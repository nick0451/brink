//! BRINK AI. Controllers decide from an [`ObserverView`] only and never see
//! the canonical world state (enforced by the `Controller` signature and a
//! source lint test).
//!
//! Every decision is a [`Score`]: labelled terms whose sum is the decision
//! value. The terms are the explanation lines shown to players and logs, so
//! "lines sum to the score" holds by construction (DESIGN §14.1 rule 4).
//! No rule anywhere branches on country identity (rule 5).

use serde::{Deserialize, Serialize};
use sim_core::{CountryId, DecisionKind, ObserverView, Order};

pub mod evaluate;
pub mod goals;
pub mod inputs;
pub mod reflexes;
pub mod sanctions;
pub mod strategist;
pub mod war;

pub use strategist::Strategist;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReasonLine {
    /// Stable key for presentation and voice selection (e.g.
    /// `security_need`, `reflex:fraternal_concern`); survives relabelling.
    pub term: String,
    /// Display text.
    pub label: String,
    pub value: f64,
}

/// Stable term key derived from a display label.
pub fn term_key(label: &str) -> String {
    label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// A utility sum. Positive total = act.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Score {
    pub lines: Vec<ReasonLine>,
}

impl Score {
    pub fn new() -> Self {
        Score::default()
    }

    /// Add a term keyed by its label. Exact zeros are kept out.
    pub fn add(&mut self, label: impl Into<String>, value: f64) -> &mut Self {
        let label = label.into();
        let term = term_key(&label);
        self.add_term(term, label, value)
    }

    /// Add a term with an explicit stable key.
    pub fn add_term(&mut self, term: impl Into<String>, label: impl Into<String>, value: f64) -> &mut Self {
        if value != 0.0 && value.is_finite() {
            self.lines.push(ReasonLine {
                term: term.into(),
                label: label.into(),
                value,
            });
        }
        self
    }

    pub fn total(&self) -> f64 {
        self.lines.iter().map(|l| l.value).sum()
    }

    /// Lines sorted by magnitude, largest first (for display).
    pub fn sorted(&self) -> Vec<ReasonLine> {
        let mut v = self.lines.clone();
        v.sort_by(|a, b| b.value.abs().total_cmp(&a.value.abs()).then(a.label.cmp(&b.label)));
        v
    }
}

/// One evaluated decision with its outcome and explanation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionRecord {
    /// What was being decided, e.g. "accept DefensiveAlliance from MAJ".
    pub subject: String,
    /// Structured decision kind (stable for presentation and voice).
    pub kind: DecisionKind,
    /// The other party of the decision, if any.
    pub counterpart: Option<CountryId>,
    pub score: f64,
    pub chosen: bool,
    pub lines: Vec<ReasonLine>,
    /// Ledger precedents the decider weighed (most relevant first).
    pub precedents: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub orders: Vec<Order>,
    pub records: Vec<DecisionRecord>,
}

pub trait Controller {
    fn decide(&mut self, view: &ObserverView) -> Decision;
}

/// Placeholder controller: holds its budget and stops deficit spending once
/// debt becomes heavy. Useful as a passive baseline in tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct Steady;

/// Debt-to-annual-GDP ratio above which controllers stop borrowing.
pub const DEBT_LIMIT: f64 = 0.9;

impl Controller for Steady {
    fn decide(&mut self, view: &ObserverView) -> Decision {
        let mut decision = Decision::default();
        if let Some(order) = debt_brake(view) {
            decision.orders.push(order);
        }
        decision
    }
}

/// Stop deficit spending above [`DEBT_LIMIT`].
pub fn debt_brake(view: &ObserverView) -> Option<Order> {
    (view.own.debt_ratio() > DEBT_LIMIT && view.own.deficit_ratio > 0.0).then_some(Order::SetDeficit(0.0))
}
