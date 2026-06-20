# Rate-limit a worker

## Problem

A worker, a background job, or a client calls something that must not be hit too
fast: a third-party API with a quota, a database, a downstream service. You want
to allow short bursts but hold a steady average rate, and to know how long to
wait when a request does not fit.

## Use

- `reliakit-ratelimit`: a token bucket. It only decides whether a request fits;
  you own the clock and the work.

## Example

```rust
use reliakit_ratelimit::RateLimiter;

fn main() {
    // Burst up to 5, refill 1 token every 200ms (≈5 requests/second sustained).
    let mut limiter = RateLimiter::new(5, 1, 200);

    // `now` is whatever monotonic millisecond tick you already track.
    let now: u64 = 0;

    if limiter.try_acquire_one(now) {
        // allowed: do the work
        let _left = limiter.available(now);
    } else {
        // throttled: ask how long until one token is available
        let wait_ms = limiter.retry_after(now, 1).unwrap_or(0);
        let _ = wait_ms; // sleep/await this yourself, then try again
    }
}
```

`try_acquire_one` consumes a token only when it returns `true`; a throttled call
costs nothing.

## Run it

```sh
cargo run -p reliakit-ratelimit --example basic
```

## Why this works

The limiter is a plain value you advance with an explicit `now`. There is no
background timer and no global state, so it is deterministic in tests (feed it the
ticks you choose), works under any runtime, and runs in `no_std`. `retry_after`
gives you a precise wait instead of a busy-loop.

## Common mistakes

- **Busy-waiting.** Looping on `try_acquire_one` without sleeping burns CPU. Use
  `retry_after` to wait exactly long enough.
- **A non-monotonic `now`.** Feed a monotonic clock (elapsed milliseconds), not
  wall-clock time that can jump backward.
- **One shared limiter without synchronization.** `RateLimiter` is a value;
  sharing it across threads needs your own `Mutex` or per-worker instances.
- **Requesting more than the capacity.** A single request larger than the bucket
  capacity can never fit and will always be refused.

## When not to use this

- Rate limiting is **not** authentication, authorization, or abuse prevention. It
  shapes traffic; it does not decide who is allowed.
- It does not coordinate across processes or machines. For a global limit, you
  need a shared store; this crate limits within one in-memory bucket.
- For bounding *in-flight concurrency* rather than *rate*, use
  `reliakit-bulkhead` instead.
