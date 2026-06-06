# reliakit-decide — design

A deterministic, zero-dependency **decision engine** for agents and control
logic. It is the fast, explainable, testable "judgment" layer that decides
*what to do*; it does not generate language or understand text — that is a job
for an LLM. In the Cole agent, `reliakit-decide` is the deterministic spine and
the LLM is the voice.

## What it is / is not

- **Is:** a utility-based decision engine. Given candidate actions and the
  current signals, it scores each action and picks the best — deterministically,
  with an explanation, in microseconds, with no allocation beyond the action
  list and no third-party dependencies.
- **Is not:** an LLM, a natural-language understander, or a learner that "knows"
  things. Any "learning" here is lightweight integer statistics (weight
  adjustment), not machine learning or comprehension. Genuine understanding /
  long-term learning lives in the host application (e.g. Cole's pgvector memory
  + LLM reflection), which can re-tune this engine's weights.

## Locked decisions (permanent once published)

1. **Fixed-point integer scores, never floating point.** A `Score` is a `u32` in
   `0..=10_000` (so `1.0` == `10_000`). Float math is non-deterministic across
   platforms; integer math is identical everywhere, which makes every decision
   reproducible and testable. This mirrors why `reliakit-codec` excludes floats.
2. **Product-veto utility.** An action's utility is its base weight multiplied by
   every consideration's score. Any near-zero consideration vetoes the action —
   all considerations must be satisfied. (Infinite-axis utility.)
3. **Saturating, panic-free arithmetic.** No input overflows or panics.
4. **Deterministic tie-breaking.** Equal utilities resolve in declaration order.
5. **Randomness is caller-supplied** (a `u32` fraction, like `reliakit-backoff`
   jitter). The engine never owns an RNG, so it stays deterministic and testable.

The score domain, the curve definitions, and the combine rule are the engine's
"wire format": changing them changes every caller's decisions, so they are
locked with exact-output tests before any release.

## Architecture (layered; core is small, advanced layers are opt-in)

Core (always on, this crate's `0.1`):

- `Score` — fixed-point `0..=1.0`.
- `Curve` — maps a raw signal to a score (`Linear`, `Inverse`, `Quadratic`,
  `Threshold`, `Constant`; more later).
- `Consideration` — one signal run through a curve.
- `Action<A>` — a candidate decision: a base weight + considerations.
- `Reasoner<A>` — holds actions, `decide()` / `rank()` by utility.
- `Decision<A>` — the chosen id + its utility (the basis for `explain`).

Planned opt-in layers (later releases, behind features — none compromise the
small core):

- `explain` — full per-consideration breakdown of why an action won.
- `variety` — weighted-random pick among the top candidates (caller RNG), so an
  agent is not monotonous.
- `bandit` — exploration vs exploitation (epsilon-greedy / softmax) so feedback
  actually improves choices over time.
- `adapt` — bounded integer weight adjustment from outcome feedback, plus
  save/load of learned weights (serialized via `reliakit-json` / `reliakit-codec`
  so the host can persist them).
- `constraint` — constraint-aware decisions that compose the rest of the Reliakit
  toolkit: skip an action when a deadline (`reliakit-timeout`) has passed, a
  limiter (`reliakit-ratelimit`) is empty, or a breaker (`reliakit-circuit`) is
  open. This makes `reliakit-decide` the capstone that ties the toolkit into one
  decision brain.
- `personas` — per-agent weight profiles, so a roster of agents decides
  distinctly.

## Honesty / positioning

Public docs describe a "deterministic decision engine", never "AI that
understands or talks like a human". Over-claiming would mislead adopters and
damage the toolkit's credibility. The engine is powerful in exactly one domain —
making fast, consistent, explainable, improvable decisions — and that is enough.

## Roadmap

- `0.1` deterministic utility core (`Score`/`Curve`/`Consideration`/`Action`/
  `Reasoner`) with exact-output tests. Can already replace rule-based routing /
  selection.
- `0.2` `explain` + `variety`.
- `0.3` `adapt` + persistable weights.
- `0.4` `bandit`, `constraint`, `personas`.
