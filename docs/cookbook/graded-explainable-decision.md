# Make a graded, explainable decision

## Problem

Some choices are not a simple `if`: you weigh several signals (urgency,
confidence, cost, availability) to pick an option, and you need to explain why
afterward. Hand-rolled scoring with magic numbers spread through nested `if`s is
hard to tune, hard to test, and impossible to explain when someone asks "why did
it pick that?"

## Use

- `reliakit-decide`: a deterministic utility engine. Score options from labeled
  considerations, apply hard constraints, abstain below a threshold, and get an
  explanation.

## Example

```rust
use reliakit_decide::{Action, Curve, Reasoner, Score};

fn main() {
    // Signals about the request, each normalized to 0.0..=1.0.
    let urgency = Score::from_ratio(80, 100);
    let confidence = Score::from_ratio(40, 100);
    let llm_available = true; // e.g. circuit closed AND rate limiter has tokens

    let mut brain = Reasoner::new();

    // Answer from a template: strong when we are already confident.
    brain.add(Action::new("answer_template").consider_labeled("confidence", Curve::Linear, confidence));

    // Escalate to an LLM: strong when urgent, but only if it is available.
    brain.add(
        Action::new("call_llm")
            .gate(llm_available) // hard constraint: skipped entirely if false
            .consider_labeled("urgency", Curve::Linear, urgency),
    );

    // A low-weight fallback so a decision still resolves.
    brain.add(Action::new("defer").with_base(Score::from_ratio(1, 10)));

    // Abstain if nothing clears the bar.
    match brain.decide_above(Score::from_ratio(5, 100)) {
        Some(decision) => println!("chose {} (utility {})", decision.id, decision.utility.raw()),
        None => println!("nothing good enough: escalate to a human"),
    }
}
```

## Run it

```sh
cargo run -p reliakit-decide --example agent_brain
```

## Why this works

Each option's utility comes from labeled considerations, so the same inputs always
produce the same choice (deterministic, testable). `gate` expresses a hard
constraint: a gated-off option is removed, not just down-weighted, so "the LLM is
down" cleanly stops that path. `decide_above` lets the reasoner abstain instead of
forcing a weak pick, and `explain` returns the contribution of each consideration
so the decision is auditable.

## Common mistakes

- **Encoding hard rules as weights.** "Never do X when Y" is a `gate`, not a tiny
  score; a weight can still be outvoted.
- **No abstain threshold.** Without `decide_above`, the engine always returns
  *something*, even when every option is weak. Set a floor and escalate below it.
- **Non-deterministic inputs.** Feed it stable, normalized signals; randomness or
  wall-clock values make the choice and its explanation irreproducible.

## When not to use this

- It does not learn. Weights and curves are yours to set and tune; it is not a
  machine-learning model.
- For a single clear rule, a plain `if` is clearer than a one-consideration
  reasoner. Reach for this when several signals genuinely trade off.
- It decides; it does not act. Carry out the chosen action yourself, ideally
  behind the relevant resilience guards (retry, circuit, rate limit).
