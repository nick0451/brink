//! Scenario data (RON) → canonical `WorldState`. This crate maps data onto
//! sim-core types and validates it; it must not add engine behaviour.

use std::path::Path;

use serde::Deserialize;
use sim_core::opinion::{Memory, OpinionModifier};
use sim_core::{
    BudgetShares, CountrySetup, ForceMix, Government, Order, OrderSet, Personality, ReputationPriors, TreatyKind,
    WorldParams, WorldState,
};

fn default_growth_shock() -> f64 {
    0.002
}
fn default_initiative_base() -> u8 {
    3
}
fn default_true() -> bool {
    true
}
fn default_intel_tech() -> u8 {
    3
}
/// Military tech treated as quality 1.0× (`sim_core::country::QUALITY_REF`).
fn default_military_tech() -> u8 {
    5
}
fn default_covert_visibility() -> f64 {
    60.0
}
fn default_one() -> f64 {
    1.0
}
fn default_trade_exposure() -> f64 {
    1.0
}
fn default_opinion_source() -> String {
    "historical relations".into()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename = "Scenario")]
pub struct ScenarioDef {
    pub name: String,
    pub seed: u64,
    pub start_year: i32,
    pub turns: u32,
    pub base_interest_rate: f64,
    #[serde(default = "default_growth_shock")]
    pub growth_shock: f64,
    #[serde(default = "default_covert_visibility")]
    pub covert_visibility: f64,
    pub countries: Vec<CountryDef>,
    #[serde(default)]
    pub opinions: Vec<OpinionDef>,
    #[serde(default)]
    pub treaties: Vec<TreatyDef>,
    #[serde(default)]
    pub sanctions: Vec<SanctionDef>,
    #[serde(default)]
    pub streams: Vec<StreamDef>,
    /// Overrides for starting tension; other pairs start at their baseline.
    #[serde(default)]
    pub tensions: Vec<TensionDef>,
    /// Seeded historical ledger entries (DESIGN §21.1).
    #[serde(default)]
    pub history: Vec<HistoryDef>,
    /// Standing claims: (claimant, target, weight 0-1).
    #[serde(default)]
    pub claims: Vec<ClaimDef>,
    /// Reflex sets available to countries (design/behaviour-and-voice.md §1).
    #[serde(default)]
    pub reflex_sets: Vec<sim_core::ReflexSet>,
    /// Reflex library files, relative to the scenario file; merged into
    /// `reflex_sets` by [`load`].
    #[serde(default)]
    pub include_reflexes: Vec<String>,
    /// Seeded ± jitter applied to every personality trait at campaign start
    /// (DESIGN §14.9 variety source). 0 = off (fixtures); 0.1 for campaigns.
    #[serde(default)]
    pub personality_jitter: f64,
    /// Event templates (DESIGN principle 8).
    #[serde(default)]
    pub events: Vec<sim_core::events::EventTemplate>,
    /// Event template files, relative to the scenario file.
    #[serde(default)]
    pub include_events: Vec<String>,
    /// Starting world energy price relative to its base (1 = balanced).
    #[serde(default = "default_one")]
    pub energy_price: f64,
    /// Energy supply from producers not modelled as countries.
    #[serde(default)]
    pub energy_outside_supply: f64,
    /// Starting world monetary stance (P7).
    #[serde(default)]
    pub monetary_stance: sim_core::MonetaryStance,
    /// Starting world inflation level (P7; 1 = severe 1970s inflation).
    #[serde(default)]
    pub inflation: f64,
    /// Countries a human may choose to play (codes; front-end only, D105
    /// #7). Empty = every active `Playable` country. Presentation data: the
    /// simulation never reads it.
    #[serde(default)]
    pub selectable: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HistoryDef {
    pub actor: String,
    pub counterpart: String,
    pub kind: sim_core::EntryKind,
    pub grade: Option<sim_core::Grade>,
    pub turns_ago: u32,
    #[serde(default)]
    pub cost_paid: f64,
    #[serde(default = "default_one")]
    pub weight: f64,
    #[serde(default)]
    pub local: bool,
    pub cause: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TreatyDef {
    pub kind: TreatyKind,
    /// For directed treaties: guarantor / based-forces owner.
    pub a: String,
    /// For directed treaties: guaranteed state / host.
    pub b: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SanctionDef {
    pub by: String,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StreamDef {
    pub from: String,
    pub to: String,
    pub amount: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClaimDef {
    pub by: String,
    pub against: String,
    pub weight: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TensionDef {
    pub a: String,
    pub b: String,
    pub value: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CountryDef {
    pub id: String,
    pub name: String,
    pub government: Government,
    pub population: f64,
    pub gdp: f64,
    pub tax_rate: f64,
    #[serde(default)]
    pub debt: f64,
    /// Foreign reserves (issue 28), money units.
    #[serde(default)]
    pub reserves: f64,
    #[serde(default)]
    pub financial_weight: f64,
    pub budget: BudgetShares,
    #[serde(default)]
    pub deficit_ratio: f64,
    pub military: f64,
    #[serde(default)]
    pub force_mix: ForceMix,
    #[serde(default = "default_military_tech")]
    pub military_tech: u8,
    #[serde(default = "default_one")]
    pub military_cost: f64,
    #[serde(default)]
    pub readiness: Option<f64>,
    /// Energy capacity per turn; 0 = not modelled (DESIGN §2.4).
    #[serde(default)]
    pub energy_capacity: f64,
    /// Strategic arsenal level 0–3 (DESIGN §11.4).
    #[serde(default)]
    pub arsenal: u8,
    /// Undeclared arsenal (P10). Default: declared.
    #[serde(default = "default_true")]
    pub arsenal_declared: bool,
    /// Covert programme already running, with its progress (P10).
    #[serde(default)]
    pub programme: Option<f64>,
    /// Personality after a reform (P3; B.18 transition vectors).
    #[serde(default)]
    pub reform_personality: Option<Personality>,
    /// Turn the current rulers came to power by revolution or conquest
    /// (negative = before the scenario starts); omitted otherwise.
    #[serde(default)]
    pub took_power: Option<i32>,
    /// Structural growth adjustment per quarter (fitted: demographics,
    /// export-led catch-up, reform era); persists.
    #[serde(default)]
    pub growth_modifier: f64,
    /// Command-economy stagnation per quarter (negative); a reform ends it.
    #[serde(default)]
    pub command_drag: f64,
    /// Export-oriented defence industry, 0–1 (D58).
    #[serde(default)]
    pub arms_industry: f64,
    /// Peripheries with loyalty and successors (P2).
    #[serde(default)]
    pub regions: Vec<sim_core::region::Region>,
    /// A successor state that starts inactive (P2).
    #[serde(default)]
    pub dormant: bool,
    pub stability: f64,
    pub legitimacy: f64,
    #[serde(default = "default_initiative_base")]
    pub initiative_base: u8,
    #[serde(default = "default_intel_tech")]
    pub intel_tech: u8,
    #[serde(default)]
    pub openness: Option<f64>,
    #[serde(default)]
    pub intel_capacity: Option<f64>,
    #[serde(default = "default_trade_exposure")]
    pub trade_exposure: f64,
    #[serde(default)]
    pub alignment: Option<String>,
    #[serde(default)]
    pub personality: Personality,
    #[serde(default)]
    pub priors: ReputationPriors,
    /// Reflex set ids to attach.
    #[serde(default)]
    pub reflex_sets: Vec<String>,
    /// Extra reflexes defined inline.
    #[serde(default)]
    pub reflexes: Vec<sim_core::Reflex>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub tier: sim_core::Tier,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpinionDef {
    pub from: String,
    pub to: String,
    pub value: f64,
    #[serde(default)]
    pub decay: f64,
    #[serde(default = "default_opinion_source")]
    pub source: String,
}

pub fn parse(text: &str) -> Result<ScenarioDef, String> {
    ron::from_str(text).map_err(|e| format!("scenario parse error: {e}"))
}

pub fn load(path: impl AsRef<Path>) -> Result<ScenarioDef, String> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut def = parse(&text)?;
    let dir = path.parent().unwrap_or(Path::new("."));
    for include in def.include_reflexes.clone() {
        def.reflex_sets.extend(load_reflex_library(dir.join(include))?);
    }
    for include in def.include_events.clone() {
        let path = dir.join(include);
        let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let events: Vec<sim_core::events::EventTemplate> =
            ron::from_str(&text).map_err(|e| format!("events {} parse error: {e}", path.display()))?;
        def.events.extend(events);
    }
    Ok(def)
}

/// Load a reflex library file: a RON list of reflex sets.
pub fn load_reflex_library(path: impl AsRef<Path>) -> Result<Vec<sim_core::ReflexSet>, String> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let sets: Vec<sim_core::ReflexSet> =
        ron::from_str(&text).map_err(|e| format!("reflex library {} parse error: {e}", path.display()))?;
    for set in &sets {
        for r in &set.reflexes {
            r.validate().map_err(|e| format!("set {}: {e}", set.id))?;
        }
    }
    Ok(sets)
}

/// Collect and validate a country's reflexes from its sets and inline list.
fn resolve_reflexes(def: &ScenarioDef, c: &CountryDef) -> Result<Vec<sim_core::Reflex>, String> {
    let mut out: Vec<sim_core::Reflex> = Vec::new();
    for set_id in &c.reflex_sets {
        let set = def
            .reflex_sets
            .iter()
            .find(|s| &s.id == set_id)
            .ok_or_else(|| format!("{}: unknown reflex set {set_id}", c.id))?;
        out.extend(set.reflexes.iter().cloned());
    }
    out.extend(c.reflexes.iter().cloned());
    let mut ids = std::collections::BTreeSet::new();
    for r in &out {
        r.validate().map_err(|e| format!("{}: {e}", c.id))?;
        if !ids.insert(r.id.as_str()) {
            return Err(format!("{}: duplicate reflex id {}", c.id, r.id));
        }
    }
    Ok(out)
}

/// Deterministic value in [-1, 1) from (seed, country, trait); independent
/// of the world RNG so jitter never shifts other random streams.
fn unit_hash(seed: u64, country: usize, t: u64) -> f64 {
    let mix = |mut x: u64| {
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    };
    let x = mix(mix(mix(seed ^ 0x6a09_e667) ^ country as u64) ^ t);
    (x >> 11) as f64 / (1u64 << 52) as f64 - 1.0
}

/// Seeded personality jitter, clamped to 0–1.
pub fn jitter(p: Personality, amount: f64, seed: u64, country: usize) -> Personality {
    if amount <= 0.0 {
        return p;
    }
    let j = |v: f64, t: u64| (v + amount * unit_hash(seed, country, t)).clamp(0.0, 1.0);
    Personality {
        aggression: j(p.aggression, 1),
        risk: j(p.risk, 2),
        paranoia: j(p.paranoia, 3),
        loyalty: j(p.loyalty, 4),
        greed: j(p.greed, 5),
        ideology: j(p.ideology, 6),
        opportunism: j(p.opportunism, 7),
    }
}

/// Build the starting world. `seed` overrides the scenario's own seed.
pub fn build(def: &ScenarioDef, seed: Option<u64>) -> Result<WorldState, String> {
    let mut seen = std::collections::BTreeSet::new();
    for c in &def.countries {
        if !seen.insert(c.id.as_str()) {
            return Err(format!("duplicate country id {}", c.id));
        }
    }

    let params = WorldParams {
        start_year: def.start_year,
        base_interest_rate: def.base_interest_rate,
        growth_shock: def.growth_shock,
        covert_visibility: def.covert_visibility,
    };
    let setups = def
        .countries
        .iter()
        .enumerate()
        .map(|(i, c)| -> Result<CountrySetup, String> {
            Ok(CountrySetup {
                code: c.id.clone(),
                name: c.name.clone(),
                government: c.government,
                population: c.population,
                gdp: c.gdp,
                tax_rate: c.tax_rate,
                debt: c.debt,
                reserves: c.reserves,
                financial_weight: c.financial_weight,
                budget: c.budget,
                deficit_ratio: c.deficit_ratio,
                military: c.military,
                force_mix: c.force_mix,
                military_tech: c.military_tech,
                military_cost: c.military_cost,
                readiness: c.readiness,
                energy_capacity: c.energy_capacity,
                arsenal: c.arsenal,
                arsenal_declared: c.arsenal_declared,
                programme: c.programme,
                reform_personality: c.reform_personality,
                took_power: c.took_power,
                growth_modifier: c.growth_modifier,
                command_drag: c.command_drag,
                arms_industry: c.arms_industry,
                regions: c.regions.clone(),
                dormant: c.dormant,
                stability: c.stability,
                legitimacy: c.legitimacy,
                initiative_base: c.initiative_base,
                intel_tech: c.intel_tech,
                openness: c.openness,
                intel_capacity: c.intel_capacity,
                trade_exposure: c.trade_exposure,
                alignment: c.alignment.clone(),
                personality: jitter(c.personality, def.personality_jitter, seed.unwrap_or(def.seed), i),
                priors: c.priors,
                reflexes: resolve_reflexes(def, c)?,
                area: c.area.clone(),
                tier: c.tier,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut world = WorldState::new(seed.unwrap_or(def.seed), params, setups)?;
    for e in &def.events {
        e.validate()?;
    }
    world.event_templates = def.events.clone();
    world.energy.price = def.energy_price;
    world.energy.outside_supply = def.energy_outside_supply;
    world.money.stance = def.monetary_stance;
    world.money.inflation = def.inflation;
    world.money.rate = sim_core::money::target_rate(params_base(&world), def.monetary_stance, 0.0);

    for o in &def.opinions {
        let from = world
            .find(&o.from)
            .ok_or_else(|| format!("opinion from unknown country {}", o.from))?;
        let to = world
            .find(&o.to)
            .ok_or_else(|| format!("opinion to unknown country {}", o.to))?;
        if from == to {
            return Err(format!("self-opinion for {}", o.from));
        }
        world.opinions.add(
            from,
            to,
            OpinionModifier {
                source: o.source.clone(),
                value: o.value,
                decay: o.decay,
                // Starting hostility is history, not a standing grievance:
                // it fades unless renewed (D81, `sim_core::grudge`).
                memory: (o.value < 0.0 && o.decay == 0.0).then(Memory::default),
            },
        );
    }
    let id = |world: &WorldState, code: &str| world.find(code).ok_or_else(|| format!("unknown country {code}"));
    for t in &def.treaties {
        let (a, b) = (id(&world, &t.a)?, id(&world, &t.b)?);
        if a == b {
            return Err(format!("treaty of {} with itself", t.a));
        }
        if world.diplomacy.find_treaty(t.kind, a, b).is_some() {
            return Err(format!("duplicate treaty {:?} {}-{}", t.kind, t.a, t.b));
        }
        sim_core::diplomacy::sign(&mut world, t.kind, a, b);
    }
    // Starting sanctions and streams go through the normal action rules so
    // their opinion and tension effects match play.
    let mut setup_orders: Vec<OrderSet> = Vec::new();
    for s in &def.sanctions {
        let (by, target) = (id(&world, &s.by)?, id(&world, &s.target)?);
        setup_orders.push(OrderSet {
            country: by,
            orders: vec![Order::Sanction { target }],
        });
    }
    for s in &def.streams {
        let (from, to) = (id(&world, &s.from)?, id(&world, &s.to)?);
        setup_orders.push(OrderSet {
            country: from,
            orders: vec![Order::StartStream { to, amount: s.amount }],
        });
    }
    for set in setup_orders {
        for order in set.orders {
            sim_core::diplomacy::apply(&mut world, set.country, order)?;
        }
    }
    // Scenario setup must not leave deep-integration shocks pending.
    for c in &mut world.countries {
        c.adjustment_shock = 0.0;
    }
    sim_core::tension::initialize(&mut world);
    for t in &def.tensions {
        let (a, b) = (id(&world, &t.a)?, id(&world, &t.b)?);
        world.tension.set(a, b, t.value);
    }
    for c in &def.claims {
        let (by, against) = (id(&world, &c.by)?, id(&world, &c.against)?);
        world.claims.push(sim_core::Claim {
            by,
            against,
            weight: c.weight.clamp(0.0, 1.0),
        });
    }
    world.global_tension = sim_core::tension::global(&world);
    for h in &def.history {
        let entry = sim_core::LedgerEntry {
            id: sim_core::ledger::EntryId(0),
            turn: 0,
            actor: id(&world, &h.actor)?,
            counterpart: id(&world, &h.counterpart)?,
            kind: h.kind,
            grade: h.grade,
            cost_paid: h.cost_paid.clamp(0.0, 1.0),
            weight: h.weight,
            local: h.local,
            visibility: sim_core::Visibility::Public,
            seen_by: Default::default(),
            cause: h.cause.clone(),
            cause_code: sim_core::CauseCode::Historical,
            statement_id: None,
        };
        sim_core::ledger::seed_history(&mut world, entry, h.turns_ago);
    }
    Ok(world)
}

fn params_base(world: &WorldState) -> f64 {
    world.params.base_interest_rate
}
