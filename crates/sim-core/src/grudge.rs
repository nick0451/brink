//! Grudges fade unless renewed (D81).
//!
//! A scenario's starting hostility is a memory of what happened before the
//! campaign (the inner-German border, a border war, a hostage crisis), not a
//! standing grievance. It fades toward zero at [`FADE_PER_YEAR`]; a hostile
//! act between the pair pauses the fade for [`RENEWAL_TURNS`]; and while the
//! two states sit in opposing blocs it stops at [`BLOC_FLOOR`] (the floor
//! lifts when a regime leaves its bloc, by reform or a coup). Generic: it
//! reads only acts, alignment tags and the turn, never identity or dates.
//!
//! Hostile acts (public only: a covert act the victim does not know about
//! renews nothing until it is exposed):
//! - imposing or joining a sanction (the act, not the standing measure);
//! - a public denunciation (coercion, as the ledger records it);
//! - declaring war, or joining a war, against the other;
//! - arming the other's enemy while that enemy is at war with it: a one-off
//!   transfer, a new public arms stream, or a standing stream that becomes a
//!   proxy involvement when the war starts (ordinary peacetime sales renew
//!   nothing);
//! - an exposed covert arms stream to the other's enemy (subversion: the
//!   engine has no other covert action against a state);
//! - an exposed weapons programme renews every grudge held against the
//!   proliferator: a secret bomb is aimed at its historical enemies, and the
//!   exposure is public, so each of them learns of it at once;
//! - nuclear use;
//! - an arms race: a state spending more than [`BUILD_UP_MARGIN`] times
//!   its habitual military share (the D59 `military_norm`, unscaled) and
//!   not cutting renews its grudge with its top threat, both ways (the
//!   build-up answers that threat and is read by it as aimed at it). Top
//!   threat = the largest hostile pressure in the builder's
//!   `domestic::security_breakdown` (power x hostility x reach), the
//!   sim-side analogue of the AI's threat estimate. Renewed every turn the
//!   build-up lasts; read from canonical budget state (no AI reads it);
//! - economic warfare: a producer on Flood renews the grudge of a
//!   same-area net energy exporter whose debt is above the level where it
//!   drags growth ([`crate::economy::DEBT_DRAG_THRESHOLD`]): Iraq's 1990
//!   charge that Kuwait over-pumped while Iraq carried the debt of its war
//!   (wars are borrowed for: `economy::war_borrowing_rate`, issue 21).
//!   Renewed every turn both conditions hold.

use crate::commodity::ProductionPolicy;
use crate::diplomacy::{DiplomaticEvent, StreamKind};
use crate::ids::CountryId;
use crate::ledger::Visibility;
use crate::opinion::HostileAct;
use crate::world::WorldState;

/// Turns per year (the simulation is quarterly; see `WorldState::year`).
pub const TURNS_PER_YEAR: u32 = 4;
/// Opinion points a historical grudge recovers per year without renewal.
pub const FADE_PER_YEAR: f64 = 1.5;
/// A hostile act pauses the fade for two years.
pub const RENEWAL_TURNS: u32 = 2 * TURNS_PER_YEAR;
/// While the pair's blocs oppose, the grudge fades no further than this.
pub const BLOC_FLOOR: f64 = -10.0;

/// A build-up is spending over a fifth above habit: the Reagan build-up
/// raised the US defence share of GDP about 25% over its late-1970s level.
pub const BUILD_UP_MARGIN: f64 = 1.2;

/// Is `c` arming beyond its habit and not cutting back?
pub fn building_up(c: &crate::country::Country) -> bool {
    c.active
        && c.military_norm > 0.0
        && c.budget.military > BUILD_UP_MARGIN * c.military_norm
        && c.budget_target.military >= c.budget.military
}

/// Do `a` and `b` belong to opposing blocs (both aligned, different tags)?
/// The same public alignment tags `domestic::bloc_backing` reads.
pub fn blocs_oppose(state: &WorldState, a: CountryId, b: CountryId) -> bool {
    matches!(
        (&state.country(a).alignment, &state.country(b).alignment),
        (Some(x), Some(y)) if x != y
    )
}

/// The pairs this turn's events set against each other.
pub fn hostile_acts(state: &WorldState, events: &[DiplomaticEvent]) -> Vec<(CountryId, CountryId, HostileAct)> {
    let mut acts = Vec::new();
    let enemies = |c: CountryId| state.wars.enemies_of(c);
    for e in events {
        match *e {
            DiplomaticEvent::SanctionImposed { by, target } => acts.push((by, target, HostileAct::Sanction)),
            DiplomaticEvent::Denounced { by, target } => acts.push((by, target, HostileAct::Denunciation)),
            DiplomaticEvent::WarDeclared { attacker, defender, .. } => acts.push((attacker, defender, HostileAct::War)),
            DiplomaticEvent::JoinedWar { war, country, side, .. } => {
                if let Some(w) = state.wars.find(war) {
                    for b in w.side(side.other()) {
                        acts.push((country, b.country, HostileAct::War));
                    }
                }
            }
            DiplomaticEvent::ArmsTransferred { from, to, .. } => {
                for x in enemies(to) {
                    acts.push((from, x, HostileAct::ArmedEnemyAtWar));
                }
            }
            DiplomaticEvent::StreamStarted { stream } if stream.kind == StreamKind::Arms && !stream.covert => {
                for x in enemies(stream.to) {
                    acts.push((stream.from, x, HostileAct::ArmedEnemyAtWar));
                }
            }
            DiplomaticEvent::CovertExposed { from, to } => {
                for x in enemies(to) {
                    acts.push((from, x, HostileAct::CovertArmsExposed));
                }
            }
            DiplomaticEvent::ProgrammeExposed { country, .. } => {
                for x in state.ids().filter(|&x| x != country) {
                    acts.push((country, x, HostileAct::ProgrammeExposed));
                }
            }
            DiplomaticEvent::NuclearStrike { by, target, .. } => acts.push((by, target, HostileAct::NuclearStrike)),
            _ => {}
        }
    }
    // A standing arms stream that became a public proxy involvement this
    // turn (the recipient's war began): arming the enemy at war.
    for i in &state.wars.involvements {
        if i.band == 2 && i.since == state.turn && i.visibility != Visibility::Covert {
            if let Some(w) = state.wars.find(i.war) {
                if let Some(side) = w.side_of(i.beneficiary) {
                    for b in w.side(side.other()) {
                        acts.push((i.actor, b.country, HostileAct::ArmedEnemyAtWar));
                    }
                }
            }
        }
    }
    // Standing condition, renewed while it lasts: an arms build-up renews
    // the builder's grudge with its top threat.
    for c in state.countries.iter().filter(|c| building_up(c)) {
        let top = crate::domestic::security_breakdown(state, c.id)
            .hostile
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)));
        if let Some((x, _)) = top {
            acts.push((c.id, x, HostileAct::ArmsBuildUp));
        }
    }
    // Standing condition: a producer flooding the market against an
    // indebted exporting neighbour (economic warfare).
    for f in state
        .countries
        .iter()
        .filter(|c| c.active && c.area.is_some() && c.energy_policy == ProductionPolicy::Flood)
    {
        for v in state.countries.iter().filter(|v| {
            v.id != f.id
                && v.active
                && v.area == f.area
                && v.energy_net_exports > 0.0
                && v.debt_ratio() > crate::economy::DEBT_DRAG_THRESHOLD
        }) {
            acts.push((f.id, v.id, HostileAct::OilFlood));
        }
    }
    acts.retain(|(a, b, _)| a != b);
    acts
}

/// Tension phase (DESIGN §4.3 step 9): renew the grudges this turn's hostile
/// acts touched (both directions: the act sits between the pair), then fade
/// every historical grudge that is neither paused nor at its floor.
pub fn run(state: &mut WorldState, events: &[DiplomaticEvent]) {
    let turn = state.turn;
    for (a, b, act) in hostile_acts(state, events) {
        state.opinions.renew(a, b, turn, act);
        state.opinions.renew(b, a, turn, act);
    }
    let step = FADE_PER_YEAR / TURNS_PER_YEAR as f64;
    let ids: Vec<CountryId> = state.ids().collect();
    for &from in &ids {
        for &to in &ids {
            if from == to || state.opinions.modifiers(from, to).iter().all(|m| m.memory.is_none()) {
                continue;
            }
            let floor = if blocs_oppose(state, from, to) { BLOC_FLOOR } else { 0.0 };
            let cell = state.opinions.modifiers_mut(from, to);
            for m in cell.iter_mut() {
                let Some(mem) = m.memory else { continue };
                let paused = mem.renewed.is_some_and(|(t, _)| turn < t + RENEWAL_TURNS);
                if !paused && m.value < floor {
                    m.value = (m.value + step).min(floor);
                }
            }
            cell.retain(|m| m.memory.is_none() || m.value != 0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renewal_window_is_two_years() {
        assert_eq!(RENEWAL_TURNS, 8);
        assert!((FADE_PER_YEAR / TURNS_PER_YEAR as f64 - 0.375).abs() < 1e-12);
    }
}
