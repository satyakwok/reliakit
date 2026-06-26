# Circuit breaker for a flaky service

## Problem

When a dependency is down, retrying every call makes things worse: you pile load
onto something already failing and make your own latency spike while every call
waits to time out. You want to stop calling for a short cool-off after a run of
failures, then probe carefully to see if it recovered.

## Use

- `reliakit-circuit`: a state machine (Closed, Open, HalfOpen) that decides
  whether a call is allowed. You own the clock and the call.

## Example

```rust
use reliakit_circuit::{CircuitBreaker, State};

fn call_dependency(attempt: u32) -> Result<(), &'static str> {
    if attempt < 5 { Err("upstream 503") } else { Ok(()) }
}

fn main() {
    // Trip after 3 consecutive failures; stay open for 500ms before probing.
    let mut breaker = CircuitBreaker::new(3, 500);
    let mut now: u64 = 0; // your monotonic clock, in milliseconds

    for attempt in 0..12u32 {
        if !breaker.allow(now) {
            // Open: reject fast, no call made.
            now += 120;
            continue;
        }
        match call_dependency(attempt) {
            Ok(()) => breaker.on_success(),
            Err(_) => breaker.on_failure(now),
        }
        now += 120;
    }

    assert_eq!(breaker.state(), State::Closed); // recovered
}
```

## Run it

```sh
cargo run -p reliakit-circuit --example basic
cargo run -p reliakit --example resilient_client --features "retry timeout ratelimit circuit backoff"
```

## Why this works

The breaker is a plain value advanced with an explicit `now`. `allow` tells you
whether to make the call; `on_success`/`on_failure` report the outcome. After
`failure_threshold` consecutive failures it opens and rejects immediately for the
cooldown, then allows a single probe (HalfOpen) and closes again only once that
succeeds. No background timer, no global state, deterministic in tests.

## Common mistakes

- **Calling the dependency before `allow`.** The point is to *not* call while
  open. Check `allow(now)` first and skip the call when it returns `false`.
- **Forgetting to report the outcome.** The breaker only learns from
  `on_success`/`on_failure`; if you skip them it never trips or recovers.
- **A backwards or non-monotonic clock.** Feed elapsed milliseconds from a
  monotonic source so the cooldown math stays correct.
- **Using it as a retry loop.** It decides *whether to call*, not *when to try
  again*. Pair it with `reliakit-retry` for the loop.

## When not to use this

- A breaker does not fix a broken dependency; it protects *you* from it. You
  still need a fallback or a clear error for callers when it is open.
- For a single one-shot call with no surrounding loop, a breaker adds little; it
  pays off when the same dependency is called repeatedly.
- It is per-instance and in-memory; it does not coordinate breaker state across
  processes or machines.
