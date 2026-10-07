use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::country::{Country, CountrySetup};
use crate::diplomacy::Diplomacy;
use crate::events::EventTemplate;
use crate::ids::CountryId;
use crate::ledger::Ledger;
use crate::opinion::OpinionBook;
use crate::tension::TensionBook;
use crate::war::Wars;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldParams {
    pub start_year: i32,
    /// Quarterly base interest rate before `financial_weight` discounts.
    pub base_interest_rate: f64,
    /// Half-width of the seeded quarterly growth shock.
    pub growth_shock: f64,
    /// Coverage needed to see a covert act when it happens (era data,
    /// DESIGN §21.1).
    pub covert_visibility: f64,
}

/// The canonical, authoritative game state. AI code never sees this type
/// directly; it receives an [`crate::view::ObserverView`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldState {
    pub seed: u64,
    pub turn: u32,
    pub params: WorldParams,
    pub countries: Vec<Country>,
    pub opinions: OpinionBook,
    pub diplomacy: Diplomacy,
    pub tension: TensionBook,
    /// Power-weighted mean tension, 0–100 (DESIGN §11.1).
    pub global_tension: f64,
    pub ledger: Ledger,
    #[serde(default)]
    pub wars: Wars,
    /// Scenario event templates (data) and their cooldowns
    /// (template, subject, object, resting until turn).
    #[serde(default)]
    pub event_templates: Vec<EventTemplate>,
    #[serde(default)]
    pub event_cooldowns: Vec<(u16, CountryId, Option<CountryId>, u32)>,
    /// World energy market (DESIGN §2.4). Inactive when no country has
    /// energy capacity.
    #[serde(default)]
    pub energy: crate::commodity::EnergyMarket,
    /// Standing claims (data): territorial or financial quarrels a state
    /// may go to war to settle. Public. A war the claimant wins settles it.
    #[serde(default)]
    pub claims: Vec<Claim>,
    /// World interest rate and the reserve holder's monetary stance (P7).
    #[serde(default)]
    pub money: crate::money::MoneyMarket,
    rng: ChaCha8Rng,
}

/// A standing claim by one state on another (a border, a province, a debt).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub by: CountryId,
    pub against: CountryId,
    /// How much the claimant cares, 0–1 (data).
    pub weight: f64,
}

impl WorldState {
    /// The weight of `by`'s claim on `against`, if any.
    pub fn claim(&self, by: CountryId, against: CountryId) -> f64 {
        self.claims
            .iter()
            .filter(|c| c.by == by && c.against == against)
            .map(|c| c.weight)
            .fold(0.0, f64::max)
    }

    pub fn new(seed: u64, params: WorldParams, setups: Vec<CountrySetup>) -> Result<Self, String> {
        if setups.is_empty() {
            return Err("scenario has no countries".into());
        }
        if setups.len() > u16::MAX as usize {
            return Err("too many countries".into());
        }
        let countries: Vec<Country> = setups
            .into_iter()
            .enumerate()
            .map(|(i, s)| Country::from_setup(CountryId(i as u16), s))
            .collect();
        let n = countries.len();
        Ok(WorldState {
            seed,
            turn: 0,
            params,
            countries,
            opinions: OpinionBook::new(n),
            diplomacy: Diplomacy::default(),
            tension: TensionBook::new(n),
            global_tension: 0.0,
            ledger: Ledger::default(),
            wars: Wars::default(),
            event_templates: Vec::new(),
            event_cooldowns: Vec::new(),
            energy: Default::default(),
            claims: Vec::new(),
            money: Default::default(),
            rng: ChaCha8Rng::seed_from_u64(seed),
        })
    }

    pub fn ids(&self) -> impl Iterator<Item = CountryId> + '_ {
        self.countries.iter().map(|c| c.id)
    }

    pub fn country(&self, id: CountryId) -> &Country {
        &self.countries[id.index()]
    }

    pub fn country_mut(&mut self, id: CountryId) -> &mut Country {
        &mut self.countries[id.index()]
    }

    pub fn find(&self, code: &str) -> Option<CountryId> {
        self.countries.iter().find(|c| c.code == code).map(|c| c.id)
    }

    pub fn year(&self) -> f64 {
        self.params.start_year as f64 + self.turn as f64 / 4.0
    }

    /// Uniform draw in [0, 1) from the world's seeded RNG.
    pub fn roll_unit(&mut self) -> f64 {
        (self.rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform draw in [-half_width, half_width).
    pub fn roll_symmetric(&mut self, half_width: f64) -> f64 {
        (self.roll_unit() * 2.0 - 1.0) * half_width
    }
}
