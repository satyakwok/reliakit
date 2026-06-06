//! Deterministic, zero-dependency **decision engine** for agents and control
//! logic.
//!
//! `reliakit-decide` answers one question well: *given the current signals,
//! which action should I take?* It scores each candidate action with
//! utility-based reasoning and picks the best — deterministically, with no
//! floating point, no allocation beyond the action list, and no third-party
//! dependencies. The same signals always produce the same decision, so the
//! choice is reproducible and testable.
//!
//! It is **not** a language model and does not understand text; it decides
//! *what to do*, not *what to say*. In an agent it is the fast, explainable
//! "judgment" layer next to a model that generates language.
//!
//! # Model
//!
//! - A [`Score`] is a fixed-point value in `0.0..=1.0` (stored as `0..=10_000`).
//! - A [`Curve`] maps a raw signal to a score (e.g. "low health" → high score).
//! - A [`Consideration`] is one signal run through a curve.
//! - An [`Action`] multiplies its considerations together (product-veto: any
//!   near-zero consideration vetoes the action) to get a utility.
//! - A [`Reasoner`] holds the candidate actions and selects the best.
//!
//! # Example
//!
//! ```
//! use reliakit_decide::{Action, Curve, Reasoner, Score};
//!
//! // A bot chooses between fleeing and fighting based on its health.
//! let health = Score::from_ratio(20, 100); // 20% health
//!
//! let mut brain = Reasoner::new();
//! brain.add(Action::new("flee").consider(Curve::Inverse, health)); // strong when health is low
//! brain.add(Action::new("fight").consider(Curve::Linear, health)); // strong when health is high
//!
//! let choice = brain.decide().unwrap();
//! assert_eq!(choice.id, "flee"); // low health -> flee wins
//! ```
//!
//! # `no_std`
//!
//! The crate is `no_std`-compatible (`default-features = false`) and always
//! requires `alloc`. The default `std` feature currently adds nothing beyond
//! `core` + `alloc`.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

use alloc::vec::Vec;

/// A fixed-point score in the inclusive range `0.0..=1.0`, stored as an integer
/// in `0..=10_000` (steps of `0.0001`).
///
/// Scores are integers so that every computation is bit-for-bit identical on
/// every platform — decisions are deterministic and exactly testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Score(u32);

impl Score {
    /// The fixed-point scale: a raw value of `SCALE` represents `1.0`.
    pub const SCALE: u32 = 10_000;
    /// The minimum score, `0.0`.
    pub const ZERO: Score = Score(0);
    /// The maximum score, `1.0`.
    pub const MAX: Score = Score(Self::SCALE);

    /// Creates a score from a raw fixed-point value, clamped to `0..=SCALE`.
    pub const fn from_raw(raw: u32) -> Score {
        Score(if raw > Self::SCALE { Self::SCALE } else { raw })
    }

    /// Returns the raw fixed-point value (`0..=SCALE`).
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Creates a score from the ratio `num / den`, clamped to `0.0..=1.0`.
    ///
    /// A zero denominator yields [`Score::ZERO`].
    pub const fn from_ratio(num: u32, den: u32) -> Score {
        if den == 0 {
            Score::ZERO
        } else {
            let v = (num as u64 * Self::SCALE as u64) / den as u64;
            Score::from_raw(if v > Self::SCALE as u64 {
                Self::SCALE
            } else {
                v as u32
            })
        }
    }

    /// Multiplies two scores in the fixed-point domain (`self * other`), staying
    /// within `0.0..=1.0`. Multiplying by [`Score::MAX`] is the identity.
    pub const fn mul(self, other: Score) -> Score {
        Score(((self.0 as u64 * other.0 as u64) / Self::SCALE as u64) as u32)
    }
}

/// Maps a raw input signal (already a [`Score`]) to a contribution score.
///
/// Curves are what make decisions feel graded rather than a rigid `if`: a low
/// signal can still contribute something, and emphasis can be shaped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Curve {
    /// Always returns the given score, ignoring the input.
    Constant(Score),
    /// Returns the input unchanged.
    Linear,
    /// Returns `1.0 - input` (high when the input is low).
    Inverse,
    /// Returns `input * input` — dampens low inputs, keeps high ones.
    Quadratic,
    /// A soft step: returns `above` when `input >= at`, otherwise `below`.
    Threshold {
        /// The input value at which the step flips.
        at: Score,
        /// The score returned below the threshold.
        below: Score,
        /// The score returned at or above the threshold.
        above: Score,
    },
}

impl Curve {
    /// Evaluates the curve for a given input signal.
    pub const fn eval(self, input: Score) -> Score {
        match self {
            Curve::Constant(s) => s,
            Curve::Linear => input,
            Curve::Inverse => Score(Score::SCALE - input.0),
            Curve::Quadratic => input.mul(input),
            Curve::Threshold { at, below, above } => {
                if input.0 >= at.0 {
                    above
                } else {
                    below
                }
            }
        }
    }
}

/// A single weighted input: a raw signal run through a [`Curve`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Consideration {
    /// The curve applied to the input.
    pub curve: Curve,
    /// The raw input signal, normalized to a [`Score`].
    pub input: Score,
}

impl Consideration {
    /// Creates a consideration from a curve and an input signal.
    pub const fn new(curve: Curve, input: Score) -> Consideration {
        Consideration { curve, input }
    }

    /// The consideration's contribution score, `curve.eval(input)`.
    pub const fn score(self) -> Score {
        self.curve.eval(self.input)
    }
}

/// A candidate decision: the value returned if chosen, plus the considerations
/// that score it.
///
/// Utility is the base weight multiplied by every consideration. Because they
/// multiply, **any near-zero consideration vetoes the action** — all of them
/// must be satisfied for a high utility.
#[derive(Debug, Clone)]
pub struct Action<A> {
    /// The value returned when this action is chosen.
    pub id: A,
    /// The base weight before considerations (defaults to [`Score::MAX`]).
    pub base: Score,
    /// The considerations multiplied together to form the utility.
    pub considerations: Vec<Consideration>,
}

impl<A> Action<A> {
    /// Creates an action with a neutral base weight and no considerations.
    pub fn new(id: A) -> Action<A> {
        Action {
            id,
            base: Score::MAX,
            considerations: Vec::new(),
        }
    }

    /// Sets the base weight (builder style).
    pub fn with_base(mut self, base: Score) -> Action<A> {
        self.base = base;
        self
    }

    /// Adds a consideration (builder style).
    pub fn consider(mut self, curve: Curve, input: Score) -> Action<A> {
        self.considerations.push(Consideration::new(curve, input));
        self
    }

    /// Computes the action's utility: `base * product(considerations)`.
    pub fn utility(&self) -> Score {
        let mut u = self.base;
        for c in &self.considerations {
            u = u.mul(c.score());
        }
        u
    }
}

/// The outcome of a decision: the chosen id and the utility it won with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision<A> {
    /// The chosen action's id.
    pub id: A,
    /// The winning utility score.
    pub utility: Score,
}

/// Holds candidate [`Action`]s and selects among them by utility.
#[derive(Debug, Clone)]
pub struct Reasoner<A> {
    actions: Vec<Action<A>>,
}

impl<A> Default for Reasoner<A> {
    fn default() -> Self {
        Reasoner {
            actions: Vec::new(),
        }
    }
}

impl<A> Reasoner<A> {
    /// Creates an empty reasoner.
    pub fn new() -> Reasoner<A> {
        Reasoner::default()
    }

    /// Adds a candidate action.
    pub fn add(&mut self, action: Action<A>) -> &mut Self {
        self.actions.push(action);
        self
    }

    /// Returns the number of candidate actions.
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Returns `true` if there are no candidate actions.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

impl<A: Clone> Reasoner<A> {
    /// Chooses the highest-utility action, or `None` if there are none.
    ///
    /// Ties resolve deterministically in favor of the earlier-declared action,
    /// so the same candidates always yield the same decision.
    pub fn decide(&self) -> Option<Decision<A>> {
        let mut best: Option<usize> = None;
        let mut best_u = Score::ZERO;
        for (i, a) in self.actions.iter().enumerate() {
            let u = a.utility();
            if best.is_none() || u > best_u {
                best = Some(i);
                best_u = u;
            }
        }
        best.map(|i| Decision {
            id: self.actions[i].id.clone(),
            utility: best_u,
        })
    }

    /// Returns every action ranked by utility, highest first.
    ///
    /// The sort is stable, so ties keep declaration order and the ranking is
    /// deterministic.
    pub fn rank(&self) -> Vec<Decision<A>> {
        let mut out: Vec<Decision<A>> = self
            .actions
            .iter()
            .map(|a| Decision {
                id: a.id.clone(),
                utility: a.utility(),
            })
            .collect();
        out.sort_by_key(|d| core::cmp::Reverse(d.utility));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_ratio_clamp_and_mul() {
        assert_eq!(Score::from_ratio(1, 2).raw(), 5_000);
        assert_eq!(Score::from_ratio(3, 0), Score::ZERO);
        assert_eq!(Score::from_raw(99_999), Score::MAX); // clamped
        assert_eq!(Score::MAX.mul(Score::from_raw(5_000)).raw(), 5_000); // identity
        assert_eq!(
            Score::from_raw(5_000).mul(Score::from_raw(5_000)).raw(),
            2_500
        ); // 0.5*0.5
    }

    #[test]
    fn curves_eval_exactly() {
        let x = Score::from_raw(3_000);
        assert_eq!(Curve::Linear.eval(x), x);
        assert_eq!(Curve::Inverse.eval(x).raw(), 7_000);
        assert_eq!(Curve::Quadratic.eval(Score::from_raw(5_000)).raw(), 2_500);
        assert_eq!(Curve::Constant(Score::MAX).eval(Score::ZERO), Score::MAX);
        let step = Curve::Threshold {
            at: Score::from_raw(5_000),
            below: Score::ZERO,
            above: Score::MAX,
        };
        assert_eq!(step.eval(Score::from_raw(4_999)), Score::ZERO);
        assert_eq!(step.eval(Score::from_raw(5_000)), Score::MAX);
    }

    #[test]
    fn utility_is_product_veto() {
        // one zero consideration vetoes the whole action
        let vetoed = Action::new(())
            .consider(Curve::Linear, Score::MAX)
            .consider(Curve::Linear, Score::ZERO);
        assert_eq!(vetoed.utility(), Score::ZERO);

        // base(1.0) * 0.8 * 0.5 = 0.4
        let a = Action::new(())
            .consider(Curve::Linear, Score::from_raw(8_000))
            .consider(Curve::Linear, Score::from_raw(5_000));
        assert_eq!(a.utility().raw(), 4_000);
    }

    #[test]
    fn decide_picks_highest_and_breaks_ties_by_order() {
        let mut r = Reasoner::new();
        r.add(Action::new("a").consider(Curve::Linear, Score::from_raw(3_000)));
        r.add(Action::new("b").consider(Curve::Linear, Score::from_raw(9_000)));
        assert_eq!(r.decide().unwrap().id, "b");

        // equal utility -> earlier-declared wins
        let mut t = Reasoner::new();
        t.add(Action::new("first").consider(Curve::Linear, Score::from_raw(5_000)));
        t.add(Action::new("second").consider(Curve::Linear, Score::from_raw(5_000)));
        assert_eq!(t.decide().unwrap().id, "first");
    }

    #[test]
    fn decide_on_empty_is_none() {
        let r: Reasoner<&str> = Reasoner::new();
        assert!(r.decide().is_none());
        assert!(r.is_empty());
    }

    #[test]
    fn rank_orders_descending_stably() {
        let mut r = Reasoner::new();
        r.add(Action::new("low").consider(Curve::Linear, Score::from_raw(2_000)));
        r.add(Action::new("high").consider(Curve::Linear, Score::from_raw(8_000)));
        r.add(Action::new("mid").consider(Curve::Linear, Score::from_raw(5_000)));
        let ranked = r.rank();
        let ids: Vec<&str> = ranked.iter().map(|d| d.id).collect();
        assert_eq!(ids, ["high", "mid", "low"]);
    }
}
