# Bound a buffer so it cannot grow without limit

## Problem

A `Vec` used as a queue, buffer, or cache grows for as long as you push to it.
Fed by untrusted or bursty input, that is an unbounded-memory risk: one busy
period or one malicious sender can exhaust memory. You want a collection that
enforces a size limit at every mutation, or that keeps only the most recent N
items and drops the rest.

## Use

- `reliakit-collections`: `BoundedVec` (a length range enforced on every change)
  and `RingBuffer` (fixed capacity, evicts the oldest).

## Example

```rust
use reliakit_collections::{BoundedVec, RingBuffer};

fn main() {
    // A buffer that must hold 1..=3 items; pushes past the bound are refused.
    let mut buffer: BoundedVec<&str, 1, 3> = BoundedVec::new(vec!["first"]).unwrap();
    buffer.push("second").unwrap();
    buffer.push("third").unwrap();
    assert!(buffer.push("fourth").is_err()); // over the upper bound

    // A window of the 3 most recent events; the oldest is evicted on overflow.
    let mut recent: RingBuffer<u32> = RingBuffer::new(3).unwrap();
    for n in 1..=5 {
        if let Some(evicted) = recent.push(n) {
            println!("evicted {evicted}");
        }
    }
    assert_eq!(recent.len(), 3);
}
```

## Run it

```sh
cargo run -p reliakit-collections --example basic
```

## Why this works

The bound is part of the type and checked at every mutation, so a `BoundedVec`
can never hold an invalid number of elements: an over-limit `push` returns an
error instead of growing, and an under-limit `pop` is refused. `RingBuffer` takes
the other stance: it always accepts the newest item and hands back whatever it
evicted, so memory stays flat. Either way the size is bounded by construction, not
by remembering to check.

## Common mistakes

- **A plain `Vec` for untrusted input.** That is the unbounded-growth bug. Bound
  the collection where external data accumulates.
- **Wrong tool for the job.** Use `BoundedVec` to *reject* overflow, `RingBuffer`
  to *evict* and keep the newest. Picking the wrong one either drops data you
  needed or rejects data you wanted to keep.
- **Forgetting the lower bound.** `BoundedVec<T, MIN, MAX>` also refuses popping
  below `MIN`; set `MIN` to `0` if you do not want a floor.

## When not to use this

- These are single-threaded values, not concurrent or lock-free structures. Share
  them behind your own synchronization.
- For a key/value or set with bounds, use `BoundedMap` / `BoundedSet` from the
  same crate; for a stack-allocated bounded string, see `reliakit-primitives`
  `InlineStr`.
- A bound caps size, not meaning. Validate the *contents* separately
  (`reliakit-validate`, `reliakit-primitives`).
