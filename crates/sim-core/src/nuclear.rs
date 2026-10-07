//! Nuclear weapons, v0.1 trim (DESIGN §11.4, D14): arsenal level and strike
//! capacity, the deterrence inputs, one use action (a limited strike on a
//! war's front) and the penalty package. Deferred: strike scales, postures,
//! detailed second strike, the Nuclear Winter meter.
//!
//! Nothing here is scripted by country: any state with an arsenal can use
//! it, and every penalty lands through ordinary state (opinion, the ledger,
//! legitimacy, tension, the energy market).

use crate::country::Government;
use crate::diplomacy::DiplomaticEvent;
use crate::ids::CountryId;
use crate::ledger::{self, CauseCode, EntryKind, LedgerLog, Visibility};
use crate::opinion::OpinionModifier;
use crate::orders::Order;
use crate::war::Side;
use crate::world::WorldState;

/// Strike capacity per arsenal level (full capacity).
pub const CAPACITY_PER_LEVEL: f64 = 2.0;
/// Strike capacity regained per turn.
pub const REGENERATION: f64 = 0.1;
/// Share of the enemy's committed strength destroyed by a limited strike.
pub const STRIKE_DESTRUCTION: f64 = 0.6;
/// Front progress gained by the striking side.
pub const STRIKE_PROGRESS: f64 = 40.0;
/// Permanent opinion every other country holds of a nuclear user.
pub const PARIAH_OPINION: f64 = -80.0;
/// Weight of the ledger entry a use writes (it dominates every reading).
pub const USE_WEIGHT: f64 = 5.0;
/// Energy price shock on any use (markets panic).
pub const PRICE_SHOCK: f64 = 1.5;

/// Full strike capacity for an arsenal level.
pub fn full_capacity(level: u8) -> f64 {
    level as f64 * CAPACITY_PER_LEVEL
}

/// Validate and resolve a nuclear strike order (orders phase): the user's
/// war, an enemy belligerent, at least one strike of capacity.
pub fn apply(state: &mut WorldState, actor: CountryId, order: Order) -> Result<Option<DiplomaticEvent>, String> {
    let Order::NuclearStrike { target } = order else {
        return Err("not a nuclear order".into());
    };
    let n = state.countries.len();
    if target.index() >= n || target == actor {
        return Err("invalid nuclear target".into());
    }
    let user = state.country(actor);
    if user.arsenal == 0 || user.strike_capacity < 1.0 {
        return Err("no strike capacity".into());
    }
    // One use action: a limited strike against forces you are at war with.
    let Some(war) = state
        .wars
        .active
        .iter()
        .find(|w| matches!((w.side_of(actor), w.side_of(target)), (Some(a), Some(b)) if a != b))
        .cloned()
    else {
        return Err("a limited strike needs a war with the target".into());
    };
    state.country_mut(actor).strike_capacity -= 1.0;
    // Use reveals any undeclared arsenal.
    state.country_mut(actor).arsenal_declared = true;
    state.country_mut(actor).programme_exposed = true;

    // Direct effect: most of the target side's committed strength on the
    // front is destroyed; the striking side gains ground.
    let side = war.side_of(actor).expect("belligerent");
    for b in war.side(side.other()) {
        let lost = state.country(b.country).forces.strength() * b.allocation.min(1.0) * STRIKE_DESTRUCTION;
        crate::war::destroy(state, b.country, lost);
    }
    if let Some(w) = state.wars.active.iter_mut().find(|w| w.id == war.id) {
        let delta = if side == Side::Attacker {
            STRIKE_PROGRESS
        } else {
            -STRIKE_PROGRESS
        };
        for f in &mut w.fronts {
            f.progress = (f.progress + delta).clamp(-100.0, 100.0);
        }
    }
    let t = state.country_mut(target);
    t.population *= 0.95;
    t.gdp *= 0.9;
    t.stability = (t.stability - 15.0).max(0.0);

    penalties(state, actor);
    Ok(Some(DiplomaticEvent::NuclearStrike {
        by: actor,
        target,
        war: war.id,
    }))
}

/// The penalty package (DESIGN §11.4): pariah status, domestic shock,
/// tension to the brink, a market panic. Ledger, allies and coalition
/// effects follow from [`process_ledger`] and the AI's own reading.
fn penalties(state: &mut WorldState, user: CountryId) {
    let ids: Vec<CountryId> = state.ids().filter(|&c| c != user).collect();
    for &o in &ids {
        state.opinions.add(
            o,
            user,
            OpinionModifier {
                source: "used nuclear weapons".into(),
                value: PARIAH_OPINION,
                decay: 0.0,
                memory: None,
            },
        );
        state.tension.add(user, o, 50.0);
    }
    let c = state.country_mut(user);
    let (legit, stab) = match c.government {
        Government::Democracy => (40.0, 25.0),
        _ => (15.0, 10.0),
    };
    c.legitimacy = (c.legitimacy - legit).max(0.0);
    c.stability = (c.stability - stab).max(0.0);
    state.energy.price *= PRICE_SHOCK;
}

/// Ledger effects of a use: a Coercion entry of overwhelming weight, seen by
/// everyone. Observers' Trust and Credibility readings follow from it.
pub fn process_ledger(state: &mut WorldState, events: &[DiplomaticEvent], log: &mut Vec<LedgerLog>) {
    for e in events {
        if let DiplomaticEvent::NuclearStrike { by, target, .. } = *e {
            ledger::record(
                state,
                by,
                target,
                EntryKind::Coercion,
                None,
                1.0,
                USE_WEIGHT,
                false,
                Visibility::Public,
                CauseCode::NuclearStrike,
                "used nuclear weapons",
                log,
            );
        }
    }
}

/// Strike capacity regenerates slowly up to the arsenal's full capacity.
pub fn regenerate(state: &mut WorldState) {
    for c in &mut state.countries {
        c.strike_capacity = (c.strike_capacity + REGENERATION).min(full_capacity(c.arsenal));
    }
}

/// Has `c` ever used nuclear weapons (public record)?
pub fn has_used(state: &WorldState, c: CountryId) -> bool {
    state
        .ledger
        .entries
        .iter()
        .any(|e| e.actor == c && e.cause_code == CauseCode::NuclearStrike)
}
