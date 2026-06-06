//! A tiny decision brain: an assistant deciding what to do with a request.
//!
//! It shows the pieces working together — graded considerations, a hard
//! constraint via `gate`, abstaining via `decide_above`, and an explanation.
//! Run with: `cargo run -p reliakit-decide --example agent_brain`

use reliakit_decide::{Action, Curve, Reasoner, Score};

fn main() {
    // Signals about the current request, each normalized to 0.0..=1.0.
    let urgency = Score::from_ratio(80, 100);
    let confidence = Score::from_ratio(40, 100); // how well we already understand it
    let llm_available = true; // e.g. circuit breaker closed AND rate limiter has tokens

    let mut brain = Reasoner::new();

    // Answer from a template — strong when we are already confident.
    brain.add(Action::new("answer_template").consider_labeled(
        "confidence",
        Curve::Linear,
        confidence,
    ));

    // Escalate to the LLM — strong when urgent, but only if the LLM is available.
    brain.add(
        Action::new("call_llm")
            .gate(llm_available) // constraint-aware: skipped entirely if the LLM is down/limited
            .consider_labeled("urgency", Curve::Linear, urgency),
    );

    // An always-available, low-weight fallback so a decision still resolves.
    brain.add(Action::new("defer").with_base(Score::from_ratio(1, 10)));

    // Abstain if nothing clears the bar — the caller would escalate to a human.
    let threshold = Score::from_ratio(5, 100);
    match brain.decide_above(threshold) {
        Some(decision) => {
            println!(
                "chose: {} (utility {}/10000)",
                decision.id,
                decision.utility.raw()
            );
            if let Some(why) = brain.explain() {
                for c in why.contributions {
                    println!(
                        "  {} : input {} -> {}",
                        c.label,
                        c.input.raw(),
                        c.output.raw()
                    );
                }
            }
        }
        None => println!("nothing good enough — escalate to a human"),
    }
}
