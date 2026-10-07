//! BRINK simulation core.
//!
//! Owns the canonical game state and turn resolution. This crate is pure and
//! deterministic: no I/O, no threads, no wall-clock time, no OS randomness.
//! All randomness flows from the seeded RNG in [`world::WorldState`], and all
//! iteration happens in `CountryId` order.
//!
//! Engine code must never branch on country identity (DESIGN §14.1 rule 5);
//! countries differ only through data and state.

pub mod commodity;
pub mod country;
pub mod diplomacy;
pub mod domestic;
pub mod economy;
pub mod events;
pub mod grudge;
pub mod ids;
pub mod initiative;
pub mod money;
pub mod intel;
pub mod ledger;
pub mod nuclear;
pub mod opinion;
pub mod orders;
pub mod programme;
pub mod reflex;
pub mod region;
pub mod reputation;
pub mod tension;
pub mod trade;
pub mod transition;
pub mod turn;
pub mod view;
pub mod war;
pub mod world;

pub use commodity::{EnergyMarket, ProductionPolicy};
pub use country::{
    BudgetShares, Country, CountrySetup, ForceMix, ForcePool, Forces, Government, Mobilization, Personality, PoolKind,
    ReputationPriors, Tier,
};
pub use diplomacy::{
    Loan,
    DiplomaticEvent, Proposal, ProposalId, Sanction, Stream, StreamKind, Treaty, TreatyId, TreatyKind,
};
pub use ids::CountryId;
pub use ledger::{CauseCode, EntryKind, Grade, LedgerEntry, LedgerLog, NormTag, Visibility};
pub use orders::{Order, OrderSet};
pub use reflex::{Condition, DecisionKind, Gate, ProposalFamily, Reflex, ReflexSet, Trait};
pub use reputation::RepKind;
pub use transition::TransitionKind;
pub use turn::{resolve_turn, TurnReport};
pub use view::{observe, Estimate, ForeignView, ObserverView};
pub use war::{Involvement, PeaceReason, PeaceResult, Side, War, WarAim, WarId, Wars};
pub use money::MonetaryStance;
pub use world::{Claim, WorldParams, WorldState};
