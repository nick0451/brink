//! Regime transition (scenario P3, v0.2; user approval item 5: a managed
//! transition is not a regime collapse).
//!
//! - **Transition crisis:** a non-democratic regime with Stability below
//!   [`CRISIS_STABILITY`] and Legitimacy below [`CRISIS_LEGITIMACY`] for
//!   [`CRISIS_TURNS`] turns in a row must choose: Reform or Crackdown.
//!   Thresholds are state, never dates.
//! - **Reform:** the government becomes a democracy, its personality moves to
//!   its reform vector (data) or a generic moderation, Legitimacy resets, and
//!   a regime whose ideology fades leaves its bloc. The country keeps
//!   playing.
//! - **Crackdown:** Stability now, Legitimacy and opinion later; every
//!   crackdown raises the per-turn chance of a hardliner coup while the
//!   crisis lasts.
//! - **Coup:** new rulers, same state. The coup resets the coup-risk count
//!   (the new junta's own crackdowns) but not the era's record of failed
//!   repression ([`TransitionState::repressions`]): the institution that
//!   seized power knows the old answer has already failed. The record clears
//!   only after a long calm ([`CALM_RESET`] turns out of crisis).
//! - **Collapse:** Stability below [`COLLAPSE_STABILITY`] forces a disorderly
//!   transition. For the player that is game over (DESIGN); AI countries
//!   continue under the new government.

use serde::{Deserialize, Serialize};

use crate::country::{Government, Personality};
use crate::diplomacy::DiplomaticEvent;
use crate::ids::CountryId;
use crate::opinion::OpinionModifier;
use crate::orders::Order;
use crate::world::WorldState;

pub const CRISIS_STABILITY: f64 = 35.0;
pub const CRISIS_LEGITIMACY: f64 = 40.0;
pub const CRISIS_TURNS: u32 = 6;
pub const COLLAPSE_STABILITY: f64 = 10.0;
/// Turns to answer a transition crisis before the regime defaults to a
/// crackdown (the status quo).
pub const ANSWER_TURNS: u32 = 2;
/// Coup chance per turn per crackdown while the crisis lasts.
pub const COUP_CHANCE: f64 = 0.05;
/// Below this ideology a reformed regime drops its bloc tag.
pub const BLOC_IDEOLOGY: f64 = 0.5;
/// Turns after any transition before a new crisis can be declared or the
/// regime can collapse again (repression and new governments buy time).
pub const REPRIEVE: u32 = 12;
/// Turns out of crisis conditions after which a regime's record of failed
/// repression is forgotten (a new era, not a lull).
pub const CALM_RESET: u32 = 2 * REPRIEVE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionKind {
    Reform,
    Crackdown,
    /// Hardliners seize power after repeated crackdowns.
    Coup,
    /// Stability collapsed: a disorderly transition.
    Collapse,
}

/// Per-country transition state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TransitionState {
    /// Consecutive turns in crisis conditions.
    pub crisis_turns: u32,
    /// A transition choice is pending since this turn.
    pub pending_since: Option<u32>,
    /// Crackdowns since the current rulers took power (drives coup risk).
    pub crackdowns: u32,
    /// Crackdowns in this crisis era, surviving coups: the record of failed
    /// repression: what the regime weighs when deciding whether repression
    /// can still work.
    #[serde(default)]
    pub repressions: u32,
    /// Consecutive turns out of crisis conditions (clears `repressions`).
    #[serde(default)]
    pub calm_turns: u32,
    /// No new crisis offer or collapse before this turn.
    #[serde(default)]
    pub quiet_until: u32,
    /// Turn of the last reform (opens the lid on secession, P2).
    #[serde(default)]
    pub reformed_turn: Option<u32>,
    /// Personality after a reform (data; e.g. the B.18 transition vectors).
    pub reform_personality: Option<Personality>,
    /// Turn the current rulers took power by revolution (a collapse) or were
    /// installed by a conqueror (negative = before the scenario starts).
    /// `None` = an established regime, or one the army itself seized (a
    /// coup purges no one). Drives [`command`].
    #[serde(default)]
    pub took_power: Option<i32>,
}

/// Turns for a purged officer corps to be rebuilt after a revolution or a
/// conquest (3 years: Iran's army, purged in 1979, broke the Iraqi
/// offensive by 1982; the Red Army purged in 1937-38 failed in Finland in
/// 1939-40 and was rebuilt by 1942).
pub const PURGE_RECOVERY: f64 = 12.0;

/// Command cohesion, 0-1: the share of the readiness it pays for that an
/// army can reach under its rulers. An army fights for a regime it trusts
/// under officers its rulers trust. In a crisis (Stability below
/// [`CRISIS_STABILITY`]) desertion and broken command lower it in proportion
/// (Russia 1917); rulers brought by a revolution or installed by a
/// conqueror purge the officer corps they inherited, and cohesion climbs
/// back over [`PURGE_RECOVERY`] turns (Iran 1979-82). A military coup
/// purges no one: the army was the coup (Turkey 1980, Argentina 1976,
/// Pakistan 1977, Korea 1979). The lower term binds. At 0 the readiness ceiling
/// is 0 and the force fights at half value (the readiness floor of D31):
/// an army that is half there.
pub fn command(c: &crate::country::Country, turn: u32) -> f64 {
    let crisis = (c.stability / CRISIS_STABILITY).clamp(0.0, 1.0);
    let purge = c
        .transition
        .took_power
        .map_or(1.0, |t| ((turn as i32 - t) as f64 / PURGE_RECOVERY).clamp(0.0, 1.0));
    crisis.min(purge)
}

fn generic_reform(p: Personality) -> Personality {
    Personality {
        aggression: p.aggression * 0.6,
        risk: p.risk * 0.8,
        paranoia: p.paranoia * 0.7,
        loyalty: p.loyalty,
        greed: (p.greed + 0.15).min(1.0),
        ideology: p.ideology * 0.6,
        opportunism: p.opportunism,
    }
}

fn hardliner(p: Personality) -> Personality {
    Personality {
        aggression: (p.aggression + 0.1).min(1.0),
        paranoia: (p.paranoia + 0.1).min(1.0),
        ideology: (p.ideology + 0.1).min(1.0),
        ..p
    }
}

fn democracies_react(state: &mut WorldState, c: CountryId, value: f64, source: &str) {
    let ids: Vec<CountryId> = state
        .ids()
        .filter(|&o| o != c && state.country(o).government == Government::Democracy)
        .collect();
    for o in ids {
        state.opinions.add(
            o,
            c,
            OpinionModifier {
                source: source.into(),
                value,
                decay: 0.5,
                memory: None,
            },
        );
    }
}

/// A reformed regime's habitual military share, relative to the old one's.
pub const REFORM_MILITARY_NORM: f64 = 0.7;

fn reform(state: &mut WorldState, c: CountryId) {
    let target = state.country(c).transition.reform_personality;
    let x = state.country_mut(c);
    x.government = Government::Democracy;
    x.personality = target.unwrap_or_else(|| generic_reform(x.personality));
    x.openness = x.openness.max(Government::Democracy.default_openness());
    x.legitimacy = 55.0;
    x.stability = (x.stability + 10.0).min(100.0);
    if x.personality.ideology < BLOC_IDEOLOGY {
        x.alignment = None;
    }
    // Market reform ends the command economy's drag, and the new regime
    // inherits a smaller army as its habit (Argentina 1983, South Africa
    // 1990s, Russia): the old doctrine's share was the old regime's.
    x.command_drag = 0.0;
    x.military_norm *= REFORM_MILITARY_NORM;
    let turn = state.turn;
    let x = state.country_mut(c);
    // A negotiated reform purges no one; a purge already under way runs on.
    let took_power = x.transition.took_power;
    x.transition = TransitionState {
        reform_personality: None,
        quiet_until: turn + REPRIEVE,
        reformed_turn: Some(turn),
        took_power,
        ..Default::default()
    };
    democracies_react(state, c, 20.0, "reformed its government");
}

fn crackdown(state: &mut WorldState, c: CountryId) {
    let x = state.country_mut(c);
    x.stability = (x.stability + 15.0).min(100.0);
    x.legitimacy = (x.legitimacy - 10.0).max(0.0);
    x.transition.crackdowns += 1;
    x.transition.repressions += 1;
    x.transition.pending_since = None;
    x.transition.crisis_turns = 0;
    let turn = state.turn;
    state.country_mut(c).transition.quiet_until = turn + REPRIEVE;
    democracies_react(state, c, -15.0, "crushed its opposition");
}

fn coup(state: &mut WorldState, c: CountryId) {
    let x = state.country_mut(c);
    x.government = Government::Authoritarian;
    x.personality = hardliner(x.personality);
    x.legitimacy = 30.0;
    x.stability = (x.stability - 10.0).max(0.0);
    x.transition.crackdowns = 0;
    x.transition.crisis_turns = 0;
    x.transition.pending_since = None;
    let turn = state.turn;
    state.country_mut(c).transition.quiet_until = turn + REPRIEVE;
}

/// Orders: answer a pending transition crisis.
pub fn apply(state: &mut WorldState, actor: CountryId, order: Order) -> Result<Option<DiplomaticEvent>, String> {
    if state.country(actor).transition.pending_since.is_none() {
        return Err("no transition crisis to answer".into());
    }
    let kind = match order {
        Order::Reform => {
            reform(state, actor);
            TransitionKind::Reform
        }
        Order::Crackdown => {
            crackdown(state, actor);
            TransitionKind::Crackdown
        }
        _ => return Err("not a transition order".into()),
    };
    Ok(Some(DiplomaticEvent::Transition { country: actor, kind }))
}

/// Transition phase (after domestic): crisis counting, offers, defaults,
/// coups and collapse.
pub fn run(state: &mut WorldState, events: &mut Vec<DiplomaticEvent>) {
    let turn = state.turn;
    let ids: Vec<CountryId> = state.ids().collect();
    for c in ids {
        let x = state.country(c);
        if !x.active || turn < x.transition.quiet_until {
            continue;
        }
        // Collapse: a disorderly transition, any government.
        if x.stability < COLLAPSE_STABILITY {
            let democratic = x.government == Government::Democracy;
            if democratic {
                coup(state, c);
            } else {
                reform(state, c);
                let y = state.country_mut(c);
                y.stability = 30.0;
                y.legitimacy = 35.0;
            }
            state.country_mut(c).stability = state.country(c).stability.max(35.0);
            // A disorderly transition is a revolution: the new rulers purge
            // the officer corps (a coup by the army itself does not).
            state.country_mut(c).transition.took_power = Some(turn as i32);
            events.push(DiplomaticEvent::Transition {
                country: c,
                kind: TransitionKind::Collapse,
            });
            continue;
        }
        if x.government == Government::Democracy {
            continue;
        }
        let in_crisis = x.stability < CRISIS_STABILITY && x.legitimacy < CRISIS_LEGITIMACY;
        let crackdowns = x.transition.crackdowns;
        let pending = x.transition.pending_since;
        {
            let t = &mut state.country_mut(c).transition;
            t.crisis_turns = if in_crisis { t.crisis_turns + 1 } else { 0 };
            t.calm_turns = if in_crisis { 0 } else { t.calm_turns + 1 };
            if t.calm_turns >= CALM_RESET {
                t.repressions = 0;
            }
        }
        // Coup risk grows with every crackdown while the crisis lasts.
        if in_crisis && crackdowns > 0 && state.roll_unit() < COUP_CHANCE * crackdowns as f64 {
            coup(state, c);
            events.push(DiplomaticEvent::Transition {
                country: c,
                kind: TransitionKind::Coup,
            });
            continue;
        }
        match pending {
            Some(since) if turn >= since + ANSWER_TURNS => {
                crackdown(state, c);
                events.push(DiplomaticEvent::Transition {
                    country: c,
                    kind: TransitionKind::Crackdown,
                });
            }
            None if state.country(c).transition.crisis_turns >= CRISIS_TURNS => {
                state.country_mut(c).transition.pending_since = Some(turn);
                events.push(DiplomaticEvent::TransitionCrisis { country: c });
            }
            _ => {}
        }
    }
}
