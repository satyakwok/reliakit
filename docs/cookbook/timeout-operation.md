# Bound an operation with a deadline

## Problem

An operation that retries, waits, or makes several calls needs an overall budget,
or it can hang far longer than a caller expects. Retry counts alone do not bound
wall-clock time: a few retries with backoff can still add up. You want one
deadline for the whole operation and a cheap way to ask "how much time is left?"

## Use

- `reliakit-timeout`: a clock-agnostic `Deadline` you check against your own
  `now`. It measures time; it does not interrupt running code.

## Example

```rust
use reliakit_timeout::{Deadline, Timeout};

fn main() {
    // One-second budget for the whole operation.
    let deadline: Deadline = Timeout::new(1_000).start(0);

    let mut now: u64 = 0; // your monotonic clock, in milliseconds
    let step = 250;

    loop {
        match deadline.check(now) {
            None => break, // budget spent; stop
            Some(remaining) => {
                // Never wait longer than the time left.
                let wait = deadline.clamp(now, step);
                let _ = remaining;
                let _ = wait; // sleep/await `wait` yourself, then continue
                now += step;
            }
        }
    }
}
```

## Run it

```sh
cargo run -p reliakit-timeout --example basic
cargo run -p reliakit --example resilient_client --features "retry timeout ratelimit circuit backoff"
```

## Why this works

The deadline is computed once (`start(now)`) and then queried with `check(now)`,
which returns `None` when the budget is gone or the milliseconds remaining
otherwise. `clamp` caps any wait so you never sleep past the deadline. Because you
pass `now`, the same code runs under any runtime, in tests with a fake clock, and
in `no_std`. Nothing is interrupted: the deadline tells you to stop; it does not
force a running call to abort.

## Common mistakes

- **Treating it as cancellation.** A `Deadline` does not kill an in-flight
  blocking call. Check it between steps, or pair it with your runtime's real
  timeout for a single call that can block.
- **Bounding only retry count.** Add a deadline when you need a wall-clock limit;
  attempt limits and time limits answer different questions.
- **Waiting past the budget.** Use `clamp(now, wait)` so the last wait fits inside
  what remains.
- **A non-monotonic clock.** Feed elapsed time from a monotonic source.

## When not to use this

- It does not interrupt synchronous, blocking work. For "abort this one call after
  N ms", use the cancellation your runtime or OS provides.
- For a steady call *rate* rather than a total *budget*, use
  `reliakit-ratelimit`; for stopping calls to a failing dependency, use
  `reliakit-circuit`.
