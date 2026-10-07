//! The world energy market (DESIGN §2.4) and production policy (scenario
//! P5, v0.2). Energy only for now; Materials and Food stay deferred (Food is
//! on probation).
//!
//! - Every country consumes energy in proportion to its GDP and produces
//!   from a capacity (scenario data) that grows slowly.
//! - The world price follows the demand ÷ supply ratio with smoothing.
//!   Wars cut belligerents' output; exporters choose Restrain / Normal /
//!   Flood.
//! - Only the price's deviation from its base matters to economies: net
//!   exporters gain growth and revenue when it is high, net importers lose,
//!   and the reverse when it is low. Effects are capped so an oil state's
//!   budget can swing hard without breaking the model.

use serde::{Deserialize, Serialize};

use crate::world::WorldState;

/// Energy consumed per unit of GDP per turn.
pub const ENERGY_INTENSITY: f64 = 1.0;
/// Capacity growth per turn at the base price. High prices draw investment:
/// growth scales with (price ÷ base)², so prices drift back toward base.
pub const CAPACITY_GROWTH: f64 = 0.006;
/// Price response to the demand ÷ supply ratio: P* = base × ratio^k.
/// Short-run energy demand is very inelastic, so small supply shocks move
/// the price a lot.
pub const PRICE_EXPONENT: f64 = 5.0;
/// Share of the gap to the target price closed each turn.
pub const PRICE_SMOOTHING: f64 = 0.25;
/// Output kept by a country at war.
pub const WAR_OUTPUT: f64 = 0.7;
/// Growth per unit of (net exports ÷ GDP) × relative price deviation,
/// capped at ±[`MAX_ENERGY_GROWTH`] per turn.
const ENERGY_GROWTH: f64 = 0.01;
pub const MAX_ENERGY_GROWTH: f64 = 0.01;
/// Revenue multiplier per unit of (net exports ÷ GDP) × relative price
/// deviation, capped to [−30%, +60%].
const ENERGY_REVENUE: f64 = 0.3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionPolicy {
    /// Hold output back to support the price.
    Restrain,
    #[default]
    Normal,
    /// Pump at full tilt: market share at the price's expense.
    Flood,
}

impl ProductionPolicy {
    pub fn factor(self) -> f64 {
        match self {
            ProductionPolicy::Restrain => 0.85,
            ProductionPolicy::Normal => 1.0,
            ProductionPolicy::Flood => 1.2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnergyMarket {
    pub price: f64,
    /// The price at which supply meets demand at scenario start.
    pub base_price: f64,
    pub supply: f64,
    pub demand: f64,
    /// Output of producers the scenario doesn't model as countries (grows
    /// like capacity).
    #[serde(default)]
    pub outside_supply: f64,
}

impl Default for EnergyMarket {
    fn default() -> Self {
        EnergyMarket {
            price: 1.0,
            base_price: 1.0,
            supply: 0.0,
            demand: 0.0,
            outside_supply: 0.0,
        }
    }
}

impl EnergyMarket {
    /// Relative deviation of the price from its base.
    pub fn deviation(&self) -> f64 {
        (self.price - self.base_price) / self.base_price.max(1e-9)
    }
}

/// Output a country puts on the market this turn.
pub fn output(state: &WorldState, i: usize) -> f64 {
    let c = &state.countries[i];
    let war = if state.wars.is_belligerent(c.id) {
        WAR_OUTPUT
    } else {
        1.0
    };
    c.energy_capacity * c.energy_policy.factor() * war
}

/// Clear the market: supply, demand, net exports, and the new price.
pub fn clear(state: &mut WorldState) {
    if state.countries.iter().all(|c| c.energy_capacity <= 0.0) {
        return;
    }
    let n = state.countries.len();
    let out: Vec<f64> = (0..n).map(|i| output(state, i)).collect();
    let use_: Vec<f64> = state.countries.iter().map(|c| c.gdp * ENERGY_INTENSITY).collect();
    let supply: f64 = out.iter().sum::<f64>() + state.energy.outside_supply;
    let investment = CAPACITY_GROWTH * (state.energy.price / state.energy.base_price.max(1e-9)).powi(2);
    state.energy.outside_supply *= 1.0 + investment;
    let demand: f64 = use_.iter().sum();
    for (i, c) in state.countries.iter_mut().enumerate() {
        c.energy_net_exports = out[i] - use_[i];
        c.energy_capacity *= 1.0 + investment;
    }
    let m = &mut state.energy;
    let target = m.base_price * (demand / supply.max(1e-9)).powf(PRICE_EXPONENT);
    m.price = (m.price + (target - m.price) * PRICE_SMOOTHING).clamp(0.3 * m.base_price, 4.0 * m.base_price);
    m.supply = supply;
    m.demand = demand;
}

/// Growth effect of the price for a country with these net exports.
pub fn growth_effect(net_exports: f64, gdp: f64, deviation: f64) -> f64 {
    (ENERGY_GROWTH * net_exports / gdp.max(1e-9) * deviation).clamp(-MAX_ENERGY_GROWTH, MAX_ENERGY_GROWTH)
}

/// Spending-pool multiplier from the price (1 = neutral).
pub fn revenue_factor(net_exports: f64, gdp: f64, deviation: f64) -> f64 {
    1.0 + (ENERGY_REVENUE * net_exports / gdp.max(1e-9) * deviation).clamp(-0.3, 0.6)
}
