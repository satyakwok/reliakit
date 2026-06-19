//! One batch of readings carried end to end through the single `reliakit` crate:
//! parse untrusted CSV, validate every field, buffer the good rows under a hard
//! cap, encode them for the wire, ship the batch to a flaky sink behind the
//! reliability stack, and finish with a health report.
//!
//! Each step is one crate name:
//!
//! - [`reliakit::csv`] + [`reliakit::derive`]: strict typed CSV decoding.
//! - [`reliakit::primitives`] + [`reliakit::validate`]: typed fields that reject
//!   bad input, with every violation collected instead of failing on the first.
//! - [`reliakit::collections`]: a bounded buffer that sheds load when full.
//! - [`reliakit::codec`] + [`reliakit::derive`]: canonical bytes for the wire.
//!   `Reading` derives both `CsvDecode` and `CanonicalEncode`, so one struct
//!   targets two formats.
//! - [`reliakit::retry`] + [`reliakit::backoff`] + [`reliakit::circuit`]: ship
//!   the batch to a flaky sink without hammering it.
//! - [`reliakit::health`]: a status page summarizing the run.
//!
//! Run it:
//!
//! ```sh
//! cargo run -p reliakit --example intake_pipeline \
//!   --features "csv derive primitives validate collections codec retry backoff circuit health"
//! ```

use std::time::{Duration, Instant};

use reliakit::backoff::Backoff;
use reliakit::circuit::CircuitBreaker;
use reliakit::codec::encode_to_vec;
use reliakit::collections::BoundedVec;
use reliakit::csv::from_csv_str;
use reliakit::derive::{CanonicalEncode, CsvDecode, CsvEncode};
use reliakit::health::{Health, HealthReport};
use reliakit::primitives::{Identifier, Port};
use reliakit::retry::{RetryPolicy, retry_with_sleep};
use reliakit::validate::{ValidationError, Violation};

/// A reading as it arrives in the CSV feed and as it goes out on the wire: the
/// same struct decodes from CSV and encodes to canonical bytes.
#[derive(Debug, Clone, PartialEq, CsvEncode, CsvDecode, CanonicalEncode)]
struct Reading {
    source: String,
    cpu: u8,
    port: u16,
}

impl Reading {
    /// Check every field, collecting a violation per problem. Constructing the
    /// typed primitive *is* the validation; the typed value is discarded here
    /// because the wire form is the plain row.
    fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = ValidationError::empty();
        if Identifier::new(self.source.clone()).is_err() {
            errors.push(Violation::with_field(
                "source",
                "must be an identifier (letters, digits, underscore)",
            ));
        }
        if self.cpu > 100 {
            errors.push(Violation::with_field("cpu", "must be 0-100"));
        }
        if Port::new(self.port).is_err() {
            errors.push(Violation::with_field("port", "must be 1-65535"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// The downstream sink: it rejects the first two flushes, then accepts.
fn flush_to_sink(attempt: u32, batch: &[Vec<u8>]) -> Result<(), &'static str> {
    if attempt < 3 {
        Err("sink unavailable")
    } else {
        let bytes: usize = batch.iter().map(Vec::len).sum();
        println!("    sink accepted {} records ({bytes} bytes)", batch.len());
        Ok(())
    }
}

fn main() {
    // A raw CSV feed: four good rows and three the validator must reject (bad
    // source character, cpu over 100, port zero). With a buffer cap of 3, the
    // fourth good row is shed.
    let feed = "source,cpu,port\n\
                web_01,42,8080\n\
                db_main,90,5432\n\
                bad host,10,9000\n\
                cache_1,30,6379\n\
                web_02,250,8081\n\
                web_03,15,0\n\
                queue_1,55,5672\n";

    println!("decoding CSV feed...");
    let rows: Vec<Reading> = match from_csv_str(feed) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("feed rejected: {err}");
            return;
        }
    };
    println!("  {} rows decoded\n", rows.len());

    // Validate, then buffer accepted rows under a hard cap of 3. A fourth
    // accepted row would be shed rather than grow the buffer without bound.
    let mut buffer: BoundedVec<Reading, 0, 3> = BoundedVec::new(Vec::new()).unwrap();
    let mut rejected = 0usize;
    let mut shed = 0usize;

    println!("validating and buffering...");
    for row in rows {
        match row.validate() {
            Err(errors) => {
                rejected += 1;
                for v in errors.violations() {
                    println!(
                        "  reject {:?}: {}: {}",
                        row.source,
                        v.field.unwrap_or("?"),
                        v.message
                    );
                }
            }
            Ok(()) => {
                let label = row.source.clone();
                if buffer.push(row).is_err() {
                    shed += 1;
                    println!("  shed {label:?}: buffer full");
                } else {
                    println!("  buffered {label:?}");
                }
            }
        }
    }
    println!();

    // Encode each buffered reading to canonical bytes: the wire payload.
    let batch: Vec<Vec<u8>> = buffer
        .iter()
        .map(|r| encode_to_vec(r).expect("canonical encoding is infallible for these fields"))
        .collect();
    println!("encoded {} records for the wire\n", batch.len());

    // Ship the batch behind retry + backoff + circuit. The sink fails twice and
    // then recovers; the breaker keeps us from hammering it in between.
    println!("flushing to sink...");
    let backoff = Backoff::exponential(Duration::from_millis(50), 2)
        .with_max_delay(Duration::from_millis(400));
    let policy = RetryPolicy::new(6, backoff).expect("max_attempts is non-zero");
    let mut breaker = CircuitBreaker::new(2, 200);
    let start = Instant::now();
    let mut attempt = 0u32;

    let outcome = retry_with_sleep(
        &policy,
        || {
            // Wait out the breaker cooldown while it is open.
            loop {
                let now = ms_since(start);
                if breaker.allow(now) {
                    break;
                }
                println!("    circuit open; waiting for cooldown");
                std::thread::sleep(Duration::from_millis(60));
            }
            attempt += 1;
            let now = ms_since(start);
            match flush_to_sink(attempt, &batch) {
                Ok(()) => {
                    breaker.on_success();
                    Ok(())
                }
                Err(e) => {
                    breaker.on_failure(now);
                    println!(
                        "    attempt {attempt} failed ({e}); circuit {:?}",
                        breaker.state()
                    );
                    Err(e)
                }
            }
        },
        |_error| true,
        std::thread::sleep,
    );

    let sink_ok = outcome.is_ok();
    match &outcome {
        Ok(()) => println!("  flushed after {attempt} attempt(s)\n"),
        Err(error) => println!("  gave up after {} attempt(s)\n", error.attempts()),
    }

    // A status page over the whole run: validation degrades if any row was
    // rejected, the buffer degrades if it shed, the sink is the critical leg.
    let report = HealthReport::new()
        .optional(
            "validation",
            if rejected == 0 {
                Health::Healthy
            } else {
                Health::Degraded
            },
        )
        .optional(
            "buffer",
            if shed == 0 {
                Health::Healthy
            } else {
                Health::Degraded
            },
        )
        .critical(
            "sink",
            if sink_ok {
                Health::Healthy
            } else {
                Health::Unhealthy
            },
        );

    println!("run summary:");
    println!(
        "  decoded={} rejected={rejected} shed={shed} shipped={}",
        rejected + shed + batch.len(),
        batch.len()
    );
    println!("  overall health: {}", report.overall());
}

/// Milliseconds elapsed since `start`, as the `u64` tick the policies use.
fn ms_since(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}
