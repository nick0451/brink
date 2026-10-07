//! Region loyalty and secession (scenario P2, v0.2), in its smallest form:
//! secession-prone countries carry a few peripheries as data (share of
//! population and GDP, loyalty, an optional successor). Successor states
//! exist from the start as dormant countries, so the country count never
//! changes mid-game. The full region map comes later.
//!
//! - Loyalty drifts toward a target set by the parent's stability,
//!   legitimacy and welfare spending, minus recent repression.
//! - Low loyalty drags on the parent (unrest).
//! - A region below [`SECESSION_LOYALTY`] may secede while its parent is in
//!   crisis, at war, or newly reformed ("opening the lid"): its successor
//!   activates with its share of population, GDP and some forces.
//!
//! Loyalty is political, never ethnic: nothing here models peoples.

use serde::{Deserialize, Serialize};

use crate::country::Forces;
use crate::diplomacy::DiplomaticEvent;
use crate::ids::CountryId;
use crate::opinion::OpinionModifier;
use crate::world::WorldState;

/// Share of the gap to target loyalty closed each turn.
const LOYALTY_RATE: f64 = 0.1;
/// Below this, a region drags on its parent.
pub const UNREST_LOYALTY: f64 = 40.0;
/// Below this, a region may secede when its parent is vulnerable.
pub const SECESSION_LOYALTY: f64 = 25.0;
/// Chance per turn of secession once the conditions hold.
pub const SECESSION_CHANCE: f64 = 0.2;
/// Turns after a reform during which the lid is open.
pub const OPEN_LID_TURNS: u32 = 12;
/// Loyalty lost per crackdown in every periphery.
pub const CRACKDOWN_LOYALTY: f64 = 10.0;
/// Share of a region's proportional forces the successor takes with it.
const FORCES_TAKEN: f64 = 0.5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub pop_share: f64,
    pub gdp_share: f64,
    pub loyalty: f64,
    /// Data offset on the loyalty target (history, distance from the centre).
    #[serde(default)]
    pub base: f64,
    /// Code of the dormant country that activates on secession.
    #[serde(default)]
    pub successor: Option<String>,
    #[serde(default)]
    pub seceded: bool,
}

/// Target loyalty for a parent in this state.
pub fn target(stability: f64, legitimacy: f64, welfare: f64, crackdowns: u32, base: f64) -> f64 {
    (30.0 + 0.4 * stability + 0.2 * legitimacy + 40.0 * (welfare - 0.3) - 10.0 * crackdowns as f64 + base)
        .clamp(0.0, 100.0)
}

/// Loyalty drift and unrest (after domestic).
pub fn run(state: &mut WorldState, events: &mut Vec<DiplomaticEvent>) {
    let turn = state.turn;
    let ids: Vec<CountryId> = state.ids().collect();
    for p in ids {
        if state.country(p).regions.is_empty() || !state.country(p).active {
            continue;
        }
        let (stab, legit, welfare, crackdowns) = {
            let c = state.country(p);
            (c.stability, c.legitimacy, c.budget.welfare, c.transition.crackdowns)
        };
        let mut drag = 0.0;
        for r in state.country_mut(p).regions.iter_mut().filter(|r| !r.seceded) {
            let t = target(stab, legit, welfare, crackdowns, r.base);
            r.loyalty += (t - r.loyalty) * LOYALTY_RATE;
            if r.loyalty < UNREST_LOYALTY {
                drag += (UNREST_LOYALTY - r.loyalty) * r.pop_share * 0.2;
            }
        }
        let c = state.country_mut(p);
        c.stability = (c.stability - drag).max(0.0);

        // Secession: low loyalty and a vulnerable centre.
        let vulnerable = c.stability < crate::transition::CRISIS_STABILITY
            || c.transition.reformed_turn.is_some_and(|t| turn < t + OPEN_LID_TURNS)
            || state.wars.is_belligerent(p);
        if !vulnerable {
            continue;
        }
        let candidates: Vec<usize> = state
            .country(p)
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.seceded && r.loyalty < SECESSION_LOYALTY && r.successor.is_some())
            .map(|(i, _)| i)
            .collect();
        for i in candidates {
            if state.roll_unit() >= SECESSION_CHANCE {
                continue;
            }
            let code = state.country(p).regions[i].successor.clone().expect("successor");
            let Some(s) = state.find(&code) else { continue };
            secede(state, p, i, s);
            events.push(DiplomaticEvent::Secession {
                parent: p,
                successor: s,
            });
        }
    }
}

fn secede(state: &mut WorldState, p: CountryId, i: usize, s: CountryId) {
    let (pop, gdp, strength, mix, tech) = {
        let c = state.country_mut(p);
        let r = &mut c.regions[i];
        r.seceded = true;
        let (ps, gs) = (r.pop_share, r.gdp_share);
        let pop = c.population * ps;
        let gdp = c.gdp * gs;
        let strength = c.forces.strength() * ps * FORCES_TAKEN;
        c.population -= pop;
        c.gdp -= gdp;
        c.debt *= 1.0 - gs;
        crate::war::destroy(state, p, strength);
        let c = state.country(p);
        (pop, gdp, strength, c.force_mix, c.military_tech as f64)
    };
    let x = state.country_mut(s);
    x.active = true;
    x.population = pop;
    x.gdp = gdp;
    x.forces = Forces::new(strength, mix, tech);
    x.stability = 50.0;
    x.legitimacy = 60.0;
    let parent = state.country_mut(p);
    parent.legitimacy = (parent.legitimacy - 10.0).max(0.0);
    parent.stability = (parent.stability - 5.0).max(0.0);
    state.opinions.add(
        p,
        s,
        OpinionModifier {
            source: "seceded from us".into(),
            value: -30.0,
            decay: 0.3,
            memory: None,
        },
    );
    state.tension.add(p, s, 40.0);
}
