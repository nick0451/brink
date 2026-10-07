//! Treaties, proposals, sanctions, support streams and diplomatic actions
//! (DESIGN §7). Actions cost Initiative; accepting a commitment-creating
//! proposal costs 1, refusing costs only Opinion (DESIGN §7.2, §21).
//!
//! Every resolved action emits a [`DiplomaticEvent`]; the Event Ledger
//! (build step 4) is built from these.

use serde::{Deserialize, Serialize};

use crate::country::Mobilization;
use crate::ids::CountryId;
use crate::opinion::OpinionModifier;
use crate::orders::Order;
use crate::war::{PeaceReason, PeaceResult, Side, WarAim, WarId};
use crate::world::WorldState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TreatyId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreatyKind {
    /// `deep` = economic integration (DESIGN §7.2).
    Trade {
        deep: bool,
    },
    NonAggression,
    DefensiveAlliance,
    /// Directed: `a` guarantees `b`. Issued unilaterally.
    Guarantee,
    /// Directed: `a`'s forces are based in `b`'s territory.
    Basing,
}

impl TreatyKind {
    /// Commitment-creating treaties cost 1 Initiative to accept (DESIGN §7.2).
    pub fn creates_commitment(self) -> bool {
        !matches!(self, TreatyKind::Trade { .. })
    }

    fn family(self) -> u8 {
        match self {
            TreatyKind::Trade { .. } => 0,
            TreatyKind::NonAggression => 1,
            TreatyKind::DefensiveAlliance => 2,
            TreatyKind::Guarantee => 3,
            TreatyKind::Basing => 4,
        }
    }

    fn is_directed(self) -> bool {
        matches!(self, TreatyKind::Guarantee | TreatyKind::Basing)
    }

    /// Opinion each party (for directed treaties: the beneficiary) gains
    /// toward the other while the treaty is in force.
    fn opinion_bonus(self) -> f64 {
        match self {
            TreatyKind::Trade { deep: false } => 5.0,
            TreatyKind::Trade { deep: true } => 10.0,
            TreatyKind::NonAggression => 5.0,
            TreatyKind::DefensiveAlliance => 20.0,
            TreatyKind::Guarantee => 15.0,
            TreatyKind::Basing => 5.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Treaty {
    pub id: TreatyId,
    pub kind: TreatyKind,
    pub a: CountryId,
    pub b: CountryId,
    pub signed: u32,
}

impl Treaty {
    pub fn involves(&self, c: CountryId) -> bool {
        self.a == c || self.b == c
    }

    pub fn other(&self, c: CountryId) -> Option<CountryId> {
        match c {
            c if c == self.a => Some(self.b),
            c if c == self.b => Some(self.a),
            _ => None,
        }
    }

    fn matches(&self, kind: TreatyKind, a: CountryId, b: CountryId) -> bool {
        if self.kind.family() != kind.family() {
            return false;
        }
        if kind.is_directed() {
            self.a == a && self.b == b
        } else {
            (self.a == a && self.b == b) || (self.a == b && self.b == a)
        }
    }
}

/// A treaty offer. Answerable during the turn after it was made, then lapses.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub from: CountryId,
    pub to: CountryId,
    /// For `Basing`, the proposer's forces would be based in `to`. For
    /// `Guarantee`, the proposer asks `to` to guarantee it.
    pub kind: TreatyKind,
    pub turn: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sanction {
    pub by: CountryId,
    pub target: CountryId,
    pub since: u32,
}

/// What a standing stream carries (DESIGN §7.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StreamKind {
    /// Money, paid from the patron's spending pool.
    #[default]
    Aid,
    /// Military strength points, moved from the patron's force pools.
    /// Permanent: the client keeps them (DESIGN §21.6).
    Arms,
}

/// A standing support stream: `amount` per turn (money for aid, strength
/// points for arms).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stream {
    pub from: CountryId,
    pub to: CountryId,
    pub amount: f64,
    pub since: u32,
    #[serde(default)]
    pub kind: StreamKind,
    /// Covert streams are hidden from observers without enough coverage of
    /// the supplier and may be exposed (DESIGN §8.4).
    #[serde(default)]
    pub covert: bool,
    /// An arms sale: the recipient pays the market price (D58).
    #[serde(default)]
    pub sale: bool,
}

impl Stream {
    /// Money-equivalent size per turn, so aid and arms compare on one scale.
    pub fn value(&self) -> f64 {
        arms_value(self.kind, self.amount)
    }
}

/// Money-equivalent of an amount of a stream kind: arms strength is valued
/// at what it would cost to build.
pub fn arms_value(kind: StreamKind, amount: f64) -> f64 {
    match kind {
        StreamKind::Aid => amount,
        StreamKind::Arms => amount / crate::economy::MILITARY_CONVERSION,
    }
}

/// A war loan (issue 24, D95): one-off aid paid to a state at war is lent,
/// not given. The money still goes into the recipient's spending pool; its
/// debt rises by the amount and the funder holds this claim (public: the
/// pledge was public). The debtor's interest on its share of the debt is
/// paid to the creditor; the principal stands until the creditor forgives
/// it ([`Order::ForgiveDebt`]).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Loan {
    pub creditor: CountryId,
    pub debtor: CountryId,
    pub amount: f64,
    /// Turn of the first loan in this claim.
    pub since: u32,
    /// The creditor has held the debtor to it (`HoldDebt`): while the
    /// debtor is in distress, a harmful standing act (D97).
    #[serde(default)]
    pub held: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Diplomacy {
    pub treaties: Vec<Treaty>,
    pub proposals: Vec<Proposal>,
    pub sanctions: Vec<Sanction>,
    pub streams: Vec<Stream>,
    /// One-off aid ordered this turn, paid during the economy phase.
    pub pending_aid: Vec<(CountryId, CountryId, f64)>,
    /// One-off arms transfers ordered this turn (strength points), moved
    /// during the economy phase.
    #[serde(default)]
    pub pending_arms: Vec<(CountryId, CountryId, f64)>,
    /// War loans outstanding, one claim per (creditor, debtor) (issue 24).
    #[serde(default)]
    pub loans: Vec<Loan>,
    next_treaty: u32,
    next_proposal: u32,
}

impl Diplomacy {
    pub fn find_treaty(&self, kind: TreatyKind, a: CountryId, b: CountryId) -> Option<&Treaty> {
        self.treaties.iter().find(|t| t.matches(kind, a, b))
    }

    pub fn has_treaty(&self, kind: TreatyKind, a: CountryId, b: CountryId) -> bool {
        self.find_treaty(kind, a, b).is_some()
    }

    pub fn allies_of(&self, c: CountryId) -> impl Iterator<Item = CountryId> + '_ {
        self.treaties
            .iter()
            .filter(move |t| t.kind == TreatyKind::DefensiveAlliance && t.involves(c))
            .filter_map(move |t| t.other(c))
    }

    /// Trade-volume multiplier for a pair from any trade agreement.
    pub fn trade_multiplier(&self, a: CountryId, b: CountryId) -> f64 {
        match self
            .find_treaty(TreatyKind::Trade { deep: false }, a, b)
            .map(|t| t.kind)
        {
            Some(TreatyKind::Trade { deep: true }) => DEEP_TRADE_MULTIPLIER,
            Some(_) => SHALLOW_TRADE_MULTIPLIER,
            None => 1.0,
        }
    }

    pub fn is_sanctioned_by(&self, target: CountryId, by: CountryId) -> bool {
        self.sanctions.iter().any(|s| s.target == target && s.by == by)
    }

    pub fn is_sanctioned(&self, target: CountryId) -> bool {
        self.sanctions.iter().any(|s| s.target == target)
    }

    /// Add `amount` to `creditor`'s claim on `debtor` (issue 24).
    pub fn lend(&mut self, creditor: CountryId, debtor: CountryId, amount: f64, turn: u32) {
        match self.loans.iter_mut().find(|l| l.creditor == creditor && l.debtor == debtor) {
            Some(l) => l.amount += amount,
            None => self.loans.push(Loan {
                creditor,
                debtor,
                amount,
                since: turn,
                held: false,
            }),
        }
    }
}

pub const SHALLOW_TRADE_MULTIPLIER: f64 = 1.6;
pub const DEEP_TRADE_MULTIPLIER: f64 = 2.5;
/// Temporary Prosperity penalty per unit of `trade_exposure` when a deep
/// agreement is signed; decays by 1 per turn (DESIGN §7.2).
pub const DEEP_ADJUSTMENT_SHOCK: f64 = 6.0;

const SANCTION_TENSION: f64 = 10.0;
/// Opinion the target holds of a sanctioner while the sanction stands.
/// Removed on lifting, leaving a fading "past sanctions" memory.
pub const SANCTION_OPINION: f64 = -25.0;
/// The "past sanctions" memory left by a lift; fades by 1 per turn.
pub const PAST_SANCTION_OPINION: f64 = -10.0;
const DENOUNCE_TENSION: f64 = 5.0;
/// Opinion a debtor gains toward a creditor that forgives its war loan:
/// the aid formula (100 x amount / GDP), capped, fading over ~20 years.
pub const FORGIVENESS_OPINION_CAP: f64 = 20.0;
const FORGIVENESS_DECAY: f64 = 0.25;

/// A debtor in distress (issue 24): at peace, with debt above the level
/// where it drags growth.
pub fn in_distress(state: &WorldState, debtor: CountryId) -> bool {
    let d = state.country(debtor);
    d.active && !state.wars.is_belligerent(debtor) && d.debt_ratio() > crate::economy::DEBT_DRAG_THRESHOLD
}
const MEDIATE_TENSION_RELIEF: f64 = 10.0;

/// Initiative cost of an order. Standing settings are free.
pub fn initiative_cost(state: &WorldState, actor: CountryId, order: &Order) -> u8 {
    match order {
        Order::SetBudget(_) | Order::SetDeficit(_) | Order::SetEnergyPolicy(_) | Order::SetMonetaryStance(_) => 0,
        Order::DeclareWar { .. }
        | Order::JoinWar { .. }
        | Order::LeaveWar { .. }
        | Order::OfferPeace { .. }
        | Order::SetMobilization(_) => crate::war::initiative_cost(state, actor, order),
        Order::NuclearStrike { .. } | Order::StartProgramme | Order::DeclareArsenal | Order::DiscloseAndDismantle => 1,
        Order::StopProgramme | Order::Reform | Order::Crackdown => 0,
        Order::Respond { proposal, accept } => {
            let commitment = state
                .diplomacy
                .proposals
                .iter()
                .find(|p| p.id == *proposal && p.to == actor)
                .is_some_and(|p| p.kind.creates_commitment());
            u8::from(*accept && commitment)
        }
        Order::LiftSanction { .. }
        | Order::ForgiveDebt { .. }
        | Order::HoldDebt { .. }
        | Order::StopStream { .. }
        | Order::StopArmsStream { .. }
        | Order::CancelArmsPurchase { .. } => 0,
        Order::ProposeTreaty { .. }
        | Order::CancelTreaty { .. }
        | Order::IssueGuarantee { .. }
        | Order::Sanction { .. }
        | Order::Aid { .. }
        | Order::StartStream { .. }
        | Order::ArmsTransfer { .. }
        | Order::StartArmsStream { .. }
        | Order::SellArms { .. }
        | Order::Denounce { .. }
        | Order::Mediate { .. } => 1,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DiplomaticEvent {
    /// The reserve-currency holder changed the world monetary stance (P7).
    MonetaryStance {
        by: CountryId,
        stance: crate::money::MonetaryStance,
    },
    Proposed {
        proposal: Proposal,
    },
    TreatySigned {
        treaty: Treaty,
    },
    TreatyCancelled {
        treaty: Treaty,
        by: CountryId,
    },
    ProposalRefused {
        proposal: Proposal,
    },
    ProposalLapsed {
        proposal: Proposal,
    },
    SanctionImposed {
        by: CountryId,
        target: CountryId,
    },
    SanctionLifted {
        by: CountryId,
        target: CountryId,
    },
    AidPledged {
        from: CountryId,
        to: CountryId,
        amount: f64,
    },
    /// One-off arms transfer ordered (strength points).
    ArmsTransferred {
        from: CountryId,
        to: CountryId,
        amount: f64,
    },
    StreamStarted {
        stream: Stream,
    },
    StreamStopped {
        stream: Stream,
    },
    Denounced {
        by: CountryId,
        target: CountryId,
    },
    /// A creditor wrote off its war loan (issue 24).
    DebtForgiven {
        creditor: CountryId,
        debtor: CountryId,
        amount: f64,
    },
    /// A creditor held a debtor in distress to its war loan: a public
    /// hostile act (issue 24).
    DebtHeld {
        creditor: CountryId,
        debtor: CountryId,
        amount: f64,
    },
    Mediated {
        by: CountryId,
        a: CountryId,
        b: CountryId,
    },
    WarDeclared {
        war: WarId,
        attacker: CountryId,
        defender: CountryId,
        aim: WarAim,
        casus_belli: bool,
        broke_pact: bool,
    },
    JoinedWar {
        war: WarId,
        country: CountryId,
        side: Side,
        band: u8,
    },
    LeftWar {
        war: WarId,
        country: CountryId,
    },
    PeaceOffered {
        war: WarId,
        by: CountryId,
    },
    /// A punitive strike the target refused became a limited war.
    WarEscalated {
        war: WarId,
        aim: WarAim,
    },
    PeaceMade {
        war: WarId,
        attacker: CountryId,
        defender: CountryId,
        aim: WarAim,
        result: PeaceResult,
        reason: PeaceReason,
        progress: f64,
    },
    /// One turn of combat on a front.
    FrontReport {
        war: WarId,
        front: u32,
        progress: f64,
        ratio: f64,
        attacker_losses: f64,
        defender_losses: f64,
    },
    Mobilized {
        country: CountryId,
        level: Mobilization,
    },
    CovertExposed {
        from: CountryId,
        to: CountryId,
    },
    /// Nuclear weapons were used (DESIGN §11.4).
    NuclearStrike {
        by: CountryId,
        target: CountryId,
        war: WarId,
    },
    /// A periphery seceded and its successor state became active (P2).
    Secession {
        parent: CountryId,
        successor: CountryId,
    },
    /// A regime faces a transition crisis: Reform or Crackdown (P3).
    TransitionCrisis {
        country: CountryId,
    },
    /// A regime transition happened (reform, crackdown, coup, collapse).
    Transition {
        country: CountryId,
        kind: crate::transition::TransitionKind,
    },
    /// Covert programmes (P10).
    ProgrammeCompleted {
        country: CountryId,
    },
    ProgrammeExposed {
        country: CountryId,
        by: CountryId,
    },
    ArsenalDeclared {
        country: CountryId,
    },
    ArsenalDismantled {
        country: CountryId,
    },
    /// An energy producer changed its production policy (public).
    EnergyPolicy {
        country: CountryId,
        policy: crate::commodity::ProductionPolicy,
    },
    /// A scenario event template fired (index into the world's templates).
    ScenarioEvent {
        template: u16,
        subject: CountryId,
        object: Option<CountryId>,
    },
}

fn modifier(source: impl Into<String>, value: f64, decay: f64) -> OpinionModifier {
    OpinionModifier {
        source: source.into(),
        value,
        decay,
        memory: None,
    }
}

fn treaty_source(id: TreatyId) -> String {
    format!("treaty #{}", id.0)
}

/// Create a treaty and its opinion effects. Used by order resolution and by
/// scenario setup.
pub fn sign(state: &mut WorldState, kind: TreatyKind, a: CountryId, b: CountryId) -> Treaty {
    let id = TreatyId(state.diplomacy.next_treaty);
    state.diplomacy.next_treaty += 1;
    let treaty = Treaty {
        id,
        kind,
        a,
        b,
        signed: state.turn,
    };
    state.diplomacy.treaties.push(treaty);

    let bonus = kind.opinion_bonus();
    state.opinions.add(b, a, modifier(treaty_source(id), bonus, 0.0));
    if !kind.is_directed() {
        state.opinions.add(a, b, modifier(treaty_source(id), bonus, 0.0));
    }
    if kind == (TreatyKind::Trade { deep: true }) {
        for c in [a, b] {
            let country = state.country_mut(c);
            country.adjustment_shock += DEEP_ADJUSTMENT_SHOCK * country.trade_exposure;
        }
    }
    treaty
}

fn remove_treaty(state: &mut WorldState, id: TreatyId) -> Option<Treaty> {
    let pos = state.diplomacy.treaties.iter().position(|t| t.id == id)?;
    let t = state.diplomacy.treaties.remove(pos);
    state.opinions.remove_source(t.b, t.a, &treaty_source(id));
    state.opinions.remove_source(t.a, t.b, &treaty_source(id));
    Some(t)
}

/// Treaty parties `(a, b)` for a proposal from `from` to `to`. A proposed
/// guarantee asks the recipient to guarantee the proposer; a proposed basing
/// treaty puts the proposer's forces in the recipient's territory.
fn treaty_parties(kind: TreatyKind, from: CountryId, to: CountryId) -> (CountryId, CountryId) {
    match kind {
        TreatyKind::Guarantee => (to, from),
        _ => (from, to),
    }
}

/// Resolve one country's diplomatic order. Returns the event, or the reason
/// it was rejected. Initiative has already been checked by the caller.
pub fn apply(state: &mut WorldState, actor: CountryId, order: Order) -> Result<Option<DiplomaticEvent>, String> {
    let n = state.countries.len();
    let valid = |c: CountryId| c.index() < n && c != actor;
    match order {
        Order::SetBudget(_)
        | Order::SetDeficit(_)
        | Order::SetEnergyPolicy(_)
        | Order::SetMonetaryStance(_) => Ok(None),
        war @ (Order::DeclareWar { .. }
        | Order::JoinWar { .. }
        | Order::LeaveWar { .. }
        | Order::OfferPeace { .. }
        | Order::SetMobilization(_)) => crate::war::apply(state, actor, war),
        strike @ Order::NuclearStrike { .. } => crate::nuclear::apply(state, actor, strike),
        p @ (Order::StartProgramme | Order::StopProgramme | Order::DeclareArsenal | Order::DiscloseAndDismantle) => {
            crate::programme::apply(state, actor, p)
        }
        t @ (Order::Reform | Order::Crackdown) => crate::transition::apply(state, actor, t),

        Order::ProposeTreaty { to, kind } => {
            if !valid(to) {
                return Err("invalid treaty partner".into());
            }
            // A proposed guarantee is a request: "guarantee me".
            let (a, b) = treaty_parties(kind, actor, to);
            if let Some(existing) = state.diplomacy.find_treaty(kind, a, b) {
                if existing.kind == kind {
                    return Err("treaty already in force".into());
                }
            }
            let proposal = Proposal {
                id: ProposalId(state.diplomacy.next_proposal),
                from: actor,
                to,
                kind,
                turn: state.turn,
            };
            state.diplomacy.next_proposal += 1;
            state.diplomacy.proposals.push(proposal);
            Ok(Some(DiplomaticEvent::Proposed { proposal }))
        }

        Order::Respond { proposal, accept } => {
            let pos = state
                .diplomacy
                .proposals
                .iter()
                .position(|p| p.id == proposal && p.to == actor && p.turn < state.turn)
                .ok_or("no such open proposal")?;
            let p = state.diplomacy.proposals.remove(pos);
            if !accept {
                state
                    .opinions
                    .add(p.from, actor, modifier("proposal refused", -5.0, 1.0));
                return Ok(Some(DiplomaticEvent::ProposalRefused { proposal: p }));
            }
            let (a, b) = treaty_parties(p.kind, p.from, p.to);
            // Upgrading a shallow trade agreement replaces it.
            if let Some(old) = state.diplomacy.find_treaty(p.kind, a, b).copied() {
                if old.kind == p.kind {
                    return Err("treaty already in force".into());
                }
                remove_treaty(state, old.id);
            }
            let treaty = sign(state, p.kind, a, b);
            Ok(Some(DiplomaticEvent::TreatySigned { treaty }))
        }

        Order::CancelTreaty { treaty } => {
            let t = *state
                .diplomacy
                .treaties
                .iter()
                .find(|t| t.id == treaty && t.involves(actor))
                .ok_or("not a party to that treaty")?;
            remove_treaty(state, t.id);
            let other = t.other(actor).expect("party checked above");
            state
                .opinions
                .add(other, actor, modifier("cancelled treaty", -20.0, 1.0));
            Ok(Some(DiplomaticEvent::TreatyCancelled { treaty: t, by: actor }))
        }

        Order::IssueGuarantee { to } => {
            if !valid(to) {
                return Err("invalid guarantee target".into());
            }
            if state.diplomacy.has_treaty(TreatyKind::Guarantee, actor, to) {
                return Err("guarantee already in force".into());
            }
            let treaty = sign(state, TreatyKind::Guarantee, actor, to);
            Ok(Some(DiplomaticEvent::TreatySigned { treaty }))
        }

        Order::Sanction { target } => {
            if !valid(target) {
                return Err("invalid sanction target".into());
            }
            if state.diplomacy.is_sanctioned_by(target, actor) {
                return Err("sanction already in force".into());
            }
            state.diplomacy.sanctions.push(Sanction {
                by: actor,
                target,
                since: state.turn,
            });
            state
                .opinions
                .add(target, actor, modifier("sanctions", SANCTION_OPINION, 0.0));
            state.tension.add(actor, target, SANCTION_TENSION);
            Ok(Some(DiplomaticEvent::SanctionImposed { by: actor, target }))
        }

        Order::LiftSanction { target } => {
            let pos = state
                .diplomacy
                .sanctions
                .iter()
                .position(|s| s.by == actor && s.target == target)
                .ok_or("no such sanction")?;
            state.diplomacy.sanctions.remove(pos);
            state.opinions.remove_source(target, actor, "sanctions");
            state
                .opinions
                .add(target, actor, modifier("past sanctions", PAST_SANCTION_OPINION, 1.0));
            Ok(Some(DiplomaticEvent::SanctionLifted { by: actor, target }))
        }

        Order::Aid { to, amount } => {
            if !valid(to) || !amount.is_finite() || amount <= 0.0 {
                return Err("invalid aid".into());
            }
            state.diplomacy.pending_aid.push((actor, to, amount));
            let gain = (100.0 * amount / state.country(to).gdp).min(20.0);
            state.opinions.add(to, actor, modifier("aid", gain, 2.0));
            Ok(Some(DiplomaticEvent::AidPledged {
                from: actor,
                to,
                amount,
            }))
        }

        Order::StartStream { to, amount } => start_stream(state, actor, to, amount, StreamKind::Aid, false, false),
        Order::StartArmsStream { to, amount, covert } => {
            start_stream(state, actor, to, amount, StreamKind::Arms, covert, false)
        }
        Order::SellArms { to, amount, covert } => {
            let event = start_stream(state, actor, to, amount, StreamKind::Arms, covert, true)?;
            if !covert {
                for x in sale_objectors(state, actor, to) {
                    state.opinions.add(x, actor, modifier("armed our rival", -15.0, 1.0));
                }
            }
            Ok(event)
        }
        Order::StopStream { to } => stop_stream(state, actor, to, StreamKind::Aid),
        Order::StopArmsStream { to } => stop_stream(state, actor, to, StreamKind::Arms),
        Order::CancelArmsPurchase { from } => {
            let pos = state
                .diplomacy
                .streams
                .iter()
                .position(|s| s.from == from && s.to == actor && s.sale)
                .ok_or("no such purchase")?;
            let stream = state.diplomacy.streams.remove(pos);
            state.opinions.remove_source(actor, from, stream_source(StreamKind::Arms));
            Ok(Some(DiplomaticEvent::StreamStopped { stream }))
        }

        Order::ArmsTransfer { to, amount } => {
            if !valid(to) || !amount.is_finite() || amount <= 0.0 {
                return Err("invalid arms transfer".into());
            }
            state.diplomacy.pending_arms.push((actor, to, amount));
            let gain = (100.0 * arms_value(StreamKind::Arms, amount) / state.country(to).gdp).min(20.0);
            state.opinions.add(to, actor, modifier("arms", gain, 2.0));
            Ok(Some(DiplomaticEvent::ArmsTransferred {
                from: actor,
                to,
                amount,
            }))
        }

        Order::Denounce { target } => {
            if !valid(target) {
                return Err("invalid denounce target".into());
            }
            state.opinions.add(target, actor, modifier("denounced us", -15.0, 1.0));
            state.tension.add(actor, target, DENOUNCE_TENSION);
            Ok(Some(DiplomaticEvent::Denounced { by: actor, target }))
        }

        Order::ForgiveDebt { debtor } => {
            let pos = state
                .diplomacy
                .loans
                .iter()
                .position(|l| l.creditor == actor && l.debtor == debtor)
                .ok_or("no such loan")?;
            let loan = state.diplomacy.loans.remove(pos);
            let d = state.country_mut(debtor);
            d.debt = (d.debt - loan.amount).max(0.0);
            let gain = (100.0 * loan.amount / d.gdp.max(1e-9)).min(FORGIVENESS_OPINION_CAP);
            state
                .opinions
                .add(debtor, actor, modifier("forgave our war debts", gain, FORGIVENESS_DECAY));
            Ok(Some(DiplomaticEvent::DebtForgiven {
                creditor: actor,
                debtor,
                amount: loan.amount,
            }))
        }

        Order::HoldDebt { debtor } => {
            let pos = state
                .diplomacy
                .loans
                .iter()
                .position(|l| l.creditor == actor && l.debtor == debtor)
                .ok_or("no such loan")?;
            if !in_distress(state, debtor) {
                return Err("debtor not in distress".into());
            }
            // The resentment is the deepening grudge while the hold stands
            // (D97, `grudge::run`), not a one-off opinion hit.
            let loan = &mut state.diplomacy.loans[pos];
            loan.held = true;
            let loan = *loan;
            Ok(Some(DiplomaticEvent::DebtHeld {
                creditor: actor,
                debtor,
                amount: loan.amount,
            }))
        }

        Order::Mediate { a, b } => {
            if !valid(a) || !valid(b) || a == b {
                return Err("invalid mediation".into());
            }
            state.tension.add(a, b, -MEDIATE_TENSION_RELIEF);
            state.opinions.add(a, actor, modifier("mediation", 5.0, 0.5));
            state.opinions.add(b, actor, modifier("mediation", 5.0, 0.5));
            Ok(Some(DiplomaticEvent::Mediated { by: actor, a, b }))
        }
    }
}

fn stream_source(kind: StreamKind) -> &'static str {
    match kind {
        StreamKind::Aid => "support stream",
        StreamKind::Arms => "arms supply",
    }
}

fn start_stream(
    state: &mut WorldState,
    actor: CountryId,
    to: CountryId,
    amount: f64,
    kind: StreamKind,
    covert: bool,
    sale: bool,
) -> Result<Option<DiplomaticEvent>, String> {
    if to.index() >= state.countries.len() || to == actor || !amount.is_finite() || amount <= 0.0 {
        return Err("invalid stream".into());
    }
    let stream = Stream {
        from: actor,
        to,
        amount,
        since: state.turn,
        kind,
        covert,
        sale,
    };
    match state
        .diplomacy
        .streams
        .iter_mut()
        .find(|s| s.from == actor && s.to == to && s.kind == kind)
    {
        Some(existing) => {
            existing.amount = amount;
            existing.covert = covert;
            existing.sale = sale;
        }
        None => {
            state.diplomacy.streams.push(stream);
            state.opinions.add(to, actor, modifier(stream_source(kind), 10.0, 0.0));
        }
    }
    Ok(Some(DiplomaticEvent::StreamStarted { stream }))
}

fn stop_stream(
    state: &mut WorldState,
    actor: CountryId,
    to: CountryId,
    kind: StreamKind,
) -> Result<Option<DiplomaticEvent>, String> {
    let pos = state
        .diplomacy
        .streams
        .iter()
        .position(|s| s.from == actor && s.to == to && s.kind == kind)
        .ok_or("no such stream")?;
    let stream = state.diplomacy.streams.remove(pos);
    state.opinions.remove_source(to, actor, stream_source(kind));
    state.opinions.add(to, actor, modifier("support cut", -10.0, 1.0));
    Ok(Some(DiplomaticEvent::StreamStopped { stream }))
}

/// Market price of arms for a buyer, per unit of build value (D58): the
/// more isolated and the more desperate the buyer, the more it pays.
pub fn arms_price(sanctioners: usize, at_war: bool) -> f64 {
    1.0 + (ARMS_SANCTION_PREMIUM * sanctioners as f64).min(ARMS_MAX_SANCTION_PREMIUM)
        + if at_war { ARMS_WAR_PREMIUM } else { 0.0 }
}

pub const ARMS_SANCTION_PREMIUM: f64 = 0.1;
pub const ARMS_MAX_SANCTION_PREMIUM: f64 = 0.5;
pub const ARMS_WAR_PREMIUM: f64 = 0.25;

/// Countries that object to `seller` selling arms to `buyer` (D58): the
/// seller's allies and patrons who are at war with, or in high tension
/// with, the buyer.
pub fn sale_objectors(state: &WorldState, seller: CountryId, buyer: CountryId) -> Vec<CountryId> {
    let mut friends: Vec<CountryId> = state.diplomacy.allies_of(seller).collect();
    friends.extend(
        state
            .diplomacy
            .streams
            .iter()
            .filter(|s| s.to == seller && !s.sale)
            .map(|s| s.from),
    );
    friends.sort();
    friends.dedup();
    friends
        .into_iter()
        .filter(|&x| x != buyer && (state.wars.at_war(x, buyer) || state.tension.get(x, buyer) >= SALE_OBJECTION_TENSION))
        .collect()
}

pub const SALE_OBJECTION_TENSION: f64 = 50.0;

/// Drop proposals made before this turn that went unanswered.
pub fn lapse_proposals(state: &mut WorldState) -> Vec<DiplomaticEvent> {
    let turn = state.turn;
    let (lapsed, open): (Vec<_>, Vec<_>) = state.diplomacy.proposals.drain(..).partition(|p| p.turn < turn);
    state.diplomacy.proposals = open;
    lapsed
        .into_iter()
        .map(|proposal| DiplomaticEvent::ProposalLapsed { proposal })
        .collect()
}
