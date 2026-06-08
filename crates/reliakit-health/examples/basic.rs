//! Roll up component checks into a `/health` decision, with one non-critical
//! dependency that only degrades the service.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p reliakit-health --example health_basic
//! ```

use reliakit_health::{Health, HealthReport};

fn main() {
    // Report each dependency's status. The cache is optional: if it is down the
    // service still serves (degraded), it is not taken offline.
    let report = HealthReport::new()
        .critical("database", Health::Healthy)
        .critical("queue", Health::Degraded)
        .detail("redelivery backlog")
        .optional("cache", Health::Unhealthy)
        .detail("primary node unreachable");

    let overall = report.overall();
    println!("overall: {overall}");

    let s = report.summary();
    println!(
        "components: {} healthy, {} degraded, {} unhealthy ({} total)",
        s.healthy,
        s.degraded,
        s.unhealthy,
        s.total()
    );

    if report.reasons().next().is_some() {
        println!("\nissues:");
        for (name, detail) in report.reasons() {
            match detail {
                Some(d) => println!("  - {name}: {d}"),
                None => println!("  - {name}"),
            }
        }
    }

    // What a /health endpoint would return.
    let (code, body) = if overall.is_operational() {
        (200, "OK")
    } else {
        (503, "Service Unavailable")
    };
    println!("\nHTTP {code} {body}");
}
