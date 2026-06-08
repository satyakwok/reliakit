//! Guard a slow dependency with a concurrency limit.
//!
//! `reliakit-bulkhead` only tracks permits; this example shows how you acquire
//! before work and release after, shedding load when the limit is reached. Run
//! with:
//!
//! ```sh
//! cargo run -p reliakit-bulkhead --example basic
//! ```

use reliakit_bulkhead::Bulkhead;

fn main() {
    // Allow at most three concurrent calls to a downstream service.
    let mut bulkhead = Bulkhead::new(3);

    // Six requests arrive while none have finished yet.
    println!("capacity: {}", bulkhead.capacity());
    for request in 0..6 {
        if bulkhead.try_acquire_one() {
            println!(
                "request {request}: admitted ({} in flight)",
                bulkhead.in_flight()
            );
        // ... start the work; release the permit when it completes ...
        } else {
            println!("request {request}: rejected (bulkhead full), shed load");
        }
    }

    // Two of the in-flight operations finish and return their permits.
    bulkhead.release(2);
    println!(
        "\nafter two completions: {} available",
        bulkhead.available()
    );

    // Room exists again, so the next request is admitted.
    if bulkhead.try_acquire_one() {
        println!("retry: admitted ({} in flight)", bulkhead.in_flight());
    }
}
