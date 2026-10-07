use serde::{Deserialize, Serialize};

use crate::country::{BudgetShares, Mobilization};
use crate::diplomacy::{ProposalId, TreatyId, TreatyKind};
use crate::ids::CountryId;
use crate::war::{Side, WarAim, WarId};

/// A standing-setting change or action submitted during planning.
/// Initiative costs are defined in [`crate::diplomacy::initiative_cost`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Order {
    // Standing settings (free).
    /// New budget target; actual shares move toward it with inertia.
    SetBudget(BudgetShares),
    /// Deficit spending as a share of revenue, clamped to 0..=MAX_DEFICIT_RATIO.
    SetDeficit(f64),
    /// Energy production policy (scenario P5): Restrain / Normal / Flood.
    SetEnergyPolicy(crate::commodity::ProductionPolicy),
    /// The reserve-currency holder's monetary stance (P7). Free; ignored
    /// from anyone else.
    SetMonetaryStance(crate::money::MonetaryStance),

    // Diplomatic actions.
    ProposeTreaty {
        to: CountryId,
        kind: TreatyKind,
    },
    /// Answer a proposal made in an earlier turn. Accepting a
    /// commitment-creating treaty costs 1 Initiative; refusing is free.
    Respond {
        proposal: ProposalId,
        accept: bool,
    },
    CancelTreaty {
        treaty: TreatyId,
    },
    IssueGuarantee {
        to: CountryId,
    },
    Sanction {
        target: CountryId,
    },
    LiftSanction {
        target: CountryId,
    },
    /// One-off aid paid from this turn's spending pool.
    Aid {
        to: CountryId,
        amount: f64,
    },
    /// Standing aid stream per turn (replaces the amount if one exists).
    StartStream {
        to: CountryId,
        amount: f64,
    },
    StopStream {
        to: CountryId,
    },
    /// One-off transfer of military strength points (DESIGN §21.6). The
    /// recipient keeps them permanently.
    ArmsTransfer {
        to: CountryId,
        amount: f64,
    },
    /// Standing arms supply: strength points per turn. A covert stream is
    /// seen only by the parties and observers with enough coverage of the
    /// supplier, and risks exposure each turn.
    StartArmsStream {
        to: CountryId,
        amount: f64,
        covert: bool,
    },
    StopArmsStream {
        to: CountryId,
    },
    /// Standing arms sale (D58): strength points per turn, paid for by the
    /// buyer at the market price. Same stream rules as a gift (permanent,
    /// route, covert exposure, both-sides ledger).
    SellArms {
        to: CountryId,
        amount: f64,
        covert: bool,
    },
    /// The buyer ends a purchase.
    CancelArmsPurchase {
        from: CountryId,
    },
    Denounce {
        target: CountryId,
    },
    Mediate {
        a: CountryId,
        b: CountryId,
    },

    // War (DESIGN §6.3–6.6, §11.2).
    /// Costs 2 Initiative. Without a casus belli (tension ≥ 40) it costs
    /// Legitimacy.
    DeclareWar {
        target: CountryId,
        aim: WarAim,
    },
    /// Enter a war at band 3 (limited) or 4 (major). Costs 1 Initiative.
    JoinWar {
        war: WarId,
        side: Side,
        band: u8,
    },
    /// A non-leader withdraws (free; the ledger records it).
    LeaveWar {
        war: WarId,
    },
    /// A leader offers peace; stays open two turns.
    OfferPeace {
        war: WarId,
    },
    /// Stepping up costs 1 Initiative; stepping down is free.
    SetMobilization(Mobilization),
    /// A limited nuclear strike on the forces of `target`, an enemy in one
    /// of our wars (DESIGN §11.4, v0.1: the one use action). 1 Initiative.
    NuclearStrike {
        target: CountryId,
    },
    // Covert programmes (scenario P10).
    /// Start a covert arsenal programme (1 Initiative).
    StartProgramme,
    StopProgramme,
    /// Make an undeclared arsenal public (1 Initiative).
    DeclareArsenal,
    /// Give up arsenal and programme for trust (1 Initiative).
    DiscloseAndDismantle,
    // Regime transition (scenario P3): answers to a transition crisis.
    Reform,
    Crackdown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderSet {
    pub country: CountryId,
    pub orders: Vec<Order>,
}
