//! Intelligence coverage and estimate quality (DESIGN §8.2–8.3).
//!
//! v0.1 sources of coverage: a base level, reconnaissance (the observer's
//! intelligence tech), the target's openness, and the observer's
//! intelligence capability against the target's counterintelligence.
//! Proximity, spy networks and allied sharing join when regions, intel
//! operations and treaties exist.

use crate::country::Country;

const BASE_COVERAGE: f64 = 5.0;
const RECON_PER_TECH: f64 = 3.0;
const OPENNESS_COVERAGE: f64 = 30.0;
const CAPABILITY_COVERAGE: f64 = 25.0;

/// Share of the gap to a defensive ally's better coverage that sharing closes.
pub const ALLY_SHARE: f64 = 0.5;

/// Coverage at or above which the stability band and debt are visible.
pub const TIER_DOMESTIC: f64 = 20.0;
/// Coverage at or above which budget allocation is visible.
pub const TIER_BUDGET: f64 = 60.0;
/// Coverage at or above which intentions become visible (once goals exist).
pub const TIER_INTENTIONS: f64 = 80.0;

/// Coverage of `target` by `observer`, 0–100.
pub fn coverage(observer: &Country, target: &Country) -> f64 {
    let capability = observer.intel_capacity / (observer.intel_capacity + target.intel_capacity + 1.0);
    (BASE_COVERAGE
        + RECON_PER_TECH * observer.intel_tech as f64
        + OPENNESS_COVERAGE * target.openness
        + CAPABILITY_COVERAGE * capability)
        .clamp(0.0, 100.0)
}

/// Relative error of an estimate at a given coverage: about ±40% at 20,
/// ±25% at 40, ±15% at 60, ±8% at 80, never below 3%.
pub fn estimate_error(coverage: f64) -> f64 {
    let gap = 1.0 - coverage.clamp(0.0, 100.0) / 100.0;
    0.5 * gap.powf(1.5) + 0.03
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::country::{BudgetShares, CountrySetup, Government};
    use crate::ids::CountryId;

    fn country(tech: u8, openness: f64, capacity: f64) -> Country {
        Country::from_setup(
            CountryId(0),
            CountrySetup {
                code: "a".into(),
                name: "a".into(),
                government: Government::Democracy,
                population: 10.0,
                gdp: 10.0,
                tax_rate: 0.2,
                debt: 0.0,
                financial_weight: 0.0,
                budget: BudgetShares {
                    military: 0.25,
                    development: 0.25,
                    welfare: 0.25,
                    intelligence: 0.25,
                },
                deficit_ratio: 0.0,
                military: 10.0,
                force_mix: Default::default(),
                military_tech: 5,
                military_cost: 1.0,
                readiness: None,
                energy_capacity: 0.0,
                arsenal: 0,
                arsenal_declared: true,
                programme: None,
                reform_personality: None,
                took_power: None,
                growth_modifier: 0.0,
                command_drag: 0.0,
                arms_industry: 0.0,
                regions: Vec::new(),
                dormant: false,
                area: None,
                tier: Default::default(),
                stability: 50.0,
                legitimacy: 50.0,
                initiative_base: 3,
                intel_tech: tech,
                openness: Some(openness),
                intel_capacity: Some(capacity),
                trade_exposure: 1.0,
                alignment: None,
                personality: Default::default(),
                priors: Default::default(),
                reflexes: Vec::new(),
            },
        )
    }

    #[test]
    fn coverage_rises_with_each_source() {
        let target = country(3, 0.3, 10.0);
        let base = coverage(&country(3, 0.5, 10.0), &target);
        assert!(coverage(&country(8, 0.5, 10.0), &target) > base, "recon");
        assert!(coverage(&country(3, 0.5, 40.0), &target) > base, "capability");
        assert!(
            coverage(&country(3, 0.5, 10.0), &country(3, 0.9, 10.0)) > base,
            "target openness"
        );
        assert!(
            coverage(&country(3, 0.5, 10.0), &country(3, 0.3, 40.0)) < base,
            "counterintelligence"
        );
    }

    #[test]
    fn coverage_is_bounded() {
        let max = coverage(&country(10, 0.0, 1e9), &country(1, 1.0, 0.0));
        let min = coverage(&country(1, 0.0, 0.0), &country(10, 0.0, 1e9));
        assert!((0.0..=100.0).contains(&max) && (0.0..=100.0).contains(&min));
        assert!(max > 80.0 && min < 15.0);
    }

    #[test]
    fn error_shrinks_with_coverage() {
        let mut prev = f64::INFINITY;
        for cov in (0..=100).step_by(10) {
            let e = estimate_error(cov as f64);
            assert!(e < prev && (0.03..1.0).contains(&e));
            prev = e;
        }
    }
}
