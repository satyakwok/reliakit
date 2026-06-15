# Benchmarks

reliakit ships a small, **zero-dependency** benchmark harness in the `benches`
crate. It does not pull in criterion or any other framework: each workload is
timed with `std::time` and kept honest with `std::hint::black_box`, so measuring
performance never breaks the project's zero-dependency rule.

## Running

```sh
cargo bench -p reliakit-benches
```

The harness grows its iteration count until a run clears a ~300 ms floor, then
reports nanoseconds per operation and throughput over the working set.

## What is measured

| Workload | What it does |
|---|---|
| `json::parse` | Parse a ~11 KB JSON document (200 records) into a `JsonValue` |
| `codec::encode_to_vec` | Encode 500 `(u64, String)` records to canonical bytes |
| `codec::decode_from_slice` | Decode those canonical bytes back into the records |

## Indicative numbers

These are single-run, host-dependent figures from one developer machine; they
are a sanity check, **not** a competitive claim. Run the harness yourself for
numbers that reflect your hardware.

```
json::parse                     333837.1 ns/op        33.7 MiB/s
codec::encode_to_vec             15032.1 ns/op       722.9 MiB/s
codec::decode_from_slice         46750.6 ns/op       232.4 MiB/s
```

## Why no criterion

criterion is a third-party dependency, and the workspace forbids third-party
crates everywhere — including dev and tooling dependencies (CI enforces this
against `Cargo.lock`). The hand-rolled harness keeps that guarantee. For the
non-runtime cost of depending on reliakit (zero dependencies, no `unsafe`,
`no_std`, compile footprint), see the Footprint section of the root README.
