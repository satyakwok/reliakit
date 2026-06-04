//! Guard a flaky dependency with a circuit breaker.
//!
//! `reliakit-circuit` only decides whether a call should be allowed; you own the
//! clock and the actual call. Run with:
//!
//! ```sh
//! cargo run -p reliakit-circuit --example basic
//! ```

use std::time::Instant;

use reliakit_circuit::{CircuitBreaker, State};

/// Stand-in for a dependency that is down for the first 5 calls, then recovers.
fn call_dependency(attempt: u32) -> Result<(), &'static str> {
    if attempt < 5 {
        Err("upstream 503")
    } else {
        Ok(())
    }
}

fn main() {
    // Trip after 3 consecutive failures; stay open for 500ms.
    let mut cb = CircuitBreaker::new(3, 500);
    let start = Instant::now();

    for attempt in 0..12u32 {
        // Our clock: milliseconds since start. The breaker never reads it itself.
        let now = start.elapsed().as_millis() as u64;

        if !cb.allow(now) {
            println!(
                "[{now:>4}ms] {:?}: rejected fast (no call made)",
                cb.state()
            );
            std::thread::sleep(std::time::Duration::from_millis(120));
            continue;
        }

        match call_dependency(attempt) {
            Ok(()) => {
                cb.on_success();
                println!("[{now:>4}ms] {:?}: call ok", cb.state());
            }
            Err(e) => {
                cb.on_failure(now);
                println!("[{now:>4}ms] {:?}: call failed ({e})", cb.state());
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    assert_eq!(cb.state(), State::Closed, "breaker should recover");
    println!("\nfinal state: {:?}", cb.state());
}
