//! World interest rate (scenario P7, built 2026-10-04 after the crisis pass
//! showed debtor crises had no driver).
//!
//! The reserve-currency holder (the country with the largest
//! `financial_weight`, if it reaches [`RESERVE_WEIGHT`]; data, not identity)
//! sets a standing monetary stance. Tight squeezes inflation everywhere but
//! raises every debtor's interest bill; Loose does the reverse. The world rate
//! moves toward the stance's target (plus a tension premium) a quarter of the
//! way each turn. Inflation pressure comes from energy prices above their
//! base; debt service comes out of each country's spending pool and shows in
//! its Prosperity.

use serde::{Deserialize, Serialize};

use crate::country::Country;
use crate::ids::CountryId;
use crate::world::WorldState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MonetaryStance {
    Tight,
    #[default]
    Neutral,
    Loose,
}

impl MonetaryStance {
    pub const ALL: [MonetaryStance; 3] = [MonetaryStance::Tight, MonetaryStance::Neutral, MonetaryStance::Loose];

    /// Target world rate as a multiple of the base rate.
    pub fn rate_factor(self) -> f64 {
        match self {
            MonetaryStance::Tight => 2.2,
            MonetaryStance::Neutral => 1.0,
            MonetaryStance::Loose => 0.6,
        }
    }

    /// How much of the energy-driven inflation pressure gets through.
    pub fn inflation_factor(self) -> f64 {
        match self {
            MonetaryStance::Tight => 0.3,
            MonetaryStance::Neutral => 1.0,
            MonetaryStance::Loose => 1.5,
        }
    }

    /// How much of the current inflation carries into expectations (a
    /// spiral sustains itself until tight money breaks it).
    pub fn persistence(self) -> f64 {
        match self {
            MonetaryStance::Tight => 0.3,
            MonetaryStance::Neutral => 0.7,
            MonetaryStance::Loose => 1.0,
        }
    }

    /// Growth per quarter the stance adds or costs, everywhere (tight money
    /// is a world recession: 1981-82).
    pub fn growth_effect(self) -> f64 {
        match self {
            MonetaryStance::Tight => -0.004,
            MonetaryStance::Neutral => 0.0,
            MonetaryStance::Loose => 0.002,
        }
    }

    /// Inflation the stance itself adds (easy money) or wrings out.
    pub fn inflation_push(self) -> f64 {
        match self {
            MonetaryStance::Tight => -0.2,
            MonetaryStance::Neutral => 0.0,
            MonetaryStance::Loose => 0.3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MoneyMarket {
    pub stance: MonetaryStance,
    /// Quarterly world interest rate.
    pub rate: f64,
    /// World inflation level (persistent: a wage-price spiral takes years
    /// to build and to wring out). 1 = a severe 1970s-style inflation.
    #[serde(default)]
    pub inflation: f64,
}

impl Default for MoneyMarket {
    fn default() -> Self {
        MoneyMarket {
            stance: MonetaryStance::Neutral,
            rate: 0.0,
            inflation: 0.0,
        }
    }
}

/// Minimum `financial_weight` to set the world's monetary stance.
pub const RESERVE_WEIGHT: f64 = 0.4;
/// Share of the gap to the target rate closed each turn.
pub const RATE_ADJUST: f64 = 0.25;
/// Rate premium at full global tension, as a share of the base rate.
pub const TENSION_PREMIUM: f64 = 0.3;
/// Prosperity lost per unit of inflation pressure (an importer at a doubled
/// energy price under a neutral stance loses this much).
pub const INFLATION_PROSPERITY: f64 = 15.0;
/// Exporters feel inflation too, but their revenue rises with the price.
pub const EXPORTER_INFLATION: f64 = 0.5;
/// Debt service up to this share of revenue is routine.
pub const ROUTINE_DEBT_SERVICE: f64 = 0.08;
/// Prosperity lost per unit of debt service above the routine share.
pub const DEBT_SERVICE_PROSPERITY: f64 = 60.0;

/// The country that sets the stance, if any.
pub fn holder(state: &WorldState) -> Option<CountryId> {
    state
        .countries
        .iter()
        .filter(|c| c.active && c.financial_weight >= RESERVE_WEIGHT)
        .max_by(|a, b| a.financial_weight.total_cmp(&b.financial_weight).then(b.id.cmp(&a.id)))
        .map(|c| c.id)
}

/// Target quarterly rate for a stance at a given global tension.
pub fn target_rate(base: f64, stance: MonetaryStance, global_tension: f64) -> f64 {
    base * (stance.rate_factor() + TENSION_PREMIUM * (global_tension / 100.0).clamp(0.0, 1.0))
}

/// Where inflation is heading under a stance from its current level:
/// energy above its base price let through by the stance, what the stance
/// adds or removes, and the spiral's own persistence.
pub fn inflation_target(energy_deviation: f64, stance: MonetaryStance, current: f64) -> f64 {
    (energy_deviation.max(0.0) * stance.inflation_factor()
        + stance.inflation_push()
        + stance.persistence() * current)
        .clamp(0.0, MAX_INFLATION)
}

/// Inflation is capped (hyperinflation is beyond this model).
pub const MAX_INFLATION: f64 = 3.0;

/// Share of the gap to the inflation target closed each turn.
pub const INFLATION_ADJUST: f64 = 0.15;

/// Debt service per quarter for a country at a world rate (the reserve
/// currency's privilege discounts it).
pub fn debt_service(c: &Country, rate: f64) -> f64 {
    c.debt * rate * (1.0 - c.financial_weight)
}

/// Prosperity effects of money for one country: (inflation, debt service).
pub fn prosperity_effects(c: &Country, pressure: f64, service: f64) -> (f64, f64) {
    let exposure = if c.energy_net_exports > 0.0 { EXPORTER_INFLATION } else { 1.0 };
    let revenue = (c.gdp * c.tax_rate).max(1e-9);
    (
        INFLATION_PROSPERITY * pressure * exposure,
        DEBT_SERVICE_PROSPERITY * (service / revenue - ROUTINE_DEBT_SERVICE).max(0.0),
    )
}

/// Move the world rate toward its target and record each country's money
/// effects (run at the start of the economy step).
pub fn update(state: &mut WorldState) {
    let target = target_rate(state.params.base_interest_rate, state.money.stance, state.global_tension);
    if state.money.rate <= 0.0 {
        state.money.rate = target;
    }
    state.money.rate += (target - state.money.rate) * RATE_ADJUST;
    let goal = inflation_target(state.energy.deviation(), state.money.stance, state.money.inflation);
    state.money.inflation += (goal - state.money.inflation) * INFLATION_ADJUST;
    let pressure = state.money.inflation;
    let rate = state.money.rate;
    for c in state.countries.iter_mut() {
        let service = debt_service(c, rate);
        let (inflation, debt) = prosperity_effects(c, pressure, service);
        c.debt_service = service;
        c.inflation_hit = inflation;
        c.debt_service_hit = debt;
    }
}
