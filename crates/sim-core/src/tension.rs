//! Bilateral and global tension (DESIGN §11.1, §11.3).

use serde::{Deserialize, Serialize};

use crate::diplomacy::TreatyKind;
use crate::ids::CountryId;
use crate::world::WorldState;

/// Share of the gap to the baseline closed each turn.
const DRIFT: f64 = 0.05;
/// Tension baseline per point of mutual hostility (mean negative opinion).
pub const HOSTILITY_TO_TENSION: f64 = 0.6;
/// Tension baseline added while either side sanctions the other.
pub const SANCTION_BASELINE: f64 = 10.0;
const PACT_RELIEF: f64 = 10.0;
/// Tension baseline between countries at war.
const WAR_BASELINE: f64 = 90.0;

/// Symmetric 0–100 tension per pair.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TensionBook {
    n: usize,
    values: Vec<f64>,
}

impl TensionBook {
    pub fn new(n: usize) -> Self {
        TensionBook {
            n,
            values: vec![0.0; n * n],
        }
    }

    fn idx(&self, a: CountryId, b: CountryId) -> usize {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        lo.index() * self.n + hi.index()
    }

    pub fn get(&self, a: CountryId, b: CountryId) -> f64 {
        if a == b {
            0.0
        } else {
            self.values[self.idx(a, b)]
        }
    }

    pub fn add(&mut self, a: CountryId, b: CountryId, delta: f64) {
        if a != b {
            let i = self.idx(a, b);
            self.values[i] = (self.values[i] + delta).clamp(0.0, 100.0);
        }
    }

    pub fn set(&mut self, a: CountryId, b: CountryId, value: f64) {
        if a != b {
            let i = self.idx(a, b);
            self.values[i] = value.clamp(0.0, 100.0);
        }
    }
}

/// Each country's power share: half economic, half military (DESIGN §11.1).
pub fn power_shares(state: &WorldState) -> Vec<f64> {
    let total_gdp: f64 = state.countries.iter().map(|c| c.gdp).sum::<f64>().max(1e-9);
    let total_mil: f64 = state.countries.iter().map(|c| c.power()).sum::<f64>().max(1e-9);
    state
        .countries
        .iter()
        .map(|c| 0.5 * c.gdp / total_gdp + 0.5 * c.power() / total_mil)
        .collect()
}

pub fn baseline(state: &WorldState, a: CountryId, b: CountryId) -> f64 {
    let hostility = -(state.opinions.opinion(a, b) + state.opinions.opinion(b, a)) / 2.0;
    let mut base = hostility.max(0.0) * HOSTILITY_TO_TENSION;
    let d = &state.diplomacy;
    if d.is_sanctioned_by(a, b) || d.is_sanctioned_by(b, a) {
        base += SANCTION_BASELINE;
    }
    if d.has_treaty(TreatyKind::NonAggression, a, b) || d.has_treaty(TreatyKind::DefensiveAlliance, a, b) {
        base -= PACT_RELIEF;
    }
    if state.wars.at_war(a, b) {
        base = base.max(WAR_BASELINE);
    }
    base.clamp(0.0, 100.0)
}

/// Set every pair to its baseline (scenario start), then recompute global.
pub fn initialize(state: &mut WorldState) {
    let ids: Vec<CountryId> = state.ids().collect();
    for (i, &a) in ids.iter().enumerate() {
        for &b in &ids[i + 1..] {
            let target = baseline(state, a, b);
            state.tension.set(a, b, target);
        }
    }
    state.global_tension = global(state);
}

/// Drift every pair toward its baseline and recompute global tension.
pub fn update(state: &mut WorldState) {
    let ids: Vec<CountryId> = state.ids().collect();
    for (i, &a) in ids.iter().enumerate() {
        for &b in &ids[i + 1..] {
            let current = state.tension.get(a, b);
            let target = baseline(state, a, b);
            state.tension.set(a, b, current + (target - current) * DRIFT);
        }
    }
    state.global_tension = global(state);
}

/// Power-weighted mean of bilateral tensions: tension between strong
/// countries matters more (DESIGN §11.1).
pub fn global(state: &WorldState) -> f64 {
    let power = power_shares(state);
    let ids: Vec<CountryId> = state.ids().collect();
    let (mut sum, mut weight) = (0.0, 0.0);
    for (i, &a) in ids.iter().enumerate() {
        for &b in &ids[i + 1..] {
            let w = power[a.index()] * power[b.index()];
            sum += w * state.tension.get(a, b);
            weight += w;
        }
    }
    if weight > 0.0 {
        sum / weight
    } else {
        0.0
    }
}

/// Readiness Condition 5 (calm) to 1 (brink).
pub fn readiness_condition(global_tension: f64) -> u8 {
    match global_tension {
        t if t < 20.0 => 5,
        t if t < 35.0 => 4,
        t if t < 50.0 => 3,
        t if t < 70.0 => 2,
        _ => 1,
    }
}
