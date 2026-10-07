//! Observer views: the only state AI code may read (DESIGN §8, §14.1).
//!
//! A view is built from the canonical state without mutating it, so building
//! views never consumes the world RNG and never depends on call order.
//! Foreign values are estimates whose error comes from intelligence coverage
//! (see [`crate::intel`]). Estimate noise is a pure function of (seed,
//! observer, target, time) and is mostly persistent: an observer stays
//! confidently wrong for a while rather than jittering every turn.

use serde::{Deserialize, Serialize};

use crate::country::{BudgetShares, Country, Government};
use crate::diplomacy::{Proposal, Sanction, Stream, Treaty};
use crate::ids::CountryId;
use crate::intel;
use crate::ledger::LedgerEntry;
use crate::reputation::{self, RepKind};
use crate::war::{Involvement, War};
use crate::world::WorldState;

/// Turns over which the persistent part of an estimate's error holds.
const NOISE_EPOCH: u32 = 8;
const PERSISTENT_SHARE: f64 = 0.7;

const SALT_MILITARY: u64 = 1;
const SALT_DEBT: u64 = 2;
const SALT_BUDGET: u64 = 3;
const SALT_POOLS: u64 = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StabilityBand {
    Collapse,
    Crisis,
    Unrest,
    Normal,
    Strong,
}

impl StabilityBand {
    pub fn from_value(v: f64) -> Self {
        match v {
            v if v < 10.0 => StabilityBand::Collapse,
            v if v < 25.0 => StabilityBand::Crisis,
            v if v < 40.0 => StabilityBand::Unrest,
            v if v < 70.0 => StabilityBand::Normal,
            _ => StabilityBand::Strong,
        }
    }
}

/// An uncertain foreign value. The true value always lies in `[low, high]`;
/// the observer doesn't know where.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Estimate {
    pub value: f64,
    pub low: f64,
    pub high: f64,
}

impl Estimate {
    fn new(truth: f64, noise: f64, error: f64) -> Self {
        let value = truth * (1.0 + noise * error);
        Estimate {
            value,
            low: value / (1.0 + error),
            high: value / (1.0 - error),
        }
    }
}

/// Estimated strength points per pool (DESIGN §6.1).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForceEstimate {
    pub land: Estimate,
    pub naval: Estimate,
    pub air: Estimate,
}

/// What an observer knows about another country.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForeignView {
    pub id: CountryId,
    pub code: String,
    pub name: String,
    pub government: Government,
    /// Public bloc tag.
    pub alignment: Option<String>,
    /// Arsenal level as the observer knows it: declared arsenals are
    /// public, undeclared ones need coverage or exposure (P10).
    pub arsenal: u8,
    /// Covert programme progress, if the observer can see it.
    pub programme: Option<f64>,
    /// Public energy statistics: capacity, policy, net exports.
    pub energy_capacity: f64,
    pub energy_policy: crate::commodity::ProductionPolicy,
    pub energy_net_exports: f64,
    /// Export-oriented defence industry (public; D58).
    pub arms_industry: f64,
    /// Military research level (public: what it fields is seen; D58).
    pub military_tech: u8,
    /// Public geography tag and tier.
    pub area: Option<String>,
    pub tier: crate::country::Tier,
    /// The observer's intelligence coverage of this country, 0–100.
    pub coverage: f64,
    /// Public statistic.
    pub gdp: f64,
    /// Estimated military power (strength × quality × readiness), on the
    /// same scale as [`Country::power`]. Estimated separately from the
    /// pools, so the two needn't agree exactly.
    pub military: Estimate,
    pub forces: ForceEstimate,
    /// Visible at coverage ≥ [`intel::TIER_DOMESTIC`].
    pub debt_ratio: Option<Estimate>,
    /// Visible at coverage ≥ [`intel::TIER_DOMESTIC`].
    pub stability_band: Option<StabilityBand>,
    /// Visible (approximately) at coverage ≥ [`intel::TIER_BUDGET`].
    pub budget: Option<BudgetShares>,
    pub their_opinion_of_us: f64,
    pub our_opinion_of_them: f64,
    /// Bilateral tension with the observer, 0–100.
    pub tension: f64,
    /// The observer's own reading of this country's reputation (DESIGN
    /// §21.3). These are the only reputation values the AI may use.
    pub credibility_back: f64,
    pub credibility_threat: f64,
    pub credibility_norm: f64,
    pub trust: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObserverView {
    pub observer: CountryId,
    pub turn: u32,
    pub year: f64,
    /// The observer's own state, known exactly.
    pub own: Country,
    pub others: Vec<ForeignView>,
    /// Treaties, sanctions and aid streams are public.
    pub treaties: Vec<Treaty>,
    pub sanctions: Vec<Sanction>,
    pub streams: Vec<Stream>,
    /// War loans outstanding (public: the pledges were public; issue 24).
    pub loans: Vec<crate::diplomacy::Loan>,
    /// Proposals addressed to the observer that it may answer this turn.
    pub incoming_proposals: Vec<Proposal>,
    pub global_tension: f64,
    /// Tension between every pair of countries (public), `(a, b, value)`
    /// with `a < b`.
    pub pair_tensions: Vec<(CountryId, CountryId, f64)>,
    /// Ledger entries the observer has seen.
    pub ledger: Vec<LedgerEntry>,
    /// Wars are public: belligerents, aims and front progress.
    pub wars: Vec<War>,
    /// The world energy market (public prices).
    pub energy: crate::commodity::EnergyMarket,
    /// Involvements the observer can see (covert ones need coverage).
    pub involvements: Vec<Involvement>,
    /// Standing claims (public).
    pub claims: Vec<crate::world::Claim>,
    /// World interest rate and monetary stance (public), and the base rate.
    pub money: crate::money::MoneyMarket,
    pub base_interest_rate: f64,
}

pub fn observe(state: &WorldState, observer: CountryId) -> ObserverView {
    let me = state.country(observer);
    let others = state
        .countries
        .iter()
        .filter(|c| c.id != observer && c.active)
        .map(|c| foreign_view(state, me, c))
        .collect();
    let d = &state.diplomacy;
    ObserverView {
        observer,
        turn: state.turn,
        year: state.year(),
        own: me.clone(),
        others,
        treaties: d.treaties.clone(),
        sanctions: d.sanctions.clone(),
        streams: d
            .streams
            .iter()
            .filter(|s| {
                !s.covert || s.from == observer || s.to == observer || crate::war::sees_covert(state, observer, s.from)
            })
            .copied()
            .collect(),
        loans: d.loans.clone(),
        incoming_proposals: d
            .proposals
            .iter()
            .filter(|p| p.to == observer && p.turn < state.turn)
            .copied()
            .collect(),
        global_tension: state.global_tension,
        pair_tensions: {
            let ids: Vec<CountryId> = state.ids().collect();
            let mut v = Vec::new();
            for (i, &a) in ids.iter().enumerate() {
                for &b in &ids[i + 1..] {
                    v.push((a, b, state.tension.get(a, b)));
                }
            }
            v
        },
        ledger: state
            .ledger
            .entries
            .iter()
            .filter(|e| e.seen_by.contains(observer))
            .cloned()
            .collect(),
        wars: state.wars.active.clone(),
        energy: state.energy.clone(),
        claims: state.claims.clone(),
        money: state.money.clone(),
        base_interest_rate: state.params.base_interest_rate,
        involvements: state
            .wars
            .involvements
            .iter()
            .filter(|i| {
                i.visibility != crate::ledger::Visibility::Covert
                    || i.actor == observer
                    || i.beneficiary == observer
                    || crate::war::sees_covert(state, observer, i.actor)
            })
            .copied()
            .collect(),
    }
}

/// Own coverage, raised toward the best coverage held by a defensive ally
/// (allied intelligence sharing, DESIGN §8.2).
fn effective_coverage(state: &WorldState, me: &Country, target: &Country) -> f64 {
    let own = intel::coverage(me, target);
    let best_ally = state
        .diplomacy
        .allies_of(me.id)
        .filter(|&a| a != target.id)
        .map(|a| intel::coverage(state.country(a), target))
        .fold(own, f64::max);
    own + intel::ALLY_SHARE * (best_ally - own)
}

fn foreign_view(state: &WorldState, me: &Country, c: &Country) -> ForeignView {
    let coverage = effective_coverage(state, me, c);
    let error = intel::estimate_error(coverage);
    let noise = |salt: u64| estimate_noise(state.seed, state.turn, salt, me.id, c.id);

    let domestic_visible = coverage >= intel::TIER_DOMESTIC;
    let budget = (coverage >= intel::TIER_BUDGET).then(|| {
        let half = error / 2.0;
        BudgetShares {
            military: c.budget.military * (1.0 + noise(SALT_BUDGET) * half),
            development: c.budget.development * (1.0 + noise(SALT_BUDGET + 10) * half),
            welfare: c.budget.welfare * (1.0 + noise(SALT_BUDGET + 20) * half),
            intelligence: c.budget.intelligence * (1.0 + noise(SALT_BUDGET + 30) * half),
        }
        .normalized()
    });

    ForeignView {
        id: c.id,
        code: c.code.clone(),
        name: c.name.clone(),
        government: c.government,
        alignment: c.alignment.clone(),
        area: c.area.clone(),
        tier: c.tier,
        arsenal: crate::programme::visible_arsenal(state, me.id, c.id),
        programme: crate::programme::visible_programme(state, me.id, c.id),
        energy_capacity: c.energy_capacity,
        energy_policy: c.energy_policy,
        energy_net_exports: c.energy_net_exports,
        arms_industry: c.arms_industry,
        military_tech: c.military_tech,
        coverage,
        gdp: c.gdp,
        military: Estimate::new(c.power(), noise(SALT_MILITARY), error),
        forces: ForceEstimate {
            land: Estimate::new(c.forces.land.strength, noise(SALT_POOLS), error),
            naval: Estimate::new(c.forces.naval.strength, noise(SALT_POOLS + 1), error),
            air: Estimate::new(c.forces.air.strength, noise(SALT_POOLS + 2), error),
        },
        debt_ratio: domestic_visible.then(|| Estimate::new(c.debt_ratio(), noise(SALT_DEBT), error)),
        stability_band: domestic_visible.then(|| StabilityBand::from_value(c.stability)),
        budget,
        their_opinion_of_us: state.opinions.opinion(c.id, me.id),
        our_opinion_of_them: state.opinions.opinion(me.id, c.id),
        tension: state.tension.get(me.id, c.id),
        credibility_back: reputation::credibility(state, me.id, c.id, RepKind::Back),
        credibility_threat: reputation::credibility(state, me.id, c.id, RepKind::Threat),
        credibility_norm: reputation::credibility(state, me.id, c.id, RepKind::Norm),
        trust: reputation::trust(state, me.id, c.id),
    }
}

/// Estimate noise in [-1, 1): a persistent part that changes every
/// [`NOISE_EPOCH`] turns plus a smaller per-turn jitter.
fn estimate_noise(seed: u64, turn: u32, salt: u64, observer: CountryId, target: CountryId) -> f64 {
    let persistent = hash_unit(seed, salt, (turn / NOISE_EPOCH) as u64, observer, target);
    let jitter = hash_unit(seed, salt ^ 0x5bd1_e995, turn as u64 + (1 << 40), observer, target);
    PERSISTENT_SHARE * persistent + (1.0 - PERSISTENT_SHARE) * jitter
}

/// Deterministic value in [-1, 1) from chained SplitMix64 mixing.
fn hash_unit(seed: u64, salt: u64, time: u64, observer: CountryId, target: CountryId) -> f64 {
    let mix = |mut x: u64| {
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    };
    let pair = ((observer.0 as u64) << 16) | target.0 as u64;
    let x = mix(mix(mix(seed ^ salt) ^ time) ^ pair);
    (x >> 11) as f64 / (1u64 << 52) as f64 - 1.0
}

impl ObserverView {
    /// The world's base interest rate (before the monetary stance).
    pub fn money_base_rate(&self) -> f64 {
        self.base_interest_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_bounded_deterministic_and_pair_specific() {
        for t in 0..200 {
            let a = estimate_noise(7, t, SALT_MILITARY, CountryId(1), CountryId(2));
            assert!((-1.0..1.0).contains(&a));
            assert_eq!(a, estimate_noise(7, t, SALT_MILITARY, CountryId(1), CountryId(2)));
        }
        assert_ne!(
            estimate_noise(7, 3, SALT_MILITARY, CountryId(1), CountryId(2)),
            estimate_noise(7, 3, SALT_MILITARY, CountryId(2), CountryId(1))
        );
    }

    #[test]
    fn noise_is_mostly_persistent_within_an_epoch() {
        for t in 0..NOISE_EPOCH - 1 {
            let a = estimate_noise(11, t, SALT_MILITARY, CountryId(0), CountryId(1));
            let b = estimate_noise(11, t + 1, SALT_MILITARY, CountryId(0), CountryId(1));
            assert!((a - b).abs() <= 2.0 * (1.0 - PERSISTENT_SHARE) + 1e-12);
        }
    }

    #[test]
    fn estimates_always_bracket_the_truth() {
        for i in 0..1000u32 {
            let noise = estimate_noise(3, i, SALT_MILITARY, CountryId(0), CountryId(1));
            let error = intel::estimate_error((i % 101) as f64);
            let e = Estimate::new(57.0, noise, error);
            assert!(e.low <= 57.0 && 57.0 <= e.high, "{e:?}");
        }
    }
}
