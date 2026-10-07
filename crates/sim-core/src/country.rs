use serde::{Deserialize, Serialize};

use crate::ids::CountryId;
use crate::initiative::{self, Initiative};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Government {
    Democracy,
    Authoritarian,
    Revolutionary,
}

impl Government {
    /// Stability driver weights `(prosperity, security, legitimacy)` (DESIGN §9.1).
    pub fn driver_weights(self) -> (f64, f64, f64) {
        match self {
            Government::Democracy => (0.45, 0.20, 0.35),
            Government::Authoritarian => (0.30, 0.35, 0.35),
            Government::Revolutionary => (0.20, 0.30, 0.50),
        }
    }

    /// Default `openness` when scenario data doesn't override it.
    pub fn default_openness(self) -> f64 {
        match self {
            Government::Democracy => 0.8,
            Government::Authoritarian => 0.3,
            Government::Revolutionary => 0.2,
        }
    }
}

/// The four budget lines (DESIGN §2.1). Always kept normalised to sum to 1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BudgetShares {
    pub military: f64,
    pub development: f64,
    pub welfare: f64,
    pub intelligence: f64,
}

impl BudgetShares {
    pub fn sum(&self) -> f64 {
        self.military + self.development + self.welfare + self.intelligence
    }

    /// Clamp negatives to zero and rescale to sum to 1. An all-zero input
    /// becomes an even split.
    pub fn normalized(self) -> Self {
        let c = |v: f64| if v.is_finite() { v.max(0.0) } else { 0.0 };
        let raw = BudgetShares {
            military: c(self.military),
            development: c(self.development),
            welfare: c(self.welfare),
            intelligence: c(self.intelligence),
        };
        let total = raw.sum();
        if total <= 0.0 {
            return BudgetShares {
                military: 0.25,
                development: 0.25,
                welfare: 0.25,
                intelligence: 0.25,
            };
        }
        BudgetShares {
            military: raw.military / total,
            development: raw.development / total,
            welfare: raw.welfare / total,
            intelligence: raw.intelligence / total,
        }
    }

    /// Move each line `rate` of the way toward `target` (budget inertia).
    pub fn approach(self, target: Self, rate: f64) -> Self {
        let l = |a: f64, b: f64| a + (b - a) * rate;
        BudgetShares {
            military: l(self.military, target.military),
            development: l(self.development, target.development),
            welfare: l(self.welfare, target.welfare),
            intelligence: l(self.intelligence, target.intelligence),
        }
        .normalized()
    }
}

/// Country tier (scenario-1980 A.2). Minors run the same AI with a
/// restricted action set; tier is data, never identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    #[default]
    Playable,
    Major,
    Minor,
}

pub fn active_default() -> bool {
    true
}

/// Quality level treated as 1.0× combat value. Quality is compared between
/// countries, so the reference only sets the scale, not the era.
pub const QUALITY_REF: f64 = 5.0;

/// Mobilization level (DESIGN §6.2). Sets the readiness cap. Changing it is
/// a step-7 order; until then every country stays at peacetime.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mobilization {
    #[default]
    Peacetime,
    Partial,
    Full,
    Total,
}

impl Mobilization {
    pub fn readiness_cap(self) -> f64 {
        match self {
            Mobilization::Peacetime => 0.5,
            Mobilization::Partial => 0.75,
            Mobilization::Full => 1.0,
            Mobilization::Total => 1.1,
        }
    }
}

/// One strength pool (DESIGN §6.1): strength points and their average quality.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForcePool {
    pub strength: f64,
    pub quality: f64,
}

impl ForcePool {
    /// Add strength of a given quality; quality becomes the weighted average.
    pub fn add(&mut self, strength: f64, quality: f64) {
        let total = self.strength + strength;
        if total > 1e-12 {
            self.quality = (self.strength * self.quality + strength * quality) / total;
        }
        self.strength = total;
    }

    /// Combat value before readiness.
    pub fn value(&self) -> f64 {
        self.strength * self.quality / QUALITY_REF
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PoolKind {
    Land,
    Naval,
    Air,
}

impl PoolKind {
    pub const ALL: [PoolKind; 3] = [PoolKind::Land, PoolKind::Naval, PoolKind::Air];
}

/// Share of new military production going to each pool (data). Normalised.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForceMix {
    pub land: f64,
    pub naval: f64,
    pub air: f64,
}

impl Default for ForceMix {
    fn default() -> Self {
        ForceMix {
            land: 0.6,
            naval: 0.2,
            air: 0.2,
        }
    }
}

impl ForceMix {
    pub fn normalized(self) -> Self {
        let c = |v: f64| if v.is_finite() { v.max(0.0) } else { 0.0 };
        let (l, n, a) = (c(self.land), c(self.naval), c(self.air));
        let total = l + n + a;
        if total <= 0.0 {
            return ForceMix::default();
        }
        ForceMix {
            land: l / total,
            naval: n / total,
            air: a / total,
        }
    }

    pub fn share(&self, kind: PoolKind) -> f64 {
        match kind {
            PoolKind::Land => self.land,
            PoolKind::Naval => self.naval,
            PoolKind::Air => self.air,
        }
    }
}

/// A country's armed forces (DESIGN §6.1–6.2).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Forces {
    pub land: ForcePool,
    pub naval: ForcePool,
    pub air: ForcePool,
    /// Share of strength usable now, 0..cap. Falls when spending no longer
    /// covers upkeep (a hollow force), capped by mobilization.
    pub readiness: f64,
    pub mobilization: Mobilization,
}

impl Forces {
    pub fn new(total: f64, mix: ForceMix, quality: f64) -> Self {
        let pool = |share: f64| ForcePool {
            strength: total.max(0.0) * share,
            quality,
        };
        let mobilization = Mobilization::Peacetime;
        Forces {
            land: pool(mix.land),
            naval: pool(mix.naval),
            air: pool(mix.air),
            readiness: mobilization.readiness_cap(),
            mobilization,
        }
    }

    pub fn pool(&self, kind: PoolKind) -> &ForcePool {
        match kind {
            PoolKind::Land => &self.land,
            PoolKind::Naval => &self.naval,
            PoolKind::Air => &self.air,
        }
    }

    pub fn pool_mut(&mut self, kind: PoolKind) -> &mut ForcePool {
        match kind {
            PoolKind::Land => &mut self.land,
            PoolKind::Naval => &mut self.naval,
            PoolKind::Air => &mut self.air,
        }
    }

    /// Total strength points (what budgets buy and upkeep wears down).
    pub fn strength(&self) -> f64 {
        self.land.strength + self.naval.strength + self.air.strength
    }

    /// Readiness multiplier: 1.0 for a fully funded peacetime force, lower
    /// for a hollow one, up to 1.6 at total mobilization.
    pub fn readiness_factor(&self) -> f64 {
        0.5 + self.readiness
    }

    /// Military power: strength × quality × readiness. The number threat,
    /// security and power shares are built from.
    pub fn power(&self) -> f64 {
        (self.land.value() + self.naval.value() + self.air.value()) * self.readiness_factor()
    }
}

/// AI personality vector (DESIGN §14.5). Stored as country data so the
/// simulation (e.g. third-party reactions) and the AI read the same values.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Personality {
    pub aggression: f64,
    pub risk: f64,
    pub paranoia: f64,
    pub loyalty: f64,
    pub greed: f64,
    pub ideology: f64,
    pub opportunism: f64,
}

impl Default for Personality {
    fn default() -> Self {
        Personality {
            aggression: 0.5,
            risk: 0.5,
            paranoia: 0.5,
            loyalty: 0.5,
            greed: 0.5,
            ideology: 0.5,
            opportunism: 0.5,
        }
    }
}

/// Starting expectations others hold about a country before any ledger
/// evidence, 0–1 (DESIGN §21.3 priors).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReputationPriors {
    pub back: f64,
    pub threat: f64,
    pub norm: f64,
}

impl Default for ReputationPriors {
    fn default() -> Self {
        ReputationPriors {
            back: 0.6,
            threat: 0.5,
            norm: 0.5,
        }
    }
}

/// Input values for creating a country. Scenario data maps onto this; derived
/// fields are filled in by [`Country::from_setup`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CountrySetup {
    pub code: String,
    pub name: String,
    pub government: Government,
    pub population: f64,
    pub gdp: f64,
    pub tax_rate: f64,
    pub debt: f64,
    pub financial_weight: f64,
    pub budget: BudgetShares,
    pub deficit_ratio: f64,
    /// Starting total military strength, split by `force_mix`.
    pub military: f64,
    pub force_mix: ForceMix,
    /// Military research level, 1–10 (DESIGN §10.1). Sets the quality of
    /// new production and caps the quality of received arms at +2.
    pub military_tech: u8,
    /// Budget cost of one strength point relative to the reference economy
    /// (data: price levels and conscription make soldiers cheaper in poor
    /// economies). 1 = reference.
    pub military_cost: f64,
    /// Starting readiness, 0–1. `None` = fully funded peacetime (0.5).
    pub readiness: Option<f64>,
    /// Energy production capacity per turn (DESIGN §2.4). 0 = the scenario
    /// doesn't model energy for this country.
    pub energy_capacity: f64,
    /// Strategic arsenal level 0–3 (DESIGN §11.4).
    pub arsenal: u8,
    /// Is the arsenal publicly declared? (P10; default true.)
    pub arsenal_declared: bool,
    /// Covert programme progress 0–100, if one is running.
    pub programme: Option<f64>,
    /// Personality after a reform (P3; e.g. B.18 transition vectors).
    pub reform_personality: Option<Personality>,
    /// Turn the current rulers came to power by revolution or conquest, if
    /// recently (negative = before the scenario; [`crate::transition::command`]).
    #[serde(default)]
    pub took_power: Option<i32>,
    /// Structural growth adjustment per quarter (data: demographics,
    /// export-led catch-up, reform era). Persists.
    pub growth_modifier: f64,
    /// Command-economy stagnation per quarter (data). Removed by a reform (P3).
    pub command_drag: f64,
    /// Export-oriented defence industry, 0–1 (data; D58). The share of sold
    /// arms that come off production lines instead of the seller's own
    /// forces, and the size of its export order book.
    pub arms_industry: f64,
    /// Peripheries with loyalty and possible successors (P2).
    pub regions: Vec<crate::region::Region>,
    /// A dormant successor state: inactive until a secession activates it.
    pub dormant: bool,
    pub stability: f64,
    pub legitimacy: f64,
    pub initiative_base: u8,
    /// Intelligence research level, 1–10 (DESIGN §10.1).
    pub intel_tech: u8,
    /// 0–1. `None` uses the government-type default.
    pub openness: Option<f64>,
    /// Starting intelligence capability. `None` starts at the equilibrium
    /// implied by the intelligence budget.
    pub intel_capacity: Option<f64>,
    /// Domestic sensitivity to trade disruption (lost exports, integration
    /// shocks). 1 = typical (DESIGN §7.3).
    pub trade_exposure: f64,
    /// Bloc or ideology tag (DESIGN §7.1). `None` = non-aligned.
    pub alignment: Option<String>,
    pub personality: Personality,
    pub priors: ReputationPriors,
    /// Institutional habits (design/behaviour-and-voice.md §1).
    pub reflexes: Vec<crate::reflex::Reflex>,
    /// Geographic area tag (data): same-area countries are in full reach.
    pub area: Option<String>,
    pub tier: Tier,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Country {
    pub id: CountryId,
    /// Data identifier only. Engine code must never branch on it.
    pub code: String,
    pub name: String,
    pub government: Government,

    pub population: f64,
    pub gdp: f64,
    pub tax_rate: f64,
    pub debt: f64,
    pub financial_weight: f64,
    pub budget_target: BudgetShares,
    pub budget: BudgetShares,
    pub deficit_ratio: f64,
    pub last_growth: f64,

    pub forces: Forces,
    pub force_mix: ForceMix,
    pub military_tech: u8,
    pub military_cost: f64,
    /// Strategic arsenal level 0–3 and remaining strike capacity.
    pub arsenal: u8,
    pub strike_capacity: f64,
    /// Undeclared arsenals are hidden from observers without coverage (P10).
    pub arsenal_declared: bool,
    /// Covert programme progress 0–100, if one is running.
    pub programme: Option<f64>,
    /// The programme or undeclared arsenal has been made public.
    pub programme_exposed: bool,
    /// Regime-transition state (P3).
    #[serde(default)]
    pub transition: crate::transition::TransitionState,
    /// Structural growth adjustment per quarter (data; persists).
    #[serde(default)]
    pub growth_modifier: f64,
    /// Command-economy drag per quarter (data; cleared by reform).
    #[serde(default)]
    pub command_drag: f64,
    /// Export-oriented defence industry, 0–1 (data; D58).
    #[serde(default)]
    pub arms_industry: f64,
    /// Habitual military share of the budget (doctrine): the scenario's
    /// starting share. The AI scales it with threat instead of every state
    /// converging on one formula.
    #[serde(default)]
    pub military_norm: f64,
    /// GDP when the norm was set: as the economy grows, the habitual share
    /// falls (guns grow with roughly the square root of butter).
    #[serde(default)]
    pub norm_gdp: f64,
    /// Smoothed growth per quarter (about a five-year memory): what people
    /// compare with the leading economy.
    #[serde(default)]
    pub growth_trend: f64,
    /// Money this turn (P7): debt service paid, and the Prosperity lost to
    /// inflation and to debt service above the routine share.
    #[serde(default)]
    pub debt_service: f64,
    #[serde(default)]
    pub inflation_hit: f64,
    #[serde(default)]
    pub debt_service_hit: f64,
    /// War consumption borrowed this turn (issue 21; 0 at peace).
    #[serde(default)]
    pub war_borrowing: f64,
    /// Strength sold abroad this turn and the money it earned (D58).
    #[serde(default)]
    pub arms_sold: f64,
    #[serde(default)]
    pub arms_income: f64,
    /// Peripheries with loyalty (P2).
    #[serde(default)]
    pub regions: Vec<crate::region::Region>,
    /// False for a dormant successor state (P2) until it secedes.
    #[serde(default = "crate::country::active_default")]
    pub active: bool,
    /// Energy production capacity (grows slowly) and the standing policy.
    pub energy_capacity: f64,
    pub energy_policy: crate::commodity::ProductionPolicy,
    /// Output minus consumption last turn (negative = importer).
    pub energy_net_exports: f64,
    /// Strength received and given as arms this turn (DESIGN §21.6).
    pub arms_in: f64,
    pub arms_out: f64,
    /// Cumulative strength given away as arms, before wear. Feeds the
    /// step-8 defence-burden term.
    pub arms_out_total: f64,
    /// Transferred strength still in service, by supplier (sorted by id).
    /// Wears down at the upkeep rate; never returns to the supplier.
    pub arms_origin: Vec<(CountryId, f64)>,

    pub stability: f64,
    pub prosperity: f64,
    pub security: f64,
    pub legitimacy: f64,
    pub war_weariness: f64,

    pub initiative_base: u8,
    pub initiative: Initiative,

    pub intel_tech: u8,
    /// How much information leaks to foreign observers, 0–1 (DESIGN §8.2).
    /// Also scales the domestic cost of exposed covert acts (DESIGN §8.4).
    pub openness: f64,
    /// Accumulated intelligence capability, built from the Intelligence
    /// budget. Drives both collection abroad and counterintelligence at home.
    pub intel_capacity: f64,

    pub trade_exposure: f64,
    /// Trade volume this turn.
    pub trade: f64,
    /// Trade lost to sanctions this turn (net of rerouting).
    pub trade_lost: f64,
    /// Share of sanction-lost trade rerouted elsewhere, 0–1. Never resets.
    pub sanction_adaptation: f64,
    /// Remaining Prosperity penalty from deep-integration adjustment.
    pub adjustment_shock: f64,
    /// Aid received and paid this turn.
    pub aid_in: f64,
    pub aid_out: f64,

    /// Bloc or ideology tag; data only, compared for equality, never matched
    /// against specific values in engine code.
    pub alignment: Option<String>,
    pub personality: Personality,
    pub priors: ReputationPriors,
    /// Institutional habits evaluated by the AI. Data only.
    pub reflexes: Vec<crate::reflex::Reflex>,
    /// Geographic area tag; compared for equality only.
    pub area: Option<String>,
    pub tier: Tier,
}

impl Country {
    pub fn from_setup(id: CountryId, s: CountrySetup) -> Self {
        let budget = s.budget.normalized();
        let mut initiative = Initiative::default();
        initiative.start_turn(initiative::allowance(s.initiative_base, s.stability));
        let intel_capacity = s.intel_capacity.unwrap_or_else(|| {
            crate::economy::intel_equilibrium(s.gdp.max(0.001) * s.tax_rate.clamp(0.0, 1.0) * budget.intelligence)
        });
        Country {
            id,
            code: s.code,
            name: s.name,
            government: s.government,
            population: s.population.max(0.001),
            gdp: s.gdp.max(0.001),
            tax_rate: s.tax_rate.clamp(0.0, 1.0),
            debt: s.debt.max(0.0),
            financial_weight: s.financial_weight.clamp(0.0, 1.0),
            budget_target: budget,
            budget,
            deficit_ratio: s.deficit_ratio.clamp(0.0, crate::economy::MAX_DEFICIT_RATIO),
            last_growth: 0.0,
            forces: {
                let mut f = Forces::new(
                    s.military,
                    s.force_mix.normalized(),
                    s.military_tech.clamp(1, 10) as f64,
                );
                if let Some(r) = s.readiness {
                    f.readiness = r.clamp(0.0, f.mobilization.readiness_cap());
                }
                f
            },
            force_mix: s.force_mix.normalized(),
            military_tech: s.military_tech.clamp(1, 10),
            military_cost: if s.military_cost > 0.0 { s.military_cost } else { 1.0 },
            arsenal: s.arsenal.min(3),
            strike_capacity: crate::nuclear::full_capacity(s.arsenal.min(3)),
            arsenal_declared: s.arsenal_declared,
            programme: s.programme.map(|p| p.clamp(0.0, 99.0)),
            programme_exposed: false,
            transition: crate::transition::TransitionState {
                reform_personality: s.reform_personality,
                took_power: s.took_power,
                ..Default::default()
            },
            growth_modifier: s.growth_modifier,
            command_drag: s.command_drag.min(0.0),
            arms_industry: s.arms_industry.clamp(0.0, 1.0),
            military_norm: s.budget.normalized().military,
            norm_gdp: s.gdp.max(0.001),
            growth_trend: crate::economy::TREND_START,
            debt_service: 0.0,
            inflation_hit: 0.0,
            debt_service_hit: 0.0,
            war_borrowing: 0.0,
            arms_sold: 0.0,
            arms_income: 0.0,
            regions: s.regions,
            active: !s.dormant,
            energy_capacity: s.energy_capacity.max(0.0),
            energy_policy: Default::default(),
            energy_net_exports: s.energy_capacity.max(0.0) - s.gdp.max(0.001) * crate::commodity::ENERGY_INTENSITY,
            arms_in: 0.0,
            arms_out: 0.0,
            arms_out_total: 0.0,
            arms_origin: Vec::new(),
            stability: s.stability.clamp(0.0, 100.0),
            prosperity: 50.0,
            security: 50.0,
            legitimacy: s.legitimacy.clamp(0.0, 100.0),
            war_weariness: 0.0,
            initiative_base: s.initiative_base,
            initiative,
            intel_tech: s.intel_tech.clamp(1, 10),
            openness: s
                .openness
                .unwrap_or_else(|| s.government.default_openness())
                .clamp(0.0, 1.0),
            intel_capacity: intel_capacity.max(0.0),
            trade_exposure: s.trade_exposure.max(0.0),
            trade: 0.0,
            trade_lost: 0.0,
            sanction_adaptation: 0.0,
            adjustment_shock: 0.0,
            aid_in: 0.0,
            aid_out: 0.0,
            alignment: s.alignment,
            personality: s.personality,
            priors: s.priors,
            reflexes: s.reflexes,
            area: s.area,
            tier: s.tier,
        }
    }

    /// Military power (see [`Forces::power`]).
    pub fn power(&self) -> f64 {
        self.forces.power()
    }

    /// Transferred strength from `supplier` still in service.
    pub fn arms_from(&self, supplier: CountryId) -> f64 {
        self.arms_origin
            .iter()
            .find(|(s, _)| *s == supplier)
            .map_or(0.0, |o| o.1)
    }

    pub fn gdp_per_capita(&self) -> f64 {
        self.gdp / self.population.max(1e-9)
    }

    /// Affinity toward another country from alignment and government type,
    /// −1..1: same bloc +1, rival blocs −1, otherwise +0.5 for the same
    /// government type.
    pub fn affinity(&self, other: &Country) -> f64 {
        match (&self.alignment, &other.alignment) {
            (Some(a), Some(b)) if a == b => 1.0,
            (Some(_), Some(_)) => -1.0,
            _ if self.government == other.government => 0.5,
            _ => 0.0,
        }
    }

    /// True if both carry the same alignment tag.
    pub fn shares_alignment(&self, other: &Country) -> bool {
        matches!((&self.alignment, &other.alignment), (Some(a), Some(b)) if a == b)
    }

    /// Debt as a share of annual GDP (one turn = one quarter).
    pub fn debt_ratio(&self) -> f64 {
        self.debt / (self.gdp * 4.0).max(1e-9)
    }
}
