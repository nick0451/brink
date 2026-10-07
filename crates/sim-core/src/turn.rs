//! Turn resolution in the fixed phase order of DESIGN §4.3. Phases without
//! systems yet (intelligence ops, movement, combat, research, events,
//! ledger) are added by later build steps in their documented positions.

use serde::{Deserialize, Serialize};

use crate::diplomacy::{self, DiplomaticEvent};
use crate::economy::MAX_DEFICIT_RATIO;
use crate::ids::CountryId;
use crate::initiative;
use crate::ledger::LedgerLog;
use crate::orders::{Order, OrderSet};
use crate::world::WorldState;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TurnReport {
    /// The turn number that was resolved.
    pub turn: u32,
    pub events: Vec<DiplomaticEvent>,
    /// Ledger development log (entries, tests, norms).
    pub ledger_log: Vec<LedgerLog>,
    pub rejected: Vec<(CountryId, Order, String)>,
}

/// Resolve one turn. Orders from all countries were planned against the same
/// start-of-turn state (simultaneous turns); they are applied in `CountryId`
/// order so the result does not depend on submission order.
pub fn resolve_turn(state: &mut WorldState, mut orders: Vec<OrderSet>) -> TurnReport {
    let mut report = TurnReport {
        turn: state.turn,
        events: Vec::new(),
        ledger_log: Vec::new(),
        rejected: Vec::new(),
    };
    orders.sort_by_key(|o| o.country);

    // 0–1. Orders lock; standing settings and diplomacy.
    for set in orders {
        let actor = set.country;
        if actor.index() < state.countries.len() && !state.country(actor).active {
            continue;
        }
        if actor.index() >= state.countries.len() {
            for order in set.orders {
                report.rejected.push((actor, order, "unknown country".into()));
            }
            continue;
        }
        for order in set.orders {
            match order {
                Order::SetBudget(shares) => state.country_mut(actor).budget_target = shares.normalized(),
                Order::SetEnergyPolicy(p) => {
                    let before = state.country(actor).energy_policy;
                    if before != p {
                        state.country_mut(actor).energy_policy = p;
                        report.events.push(crate::diplomacy::DiplomaticEvent::EnergyPolicy {
                            country: actor,
                            policy: p,
                        });
                    }
                }
                Order::SetMonetaryStance(stance) => {
                    if crate::money::holder(state) != Some(actor) {
                        report
                            .rejected
                            .push((actor, order, "only the reserve-currency holder sets the stance".into()));
                    } else if state.money.stance != stance {
                        state.money.stance = stance;
                        report
                            .events
                            .push(crate::diplomacy::DiplomaticEvent::MonetaryStance { by: actor, stance });
                    }
                }
                Order::SetDeficit(ratio) => {
                    state.country_mut(actor).deficit_ratio = if ratio.is_finite() {
                        ratio.clamp(0.0, MAX_DEFICIT_RATIO)
                    } else {
                        0.0
                    }
                }
                action => {
                    let cost = diplomacy::initiative_cost(state, actor, &action);
                    if state.country(actor).initiative.available() < cost {
                        report.rejected.push((actor, action, "not enough Initiative".into()));
                        continue;
                    }
                    match diplomacy::apply(state, actor, action.clone()) {
                        Ok(event) => {
                            state.country_mut(actor).initiative.spend(cost);
                            report.events.extend(event);
                        }
                        Err(reason) => report.rejected.push((actor, action, reason)),
                    }
                }
            }
        }
    }
    report.events.extend(diplomacy::lapse_proposals(state));

    // 4. Combat: covert exposure, fronts, wars that end.
    crate::war::resolve(state, &mut report.events);

    // 5. Economy (trade, aid, growth).
    crate::economy::run(state);
    // 7. Domestic, then regime transitions.
    crate::domestic::run(state);
    crate::transition::run(state, &mut report.events);
    crate::region::run(state, &mut report.events);
    // 8. Events: templates that read state; covert programmes advance and
    // may be exposed.
    crate::events::run(state, &mut report.events);
    crate::programme::advance(state, &mut report.events);
    crate::programme::expose(state, &mut report.events);
    // 9. Tension & ledger: opinion decay, tension drift, then the Event
    // Ledger (entries, commitment tests, third-party reactions).
    state.opinions.decay();
    crate::tension::update(state);
    let events = report.events.clone();
    crate::ledger::process(state, &events, &mut report.ledger_log);
    crate::war::process_ledger(state, &events, &mut report.ledger_log);
    crate::nuclear::process_ledger(state, &events, &mut report.ledger_log);
    crate::programme::process_ledger(state, &events, &mut report.ledger_log);
    // Historical grudges: renewed by this turn's hostile acts, else fading
    // (D81). After the ledger so this turn's proxy involvements are known.
    crate::grudge::run(state, &events);
    crate::nuclear::regenerate(state);

    // Initiative for the next planning phase.
    for c in &mut state.countries {
        c.initiative.end_turn();
        c.initiative
            .start_turn(initiative::allowance(c.initiative_base, c.stability));
    }

    state.turn += 1;
    report
}
