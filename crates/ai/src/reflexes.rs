//! Reflex evaluation (design/behaviour-and-voice.md §1). Reads only the
//! observer's view. Matching reflexes add labelled `Score` lines with term
//! `reflex:<id>`; their total is scaled into ±REFLEX_CAP so explanation
//! lines still sum exactly to the score.

use sim_core::reflex::{Condition, Gate, Trait, REFLEX_CAP};
use sim_core::view::{ForeignView, StabilityBand};
use sim_core::{DecisionKind, Government, ObserverView, Personality, TreatyKind};

use crate::inputs::{has_treaty, top_threat};
use crate::Score;

/// Estimated power share at or above which a country counts as a great power.
pub const GREAT_POWER_SHARE: f64 = 0.15;

/// Who the decision is about.
#[derive(Clone, Copy, Default)]
pub struct Parties<'a> {
    pub counterpart: Option<&'a ForeignView>,
    pub secondary: Option<&'a ForeignView>,
}

fn trait_value(p: &Personality, t: Trait) -> f64 {
    match t {
        Trait::Aggression => p.aggression,
        Trait::Risk => p.risk,
        Trait::Paranoia => p.paranoia,
        Trait::Loyalty => p.loyalty,
        Trait::Greed => p.greed,
        Trait::Ideology => p.ideology,
        Trait::Opportunism => p.opportunism,
    }
}

fn support_to_me(view: &ObserverView) -> f64 {
    view.streams
        .iter()
        .filter(|s| s.to == view.observer)
        .map(|s| s.value())
        .sum()
}

fn gate_open(view: &ObserverView, g: &Gate) -> bool {
    let own = &view.own;
    match g {
        Gate::PersonalityAtLeast(t, v) => trait_value(&own.personality, *t) >= *v,
        Gate::PersonalityBelow(t, v) => trait_value(&own.personality, *t) < *v,
        Gate::Government(gov) => own.government == *gov,
        Gate::NotGovernment(gov) => own.government != *gov,
        Gate::ReceivingSupport => support_to_me(view) > 0.0,
        Gate::NotReceivingSupport => support_to_me(view) == 0.0,
    }
}

/// Estimated power share of a foreign country (economy + military, half each).
pub fn power_share(view: &ObserverView, f: &ForeignView) -> f64 {
    let gdp_total: f64 = view.own.gdp + view.others.iter().map(|o| o.gdp).sum::<f64>();
    let mil_total: f64 = view.own.power() + view.others.iter().map(|o| o.military.value).sum::<f64>();
    0.5 * f.gdp / gdp_total.max(1e-9) + 0.5 * f.military.value / mil_total.max(1e-9)
}

fn band_value(b: StabilityBand) -> f64 {
    // Upper bound of each band, so "below X" is only claimed when certain.
    match b {
        StabilityBand::Collapse => 10.0,
        StabilityBand::Crisis => 25.0,
        StabilityBand::Unrest => 40.0,
        StabilityBand::Normal => 70.0,
        StabilityBand::Strong => 100.0,
    }
}

fn same_bloc(a: &Option<String>, b: &Option<String>) -> bool {
    matches!((a, b), (Some(x), Some(y)) if x == y)
}

fn rival_bloc(a: &Option<String>, b: &Option<String>) -> bool {
    matches!((a, b), (Some(x), Some(y)) if x != y)
}

fn condition_holds(view: &ObserverView, parties: Parties, c: &Condition) -> bool {
    let own = &view.own;
    let me = view.observer;
    let cp = parties.counterpart;
    let sec = parties.secondary;
    match c {
        Condition::CounterpartSharesAlignment => cp.is_some_and(|f| same_bloc(&own.alignment, &f.alignment)),
        Condition::CounterpartRivalBloc => cp.is_some_and(|f| rival_bloc(&own.alignment, &f.alignment)),
        Condition::CounterpartNonAligned => cp.is_some_and(|f| f.alignment.is_none()),
        Condition::CounterpartIsGreatPower => cp.is_some_and(|f| power_share(view, f) >= GREAT_POWER_SHARE),
        Condition::CounterpartGovernment(g) => cp.is_some_and(|f| f.government == *g),
        Condition::CounterpartStabilityBelow(v) => {
            cp.and_then(|f| f.stability_band).is_some_and(|b| band_value(b) <= *v)
        }
        Condition::CounterpartCredibilityBackBelow(v) => cp.is_some_and(|f| f.credibility_back < *v),
        Condition::CounterpartCredibilityBackAtLeast(v) => cp.is_some_and(|f| f.credibility_back >= *v),
        Condition::CounterpartCredibilityThreatBelow(v) => cp.is_some_and(|f| f.credibility_threat < *v),
        Condition::CounterpartCredibilityThreatAtLeast(v) => cp.is_some_and(|f| f.credibility_threat >= *v),
        Condition::CounterpartIsMyAlly => cp.is_some_and(|f| has_treaty(view, TreatyKind::DefensiveAlliance, me, f.id)),
        Condition::CounterpartFundsMe => cp.is_some_and(|f| view.streams.iter().any(|s| s.from == f.id && s.to == me)),
        Condition::CounterpartCutMySupportWithin(turns) => cp.is_some_and(|f| {
            view.ledger.iter().any(|e| {
                e.actor == f.id
                    && e.counterpart == me
                    && e.cause_code == sim_core::CauseCode::SupportCut
                    && e.turn >= view.turn as i32 - *turns as i32
            })
        }),
        Condition::CounterpartSanctionsMe => {
            cp.is_some_and(|f| view.sanctions.iter().any(|s| s.by == f.id && s.target == me))
        }
        Condition::SecondarySharesAlignment => sec.is_some_and(|f| same_bloc(&own.alignment, &f.alignment)),
        Condition::SecondaryRivalBloc => sec.is_some_and(|f| rival_bloc(&own.alignment, &f.alignment)),
        Condition::SecondaryIsMyAlly => sec.is_some_and(|f| has_treaty(view, TreatyKind::DefensiveAlliance, me, f.id)),
        Condition::MyThreatAtLeast(v) => top_threat(view).is_some_and(|(_, t)| t >= *v),
        Condition::MyThreatBelow(v) => top_threat(view).is_none_or(|(_, t)| t < *v),
        Condition::GlobalTensionAtLeast(v) => view.global_tension >= *v,
        Condition::MyDebtRatioAbove(v) => own.debt_ratio() > *v,
        Condition::MyStabilityBelow(v) => own.stability < *v,
        Condition::HostingForeignTroops => view.treaties.iter().any(|t| t.kind == TreatyKind::Basing && t.b == me),
        Condition::SupportShareAtLeast(v) => support_to_me(view) / (own.gdp * own.tax_rate).max(1e-9) >= *v,
        Condition::SanctionedByDemocracy => view.sanctions.iter().any(|s| {
            s.target == me
                && view
                    .others
                    .iter()
                    .any(|f| f.id == s.by && f.government == Government::Democracy)
        }),
    }
}

/// Add every active, matching reflex of the observer to `score`.
pub fn apply(view: &ObserverView, decision: DecisionKind, parties: Parties, score: &mut Score) {
    let fired: Vec<_> = view
        .own
        .reflexes
        .iter()
        .filter(|r| r.decision.applies_to(decision))
        .filter(|r| r.active_if.iter().all(|g| gate_open(view, g)))
        .filter(|r| r.when.iter().all(|c| condition_holds(view, parties, c)))
        .collect();
    let total: f64 = fired.iter().map(|r| r.weight).sum();
    let scale = if total.abs() > REFLEX_CAP {
        REFLEX_CAP / total.abs()
    } else {
        1.0
    };
    for r in fired {
        score.add_term(format!("reflex:{}", r.id), r.label.clone(), r.weight * scale);
    }
}
