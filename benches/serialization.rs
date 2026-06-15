//! Throughput benchmarks for reliakit's serialization crates.
//!
//! There is no third-party benchmark framework here: each workload is timed with
//! [`std::time`] and kept honest with [`std::hint::black_box`], so the benchmark
//! itself stays within the zero-dependency rule. Numbers are indicative
//! (single-run, host-dependent), not a competitive claim.
//!
//! ```sh
//! cargo bench -p reliakit-benches
//! ```

use std::hint::black_box;
use std::time::{Duration, Instant};

use reliakit_codec::{decode_from_slice, encode_to_vec};
use reliakit_json::parse;

/// Run `op` for at least ~300ms, then report nanoseconds per operation and
/// throughput over a `payload`-byte working set. The iteration count grows until
/// the measured run clears the time floor, which keeps short and long workloads
/// comparable without a fixed, guessed iteration count.
fn bench(label: &str, payload: usize, mut op: impl FnMut()) {
    let mut iters: u64 = 32;
    loop {
        let start = Instant::now();
        for _ in 0..iters {
            op();
        }
        let elapsed = start.elapsed();
        if elapsed >= Duration::from_millis(300) || iters >= 1 << 30 {
            let ns = elapsed.as_secs_f64() * 1e9 / iters as f64;
            let mib = (payload as f64 * iters as f64) / (1024.0 * 1024.0) / elapsed.as_secs_f64();
            println!("{label:<28} {ns:>11.1} ns/op   {mib:>9.1} MiB/s");
            return;
        }
        iters *= 2;
    }
}

fn json_payload(records: usize) -> String {
    let mut s = String::from("{\"items\":[");
    for i in 0..records {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"id\":{i},\"name\":\"service-{i}\",\"active\":{},\"score\":{i}.5}}",
            i % 2 == 0
        ));
    }
    s.push_str("]}");
    s
}

fn codec_records(records: u64) -> Vec<(u64, String)> {
    (0..records).map(|i| (i, format!("service-{i}"))).collect()
}

fn main() {
    println!("reliakit serialization benchmarks (zero-dependency harness)\n");

    let json = json_payload(200);
    let json_bytes = json.len();
    bench("json::parse", json_bytes, || {
        black_box(parse(black_box(json.as_bytes())).unwrap());
    });

    let records = codec_records(500);
    let encoded = encode_to_vec(&records).unwrap();
    let enc_len = encoded.len();

    bench("codec::encode_to_vec", enc_len, || {
        black_box(encode_to_vec(black_box(&records)).unwrap());
    });

    bench("codec::decode_from_slice", enc_len, || {
        let decoded: (Vec<(u64, String)>, usize) = decode_from_slice(black_box(&encoded)).unwrap();
        black_box(decoded);
    });

    println!("\npayloads: json {json_bytes} B (200 records), codec {enc_len} B (500 records)");
}
