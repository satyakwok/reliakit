# Limit concurrency with a bulkhead

## Problem

A slow dependency can sink the whole service: if calls to it pile up, you run out
of threads, connections, or memory while every request waits. Rate limiting caps
how *often* you call; it does not cap how many calls are *in flight at once*. You
want a hard ceiling on concurrent work, and to shed load fast when that ceiling is
reached instead of queueing without bound.

## Use

- `reliakit-bulkhead`: a counting semaphore. Acquire a permit before the work,
  release it after, and it tells you when to shed.

## Example

```rust
use reliakit_bulkhead::Bulkhead;

fn main() {
    // At most 3 concurrent calls to a downstream service.
    let mut bulkhead = Bulkhead::new(3);

    if bulkhead.try_acquire_one() {
        // Admitted: do the work, then return the permit.
        // (in real code this happens on the worker that ran the call)
        let _in_flight = bulkhead.in_flight();
        bulkhead.release(1);
    } else {
        // Full: shed load now (fail fast) instead of waiting.
        eprintln!("bulkhead full, rejecting");
    }
}
```

For metrics, `try_acquire_observed` reports admitted-vs-rejected and the free
permits left after each decision, without changing the acquire path.

## Run it

```sh
cargo run -p reliakit-bulkhead --example basic
```

## Why this works

The bulkhead is a permit count you advance yourself: `try_acquire_one` succeeds
only while capacity remains, and you `release` when work finishes. A full bulkhead
returns `false` immediately, so excess load is shed at the edge rather than
queued. It is a plain value with no background threads, so it works under any
runtime and in tests.

## Common mistakes

- **Forgetting to release.** A permit not returned is leaked, and the bulkhead
  slowly closes. Release on every exit path of the work, including errors.
- **Confusing it with rate limiting.** A bulkhead bounds *concurrency* (how many
  at once); `reliakit-ratelimit` bounds *rate* (how many per interval). You often
  want both.
- **Queueing instead of shedding.** Buffering rejected work in an unbounded queue
  recreates the problem. Shed, or buffer with a bound (`reliakit-collections`).
- **Sharing one bulkhead across threads without synchronization.** It is a value;
  wrap it in your own `Mutex` or use one per worker.

## When not to use this

- It is not a thread pool or executor: it does not run the work, only counts
  permits. You still schedule and run the calls.
- It does not pace a steady rate; for that use `reliakit-ratelimit`.
- It coordinates within one process. A cluster-wide concurrency cap needs a
  shared store.
