//! Deterministic, zero-dependency **decision engine** for agents and control
//! logic.
//!
//! `reliakit-decide` answers one question well: *given the current signals,
//! which action should I take?* It scores each candidate action with
//! utility-based reasoning and picks the best — deterministically, with no
//! floating point and no third-party dependencies. [`Reasoner::decide`]
//! allocates nothing; [`Reasoner::rank`] and [`Reasoner::explain`] allocate only
//! the result they return. The same signals always produce the same decision, so
//! the choice is reproducible and testable.
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
            Curve::Inverse => Score(Score::SCALE.saturating_sub(input.0)),
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
    /// A short static label naming this signal; shown by [`Reasoner::explain`].
    /// Empty by default.
    pub label: &'static str,
    /// The curve applied to the input.
    pub curve: Curve,
    /// The raw input signal, normalized to a [`Score`].
    pub input: Score,
}

impl Consideration {
    /// Creates an unlabeled consideration from a curve and an input signal.
    pub const fn new(curve: Curve, input: Score) -> Consideration {
        Consideration {
            label: "",
            curve,
            input,
        }
    }

    /// Creates a consideration with a static label (shown in explanations).
    pub const fn labeled(label: &'static str, curve: Curve, input: Score) -> Consideration {
        Consideration {
            label,
            curve,
            input,
        }
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
    /// Whether this action is permitted. A gated-off action (`false`) always has
    /// zero utility, so it is never chosen by `decide`/`decide_weighted`. Set it
    /// with [`gate`](Action::gate). Defaults to `true`.
    pub allowed: bool,
    /// The considerations multiplied together to form the utility.
    pub considerations: Vec<Consideration>,
}

impl<A> Action<A> {
    /// Creates an action with a neutral base weight and no considerations.
    pub fn new(id: A) -> Action<A> {
        Action {
            id,
            base: Score::MAX,
            allowed: true,
            considerations: Vec::new(),
        }
    }

    /// Gates the action on a caller-supplied condition (builder style). Calls
    /// combine with AND, so `.gate(a).gate(b)` is allowed only when both hold.
    ///
    /// This is how decisions become constraint-aware without any dependency: the
    /// caller passes whatever it already knows — a deadline, a rate limiter, a
    /// circuit breaker, business hours, a feature flag — as a `bool`. A gated-off
    /// action has zero utility and is never chosen. Keep one ungated fallback
    /// action so a decision still resolves when everything else is gated off.
    pub fn gate(mut self, allowed: bool) -> Action<A> {
        self.allowed = self.allowed && allowed;
        self
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

    /// Adds a labeled consideration (builder style); the label appears in
    /// [`Reasoner::explain`] output.
    pub fn consider_labeled(
        mut self,
        label: &'static str,
        curve: Curve,
        input: Score,
    ) -> Action<A> {
        self.considerations
            .push(Consideration::labeled(label, curve, input));
        self
    }

    /// Computes the action's utility: `base * product(considerations)`, or
    /// [`Score::ZERO`] if the action is gated off ([`allowed`](Action::allowed)
    /// is `false`).
    pub fn utility(&self) -> Score {
        if !self.allowed {
            return Score::ZERO;
        }
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

/// One line of an explanation: a consideration's label and the score it produced
/// for the chosen action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contribution {
    /// The consideration's label (empty if it was unlabeled).
    pub label: &'static str,
    /// The raw input signal.
    pub input: Score,
    /// The score the curve produced for that input.
    pub output: Score,
}

/// Why an action was chosen: its id, final utility, and the per-consideration
/// breakdown (in declaration order) that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation<A> {
    /// The chosen action's id.
    pub id: A,
    /// Whether the chosen action was permitted. `false` means every action was
    /// gated off and this one won only by tie-break — its utility is zero.
    pub allowed: bool,
    /// The winning utility score.
    pub utility: Score,
    /// One entry per consideration, in declaration order.
    pub contributions: Vec<Contribution>,
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

    /// Index of the highest-utility action (earlier-declared wins ties), or
    /// `None` if there are none. Shared by `decide` and `explain`.
    fn best_index(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_u = Score::ZERO;
        for (i, a) in self.actions.iter().enumerate() {
            let u = a.utility();
            if best.is_none() || u > best_u {
                best = Some(i);
                best_u = u;
            }
        }
        best
    }
}

impl<A: Clone> Reasoner<A> {
    /// Chooses the highest-utility action, or `None` if there are none.
    ///
    /// Ties resolve deterministically in favor of the earlier-declared action,
    /// so the same candidates always yield the same decision.
    pub fn decide(&self) -> Option<Decision<A>> {
        self.best_index().map(|i| Decision {
            id: self.actions[i].id.clone(),
            utility: self.actions[i].utility(),
        })
    }

    /// Chooses an action at random with probability proportional to its utility
    /// (roulette selection), so repeated decisions vary instead of always
    /// returning the single best.
    ///
    /// `rand` is any uniformly-distributed `u32` you supply (e.g. from `rand` or
    /// `getrandom`), interpreted as the fraction `rand / 2^32`. The same `rand`
    /// and candidates always yield the same choice — the engine never owns a
    /// random source, so it stays deterministic and testable.
    ///
    /// Returns `None` if there are no actions. If every utility is zero, the
    /// earliest-declared action is returned.
    pub fn decide_weighted(&self, rand: u32) -> Option<Decision<A>> {
        if self.actions.is_empty() {
            return None;
        }
        let total: u64 = self.actions.iter().map(|a| a.utility().raw() as u64).sum();
        if total == 0 {
            let a = &self.actions[0];
            return Some(Decision {
                id: a.id.clone(),
                utility: a.utility(),
            });
        }
        // `target` lands in `0..total` because `rand <= u32::MAX`; the u128
        // multiply cannot overflow.
        let target = ((rand as u128 * total as u128) >> 32) as u64;
        let mut cumulative: u64 = 0;
        for a in &self.actions {
            cumulative += a.utility().raw() as u64;
            if target < cumulative {
                return Some(Decision {
                    id: a.id.clone(),
                    utility: a.utility(),
                });
            }
        }
        // Unreachable while `target < total`; the index is valid because
        // `actions` is non-empty.
        let a = &self.actions[self.actions.len() - 1];
        Some(Decision {
            id: a.id.clone(),
            utility: a.utility(),
        })
    }

    /// Explains the winning decision: the chosen id, its utility, and the
    /// per-consideration breakdown that produced it. `None` if there are no
    /// actions. The winner matches [`decide`](Reasoner::decide).
    pub fn explain(&self) -> Option<Explanation<A>> {
        self.best_index().map(|i| {
            let a = &self.actions[i];
            let contributions = a
                .considerations
                .iter()
                .map(|c| Contribution {
                    label: c.label,
                    input: c.input,
                    output: c.score(),
                })
                .collect();
            Explanation {
                id: a.id.clone(),
                allowed: a.allowed,
                utility: a.utility(),
                contributions,
            }
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

/// A persistent table of learned weights, one per key, nudged by feedback.
///
/// Decision logic is stateless per call; `Policy` is the small mutable state that
/// lets an agent improve over time. Read a key's learned weight and fold it into
/// an action (as its base or a consideration); after an outcome, call
/// [`reward`](Policy::reward) to move that weight toward what actually worked.
///
/// The update is a **bounded integer moving average**, so it is deterministic and
/// can never run away — it is not machine learning, just `weight += rate * (outcome
/// - weight)` in fixed point, clamped to `0.0..=1.0`.
///
/// # Example
///
/// ```
/// use reliakit_decide::{Policy, Score};
///
/// // Start every key at 0.5, learning at rate 0.5.
/// let mut policy = Policy::new(Score::from_ratio(1, 2), Score::from_ratio(1, 2));
/// assert_eq!(policy.weight(&"route_a"), Score::from_ratio(1, 2)); // unseen -> default
///
/// // "route_a" worked well (outcome 1.0): its weight rises toward 1.0.
/// policy.reward("route_a", Score::MAX);
/// assert_eq!(policy.weight(&"route_a").raw(), 7_500); // 0.5 + 0.5*(1.0-0.5)
/// ```
#[derive(Debug, Clone)]
pub struct Policy<K> {
    entries: Vec<(K, Score)>,
    rate: Score,
    default: Score,
}

impl<K> Policy<K> {
    /// Creates an empty policy with a learning `rate` and a `default` weight for
    /// keys that have not been seen yet.
    pub fn new(rate: Score, default: Score) -> Policy<K> {
        Policy {
            entries: Vec::new(),
            rate,
            default,
        }
    }

    /// The number of keys that have learned weights.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no key has a learned weight yet.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The learned `(key, weight)` pairs, for snapshotting to storage. The host
    /// serializes these however it likes — the engine pulls in no serializer.
    /// Restore them later with [`set`](Policy::set).
    pub fn entries(&self) -> &[(K, Score)] {
        &self.entries
    }

    /// Bounded integer EMA: `w + rate * (outcome - w)`, clamped to `0.0..=1.0`.
    fn step(rate: Score, current: Score, outcome: Score) -> Score {
        let delta = outcome.raw() as i64 - current.raw() as i64; // [-SCALE, SCALE]
        let moved = current.raw() as i64 + (rate.raw() as i64 * delta) / Score::SCALE as i64;
        let clamped = moved.clamp(0, Score::SCALE as i64);
        Score::from_raw(clamped as u32)
    }
}

impl<K: PartialEq> Policy<K> {
    /// The learned weight for `key`, or the configured default if it is unseen.
    pub fn weight(&self, key: &K) -> Score {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, w)| *w)
            .unwrap_or(self.default)
    }

    /// Nudges `key`'s weight toward `outcome` by the learning rate. A previously
    /// unseen key starts from the default before moving.
    pub fn reward(&mut self, key: K, outcome: Score) {
        if let Some(entry) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            entry.1 = Self::step(self.rate, entry.1, outcome);
        } else {
            let moved = Self::step(self.rate, self.default, outcome);
            self.entries.push((key, moved));
        }
    }

    /// Sets `key`'s weight directly (insert or replace) — used to restore learned
    /// weights from storage. Unlike [`reward`](Policy::reward) this does not apply
    /// the learning rate; it stores the value as given.
    pub fn set(&mut self, key: K, weight: Score) {
        if let Some(entry) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            entry.1 = weight;
        } else {
            self.entries.push((key, weight));
        }
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

    #[test]
    fn with_base_scales_and_vetoes() {
        // base 0.5 * consideration 0.8 = 0.4
        let scaled = Action::new(())
            .with_base(Score::from_raw(5_000))
            .consider(Curve::Linear, Score::from_raw(8_000));
        assert_eq!(scaled.utility().raw(), 4_000);

        // base 0.0 vetoes the whole action regardless of considerations
        let vetoed = Action::new(())
            .with_base(Score::ZERO)
            .consider(Curve::Linear, Score::MAX);
        assert_eq!(vetoed.utility(), Score::ZERO);
    }

    #[test]
    fn explain_breaks_down_the_winner() {
        let health = Score::from_ratio(20, 100);
        let mut r = Reasoner::new();
        r.add(Action::new("flee").consider_labeled("low_health", Curve::Inverse, health));
        r.add(Action::new("fight").consider_labeled("high_health", Curve::Linear, health));

        let ex = r.explain().unwrap();
        assert_eq!(ex.id, "flee");
        assert_eq!(ex.utility.raw(), 8_000); // base 1.0 * Inverse(0.2) = 0.8
        assert_eq!(ex.contributions.len(), 1);
        assert_eq!(ex.contributions[0].label, "low_health");
        assert_eq!(ex.contributions[0].input, health);
        assert_eq!(ex.contributions[0].output.raw(), 8_000);
    }

    #[test]
    fn explain_on_empty_is_none() {
        let r: Reasoner<&str> = Reasoner::new();
        assert!(r.explain().is_none());
    }

    #[test]
    fn explain_lists_all_considerations_in_order() {
        let mut r = Reasoner::new();
        r.add(
            Action::new("act")
                .consider_labeled("a", Curve::Linear, Score::from_raw(8_000))
                .consider_labeled("b", Curve::Linear, Score::from_raw(5_000)),
        );
        let ex = r.explain().unwrap();
        assert_eq!(ex.utility.raw(), 4_000); // 1.0 * 0.8 * 0.5
        let labels: Vec<&str> = ex.contributions.iter().map(|c| c.label).collect();
        assert_eq!(labels, ["a", "b"]); // declaration order preserved
        assert_eq!(ex.contributions[0].output.raw(), 8_000);
        assert_eq!(ex.contributions[1].output.raw(), 5_000);
    }

    #[test]
    fn rank_keeps_declaration_order_on_ties() {
        let mut r = Reasoner::new();
        r.add(Action::new("first").consider(Curve::Linear, Score::from_raw(5_000)));
        r.add(Action::new("second").consider(Curve::Linear, Score::from_raw(5_000)));
        let ids: Vec<&str> = r.rank().iter().map(|d| d.id).collect();
        assert_eq!(ids, ["first", "second"]); // equal utility -> stable order
    }

    #[test]
    fn from_ratio_above_one_clamps() {
        assert_eq!(Score::from_ratio(3, 2), Score::MAX); // 1.5 -> clamp to 1.0
        assert_eq!(Score::from_ratio(10, 10), Score::MAX); // exactly 1.0
    }

    #[test]
    fn quadratic_extremes() {
        assert_eq!(Curve::Quadratic.eval(Score::MAX), Score::MAX); // 1.0^2 = 1.0
        assert_eq!(Curve::Quadratic.eval(Score::ZERO), Score::ZERO); // 0^2 = 0
    }

    #[test]
    fn decide_weighted_is_proportional_and_deterministic() {
        let mut r = Reasoner::new();
        r.add(Action::new("a").consider(Curve::Linear, Score::from_raw(2_500))); // util 0.25
        r.add(Action::new("b").consider(Curve::Linear, Score::from_raw(7_500))); // util 0.75

        // bottom of the range picks the first slice, top picks the last
        assert_eq!(r.decide_weighted(0).unwrap().id, "a");
        assert_eq!(r.decide_weighted(u32::MAX).unwrap().id, "b");
        // the split sits at 25%: rand = 2^30 maps to target == 2500 exactly, so
        // one below stays in "a" (target 2499) and the boundary crosses to "b".
        assert_eq!(r.decide_weighted(1_073_741_823).unwrap().id, "a"); // target 2499 < 2500
        assert_eq!(r.decide_weighted(1_073_741_824).unwrap().id, "b"); // target 2500, crosses
                                                                       // deterministic: the same rand always yields the same choice
        assert_eq!(
            r.decide_weighted(1_234_567).unwrap().id,
            r.decide_weighted(1_234_567).unwrap().id
        );
    }

    #[test]
    fn decide_weighted_zero_total_returns_first() {
        let mut r = Reasoner::new();
        r.add(Action::new("x").with_base(Score::ZERO));
        r.add(Action::new("y").with_base(Score::ZERO));
        assert_eq!(r.decide_weighted(999).unwrap().id, "x");
    }

    #[test]
    fn decide_weighted_empty_is_none() {
        let r: Reasoner<&str> = Reasoner::new();
        assert!(r.decide_weighted(0).is_none());
    }

    #[test]
    fn policy_unseen_key_returns_default() {
        let p: Policy<&str> = Policy::new(Score::from_ratio(1, 2), Score::from_raw(3_000));
        assert_eq!(p.weight(&"x").raw(), 3_000);
        assert!(p.is_empty());
    }

    #[test]
    fn policy_reward_moves_toward_outcome_and_converges() {
        // rate 0.5, default 0.5
        let mut p = Policy::new(Score::from_ratio(1, 2), Score::from_ratio(1, 2));
        p.reward("a", Score::MAX); // 0.5 + 0.5*(1.0-0.5) = 0.75
        assert_eq!(p.weight(&"a").raw(), 7_500);
        p.reward("a", Score::MAX); // 0.75 + 0.5*0.25 = 0.875
        assert_eq!(p.weight(&"a").raw(), 8_750);
        for _ in 0..50 {
            p.reward("a", Score::MAX);
        }
        // converges upward toward 1.0, never exceeding it
        assert!(p.weight(&"a").raw() > 9_900);
        assert!(p.weight(&"a").raw() <= Score::SCALE);
    }

    #[test]
    fn policy_rate_extremes() {
        // rate 0.0 -> never changes
        let mut still = Policy::new(Score::ZERO, Score::from_ratio(1, 2));
        still.reward("a", Score::MAX);
        assert_eq!(still.weight(&"a").raw(), 5_000);
        // rate 1.0 -> jumps straight to the outcome
        let mut fast = Policy::new(Score::MAX, Score::from_ratio(1, 2));
        fast.reward("a", Score::from_raw(2_000));
        assert_eq!(fast.weight(&"a").raw(), 2_000);
    }

    #[test]
    fn policy_reward_toward_zero_clamps_at_zero() {
        let mut p = Policy::new(Score::MAX, Score::from_ratio(1, 2)); // rate 1.0
        p.reward("a", Score::ZERO);
        assert_eq!(p.weight(&"a"), Score::ZERO);
    }

    #[test]
    fn policy_reward_same_key_updates_in_place() {
        let mut p = Policy::new(Score::from_ratio(1, 2), Score::from_ratio(1, 2));
        p.reward("a", Score::MAX);
        p.reward("a", Score::MAX); // same key again
        assert_eq!(p.len(), 1); // updated, not duplicated
        p.reward("b", Score::MAX); // distinct key grows the table
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn policy_set_replaces_existing() {
        let mut p = Policy::new(Score::MAX, Score::ZERO);
        p.set("a", Score::from_raw(1_000));
        p.set("a", Score::from_raw(9_000)); // replace, not duplicate
        assert_eq!(p.len(), 1);
        assert_eq!(p.weight(&"a").raw(), 9_000);
    }

    #[test]
    fn policy_entries_snapshot_round_trips_via_set() {
        let mut p = Policy::new(Score::from_ratio(1, 2), Score::ZERO);
        p.reward("a", Score::MAX);
        p.set("b", Score::from_raw(3_000));

        // snapshot for "storage", then restore into a fresh policy
        let saved: Vec<(&str, Score)> = p.entries().to_vec();
        let mut restored = Policy::new(Score::from_ratio(1, 2), Score::ZERO);
        for (k, w) in saved {
            restored.set(k, w);
        }
        assert_eq!(restored.weight(&"a"), p.weight(&"a"));
        assert_eq!(restored.weight(&"b").raw(), 3_000);
        assert_eq!(restored.len(), p.len());
    }

    #[test]
    fn gate_vetoes_and_combines() {
        // gate(true) is a no-op
        let ok = Action::new("x")
            .gate(true)
            .consider(Curve::Linear, Score::from_raw(8_000));
        assert_eq!(ok.utility().raw(), 8_000);

        // gate(false) zeroes the action even with a maxed consideration
        let blocked = Action::new("x")
            .gate(false)
            .consider(Curve::Linear, Score::MAX);
        assert_eq!(blocked.utility(), Score::ZERO);

        // gates AND together
        assert!(Action::new("x").gate(true).gate(true).allowed);
        assert!(!Action::new("x").gate(true).gate(false).allowed);
    }

    #[test]
    fn gated_action_loses_to_ungated_fallback() {
        let mut r = Reasoner::new();
        r.add(
            Action::new("call_llm")
                .gate(false) // blocked despite high utility
                .consider(Curve::Linear, Score::MAX),
        );
        r.add(Action::new("defer").consider(Curve::Linear, Score::from_raw(1_000)));
        assert_eq!(r.decide().unwrap().id, "defer");
    }

    #[test]
    fn explain_surfaces_gated_winner() {
        let mut r = Reasoner::new();
        r.add(
            Action::new("only")
                .gate(false)
                .consider(Curve::Linear, Score::MAX),
        );
        let ex = r.explain().unwrap();
        assert_eq!(ex.id, "only");
        assert!(!ex.allowed); // surfaced: it was gated off
        assert_eq!(ex.utility, Score::ZERO);
    }

    #[test]
    fn gated_action_excluded_from_weighted() {
        let mut r = Reasoner::new();
        r.add(
            Action::new("blocked")
                .gate(false)
                .consider(Curve::Linear, Score::MAX),
        );
        r.add(Action::new("open").consider(Curve::Linear, Score::from_raw(5_000)));
        // even at the bottom of the random range, a zero-weight action is skipped
        assert_eq!(r.decide_weighted(0).unwrap().id, "open");
    }

    #[test]
    fn personas_emerge_from_per_agent_policy_keys() {
        // One Policy keyed by (agent, action) gives each agent its own weights,
        // so distinct "personalities" emerge with no new machinery.
        let mut p = Policy::new(Score::MAX, Score::from_ratio(1, 2)); // rate 1.0 for a sharp test
        p.reward(("vale", "trade"), Score::MAX); // Vale learns trading works
        p.reward(("mason", "trade"), Score::ZERO); // Mason learns it does not

        let vale = Action::new("trade").with_base(p.weight(&("vale", "trade")));
        let mason = Action::new("trade").with_base(p.weight(&("mason", "trade")));
        assert_eq!(vale.utility(), Score::MAX); // same action, opposite learned bias
        assert_eq!(mason.utility(), Score::ZERO);
    }
}
