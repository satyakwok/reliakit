# Aggregate checks for a health endpoint

## Problem

A `/health` or `/ready` endpoint has to roll several dependency checks into one
answer. Not every dependency is equal: if the database is down the service is
unavailable, but if an optional cache is down the service still works, just
degraded. Treating them all the same either takes you offline for a minor outage
or reports healthy when you are not.

## Use

- `reliakit-health`: status types and a criticality-aware aggregator. It reports;
  it never acts.

## Example

```rust
use reliakit_health::{Health, HealthReport};

fn main() {
    // You run the probes; report each result with its criticality.
    let report = HealthReport::new()
        .critical("database", Health::Healthy)
        .critical("queue", Health::Degraded)
        .detail("redelivery backlog")
        .optional("cache", Health::Unhealthy) // optional: degrades, does not fail
        .detail("primary node unreachable");

    let overall = report.overall();

    // What the endpoint returns.
    let (code, body) = if overall.is_operational() {
        (200, "OK")
    } else {
        (503, "Service Unavailable")
    };
    println!("HTTP {code} {body} (overall: {overall})");

    for (name, detail) in report.reasons() {
        match detail {
            Some(d) => println!("  {name}: {d}"),
            None => println!("  {name}"),
        }
    }
}
```

## Run it

```sh
cargo run -p reliakit-health --example basic
```

## Why this works

Criticality is part of each check. A failed `critical` dependency makes the whole
report non-operational; a failed `optional` one only degrades it, so a minor
outage does not take you offline. `overall().is_operational()` collapses the roll-
up to the yes/no a load balancer needs, while `reasons()` and `summary()` give the
detail for a status page. The crate only computes status from what you report; it
runs no probes and takes no action.

## Common mistakes

- **Marking everything critical.** Then one optional cache blip returns 503 and
  sheds all traffic. Use `optional` for dependencies you can serve without.
- **Doing real work in the endpoint.** A health check that runs slow queries
  becomes a load amplifier under probing. Report cached or cheap probe results.
- **Conflating liveness and readiness.** "Am I alive?" and "should I get traffic?"
  are different questions; build a separate report for each if your platform
  distinguishes them.

## When not to use this

- It aggregates results you supply; it does not probe dependencies for you. You
  own the checks.
- It is not a monitoring or alerting system. It answers one request; it does not
  store history or page anyone.
