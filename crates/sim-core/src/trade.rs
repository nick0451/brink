//! Bilateral trade, sanctions and substitution (DESIGN §7.2–7.3).
//!
//! Trade volume per pair = gravity (GDP_a × GDP_b ÷ world GDP) × openness ×
//! relations × agreement multiplier, cut by sanctions in either direction.
//! The normalised gravity keeps each country's trade a sensible share of its
//! own economy however many partners exist (a √(GDP_a × GDP_b) form gave
//! small economies trade many times their GDP in a 28-country world). A sanctioned country
//! reroutes a growing share of its lost trade to non-participants; that
//! adaptation never resets, so on/off sanction cycling erodes the weapon.
//! Sanctioners recover nothing: their lost exports are the visible cost.

use crate::ids::CountryId;
use crate::world::WorldState;

/// Trade as a share of GDP for a fully open economy with average relations
/// and no agreements, before its own share of world GDP is excluded.
const TRADE_BASE: f64 = 0.3;

/// Gravity trade between two economies (no agreements, sanctions or
/// relations): `TRADE_BASE × GDP_a × GDP_b ÷ world GDP`.
pub fn gravity(gdp_a: f64, gdp_b: f64, world_gdp: f64) -> f64 {
    TRADE_BASE * gdp_a * gdp_b / world_gdp.max(1e-9)
}
/// Share of bilateral trade cut by a sanction.
pub const SANCTION_CUT: f64 = 0.8;
/// Adaptation gained per turn under sanction (share of lost trade rerouted).
pub const ADAPTATION_PER_TURN: f64 = 0.10;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TradeOutcome {
    /// Trade volume per country this turn (after sanctions and rerouting).
    pub volume: Vec<f64>,
    /// Trade lost to sanctions per country, net of rerouting.
    pub lost: Vec<f64>,
    /// Gravity-only trade with no agreements or sanctions. Ordinary trade is
    /// part of base growth; only the difference from this moves growth.
    pub baseline: Vec<f64>,
}

pub fn compute(state: &WorldState) -> TradeOutcome {
    let n = state.countries.len();
    let mut volume = vec![0.0; n];
    let mut baseline = vec![0.0; n];
    let mut lost_as_target = vec![0.0; n];
    let mut lost_as_sanctioner = vec![0.0; n];
    let d = &state.diplomacy;
    let world: f64 = state.countries.iter().map(|c| c.gdp).sum();

    for i in 0..n {
        for j in i + 1..n {
            let (a, b) = (CountryId(i as u16), CountryId(j as u16));
            let (ca, cb) = (state.country(a), state.country(b));
            if !ca.active || !cb.active {
                continue;
            }
            let openness = 0.5 + 0.25 * (ca.openness + cb.openness);
            let relations =
                (1.0 + (state.opinions.opinion(a, b) + state.opinions.opinion(b, a)) / 400.0).clamp(0.5, 1.5);
            let gravity = gravity(ca.gdp, cb.gdp, world) * openness * relations;
            baseline[i] += gravity;
            baseline[j] += gravity;
            let full = gravity * d.trade_multiplier(a, b);

            if state.wars.at_war(a, b) {
                // Enemies don't trade; nothing is rerouted.
                lost_as_sanctioner[i] += full;
                lost_as_sanctioner[j] += full;
                continue;
            }
            let a_sanctions_b = d.is_sanctioned_by(b, a);
            let b_sanctions_a = d.is_sanctioned_by(a, b);
            let cut = if a_sanctions_b || b_sanctions_a {
                full * SANCTION_CUT
            } else {
                0.0
            };
            volume[i] += full - cut;
            volume[j] += full - cut;
            // Each side's loss counts as "target" loss if it is sanctioned in
            // this pair (it can reroute), otherwise as sanctioner loss.
            for (side, sanctioned) in [(i, b_sanctions_a), (j, a_sanctions_b)] {
                if cut > 0.0 {
                    if sanctioned {
                        lost_as_target[side] += cut;
                    } else {
                        lost_as_sanctioner[side] += cut;
                    }
                }
            }
        }
    }

    let mut lost = vec![0.0; n];
    for i in 0..n {
        let rerouted = lost_as_target[i] * state.countries[i].sanction_adaptation;
        volume[i] += rerouted;
        lost[i] = lost_as_target[i] - rerouted + lost_as_sanctioner[i];
    }
    TradeOutcome { volume, lost, baseline }
}

/// Sanctioned countries adapt; adaptation persists after sanctions lift.
pub fn adapt(state: &mut WorldState) {
    for i in 0..state.countries.len() {
        if state.diplomacy.is_sanctioned(CountryId(i as u16)) {
            let c = &mut state.countries[i];
            c.sanction_adaptation = (c.sanction_adaptation + ADAPTATION_PER_TURN).min(1.0);
        }
    }
}
