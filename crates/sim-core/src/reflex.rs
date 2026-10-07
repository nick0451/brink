//! Reflexes: data rules for institutional habits (design/behaviour-and-voice.md §1).
//!
//! A reflex is `WHEN <generic conditions> THEN <labelled utility term> on
//! <decision>`, gated by state (personality ranges, government type,
//! support status), never by calendar date. Traits describe structural
//! characteristics; reflexes describe habits, so a changed government or
//! personality can stop behaving historically.
//!
//! This module holds data types and validation only. The AI evaluates
//! conditions against its observer view.

use serde::{Deserialize, Serialize};

use crate::country::Government;
use crate::diplomacy::TreatyKind;

/// Total reflex contribution per decision is scaled into ±this.
pub const REFLEX_CAP: f64 = 40.0;
/// Reflexes expressing ideology may not exceed this magnitude (keeps the
/// anti-ideology-lock safeguard intact).
pub const IDEOLOGICAL_CAP: f64 = 30.0;

/// Treaty family for proposal decisions; `Any` matches every kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalFamily {
    Any,
    Trade,
    NonAggression,
    DefensiveAlliance,
    Guarantee,
    Basing,
}

impl ProposalFamily {
    pub fn of(kind: TreatyKind) -> Self {
        match kind {
            TreatyKind::Trade { .. } => ProposalFamily::Trade,
            TreatyKind::NonAggression => ProposalFamily::NonAggression,
            TreatyKind::DefensiveAlliance => ProposalFamily::DefensiveAlliance,
            TreatyKind::Guarantee => ProposalFamily::Guarantee,
            TreatyKind::Basing => ProposalFamily::Basing,
        }
    }
}

/// The decision a reflex (or a decision record) concerns. Kinds without an
/// AI decision yet are accepted in data and stay dormant until their build
/// step adds the decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionKind {
    AcceptProposal(ProposalFamily),
    RequestGuarantee,
    JoinSanction,
    StayInAlliance,
    // Dormant until later build steps:
    ImposeSanction,
    StreamStart,
    StreamStop,
    /// Involvement band 0–4 (step 7).
    Band(u8),
    Withdraw,
    /// Weapons test (v0.2, P10).
    Test,
    /// Commodity production flood (v0.2, P5).
    Flood,
    /// Strategic goal selection (step 8).
    Goals,
    /// Military budget policy (step 8).
    Budget,
    /// Making a treaty proposal ourselves (step 8).
    Propose(ProposalFamily),
    /// Starting a covert arsenal programme (P10).
    Programme,
    /// Disclosing and dismantling an arsenal (P10).
    Dismantle,
    /// Choosing reform over crackdown in a transition crisis (P3).
    Reform,
    /// Selling arms abroad (D58).
    SellArms,
    /// The reserve-currency holder's monetary stance (P7).
    MonetaryStance,
    /// Keeping (or lifting) a sanction we impose (issue 15).
    KeepSanction,
    /// Forgiving (or holding) a war loan to a debtor in distress (issue 24).
    ForgiveDebt,
}

impl DecisionKind {
    /// Does a reflex written for `self` apply to the concrete decision `actual`?
    pub fn applies_to(self, actual: DecisionKind) -> bool {
        match (self, actual) {
            (DecisionKind::AcceptProposal(ProposalFamily::Any), DecisionKind::AcceptProposal(_)) => true,
            (a, b) => a == b,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trait {
    Aggression,
    Risk,
    Paranoia,
    Loyalty,
    Greed,
    Ideology,
    Opportunism,
}

/// Generic conditions, evaluated from the observer's view only. The
/// "counterpart" is the other party of the decision (proposer, patron,
/// sanctioner, alliance partner); the "secondary" is a third party (the
/// sanction target, or the threat behind a guarantee).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    CounterpartSharesAlignment,
    /// Both carry alignment tags and they differ.
    CounterpartRivalBloc,
    CounterpartNonAligned,
    /// Estimated power share (economy + military) ≥ 15%.
    CounterpartIsGreatPower,
    CounterpartGovernment(Government),
    /// Needs the stability band to be visible (coverage); false otherwise.
    CounterpartStabilityBelow(f64),
    CounterpartCredibilityBackBelow(f64),
    CounterpartCredibilityBackAtLeast(f64),
    CounterpartCredibilityThreatBelow(f64),
    CounterpartCredibilityThreatAtLeast(f64),
    CounterpartIsMyAlly,
    /// The counterpart runs a support stream to me.
    CounterpartFundsMe,
    /// The counterpart cut a support stream to me within N turns (ledger).
    CounterpartCutMySupportWithin(u32),
    CounterpartSanctionsMe,
    SecondarySharesAlignment,
    SecondaryRivalBloc,
    SecondaryIsMyAlly,
    MyThreatAtLeast(f64),
    MyThreatBelow(f64),
    GlobalTensionAtLeast(f64),
    MyDebtRatioAbove(f64),
    MyStabilityBelow(f64),
    /// Another country's forces are based on my soil.
    HostingForeignTroops,
    /// Support streams to me as a share of my revenue.
    SupportShareAtLeast(f64),
    /// A democracy currently sanctions me.
    SanctionedByDemocracy,
}

/// State gates: when the reflex is part of this institution's repertoire.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Gate {
    PersonalityAtLeast(Trait, f64),
    PersonalityBelow(Trait, f64),
    Government(Government),
    NotGovernment(Government),
    ReceivingSupport,
    NotReceivingSupport,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reflex {
    pub id: String,
    pub decision: DecisionKind,
    #[serde(default)]
    pub when: Vec<Condition>,
    /// −40..+40 (−30..+30 if `ideological`).
    pub weight: f64,
    /// In-world label; becomes the explanation line and the voice hook.
    pub label: String,
    #[serde(default)]
    pub active_if: Vec<Gate>,
    #[serde(default)]
    pub ideological: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflexSet {
    pub id: String,
    pub reflexes: Vec<Reflex>,
}

impl Reflex {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || self.label.trim().is_empty() {
            return Err("reflex needs an id and a label".into());
        }
        let cap = if self.ideological { IDEOLOGICAL_CAP } else { REFLEX_CAP };
        if !self.weight.is_finite() || self.weight.abs() > cap {
            return Err(format!("reflex {}: weight {} outside ±{cap}", self.id, self.weight));
        }
        for c in &self.when {
            let bad = match c {
                Condition::CounterpartStabilityBelow(v)
                | Condition::CounterpartCredibilityBackBelow(v)
                | Condition::CounterpartCredibilityBackAtLeast(v)
                | Condition::CounterpartCredibilityThreatBelow(v)
                | Condition::CounterpartCredibilityThreatAtLeast(v)
                | Condition::MyThreatAtLeast(v)
                | Condition::MyThreatBelow(v)
                | Condition::GlobalTensionAtLeast(v)
                | Condition::MyStabilityBelow(v) => !(0.0..=100.0).contains(v),
                Condition::MyDebtRatioAbove(v) | Condition::SupportShareAtLeast(v) => !v.is_finite() || *v < 0.0,
                _ => false,
            };
            if bad {
                return Err(format!("reflex {}: condition {c:?} out of range", self.id));
            }
        }
        for g in &self.active_if {
            if let Gate::PersonalityAtLeast(_, v) | Gate::PersonalityBelow(_, v) = g {
                if !(0.0..=1.0).contains(v) {
                    return Err(format!("reflex {}: gate {g:?} out of range", self.id));
                }
            }
        }
        Ok(())
    }
}
