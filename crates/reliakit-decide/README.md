<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-decide

[![Crates.io](https://img.shields.io/crates/v/reliakit-decide.svg)](https://crates.io/crates/reliakit-decide)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-decide.svg)](https://crates.io/crates/reliakit-decide)
[![Docs.rs](https://docs.rs/reliakit-decide/badge.svg)](https://docs.rs/reliakit-decide)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-decide)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-decide)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

A deterministic, zero-dependency **decision engine** for agents and control
logic.

`reliakit-decide` answers one question well: *given the current signals, which
action should I take?* It scores candidate actions with utility-based reasoning
and picks the best — deterministically, with no floating point and no third-party
dependencies. `decide()` allocates nothing; `rank()` and `explain()` allocate
only the result they return. The same signals always produce the same decision,
so choices are reproducible and exactly testable.

It is **not** a language model and does not understand text. It decides *what to
do*, not *what to say* — the fast, explainable judgment layer that sits next to a
model which generates language.

## What This Crate Does

- Scores candidate `Action`s by **utility** and picks the best (`decide`), ranks
  them all (`rank`), or samples one by weight (`decide_weighted`).
- **Explains** why an action won, per consideration (`explain`).
- **Abstains** when nothing clears a threshold (`decide_above`) so the caller can
  escalate — for example, to an LLM.
- Makes decisions **constraint-aware** with no dependency (`Action::gate`).
- **Tunes** per-key weights from feedback (`Policy`), which the host can persist.

## When To Use It

- Routing or selecting among options from weighted signals (intent/agent routing,
  skill or tool selection, prioritization) where you want a real score and a
  reason, not a rigid first-match rule.
- Deciding *when* to spend an expensive resource (an LLM call, a network request)
  versus answering with a cheap path — `gate` and `decide_above` make that explicit.
- Embedded or `no_std` control logic that needs graded, testable decisions.

## When Not To Use It

- You need to generate text or understand language — that is a language model's
  job; this crate only chooses *what to do*, not *what to say*.
- You need statistical/ML learning from data — `Policy` is a bounded feedback
  average, not training.
- A plain `if` is genuinely enough — don't reach for a scorer.

## Installation

```toml
[dependencies]
reliakit-decide = "0.1"
```

## Examples

```rust
use reliakit_decide::{Action, Curve, Reasoner, Score};

// A bot chooses between fleeing and fighting based on its health.
let health = Score::from_ratio(20, 100); // 20% health

let mut brain = Reasoner::new();
brain.add(Action::new("flee").consider(Curve::Inverse, health));  // strong when health is low
brain.add(Action::new("fight").consider(Curve::Linear, health));  // strong when health is high

assert_eq!(brain.decide().unwrap().id, "flee"); // low health -> flee wins
```

A complete runnable example is in [`examples/agent_brain.rs`](./examples/agent_brain.rs):

```sh
cargo run -p reliakit-decide --example agent_brain
```

## Core Concepts

- `Score` — a fixed-point value in `0.0..=1.0` (stored as `0..=10_000`), so all
  math is integer and identical on every platform.
- `Curve` — maps a raw signal to a score (`Linear`, `Inverse`, `Quadratic`,
  `Threshold`, `Constant`).
- `Consideration` — one signal run through a curve.
- `Action` — multiplies its considerations (product-veto: any near-zero
  consideration vetoes the action) to form a utility. `gate(allowed)` makes a
  decision constraint-aware with no dependency: pass whatever you already know (a
  deadline, rate limiter, circuit breaker, business hours, a feature flag) as a
  `bool`; a gated-off action has zero utility. Keep one ungated fallback.
- `Reasoner` — holds the candidate actions: `decide()` / `rank()` by utility,
  `explain()` for the per-consideration breakdown of why an action won,
  `decide_weighted(rand)` for roulette selection (caller-supplied RNG) so an agent
  varies instead of always repeating the single best, and `decide_above(threshold)`
  to **abstain** — return `None` when nothing is good enough so the caller can
  escalate instead of forcing a weak choice.
- `Policy` — an optional persistent table of learned weights per key. `reward(key,
  outcome)` nudges a weight toward what worked (bounded integer moving average,
  deterministic); fold `weight(&key)` back into an action so choices improve over
  time. `entries()` / `set()` snapshot and restore the weights so the host can
  persist them (no built-in serializer). Not machine learning — just feedback-tuned
  weights. Key it by `(agent, action)` to give each agent its own learned weights —
  distinct "personas" with no extra types.

## Feature Flags

| Feature | Default | Effect |
|---|---|---|
| `std` | yes | Currently adds nothing beyond `core` + `alloc`; reserved for future `std`-only conveniences. |

## `no_std`

`no_std`-compatible (`default-features = false`); always requires `alloc`.

## Safety

`#![forbid(unsafe_code)]`. All score arithmetic is saturating and panic-free.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Published to crates.io and pre-1.0. The API is settling; it may receive
backward-compatible refinements before a `1.0` release.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).
