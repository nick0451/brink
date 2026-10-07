//! Per-observer reputation read from the Event Ledger (DESIGN §21.3).
//!
//! There is no stored or authoritative global credibility. Each observer
//! weighs the entries it has seen by recency, relevance to its own case and
//! cost, so different observers can rationally disagree about the same
//! country. The global figure exists for display only and must never be an
//! AI input.

use serde::{Deserialize, Serialize};

use crate::ids::CountryId;
use crate::ledger::{EntryKind, LedgerEntry};
use crate::tension;
use crate::world::WorldState;

/// Prior strength, in pseudo-entries.
pub const PRIOR_WEIGHT: f64 = 3.0;
/// Turns for an entry's weight to halve.
pub const HALF_LIFE: f64 = 20.0;
/// Abandonments weigh this much more than honours ("slow to build, fast to lose").
pub const ABANDON_WEIGHT: f64 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepKind {
    /// "Will they defend / support me?"
    Back,
    /// "Will they follow through on threats?"
    Threat,
    /// "Do they enforce the principles they have acted on?"
    Norm,
}

impl RepKind {
    fn matches(self, kind: EntryKind) -> bool {
        matches!(
            (self, kind),
            (RepKind::Back, EntryKind::Back)
                | (RepKind::Threat, EntryKind::Threat)
                | (RepKind::Norm, EntryKind::Norm(_))
        )
    }
}

/// How relevant an entry is to an observer's own situation.
pub fn relevance(state: &WorldState, observer: CountryId, entry: &LedgerEntry) -> f64 {
    if entry.counterpart == observer {
        return 1.0;
    }
    if entry.local {
        return 0.0;
    }
    // Region comes with the map (step 9); alignment is available now.
    if state
        .country(observer)
        .shares_alignment(state.country(entry.counterpart))
    {
        0.5
    } else {
        0.2
    }
}

/// Weight of one entry for one observer (0 if unseen or ungraded).
pub fn entry_weight(state: &WorldState, observer: CountryId, entry: &LedgerEntry) -> f64 {
    if entry.actor == observer || !entry.seen_by.contains(observer) {
        return 0.0;
    }
    let Some(score) = entry.grade.and_then(|g| g.score()) else {
        return 0.0;
    };
    let age = (state.turn as i32 - entry.turn).max(0) as f64;
    let abandon = if score == 0.0 { ABANDON_WEIGHT } else { 1.0 };
    0.5f64.powf(age / HALF_LIFE) * relevance(state, observer, entry) * (1.0 + entry.cost_paid) * abandon * entry.weight
}

fn prior(state: &WorldState, actor: CountryId, kind: RepKind) -> f64 {
    let p = state.country(actor).priors;
    match kind {
        RepKind::Back => p.back,
        RepKind::Threat => p.threat,
        RepKind::Norm => p.norm,
    }
}

/// `Exp_k(E, P)`: what `observer` expects of `actor`, 0–1.
pub fn expectation(state: &WorldState, observer: CountryId, actor: CountryId, kind: RepKind) -> f64 {
    let mut num = PRIOR_WEIGHT * prior(state, actor, kind);
    let mut den = PRIOR_WEIGHT;
    for e in state
        .ledger
        .entries
        .iter()
        .filter(|e| e.actor == actor && kind.matches(e.kind))
    {
        let w = entry_weight(state, observer, e);
        if w > 0.0 {
            num += w * e.grade.and_then(|g| g.score()).unwrap_or(0.0);
            den += w;
        }
    }
    num / den
}

/// Credibility(E, P), 0–100, for the given kind.
pub fn credibility(state: &WorldState, observer: CountryId, actor: CountryId, kind: RepKind) -> f64 {
    100.0 * expectation(state, observer, actor, kind)
}

/// Trust(E, P), 0–100: only commitments where the observer was the counterpart.
pub fn trust(state: &WorldState, observer: CountryId, actor: CountryId) -> f64 {
    let mut num = PRIOR_WEIGHT * prior(state, actor, RepKind::Back);
    let mut den = PRIOR_WEIGHT;
    for e in state.ledger.entries.iter().filter(|e| {
        e.actor == actor && e.counterpart == observer && matches!(e.kind, EntryKind::Back | EntryKind::Threat)
    }) {
        let w = entry_weight(state, observer, e);
        if w > 0.0 {
            num += w * e.grade.and_then(|g| g.score()).unwrap_or(0.0);
            den += w;
        }
    }
    100.0 * num / den
}

/// Display-only power-weighted average over observers. **Not an AI input.**
pub fn global_credibility_display(state: &WorldState, actor: CountryId, kind: RepKind) -> f64 {
    let power = tension::power_shares(state);
    let (mut sum, mut weight) = (0.0, 0.0);
    for o in state.ids().filter(|&o| o != actor) {
        sum += power[o.index()] * credibility(state, o, actor, kind);
        weight += power[o.index()];
    }
    if weight > 0.0 {
        sum / weight
    } else {
        100.0 * prior(state, actor, kind)
    }
}

/// The `k` ledger entries that weigh most in `observer`'s view of `actor`
/// (for reasoning logs: "remembers: you cut support to X, 1986").
pub fn top_precedents(
    state: &WorldState,
    observer: CountryId,
    actor: CountryId,
    kind: RepKind,
    k: usize,
) -> Vec<(&LedgerEntry, f64)> {
    let mut v: Vec<_> = state
        .ledger
        .entries
        .iter()
        .filter(|e| e.actor == actor && kind.matches(e.kind))
        .map(|e| (e, entry_weight(state, observer, e)))
        .filter(|(_, w)| *w > 0.0)
        .collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.id.cmp(&b.0.id)));
    v.truncate(k);
    v
}
