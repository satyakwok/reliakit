<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-decide

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

> **Status: in development, not yet published to crates.io.** The API may change
> before the first release. See [`DESIGN.md`](./DESIGN.md) for the full design,
> the locked decisions, and the roadmap.

## Example

```rust
use reliakit_decide::{Action, Curve, Reasoner, Score};

// A bot chooses between fleeing and fighting based on its health.
let health = Score::from_ratio(20, 100); // 20% health

let mut brain = Reasoner::new();
brain.add(Action::new("flee").consider(Curve::Inverse, health));  // strong when health is low
brain.add(Action::new("fight").consider(Curve::Linear, health));  // strong when health is high

assert_eq!(brain.decide().unwrap().id, "flee"); // low health -> flee wins
```

## Core concepts

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
  `explain()` for the per-consideration breakdown of why an action won, and
  `decide_weighted(rand)` for roulette selection (caller-supplied RNG) so an agent
  varies instead of always repeating the single best.
- `Policy` — an optional persistent table of learned weights per key. `reward(key,
  outcome)` nudges a weight toward what worked (bounded integer moving average,
  deterministic); fold `weight(&key)` back into an action so choices improve over
  time. `entries()` / `set()` snapshot and restore the weights so the host can
  persist them (no built-in serializer). Not machine learning — just feedback-tuned
  weights. Key it by `(agent, action)` to give each agent its own learned weights —
  distinct "personas" with no extra types.

## `no_std`

`no_std`-compatible (`default-features = false`); always requires `alloc`.

## Safety

`#![forbid(unsafe_code)]`. All score arithmetic is saturating and panic-free.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).
