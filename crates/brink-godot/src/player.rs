//! Player-facing data (V-2a, principle 7, D105 #5). Every function here
//! reads only the player's own fog-filtered [`ObserverView`]: the human sees
//! exactly what the AI in the same seat would see. A source lint in
//! `tests/player.rs` forbids the canonical world type in this file.
//!
//! Plain Rust (no Godot types), so it is unit-testable; `lib.rs` flattens
//! these structs into Godot dictionaries.

use sim_core::commodity::ProductionPolicy;
use sim_core::diplomacy::DiplomaticEvent;
use sim_core::view::StabilityBand;
use sim_core::{
    BudgetShares, CountryId, DecisionKind, Estimate, ForeignView, LedgerLog, Mobilization, MonetaryStance,
    ObserverView, Order, ProposalId, Side, StreamKind, TransitionKind, TreatyId, TreatyKind, TurnReport, WarAim, WarId,
};
use voice::lines::Trigger;

// ---------------------------------------------------------------- orders --

/// A player order as the front-end describes it (the V-2a subset, plan §2).
/// `lib.rs` fills it from a Godot dictionary with the same keys.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OrderSpec {
    pub kind: String,
    /// Country code.
    pub target: Option<String>,
    /// War aim: "Punitive" | "Limited" | "Major".
    pub aim: Option<String>,
    pub amount: Option<f64>,
    /// Respond: accept (true) or refuse (false).
    pub flag: Option<bool>,
    /// Proposal, treaty or war id.
    pub id: Option<u32>,
    /// Mobilization / energy policy / monetary stance level.
    pub level: Option<String>,
    /// Treaty kind: "Trade" | "NonAggression" | "DefensiveAlliance".
    pub treaty: Option<String>,
    /// JoinWar side: "Attacker" | "Defender".
    pub side: Option<String>,
    /// JoinWar band: 3 (limited) or 4 (major).
    pub band: Option<u8>,
    /// SetBudget shares: military, development, welfare, intelligence.
    pub shares: Option<[f64; 4]>,
}

/// Every order kind the V-2a bridge accepts.
pub const ORDER_KINDS: &[&str] = &[
    "budget",
    "deficit",
    "mobilization",
    "energy_policy",
    "monetary_stance",
    "respond",
    "reform",
    "crackdown",
    "propose_treaty",
    "guarantee",
    "sanction",
    "lift_sanction",
    "denounce",
    "aid",
    "cancel_treaty",
    "declare_war",
    "join_war",
    "offer_peace",
    "leave_war",
];

fn code_of(view: &ObserverView, id: CountryId) -> String {
    if id == view.observer {
        return view.own.code.clone();
    }
    view.others
        .iter()
        .find(|f| f.id == id)
        .map_or_else(|| format!("#{}", id.0), |f| f.code.clone())
}

fn name_of(view: &ObserverView, id: CountryId) -> String {
    if id == view.observer {
        return view.own.name.clone();
    }
    view.others
        .iter()
        .find(|f| f.id == id)
        .map_or_else(|| format!("#{}", id.0), |f| f.name.clone())
}

fn foreign<'a>(view: &'a ObserverView, code: &str) -> Option<&'a ForeignView> {
    view.others.iter().find(|f| f.code == code)
}

fn target_id(view: &ObserverView, spec: &OrderSpec) -> Result<CountryId, String> {
    let code = spec.target.as_deref().ok_or("missing target")?;
    foreign(view, code)
        .map(|f| f.id)
        .ok_or_else(|| format!("unknown country {code}"))
}

fn level<T: Copy>(spec: &OrderSpec, options: &[(&str, T)]) -> Result<T, String> {
    let l = spec.level.as_deref().ok_or("missing level")?;
    options
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(l))
        .map(|(_, v)| *v)
        .ok_or_else(|| format!("unknown level {l}"))
}

const MOBILIZATION: &[(&str, Mobilization)] = &[
    ("Peacetime", Mobilization::Peacetime),
    ("Partial", Mobilization::Partial),
    ("Full", Mobilization::Full),
    ("Total", Mobilization::Total),
];
const ENERGY: &[(&str, ProductionPolicy)] = &[
    ("Restrain", ProductionPolicy::Restrain),
    ("Normal", ProductionPolicy::Normal),
    ("Flood", ProductionPolicy::Flood),
];
const STANCE: &[(&str, MonetaryStance)] = &[
    ("Tight", MonetaryStance::Tight),
    ("Neutral", MonetaryStance::Neutral),
    ("Loose", MonetaryStance::Loose),
];
const AIMS: &[(&str, WarAim)] = &[
    ("Punitive", WarAim::Punitive),
    ("Limited", WarAim::Limited),
    ("Major", WarAim::Major),
];
const TREATIES: &[(&str, TreatyKind)] = &[
    ("Trade", TreatyKind::Trade { deep: false }),
    ("NonAggression", TreatyKind::NonAggression),
    ("DefensiveAlliance", TreatyKind::DefensiveAlliance),
];

fn label<T: PartialEq>(options: &[(&'static str, T)], v: &T) -> &'static str {
    options.iter().find(|(_, x)| x == v).map_or("?", |(k, _)| k)
}

fn pick<T: Copy>(value: Option<&str>, options: &[(&str, T)], what: &str) -> Result<T, String> {
    let v = value.ok_or_else(|| format!("missing {what}"))?;
    options
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(v))
        .map(|(_, x)| *x)
        .ok_or_else(|| format!("unknown {what} {v}"))
}

/// Parse a front-end order into a simulation [`Order`], resolving country
/// codes through the player's view. Only the V-2a subset is accepted.
pub fn parse_order(spec: &OrderSpec, view: &ObserverView) -> Result<Order, String> {
    let id = || spec.id.ok_or_else(|| "missing id".to_string());
    let amount = || {
        spec.amount
            .filter(|a| a.is_finite())
            .ok_or_else(|| "missing amount".to_string())
    };
    Ok(match spec.kind.as_str() {
        "budget" => {
            let [military, development, welfare, intelligence] = spec.shares.ok_or("missing budget shares")?;
            Order::SetBudget(BudgetShares {
                military,
                development,
                welfare,
                intelligence,
            })
        }
        "deficit" => Order::SetDeficit(amount()?),
        "mobilization" => Order::SetMobilization(level(spec, MOBILIZATION)?),
        "energy_policy" => Order::SetEnergyPolicy(level(spec, ENERGY)?),
        "monetary_stance" => Order::SetMonetaryStance(level(spec, STANCE)?),
        "respond" => Order::Respond {
            proposal: ProposalId(id()?),
            accept: spec.flag.ok_or("missing flag (accept)")?,
        },
        "reform" => Order::Reform,
        "crackdown" => Order::Crackdown,
        "propose_treaty" => Order::ProposeTreaty {
            to: target_id(view, spec)?,
            kind: pick(spec.treaty.as_deref(), TREATIES, "treaty")?,
        },
        "guarantee" => Order::IssueGuarantee {
            to: target_id(view, spec)?,
        },
        "sanction" => Order::Sanction {
            target: target_id(view, spec)?,
        },
        "lift_sanction" => Order::LiftSanction {
            target: target_id(view, spec)?,
        },
        "denounce" => Order::Denounce {
            target: target_id(view, spec)?,
        },
        "aid" => Order::Aid {
            to: target_id(view, spec)?,
            amount: amount()?,
        },
        "cancel_treaty" => Order::CancelTreaty {
            treaty: TreatyId(id()?),
        },
        "declare_war" => Order::DeclareWar {
            target: target_id(view, spec)?,
            aim: pick(spec.aim.as_deref(), AIMS, "aim")?,
        },
        "join_war" => Order::JoinWar {
            war: WarId(id()?),
            side: pick(
                spec.side.as_deref(),
                &[("Attacker", Side::Attacker), ("Defender", Side::Defender)],
                "side",
            )?,
            band: match spec.band.unwrap_or(3) {
                b @ (3 | 4) => b,
                b => return Err(format!("band {b}: join a war at band 3 or 4")),
            },
        },
        "offer_peace" => Order::OfferPeace { war: WarId(id()?) },
        "leave_war" => Order::LeaveWar { war: WarId(id()?) },
        other => return Err(format!("unsupported order kind {other}")),
    })
}

/// The inverse of [`parse_order`] for display (pending orders, advice).
/// `None` for orders outside the V-2a subset (the advisor may still
/// recommend them; they show as text only).
pub fn spec_of(order: &Order, view: &ObserverView) -> Option<OrderSpec> {
    let mut s = OrderSpec::default();
    let target = |c: CountryId| Some(code_of(view, c));
    match order {
        Order::SetBudget(b) => {
            s.kind = "budget".into();
            s.shares = Some([b.military, b.development, b.welfare, b.intelligence]);
        }
        Order::SetDeficit(r) => {
            s.kind = "deficit".into();
            s.amount = Some(*r);
        }
        Order::SetMobilization(m) => {
            s.kind = "mobilization".into();
            s.level = Some(label(MOBILIZATION, m).into());
        }
        Order::SetEnergyPolicy(p) => {
            s.kind = "energy_policy".into();
            s.level = Some(label(ENERGY, p).into());
        }
        Order::SetMonetaryStance(m) => {
            s.kind = "monetary_stance".into();
            s.level = Some(label(STANCE, m).into());
        }
        Order::Respond { proposal, accept } => {
            s.kind = "respond".into();
            s.id = Some(proposal.0);
            s.flag = Some(*accept);
        }
        Order::Reform => s.kind = "reform".into(),
        Order::Crackdown => s.kind = "crackdown".into(),
        Order::ProposeTreaty { to, kind } => {
            if !TREATIES.iter().any(|(_, k)| k == kind) {
                return None;
            }
            s.kind = "propose_treaty".into();
            s.target = target(*to);
            s.treaty = Some(label(TREATIES, kind).into());
        }
        Order::IssueGuarantee { to } => {
            s.kind = "guarantee".into();
            s.target = target(*to);
        }
        Order::Sanction { target: t } => {
            s.kind = "sanction".into();
            s.target = target(*t);
        }
        Order::LiftSanction { target: t } => {
            s.kind = "lift_sanction".into();
            s.target = target(*t);
        }
        Order::Denounce { target: t } => {
            s.kind = "denounce".into();
            s.target = target(*t);
        }
        Order::Aid { to, amount } => {
            s.kind = "aid".into();
            s.target = target(*to);
            s.amount = Some(*amount);
        }
        Order::CancelTreaty { treaty } => {
            s.kind = "cancel_treaty".into();
            s.id = Some(treaty.0);
        }
        Order::DeclareWar { target: t, aim } => {
            s.kind = "declare_war".into();
            s.target = target(*t);
            s.aim = Some(label(AIMS, aim).into());
        }
        Order::JoinWar { war, side, band } => {
            s.kind = "join_war".into();
            s.id = Some(war.0);
            s.side = Some(format!("{side:?}"));
            s.band = Some(*band);
        }
        Order::OfferPeace { war } => {
            s.kind = "offer_peace".into();
            s.id = Some(war.0);
        }
        Order::LeaveWar { war } => {
            s.kind = "leave_war".into();
            s.id = Some(war.0);
        }
        _ => return None,
    }
    Some(s)
}

/// One-line description of any order, in the player's terms.
pub fn describe_order(order: &Order, view: &ObserverView) -> String {
    let n = |c: CountryId| name_of(view, c);
    match order {
        Order::SetBudget(b) => format!(
            "Set budget: military {:.0}%, development {:.0}%, welfare {:.0}%, intelligence {:.0}%",
            100.0 * b.military,
            100.0 * b.development,
            100.0 * b.welfare,
            100.0 * b.intelligence
        ),
        Order::SetDeficit(r) => format!("Set deficit spending to {:.0}% of revenue", 100.0 * r),
        Order::SetMobilization(m) => format!("Mobilization: {m:?}"),
        Order::SetEnergyPolicy(p) => format!("Energy policy: {p:?}"),
        Order::SetMonetaryStance(m) => format!("Monetary stance: {m:?}"),
        Order::Respond { proposal, accept } => {
            let what = view.incoming_proposals.iter().find(|p| p.id == *proposal).map_or_else(
                || format!("proposal #{}", proposal.0),
                |p| format!("{}'s {}", n(p.from), treaty_name(p.kind)),
            );
            format!("{} {what}", if *accept { "Accept" } else { "Refuse" })
        }
        Order::Reform => "Answer the crisis: Reform".into(),
        Order::Crackdown => "Answer the crisis: Crackdown".into(),
        Order::ProposeTreaty { to, kind } => format!("Propose a {} to {}", treaty_name(*kind), n(*to)),
        Order::IssueGuarantee { to } => format!("Guarantee {}", n(*to)),
        Order::Sanction { target } => format!("Sanction {}", n(*target)),
        Order::LiftSanction { target } => format!("Lift sanctions on {}", n(*target)),
        Order::Denounce { target } => format!("Denounce {}", n(*target)),
        Order::Aid { to, amount } => format!("Send {amount:.1} aid to {}", n(*to)),
        Order::CancelTreaty { treaty } => match view.treaties.iter().find(|t| t.id == *treaty) {
            Some(t) => {
                let other = t.other(view.observer).unwrap_or(t.b);
                format!("Cancel the {} with {}", treaty_name(t.kind), n(other))
            }
            None => format!("Cancel treaty #{}", treaty.0),
        },
        Order::DeclareWar { target, aim } => format!("Declare a {aim:?} war on {}", n(*target)),
        Order::JoinWar { war, side, band } => format!("Join war #{} on the {side:?} side (band {band})", war.0),
        Order::OfferPeace { war } => format!("Offer peace in war #{}", war.0),
        Order::LeaveWar { war } => format!("Leave war #{}", war.0),
        Order::StartStream { to, amount } => format!("Fund {} with {amount:.1} per turn", n(*to)),
        Order::StopStream { to } => format!("Stop funding {}", n(*to)),
        Order::StartArmsStream { to, amount, covert } => format!(
            "{}rm {} with {amount:.1} strength per turn",
            if *covert { "Covertly a" } else { "A" },
            n(*to)
        ),
        Order::StopArmsStream { to } => format!("Stop arming {}", n(*to)),
        Order::SellArms { to, amount, .. } => format!("Sell {amount:.1} strength of arms per turn to {}", n(*to)),
        other => format!("{other:?}"),
    }
}

pub fn treaty_name(kind: TreatyKind) -> &'static str {
    match kind {
        TreatyKind::Trade { deep: true } => "deep trade agreement",
        TreatyKind::Trade { deep: false } => "trade agreement",
        TreatyKind::NonAggression => "non-aggression pact",
        TreatyKind::DefensiveAlliance => "defensive alliance",
        TreatyKind::Guarantee => "guarantee",
        TreatyKind::Basing => "basing agreement",
    }
}

// ----------------------------------------------------------- own country --

#[derive(Clone, Debug, PartialEq)]
pub struct TreatyLine {
    pub id: u32,
    pub kind: &'static str,
    /// The other party's code.
    pub with: String,
    /// For a directed treaty: does the player give (true) or receive it?
    pub ours: bool,
}

/// The player's own country, known exactly (it is the player's own state).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerState {
    pub code: String,
    pub name: String,
    pub government: String,
    pub turn: u32,
    pub year: f64,
    pub gdp: f64,
    /// Last quarter's growth (fraction).
    pub growth: f64,
    /// Debt / annual GDP.
    pub debt_ratio: f64,
    /// Deficit spending as a share of revenue.
    pub deficit: f64,
    pub reserves: f64,
    pub stability: f64,
    pub prosperity: f64,
    pub security: f64,
    pub legitimacy: f64,
    pub war_weariness: f64,
    pub initiative_allowance: u8,
    pub initiative_banked: u8,
    pub initiative_available: u8,
    /// [military, development, welfare, intelligence]
    pub budget: [f64; 4],
    pub budget_target: [f64; 4],
    pub mobilization: String,
    pub energy_policy: String,
    pub energy_capacity: f64,
    pub energy_net_exports: f64,
    pub military: f64,
    pub arsenal: u8,
    pub at_war: bool,
    /// A transition crisis awaits Reform or Crackdown (the main way to lose).
    pub crisis_pending: bool,
    /// Turn the crisis opened (unanswered crises default to Crackdown).
    pub crisis_since: Option<u32>,
    pub treaties: Vec<TreatyLine>,
    /// Codes of countries we sanction / that sanction us.
    pub sanctioning: Vec<String>,
    pub sanctioned_by: Vec<String>,
}

fn shares(b: &BudgetShares) -> [f64; 4] {
    [b.military, b.development, b.welfare, b.intelligence]
}

pub fn player_state(view: &ObserverView) -> PlayerState {
    let me = &view.own;
    let p = view.observer;
    PlayerState {
        code: me.code.clone(),
        name: me.name.clone(),
        government: format!("{:?}", me.government),
        turn: view.turn,
        year: view.year,
        gdp: me.gdp,
        growth: me.last_growth,
        debt_ratio: me.debt_ratio(),
        deficit: me.deficit_ratio,
        reserves: me.reserves,
        stability: me.stability,
        prosperity: me.prosperity,
        security: me.security,
        legitimacy: me.legitimacy,
        war_weariness: me.war_weariness,
        initiative_allowance: me.initiative.allowance,
        initiative_banked: me.initiative.banked,
        initiative_available: me.initiative.available(),
        budget: shares(&me.budget),
        budget_target: shares(&me.budget_target),
        mobilization: format!("{:?}", me.forces.mobilization),
        energy_policy: format!("{:?}", me.energy_policy),
        energy_capacity: me.energy_capacity,
        energy_net_exports: me.energy_net_exports,
        military: me.power(),
        arsenal: me.arsenal,
        at_war: view.wars.iter().any(|w| w.belligerent(p).is_some()),
        crisis_pending: me.transition.pending_since.is_some(),
        crisis_since: me.transition.pending_since,
        treaties: view
            .treaties
            .iter()
            .filter(|t| t.involves(p))
            .map(|t| TreatyLine {
                id: t.id.0,
                kind: treaty_name(t.kind),
                with: code_of(view, t.other(p).unwrap_or(t.b)),
                ours: t.a == p,
            })
            .collect(),
        sanctioning: view
            .sanctions
            .iter()
            .filter(|s| s.by == p)
            .map(|s| code_of(view, s.target))
            .collect(),
        sanctioned_by: view
            .sanctions
            .iter()
            .filter(|s| s.target == p)
            .map(|s| code_of(view, s.by))
            .collect(),
    }
}

// ------------------------------------------------------- foreign country --

/// What the player knows about another country: public facts, estimates
/// with their bands, and `None` where coverage is too low.
#[derive(Clone, Debug, PartialEq)]
pub struct ForeignInfo {
    pub code: String,
    pub name: String,
    pub government: String,
    pub alignment: Option<String>,
    pub area: Option<String>,
    pub coverage: f64,
    pub gdp: f64,
    /// `[value, low, high]`
    pub military: [f64; 3],
    pub land: [f64; 3],
    pub naval: [f64; 3],
    pub air: [f64; 3],
    pub debt_ratio: Option<[f64; 3]>,
    pub stability_band: Option<&'static str>,
    pub budget: Option<[f64; 4]>,
    pub arsenal: u8,
    pub programme: Option<f64>,
    pub energy_policy: String,
    pub energy_capacity: f64,
    pub energy_net_exports: f64,
    pub arms_industry: f64,
    pub military_tech: u8,
    pub their_opinion_of_us: f64,
    pub our_opinion_of_them: f64,
    pub tension: f64,
    pub credibility_back: f64,
    pub credibility_threat: f64,
    pub credibility_norm: f64,
    pub trust: f64,
    pub at_war: bool,
    pub at_war_with_us: bool,
    /// Treaties between us and them.
    pub treaties_with_us: Vec<&'static str>,
    pub we_sanction: bool,
    pub sanctions_us: bool,
}

fn est(e: &Estimate) -> [f64; 3] {
    [e.value, e.low, e.high]
}

pub fn band_name(b: StabilityBand) -> &'static str {
    match b {
        StabilityBand::Collapse => "Collapse",
        StabilityBand::Crisis => "Crisis",
        StabilityBand::Unrest => "Unrest",
        StabilityBand::Normal => "Normal",
        StabilityBand::Strong => "Strong",
    }
}

pub fn foreign_info(view: &ObserverView, code: &str) -> Option<ForeignInfo> {
    let f = foreign(view, code)?;
    let p = view.observer;
    let at_war_with_us = view
        .wars
        .iter()
        .any(|w| matches!((w.side_of(p), w.side_of(f.id)), (Some(a), Some(b)) if a != b));
    Some(ForeignInfo {
        code: f.code.clone(),
        name: f.name.clone(),
        government: format!("{:?}", f.government),
        alignment: f.alignment.clone(),
        area: f.area.clone(),
        coverage: f.coverage,
        gdp: f.gdp,
        military: est(&f.military),
        land: est(&f.forces.land),
        naval: est(&f.forces.naval),
        air: est(&f.forces.air),
        debt_ratio: f.debt_ratio.as_ref().map(est),
        stability_band: f.stability_band.map(band_name),
        budget: f.budget.as_ref().map(shares),
        arsenal: f.arsenal,
        programme: f.programme,
        energy_policy: format!("{:?}", f.energy_policy),
        energy_capacity: f.energy_capacity,
        energy_net_exports: f.energy_net_exports,
        arms_industry: f.arms_industry,
        military_tech: f.military_tech,
        their_opinion_of_us: f.their_opinion_of_us,
        our_opinion_of_them: f.our_opinion_of_them,
        tension: f.tension,
        credibility_back: f.credibility_back,
        credibility_threat: f.credibility_threat,
        credibility_norm: f.credibility_norm,
        trust: f.trust,
        at_war: view.wars.iter().any(|w| w.belligerent(f.id).is_some()),
        at_war_with_us,
        treaties_with_us: view
            .treaties
            .iter()
            .filter(|t| t.involves(p) && t.involves(f.id))
            .map(|t| treaty_name(t.kind))
            .collect(),
        we_sanction: view.sanctions.iter().any(|s| s.by == p && s.target == f.id),
        sanctions_us: view.sanctions.iter().any(|s| s.by == f.id && s.target == p),
    })
}

/// A map row for `countries()` in player mode: exact for the player's own
/// country, the player's estimate for everyone else.
#[derive(Clone, Debug, PartialEq)]
pub struct MapRow {
    pub id: CountryId,
    pub code: String,
    pub name: String,
    pub area: Option<String>,
    pub government: String,
    pub alignment: Option<String>,
    pub gdp: f64,
    /// Exact for the player; `None` when the band is hidden.
    pub stability: Option<f64>,
    pub stability_band: Option<&'static str>,
    /// Exact for the player; the estimate (with band) for others.
    pub power: [f64; 3],
    pub arsenal: u8,
    pub at_war: bool,
    pub own: bool,
}

pub fn map_rows(view: &ObserverView) -> Vec<MapRow> {
    let at_war = |c: CountryId| view.wars.iter().any(|w| w.belligerent(c).is_some());
    let me = &view.own;
    let mut rows = vec![MapRow {
        id: me.id,
        code: me.code.clone(),
        name: me.name.clone(),
        area: me.area.clone(),
        government: format!("{:?}", me.government),
        alignment: me.alignment.clone(),
        gdp: me.gdp,
        stability: Some(me.stability),
        stability_band: Some(band_name(StabilityBand::from_value(me.stability))),
        power: [me.power(); 3],
        arsenal: me.arsenal,
        at_war: at_war(me.id),
        own: true,
    }];
    rows.extend(view.others.iter().map(|f| MapRow {
        id: f.id,
        code: f.code.clone(),
        name: f.name.clone(),
        area: f.area.clone(),
        government: format!("{:?}", f.government),
        alignment: f.alignment.clone(),
        gdp: f.gdp,
        stability: None,
        stability_band: f.stability_band.map(band_name),
        power: est(&f.military),
        arsenal: f.arsenal,
        at_war: at_war(f.id),
        own: false,
    }));
    rows.sort_by_key(|r| r.id);
    rows
}

/// Public pair tension from the view (0 for unknown pairs).
pub fn tension(view: &ObserverView, a: &str, b: &str) -> f64 {
    let id = |code: &str| {
        if code == view.own.code {
            Some(view.observer)
        } else {
            foreign(view, code).map(|f| f.id)
        }
    };
    let (Some(x), Some(y)) = (id(a), id(b)) else { return 0.0 };
    let (lo, hi) = if x < y { (x, y) } else { (y, x) };
    view.pair_tensions
        .iter()
        .find(|(a, b, _)| *a == lo && *b == hi)
        .map_or(0.0, |t| t.2)
}

// ------------------------------------------------------------- proposals --

#[derive(Clone, Debug, PartialEq)]
pub struct ProposalLine {
    pub id: u32,
    pub from: String,
    pub from_name: String,
    pub kind: &'static str,
    pub turn: u32,
    /// Initiative an acceptance costs (refusing is free).
    pub accept_cost: u8,
    /// Proposals lapse if not answered this turn.
    pub text: String,
}

pub fn proposals(view: &ObserverView) -> Vec<ProposalLine> {
    view.incoming_proposals
        .iter()
        .map(|p| ProposalLine {
            id: p.id.0,
            from: code_of(view, p.from),
            from_name: name_of(view, p.from),
            kind: treaty_name(p.kind),
            turn: p.turn,
            accept_cost: u8::from(p.kind.creates_commitment()),
            text: format!("{} proposes a {}.", name_of(view, p.from), treaty_name(p.kind)),
        })
        .collect()
}

// --------------------------------------------------------------- advice --

/// Which decision kinds explain an order (linking advice to its reasons).
pub fn explains(order: &Order, kind: DecisionKind) -> bool {
    use DecisionKind as K;
    match order {
        Order::SetBudget(_) | Order::SetDeficit(_) => kind == K::Budget,
        Order::SetEnergyPolicy(_) => kind == K::Flood,
        Order::SetMonetaryStance(_) => kind == K::MonetaryStance,
        Order::Respond { .. } => matches!(kind, K::AcceptProposal(_)),
        Order::ProposeTreaty { .. } => matches!(kind, K::Propose(_) | K::RequestGuarantee),
        Order::Sanction { .. } => matches!(kind, K::ImposeSanction | K::JoinSanction),
        Order::LiftSanction { .. } => kind == K::KeepSanction,
        Order::DeclareWar { .. } | Order::JoinWar { .. } | Order::SetMobilization(_) => matches!(kind, K::Band(_)),
        Order::LeaveWar { .. } | Order::OfferPeace { .. } => matches!(kind, K::Withdraw | K::Band(_)),
        Order::CancelTreaty { .. } => kind == K::StayInAlliance,
        Order::Reform | Order::Crackdown => kind == K::Reform,
        Order::StartStream { .. } | Order::StartArmsStream { .. } | Order::ArmsTransfer { .. } => {
            kind == K::StreamStart
        }
        Order::StopStream { .. } | Order::StopArmsStream { .. } => kind == K::StreamStop,
        Order::SellArms { .. } => kind == K::SellArms,
        Order::StartProgramme => kind == K::Programme,
        Order::DiscloseAndDismantle => kind == K::Dismantle,
        Order::ForgiveDebt { .. } | Order::HoldDebt { .. } => kind == K::ForgiveDebt,
        _ => false,
    }
}

/// The counterpart an order concerns, if any (for linking advice).
pub fn order_counterpart(order: &Order, view: &ObserverView) -> Option<CountryId> {
    match *order {
        Order::ProposeTreaty { to, .. }
        | Order::IssueGuarantee { to }
        | Order::Aid { to, .. }
        | Order::StartStream { to, .. }
        | Order::StopStream { to }
        | Order::ArmsTransfer { to, .. }
        | Order::StartArmsStream { to, .. }
        | Order::StopArmsStream { to }
        | Order::SellArms { to, .. } => Some(to),
        Order::Sanction { target }
        | Order::LiftSanction { target }
        | Order::Denounce { target }
        | Order::DeclareWar { target, .. }
        | Order::NuclearStrike { target } => Some(target),
        Order::ForgiveDebt { debtor } | Order::HoldDebt { debtor } => Some(debtor),
        Order::Respond { proposal, .. } => view
            .incoming_proposals
            .iter()
            .find(|p| p.id == proposal)
            .map(|p| p.from),
        _ => None,
    }
}

// ------------------------------------------------------------- game over --

/// The player's regime is gone (D105 #1): collapse or coup, or the country
/// stopped existing. The player's own Reform (and an unanswered crisis
/// defaulting to Crackdown) does not end the game.
pub fn game_over(player: CountryId, events: &[DiplomaticEvent], own_active: bool) -> Option<String> {
    for e in events {
        if let DiplomaticEvent::Transition { country, kind } = e {
            if *country == player {
                match kind {
                    TransitionKind::Collapse => return Some("Collapse".into()),
                    TransitionKind::Coup => return Some("Coup".into()),
                    _ => {}
                }
            }
        }
    }
    (!own_active).then(|| "Extinct".into())
}

// --------------------------------------------------------- narration fog --

fn stream_seen(view: &ObserverView, from: CountryId, to: CountryId, kind: StreamKind) -> bool {
    view.streams
        .iter()
        .any(|s| s.from == from && s.to == to && s.kind == kind)
}

fn any_stream_seen(view: &ObserverView, from: CountryId, to: CountryId) -> bool {
    view.streams.iter().any(|s| s.from == from && s.to == to)
}

/// Did the player see `actor` claim `norm` (its ledger entry is in view)?
fn norm_seen(view: &ObserverView, actor: CountryId, norm: sim_core::NormTag) -> bool {
    view.ledger
        .iter()
        .any(|e| e.actor == actor && e.kind == sim_core::EntryKind::Norm(norm))
}

/// Could the player know this event happened? `pre` is the player's view
/// taken before the turn resolved, `post` after.
pub fn event_visible(pre: &ObserverView, post: &ObserverView, e: &DiplomaticEvent) -> bool {
    let p = post.observer;
    match e {
        DiplomaticEvent::StreamStarted { stream } => {
            !stream.covert
                || stream.from == p
                || stream.to == p
                || stream_seen(post, stream.from, stream.to, stream.kind)
        }
        DiplomaticEvent::StreamStopped { stream } => {
            !stream.covert
                || stream.from == p
                || stream.to == p
                || stream_seen(pre, stream.from, stream.to, stream.kind)
        }
        // A covert programme's completion is news only to those who could
        // already see the programme.
        DiplomaticEvent::ProgrammeCompleted { country } => {
            *country == p || pre.others.iter().any(|f| f.id == *country && f.programme.is_some())
        }
        // Proposals are private between the parties: the view shows only
        // those addressed to the observer.
        DiplomaticEvent::Proposed { proposal }
        | DiplomaticEvent::ProposalRefused { proposal }
        | DiplomaticEvent::ProposalLapsed { proposal } => proposal.from == p || proposal.to == p,
        _ => true,
    }
}

/// The turn report as the player could know it: events it could see, ledger
/// developments it saw, and only its own rejected orders.
pub fn visible_report(pre: &ObserverView, post: &ObserverView, report: &TurnReport) -> TurnReport {
    let p = post.observer;
    TurnReport {
        turn: report.turn,
        events: report
            .events
            .iter()
            .filter(|e| event_visible(pre, post, e))
            .cloned()
            .collect(),
        ledger_log: report
            .ledger_log
            .iter()
            .filter(|l| match l {
                LedgerLog::EntryWritten { id, .. } => post.ledger.iter().any(|e| e.id == *id),
                LedgerLog::NormCreated { seen_by, .. } => seen_by.contains(&p),
                // A norm test is news to the parties and to those who saw
                // the norm claimed.
                LedgerLog::NormTest {
                    norm, actor, violator, ..
                } => *actor == p || *violator == p || norm_seen(post, *actor, *norm),
                _ => true,
            })
            .cloned()
            .collect(),
        rejected: report.rejected.iter().filter(|r| r.0 == p).cloned().collect(),
    }
}

/// Second pass over narration whose facts the narrator reads from state
/// rather than from the (already filtered) report. `actor`/`target` are the
/// narration's subjects.
pub fn narration_visible(
    pre: &ObserverView,
    post: &ObserverView,
    trigger: Trigger,
    actor: CountryId,
    target: CountryId,
) -> bool {
    let p = post.observer;
    let party = actor == p || target == p;
    match trigger {
        // Another government's change of goals is its private reasoning
        // (D105 #3: reasoning is shown only after the fact, in history).
        Trigger::StrategyReversed => actor == p,
        Trigger::ArmedClient => party || stream_seen(post, actor, target, StreamKind::Arms),
        // Arms a supplier sent may have come covertly.
        Trigger::TransferredArmsUsedAgainstSupplier => {
            party
                || stream_seen(pre, actor, target, StreamKind::Arms)
                || stream_seen(post, actor, target, StreamKind::Arms)
        }
        // The fact quotes exact debt: only our own.
        Trigger::DebtBrake | Trigger::OutOfInitiative => actor == p,
        // "...which funds it": the funding stream may be covert.
        Trigger::RefusedPatron => party || any_stream_seen(post, target, actor),
        // Proposals and the refuser's "decisive factor" (its private
        // DecisionRecord, D105 #3) are known to the parties only.
        Trigger::GuaranteeRefused
        | Trigger::RepeatedRequestRefused
        | Trigger::StatusQuoRefusal
        | Trigger::ProtectionFromDenounced
        | Trigger::CoalitionByDependence => party,
        // Exact values the player's view only estimates: a third party's
        // opinion, readiness, trade lost to sanctions, rerouting share.
        Trigger::NormIgnoredByFriend
        | Trigger::CutMilitarySpendingThenThreatened
        | Trigger::SanctionsHurtSender
        | Trigger::SanctionsSubstituted
        | Trigger::SanctionsReimposed => party,
        _ => true,
    }
}
