//! Event templates (DESIGN principle 8, scenario-1980 P12): parameterised
//! families that fire from simulation conditions and feed back into the
//! systems. No flat random bonuses: every template has state conditions,
//! a per-turn chance once they hold, a cooldown, and effects on ordinary
//! state (Legitimacy, Stability, debt, tension, opinion, trade partners).
//!
//! Templates are scenario data. Engine code never names a country.

use serde::{Deserialize, Serialize};

use crate::country::{Government, Mobilization};
use crate::diplomacy::{DiplomaticEvent, TreatyKind};
use crate::ids::CountryId;
use crate::opinion::OpinionModifier;
use crate::world::WorldState;

fn default_cooldown() -> u32 {
    12
}

/// A condition on one country (the subject or object), or on the pair.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventCondition {
    StabilityBelow(f64),
    ProsperityBelow(f64),
    LegitimacyBelow(f64),
    DebtRatioAbove(f64),
    /// Last quarter's growth below this (e.g. -0.002).
    GrowthBelow(f64),
    Government(Government),
    NotGovernment(Government),
    OpennessAbove(f64),
    MobilizedAtLeast(Mobilization),
    ReadinessBelow(f64),
    AtWar,
    AtPeace,
    /// Power share (economic + military) at least this.
    PowerShareAtLeast(f64),
    // Pair conditions (object templates only; "other" is the subject).
    SameArea,
    TensionAbove(f64),
    /// Either side's opinion of the other at or below this.
    OpinionAtMost(f64),
    /// The object's power is below the subject's × this factor.
    WeakerThanSubject(f64),
    NoDefensivePact,
}

/// What a firing does. Subject = the country the template fired for;
/// object = the second country of a pair template.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventEffect {
    SubjectLegitimacy(f64),
    SubjectStability(f64),
    /// Temporary Prosperity penalty (decays 1 per turn).
    SubjectProsperityShock(f64),
    /// Share of debt written off (a default).
    SubjectDebtWriteDown(f64),
    /// Change in access to credit (0–1).
    SubjectFinancialWeight(f64),
    ObjectLegitimacy(f64),
    ObjectStability(f64),
    PairTension(f64),
    /// The object's opinion of the subject (decays 1/turn).
    ObjectOpinion(f64),
    /// The subject's opinion of the object (decays 1/turn).
    SubjectOpinion(f64),
    /// Prosperity shock to every trade-agreement partner, × its exposure
    /// (contagion).
    PartnersProsperityShock(f64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventTemplate {
    pub id: String,
    /// Plain title for logs (the voice layer may add a register later).
    pub title: String,
    pub subject: Vec<EventCondition>,
    /// Present for pair templates: conditions on the object.
    #[serde(default)]
    pub object: Option<Vec<EventCondition>>,
    /// Chance per turn once the conditions hold.
    pub chance: f64,
    #[serde(default = "default_cooldown")]
    pub cooldown: u32,
    pub effects: Vec<EventEffect>,
}

impl EventTemplate {
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.chance) {
            return Err(format!("event {}: chance must be 0–1", self.id));
        }
        let pairish = |c: &EventCondition| {
            matches!(
                c,
                EventCondition::SameArea
                    | EventCondition::TensionAbove(_)
                    | EventCondition::OpinionAtMost(_)
                    | EventCondition::WeakerThanSubject(_)
                    | EventCondition::NoDefensivePact
            )
        };
        if self.subject.iter().any(pairish) {
            return Err(format!("event {}: pair conditions belong on the object", self.id));
        }
        let object_effect = self.effects.iter().any(|e| {
            matches!(
                e,
                EventEffect::ObjectLegitimacy(_)
                    | EventEffect::ObjectStability(_)
                    | EventEffect::PairTension(_)
                    | EventEffect::ObjectOpinion(_)
                    | EventEffect::SubjectOpinion(_)
            )
        });
        if object_effect && self.object.is_none() {
            return Err(format!("event {}: object effects need object conditions", self.id));
        }
        Ok(())
    }
}

fn holds(state: &WorldState, c: &EventCondition, who: CountryId, other: Option<CountryId>, power: &[f64]) -> bool {
    let x = state.country(who);
    match *c {
        EventCondition::StabilityBelow(v) => x.stability < v,
        EventCondition::ProsperityBelow(v) => x.prosperity < v,
        EventCondition::LegitimacyBelow(v) => x.legitimacy < v,
        EventCondition::DebtRatioAbove(v) => x.debt_ratio() > v,
        EventCondition::GrowthBelow(v) => x.last_growth < v,
        EventCondition::Government(g) => x.government == g,
        EventCondition::NotGovernment(g) => x.government != g,
        EventCondition::OpennessAbove(v) => x.openness > v,
        EventCondition::MobilizedAtLeast(m) => x.forces.mobilization.level() >= m.level(),
        EventCondition::ReadinessBelow(v) => x.forces.readiness < v,
        EventCondition::AtWar => state.wars.is_belligerent(who),
        EventCondition::AtPeace => !state.wars.is_belligerent(who),
        EventCondition::PowerShareAtLeast(v) => power[who.index()] >= v,
        EventCondition::SameArea => {
            other.is_some_and(|o| matches!((&x.area, &state.country(o).area), (Some(a), Some(b)) if a == b))
        }
        EventCondition::TensionAbove(v) => other.is_some_and(|o| state.tension.get(who, o) > v),
        EventCondition::OpinionAtMost(v) => {
            other.is_some_and(|o| state.opinions.opinion(who, o) <= v || state.opinions.opinion(o, who) <= v)
        }
        EventCondition::WeakerThanSubject(k) => other.is_some_and(|o| x.power() < state.country(o).power() * k),
        EventCondition::NoDefensivePact => other.is_some_and(|o| {
            !state.diplomacy.has_treaty(TreatyKind::DefensiveAlliance, who, o)
                && !state.diplomacy.has_treaty(TreatyKind::NonAggression, who, o)
        }),
    }
}

fn modifier(source: &str, value: f64) -> OpinionModifier {
    OpinionModifier {
        source: source.into(),
        value,
        decay: 1.0,
        memory: None,
    }
}

fn apply(state: &mut WorldState, t: &EventTemplate, subject: CountryId, object: Option<CountryId>) {
    for e in &t.effects {
        match *e {
            EventEffect::SubjectLegitimacy(d) => {
                let c = state.country_mut(subject);
                c.legitimacy = (c.legitimacy + d).clamp(0.0, 100.0);
            }
            EventEffect::SubjectStability(d) => {
                let c = state.country_mut(subject);
                c.stability = (c.stability + d).clamp(0.0, 100.0);
            }
            EventEffect::SubjectProsperityShock(d) => state.country_mut(subject).adjustment_shock += d.max(0.0),
            EventEffect::SubjectDebtWriteDown(f) => {
                let c = state.country_mut(subject);
                c.debt *= 1.0 - f.clamp(0.0, 1.0);
            }
            EventEffect::SubjectFinancialWeight(d) => {
                let c = state.country_mut(subject);
                c.financial_weight = (c.financial_weight + d).clamp(0.0, 1.0);
            }
            EventEffect::PartnersProsperityShock(d) => {
                let partners: Vec<CountryId> = state
                    .diplomacy
                    .treaties
                    .iter()
                    .filter(|tr| matches!(tr.kind, TreatyKind::Trade { .. }) && tr.involves(subject))
                    .filter_map(|tr| tr.other(subject))
                    .collect();
                for p in partners {
                    let c = state.country_mut(p);
                    c.adjustment_shock += d.max(0.0) * c.trade_exposure;
                }
            }
            _ => {}
        }
        if let Some(o) = object {
            match *e {
                EventEffect::ObjectLegitimacy(d) => {
                    let c = state.country_mut(o);
                    c.legitimacy = (c.legitimacy + d).clamp(0.0, 100.0);
                }
                EventEffect::ObjectStability(d) => {
                    let c = state.country_mut(o);
                    c.stability = (c.stability + d).clamp(0.0, 100.0);
                }
                EventEffect::PairTension(d) => state.tension.add(subject, o, d),
                EventEffect::ObjectOpinion(d) => state.opinions.add(o, subject, modifier(&t.title, d)),
                EventEffect::SubjectOpinion(d) => state.opinions.add(subject, o, modifier(&t.title, d)),
                _ => {}
            }
        }
    }
}

/// Events phase (DESIGN §4.3): every template is checked for every subject
/// (and object) in id order; once its conditions hold it fires with its
/// chance, then rests for its cooldown.
pub fn run(state: &mut WorldState, events: &mut Vec<DiplomaticEvent>) {
    if state.event_templates.is_empty() {
        return;
    }
    let templates = state.event_templates.clone();
    let power = crate::tension::power_shares(state);
    let ids: Vec<CountryId> = state.ids().filter(|&c| state.country(c).active).collect();
    let turn = state.turn;
    for (ti, t) in templates.iter().enumerate() {
        for &s in &ids {
            if !t.subject.iter().all(|c| holds(state, c, s, None, &power)) {
                continue;
            }
            let objects: Vec<Option<CountryId>> = match &t.object {
                None => vec![None],
                Some(conds) => ids
                    .iter()
                    .filter(|&&o| o != s && conds.iter().all(|c| holds(state, c, o, Some(s), &power)))
                    .map(|&o| Some(o))
                    .collect(),
            };
            for o in objects {
                let key = (ti as u16, s, o);
                let resting = state
                    .event_cooldowns
                    .iter()
                    .any(|&(a, b, c, until)| (a, b, c) == key && turn < until);
                if resting {
                    continue;
                }
                if state.roll_unit() >= t.chance {
                    continue;
                }
                state.event_cooldowns.retain(|&(a, b, c, _)| (a, b, c) != key);
                state.event_cooldowns.push((key.0, key.1, key.2, turn + t.cooldown));
                apply(state, t, s, o);
                events.push(DiplomaticEvent::ScenarioEvent {
                    template: ti as u16,
                    subject: s,
                    object: o,
                });
            }
        }
    }
}
