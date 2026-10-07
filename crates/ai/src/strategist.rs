//! The v0.1 strategic AI controller. Responds to proposals every turn;
//! makes strategic moves (guarantee requests, joining sanctions, alliance
//! re-evaluation) on a staggered 4-turn assessment cadence (DESIGN §14.2).

use std::collections::{BTreeMap, BTreeSet};

use sim_core::{
    CountryId, DecisionKind, ObserverView, Order, ProposalFamily, Side, TreatyId, TreatyKind, War, WarAim, WarId,
};

use crate::evaluate::{self, EXIT_THRESHOLD};
use crate::goals::{self, Goal};
use crate::inputs::{foreign, is_protected, precedents, top_threat};
use crate::sanctions;
use crate::war::{self as wa, BandOption, BAND_HYSTERESIS};
use crate::{debt_brake, Controller, Decision, DecisionRecord, Score};

/// Turns between strategic assessments.
pub const ASSESSMENT_INTERVAL: u32 = 4;
/// Assessments in a row an alliance must score badly before exit.
pub const EXIT_ASSESSMENTS: u32 = 2;

#[derive(Clone, Debug, Default)]
pub struct Strategist {
    /// Consecutive bad assessments per alliance (hysteresis).
    exit_pressure: BTreeMap<TreatyId, u32>,
    last_guarantee_request: Option<u32>,
    /// Wars whose crisis response has been decided (re-scored on
    /// assessment turns afterwards).
    decided_wars: BTreeSet<WarId>,
    /// Active strategic goals (persist between assessments).
    pub goals: Vec<Goal>,
    /// Consecutive bad assessments per guarantee we issued.
    guarantee_pressure: BTreeMap<TreatyId, u32>,
    /// Last turn we proposed something to a partner and how many times in
    /// a row, by (partner, kind). Each repeat waits longer: a government
    /// that keeps getting no answer stops asking.
    last_proposed: BTreeMap<(CountryId, u8), (u32, u32)>,
    /// Consecutive bad assessments per stream we fund, by (client, kind).
    stream_pressure: BTreeMap<(CountryId, u8), u32>,
    /// Consecutive bad assessments of our own weapons programme.
    programme_pressure: u32,
    /// Consecutive bad assessments per arms purchase, by seller (D58).
    purchase_pressure: BTreeMap<CountryId, u32>,
    /// Consecutive bad assessments per sanction we impose, by target.
    sanction_pressure: BTreeMap<CountryId, u32>,
    /// Turn we last lifted our sanction on each target. A sanction lifted
    /// because its cause passed returns only for a new cause: we re-impose
    /// or re-join it only after the target commits a new hostile act
    /// ([`sanctions::new_hostile_act`]). Forgotten once one is seen. This
    /// replaces issue 15's `lifted_beside`, which barred only the
    /// sanctions in force at the lift, so a newly arrived sanctioner
    /// re-opened the same grievance (coalition churn, issue 19).
    lifted_at: BTreeMap<CountryId, u32>,
    /// Seed for the AI's own near-tie choices (deterministic per campaign).
    seed: u64,
}

impl Strategist {
    pub fn new() -> Self {
        Self::default()
    }

    /// A strategist whose near-tie choices are seeded per campaign.
    pub fn with_seed(seed: u64) -> Self {
        Strategist {
            seed,
            ..Self::default()
        }
    }

    fn is_assessment_turn(view: &ObserverView) -> bool {
        (view.turn + view.observer.0 as u32).is_multiple_of(ASSESSMENT_INTERVAL)
    }
}

struct Budget(u8);

impl Budget {
    fn take(&mut self, cost: u8) -> bool {
        if self.0 >= cost {
            self.0 -= cost;
            true
        } else {
            false
        }
    }
}

fn record(
    subject: String,
    kind: DecisionKind,
    counterpart: Option<CountryId>,
    score: &Score,
    chosen: bool,
    precedents: Vec<String>,
) -> DecisionRecord {
    DecisionRecord {
        subject,
        kind,
        counterpart,
        score: score.total(),
        chosen,
        lines: score.sorted(),
        precedents,
    }
}

fn code(view: &ObserverView, id: CountryId) -> String {
    foreign(view, id).map_or_else(|| format!("#{}", id.0), |f| f.code.clone())
}

/// "Why didn't we intervene?" (instrumentation; `brink diagnose`): for
/// each side of a war we don't lead, the best proxy option (band 2) and
/// the best use of force (band 3–4), with the stake and terms. Never
/// chosen: the decision itself is [`Strategist::involvement`], which only
/// runs for a side with a stake.
fn weigh_intervention(view: &ObserverView, w: &sim_core::War) -> Vec<DecisionRecord> {
    let options = [Side::Attacker, Side::Defender]
        .into_iter()
        .flat_map(|side| [(side, 2..=2), (side, 3..=4)]);
    options
        .filter_map(|(side, bands)| {
            let (o, s) = BandOption::ALL
                .iter()
                .filter(|o| bands.contains(&o.band))
                .map(|&o| (o, wa::band(view, w, side, o)))
                .max_by(|a, b| a.1.total().total_cmp(&b.1.total()).then(b.0.band.cmp(&a.0.band)))?;
            Some(record(
                format!(
                    "weigh intervention in {}–{} war for {} (stake {:.2}): {}",
                    code(view, w.attacker),
                    code(view, w.defender),
                    code(view, w.leader(side)),
                    wa::stake(view, w, side),
                    o.label()
                ),
                DecisionKind::Band(o.band),
                Some(w.leader(side)),
                &s,
                false,
                Vec::new(),
            ))
        })
        .collect()
}

impl Controller for Strategist {
    fn decide(&mut self, view: &ObserverView) -> Decision {
        let mut d = Decision::default();
        if !view.own.active {
            return d; // A dormant successor state has no government yet.
        }
        let mut budget = Budget(view.own.initiative.available());
        d.orders.extend(debt_brake(view));

        // 1. Answer proposals (every turn). Accepting a commitment costs 1.
        for p in &view.incoming_proposals {
            let score = evaluate::proposal(view, p);
            let cost = u8::from(p.kind.creates_commitment());
            let accept = score.total() > 0.0 && budget.take(cost);
            d.orders.push(Order::Respond { proposal: p.id, accept });
            d.records.push(record(
                format!("accept {:?} from {}", p.kind, code(view, p.from)),
                DecisionKind::AcceptProposal(ProposalFamily::of(p.kind)),
                Some(p.from),
                &score,
                accept,
                precedents(view, p.from, 3),
            ));
        }

        let assessment = Self::is_assessment_turn(view);
        self.decided_wars.retain(|id| view.wars.iter().any(|w| w.id == *id));

        // 2a. Wars we lead: continue or offer peace.
        for w in view.wars.iter().filter(|w| w.is_leader(view.observer)) {
            let side = w.side_of(view.observer).expect("leader");
            let other = w.leader(side.other());
            let offer_pending = w.peace_offers.iter().any(|o| o.0 == other);
            let ours_open = w
                .peace_offers
                .iter()
                .any(|o| o.0 == view.observer && o.1 + 2 > view.turn);
            if ours_open || !(assessment || offer_pending || w.started + 1 >= view.turn) {
                continue;
            }
            let score = wa::continue_war(view, w);
            let fight_on = score.total() > 0.0;
            if !fight_on {
                d.orders.push(Order::OfferPeace { war: w.id });
            }
            d.records.push(record(
                format!("continue the war with {}", code(view, other)),
                DecisionKind::Withdraw,
                Some(other),
                &score,
                fight_on,
                precedents(view, other, 3),
            ));
        }

        // 2a''. A transition crisis must be answered (scenario P3).
        if view.own.transition.pending_since.is_some() {
            let s = evaluate::reform(view);
            let reform = s.total() > 0.0;
            d.orders.push(if reform { Order::Reform } else { Order::Crackdown });
            let mut rec = s.clone();
            if !reform {
                // Recorded as the crackdown decision: its value is the
                // reform score's mirror, so the chosen record is positive.
                rec = Score::new();
                for l in &s.lines {
                    rec.add_term(l.term.clone(), l.label.clone(), -l.value);
                }
            }
            d.records.push(record(
                if reform {
                    "reform the regime".into()
                } else {
                    "crack down".into()
                },
                DecisionKind::Reform,
                None,
                &rec,
                rec.total() > 0.0,
                Vec::new(),
            ));
        }

        // 2a'. Nuclear use: existential situations only (DESIGN §11.4).
        if view.own.arsenal > 0 && view.own.strike_capacity >= 1.0 {
            for w in view.wars.iter().filter(|w| w.side_of(view.observer).is_some()) {
                let side = w.side_of(view.observer).expect("belligerent");
                let enemy = w.leader(side.other());
                let Some(target) = foreign(view, enemy) else { continue };
                if let Some(score) = wa::nuclear_use(view, w, target) {
                    let fire = score.total() > 0.0 && budget.take(1);
                    if fire {
                        d.orders.push(Order::NuclearStrike { target: enemy });
                    }
                    d.records.push(record(
                        format!("nuclear strike on {}", target.code),
                        DecisionKind::Band(4),
                        Some(enemy),
                        &score,
                        fire,
                        precedents(view, enemy, 3),
                    ));
                }
            }
        }

        // 2b. Mobilization (at war: every turn; otherwise on assessments).
        let fighting = view.wars.iter().any(|w| w.side_of(view.observer).is_some());
        if fighting || assessment {
            let (level, score) = wa::mobilization(view);
            let current = view.own.forces.mobilization;
            if level != current {
                let cost = u8::from(level.level() > current.level());
                if budget.take(cost) {
                    d.orders.push(Order::SetMobilization(level));
                    d.records.push(record(
                        format!("mobilize: {current:?} -> {level:?}"),
                        DecisionKind::Band(2),
                        None,
                        &score,
                        true,
                        Vec::new(),
                    ));
                }
            }
        }

        // 2c. Crisis involvement in others' wars: decide at once, re-score
        // (hold / withdraw / escalate) on assessment turns.
        for w in view.wars.iter().filter(|w| !w.is_leader(view.observer)) {
            let fresh = !self.decided_wars.contains(&w.id);
            if (fresh || assessment) && view.own.tier != sim_core::Tier::Minor {
                d.records.extend(weigh_intervention(view, w));
            }
            if !(fresh || assessment) {
                continue;
            }
            let Some((side, _)) = wa::favoured_side(view, w) else {
                continue;
            };
            self.decided_wars.insert(w.id);
            if let Some(r) = self.involvement(view, w, side, fresh, &mut budget, &mut d.orders) {
                d.records.push(r);
            }
        }

        if !assessment {
            return d;
        }

        // 2. Request a guarantee if threatened and unprotected.
        let cooled = self
            .last_guarantee_request
            .is_none_or(|t| view.turn >= t + 2 * ASSESSMENT_INTERVAL);
        if cooled && !is_protected(view) && top_threat(view).is_some_and(|(_, t)| t >= 30.0) {
            let best = view
                .others
                .iter()
                .map(|f| (f, evaluate::request_guarantee(view, f)))
                .max_by(|a, b| a.1.total().total_cmp(&b.1.total()).then(b.0.id.cmp(&a.0.id)));
            if let Some((patron, score)) = best {
                let ask = score.total() > 0.0 && budget.take(1);
                if ask {
                    d.orders.push(Order::ProposeTreaty {
                        to: patron.id,
                        kind: TreatyKind::Guarantee,
                    });
                    self.last_guarantee_request = Some(view.turn);
                }
                d.records.push(record(
                    format!("request guarantee from {}", patron.code),
                    DecisionKind::RequestGuarantee,
                    Some(patron.id),
                    &score,
                    ask,
                    precedents(view, patron.id, 3),
                ));
            }
        }

        // 3. Join others' sanctions (once per target).
        for s in &view.sanctions {
            let already = view
                .sanctions
                .iter()
                .any(|x| x.by == view.observer && x.target == s.target)
                || d.orders
                    .iter()
                    .any(|o| matches!(o, Order::Sanction { target } if *target == s.target));
            if s.by == view.observer || s.target == view.observer || already {
                continue;
            }
            if !self.may_resanction(view, s.target) {
                continue;
            }
            let mut score = evaluate::join_sanction(view, s);
            if goals::contains_target(&self.goals, s.target) {
                score.add("goal: pressure our main rival", 10.0);
            }
            let join = score.total() > 0.0 && budget.take(1);
            if join {
                d.orders.push(Order::Sanction { target: s.target });
            }
            d.records.push(record(
                format!("join {}'s sanctions on {}", code(view, s.by), code(view, s.target)),
                DecisionKind::JoinSanction,
                Some(s.by),
                &score,
                join,
                Vec::new(),
            ));
        }

        // 3b. Re-justify our own sanctions; lift with hysteresis (issue 15).
        self.review_sanctions(view, &mut d);

        // 4. Re-evaluate alliances, with two-assessment hysteresis.
        let alliances: Vec<_> = view
            .treaties
            .iter()
            .filter(|t| t.kind == TreatyKind::DefensiveAlliance && t.involves(view.observer))
            .copied()
            .collect();
        self.exit_pressure.retain(|id, _| alliances.iter().any(|t| t.id == *id));
        for t in alliances {
            let score = evaluate::stay_in_alliance(view, &t);
            let pressure = self.exit_pressure.entry(t.id).or_insert(0);
            *pressure = if score.total() < EXIT_THRESHOLD {
                *pressure + 1
            } else {
                0
            };
            let leave = *pressure >= EXIT_ASSESSMENTS && budget.take(1);
            if leave {
                d.orders.push(Order::CancelTreaty { treaty: t.id });
            }
            let partner = t.other(view.observer).expect("party");
            d.records.push(record(
                format!(
                    "stay in alliance with {} (bad assessments: {})",
                    code(view, partner),
                    *pressure
                ),
                DecisionKind::StayInAlliance,
                Some(partner),
                &score,
                !leave,
                precedents(view, partner, 3),
            ));
        }

        // 5. Strategic goals and the policies they drive.
        self.pursue_goals(view, &mut budget, &mut d);

        // 5b. Energy production policy for exporters (scenario P5, free).
        if view.own.energy_capacity > 0.0 && view.own.energy_net_exports > 0.3 * view.own.energy_capacity {
            let mut best: Option<(sim_core::ProductionPolicy, Score)> = None;
            for p in [
                sim_core::ProductionPolicy::Restrain,
                sim_core::ProductionPolicy::Normal,
                sim_core::ProductionPolicy::Flood,
            ] {
                let sc = evaluate::energy_policy(view, p);
                if best.as_ref().is_none_or(|b| sc.total() > b.1.total()) {
                    best = Some((p, sc));
                }
            }
            if let Some((p, sc)) = best {
                if p != view.own.energy_policy {
                    let stay = evaluate::energy_policy(view, view.own.energy_policy).total();
                    let mut rec = sc.clone();
                    rec.add_term("best_alternative", "staying with the current policy", -stay);
                    if rec.total() > 0.0 {
                        d.orders.push(Order::SetEnergyPolicy(p));
                        d.records.push(record(
                            format!("energy policy: {p:?}"),
                            DecisionKind::Flood,
                            None,
                            &rec,
                            true,
                            Vec::new(),
                        ));
                    }
                }
            }
        }

        // 5b'. World monetary stance, if we hold the reserve currency (P7, free).
        if view.own.financial_weight >= sim_core::money::RESERVE_WEIGHT {
            let current = view.money.stance;
            let stay = evaluate::monetary_stance(view, current);
            let best = sim_core::MonetaryStance::ALL
                .into_iter()
                .map(|st| (st, evaluate::monetary_stance(view, st)))
                .max_by(|a, b| a.1.total().total_cmp(&b.1.total()).then(b.0.cmp(&a.0)));
            if let Some((st, sc)) = best.filter(|b| b.0 != current) {
                let mut rec = sc.clone();
                rec.add_term("best_alternative", "keeping the current stance", -stay.total());
                if rec.total() > 0.0 {
                    d.orders.push(Order::SetMonetaryStance(st));
                    d.records.push(record(
                        format!("monetary stance: {st:?}"),
                        DecisionKind::MonetaryStance,
                        None,
                        &rec,
                        true,
                        Vec::new(),
                    ));
                }
            }
        }

        // 5c. Covert programmes (P10).
        let own = &view.own;
        // Minors (restricted actions) never start a programme of their own.
        if own.programme.is_none() && own.arsenal == 0 && own.tier != sim_core::Tier::Minor && budget.0 >= 1 {
            let s = evaluate::start_programme(view);
            if s.total() > 0.0 && budget.take(1) {
                d.orders.push(Order::StartProgramme);
                d.records.push(record(
                    "start a covert weapons programme".into(),
                    DecisionKind::Programme,
                    None,
                    &s,
                    true,
                    Vec::new(),
                ));
            }
        }
        if own.programme.is_some() {
            let s = evaluate::keep_programme(view);
            self.programme_pressure = if s.total() < EXIT_THRESHOLD {
                self.programme_pressure + 1
            } else {
                0
            };
            let stop = self.programme_pressure >= EXIT_ASSESSMENTS;
            if stop {
                d.orders.push(Order::StopProgramme);
                self.programme_pressure = 0;
            }
            d.records.push(record(
                "continue our weapons programme".into(),
                DecisionKind::Programme,
                None,
                &s,
                !stop,
                Vec::new(),
            ));
        }
        if own.arsenal > 0 && !own.arsenal_declared && budget.0 >= 1 {
            let s = evaluate::declare_arsenal(view);
            if s.total() > 0.0 && budget.take(1) {
                d.orders.push(Order::DeclareArsenal);
                d.records.push(record(
                    "declare our arsenal".into(),
                    DecisionKind::Test,
                    None,
                    &s,
                    true,
                    Vec::new(),
                ));
            }
        }
        // Only a small arsenal or a programme is ever given up.
        if own.arsenal <= 1 && (own.arsenal > 0 || own.programme.is_some()) {
            let s = evaluate::dismantle(view, self.goals.contains(&Goal::DevelopEconomy));
            if s.total() > 0.0 && budget.take(1) {
                d.orders.push(Order::DiscloseAndDismantle);
                d.records.push(record(
                    "disclose and dismantle".into(),
                    DecisionKind::Dismantle,
                    None,
                    &s,
                    true,
                    Vec::new(),
                ));
            }
        }

        // 6. War: only from a real quarrel, and never while already leading one.
        let leading = view.wars.iter().any(|w| w.is_leader(view.observer));
        if !leading && budget.0 >= 2 && view.own.tier != sim_core::Tier::Minor {
            let best = view
                .others
                .iter()
                .filter(|f| f.tension >= 30.0 || f.our_opinion_of_them <= -30.0)
                .filter(|f| {
                    !view
                        .wars
                        .iter()
                        .any(|w| w.side_of(f.id).is_some() && w.side_of(view.observer).is_some())
                })
                .flat_map(|f| {
                    [WarAim::Punitive, WarAim::Limited, WarAim::Major]
                        .map(|aim| (f, aim, wa::declare_war(view, f, aim)))
                })
                .max_by(|a, b| a.2.total().total_cmp(&b.2.total()).then(b.0.id.cmp(&a.0.id)));
            if let Some((target, aim, mut score)) = best {
                // An opening is worth only what can be won through it: like
                // every other victory-contingent gain in `declare_war` (D78
                // H1), the Exploit goal's bonus is weighted by our odds.
                if self.goals.contains(&Goal::Exploit(target.id)) {
                    score.add(
                        "goal: exploit the opening",
                        15.0 * (0.5 + view.own.personality.opportunism) * wa::war_odds(view, target, aim),
                    );
                }
                let go = score.total() > 0.0 && budget.take(2);
                if go {
                    d.orders.push(Order::DeclareWar { target: target.id, aim });
                }
                d.records.push(record(
                    format!("declare {aim:?} war on {}", target.code),
                    DecisionKind::Band(aim.band()),
                    Some(target.id),
                    &score,
                    go,
                    precedents(view, target.id, 3),
                ));
            }
        }
        d
    }
}

impl Strategist {
    /// May we sanction `target`? Always, unless we lifted a sanction on it
    /// and it has committed no new hostile act since.
    fn may_resanction(&self, view: &ObserverView, target: CountryId) -> bool {
        self.lifted_at
            .get(&target)
            .is_none_or(|&at| sanctions::new_hostile_act(view, target, at))
    }

    fn proposed_recently(&self, partner: CountryId, kind: u8, turn: u32) -> bool {
        self.last_proposed
            .get(&(partner, kind))
            .is_some_and(|&(t, n)| turn < t + 3 * ASSESSMENT_INTERVAL * n * n)
    }

    fn note_proposal(&mut self, partner: CountryId, kind: u8, turn: u32) {
        let e = self.last_proposed.entry((partner, kind)).or_insert((turn, 0));
        *e = (turn, e.1 + 1);
    }

    /// Strategic reassessment (DESIGN §14.2): re-score goals, then let them
    /// set the military budget, propose trade and alliances, offer
    /// guarantees, retrench, and sanction the main rival.
    fn pursue_goals(&mut self, view: &ObserverView, budget: &mut Budget, d: &mut Decision) {
        let me = view.observer;
        let chosen = goals::choose(view, &self.goals, self.seed);
        let new_goals: Vec<Goal> = chosen.iter().map(|g| g.0).collect();
        if new_goals != self.goals {
            let mut s = Score::new();
            for (g, gs) in &chosen {
                s.add_term(
                    format!("goal:{g:?}"),
                    format!("{} ({:.0})", g.label(view), gs.total()),
                    gs.total(),
                );
            }
            let names: Vec<String> = new_goals.iter().map(|g| g.label(view)).collect();
            d.records.push(record(
                format!(
                    "strategic goals: {}",
                    if names.is_empty() {
                        "none".into()
                    } else {
                        names.join(", ")
                    }
                ),
                DecisionKind::Goals,
                None,
                &s,
                s.total() > 0.0,
                Vec::new(),
            ));
        }
        self.goals = new_goals;
        let g = self.goals.clone();
        let has = |x: Goal| g.contains(&x);

        // Budget: military share toward what threats and goals justify.
        let target = goals::military_target(view, &g);
        let current = view.own.budget_target.military;
        let delta = target.total() / 100.0 - current;
        if delta.abs() >= 0.03 {
            let mut s = Score::new();
            let sign = if delta > 0.0 { 1.0 } else { -1.0 };
            for l in &target.lines {
                s.add_term(l.term.clone(), l.label.clone(), sign * l.value);
            }
            s.add("current military share", -sign * 100.0 * current);
            let shares = goals::budget_for(view.own.budget_target, target.total() / 100.0, &g);
            d.orders.push(Order::SetBudget(shares));
            d.records.push(record(
                format!(
                    "{} military spending to {:.0}% of the budget",
                    if delta > 0.0 { "raise" } else { "cut" },
                    100.0 * shares.military
                ),
                DecisionKind::Budget,
                None,
                &s,
                true,
                Vec::new(),
            ));
        }

        // Trade: development and bloc leadership seek agreements.
        if (has(Goal::DevelopEconomy) || has(Goal::LeadBloc)) && budget.0 >= 1 {
            let best = view
                .others
                .iter()
                .filter(|f| {
                    !view
                        .wars
                        .iter()
                        .any(|w| w.side_of(f.id).is_some() && w.side_of(me).is_some())
                })
                .filter_map(|f| {
                    let shallow = view
                        .treaties
                        .iter()
                        .find(|t| matches!(t.kind, TreatyKind::Trade { .. }) && t.involves(me) && t.involves(f.id));
                    match shallow.map(|t| t.kind) {
                        None if f.our_opinion_of_them >= 0.0 => Some((f, false)),
                        Some(TreatyKind::Trade { deep: false })
                            if f.our_opinion_of_them >= 25.0 && has(Goal::DevelopEconomy) =>
                        {
                            Some((f, true))
                        }
                        _ => None,
                    }
                })
                .filter(|(f, deep)| !self.proposed_recently(f.id, u8::from(*deep), view.turn))
                .map(|(f, deep)| (f, deep, evaluate::propose_trade(view, f, deep)))
                .max_by(|a, b| a.2.total().total_cmp(&b.2.total()).then(b.0.id.cmp(&a.0.id)));
            if let Some((f, deep, s)) = best {
                let go = s.total() > 0.0 && budget.take(1);
                if go {
                    d.orders.push(Order::ProposeTreaty {
                        to: f.id,
                        kind: TreatyKind::Trade { deep },
                    });
                    self.note_proposal(f.id, u8::from(deep), view.turn);
                }
                d.records.push(record(
                    format!(
                        "propose {} trade with {}",
                        if deep { "deep" } else { "shallow" },
                        f.code
                    ),
                    DecisionKind::Propose(ProposalFamily::Trade),
                    Some(f.id),
                    &s,
                    go,
                    Vec::new(),
                ));
            }
        }

        // Minors (restricted action set, scenario-1980 P1): budget and trade
        // only; no coalition building, guarantees or sanctions of their own.
        if view.own.tier == sim_core::Tier::Minor {
            return;
        }

        // Coalition: an alliance against the main threat.
        let threat_target = g.iter().find_map(|x| match x {
            Goal::Contain(c) | Goal::BalanceHegemon(c) => Some(*c),
            _ => None,
        });
        if let (Some(t), true) = (threat_target, has(Goal::LeadBloc) || threat_target.is_some()) {
            if let Some(against) = foreign(view, t) {
                let best = view
                    .others
                    .iter()
                    .filter(|f| f.id != t && !crate::inputs::has_treaty(view, TreatyKind::DefensiveAlliance, me, f.id))
                    .filter(|f| f.our_opinion_of_them >= 15.0)
                    .filter(|f| !self.proposed_recently(f.id, 2, view.turn))
                    .map(|f| (f, evaluate::propose_alliance(view, f, against)))
                    .max_by(|a, b| a.1.total().total_cmp(&b.1.total()).then(b.0.id.cmp(&a.0.id)));
                if let Some((f, s)) = best {
                    let go = s.total() > 0.0 && budget.take(1);
                    if go {
                        d.orders.push(Order::ProposeTreaty {
                            to: f.id,
                            kind: TreatyKind::DefensiveAlliance,
                        });
                        self.note_proposal(f.id, 2, view.turn);
                    }
                    d.records.push(record(
                        format!("propose an alliance to {} against {}", f.code, against.code),
                        DecisionKind::Propose(ProposalFamily::DefensiveAlliance),
                        Some(f.id),
                        &s,
                        go,
                        Vec::new(),
                    ));
                }
            }
        }

        // Guarantees for threatened friends (protect clients / lead the bloc).
        if (has(Goal::ProtectClients) || has(Goal::LeadBloc)) && budget.0 >= 1 {
            let best = view
                .others
                .iter()
                .filter(|f| !crate::inputs::has_treaty(view, TreatyKind::Guarantee, me, f.id))
                .filter(|f| !crate::inputs::has_treaty(view, TreatyKind::DefensiveAlliance, me, f.id))
                .filter(|f| goals::danger(view, f.id) >= 0.4 && f.our_opinion_of_them >= 10.0)
                .filter(|f| !self.proposed_recently(f.id, 3, view.turn))
                .map(|f| (f, evaluate::offer_guarantee(view, f)))
                .max_by(|a, b| a.1.total().total_cmp(&b.1.total()).then(b.0.id.cmp(&a.0.id)));
            if let Some((f, s)) = best {
                let go = s.total() > 0.0 && budget.take(1);
                if go {
                    d.orders.push(Order::IssueGuarantee { to: f.id });
                    self.note_proposal(f.id, 3, view.turn);
                }
                d.records.push(record(
                    format!("guarantee {} unasked", f.code),
                    DecisionKind::Propose(ProposalFamily::Guarantee),
                    Some(f.id),
                    &s,
                    go,
                    Vec::new(),
                ));
            }
        }

        // Retrenchment: re-evaluate guarantees we issued (2-assessment rule).
        let issued: Vec<_> = view
            .treaties
            .iter()
            .filter(|t| t.kind == TreatyKind::Guarantee && t.a == me)
            .copied()
            .collect();
        self.guarantee_pressure
            .retain(|id, _| issued.iter().any(|t| t.id == *id));
        for t in issued {
            let Some(client) = foreign(view, t.b) else { continue };
            let s = evaluate::keep_guarantee(view, client, has(Goal::DevelopEconomy) && !has(Goal::ProtectClients));
            let pressure = self.guarantee_pressure.entry(t.id).or_insert(0);
            *pressure = if s.total() < EXIT_THRESHOLD { *pressure + 1 } else { 0 };
            let drop = *pressure >= EXIT_ASSESSMENTS && budget.take(1);
            if drop {
                d.orders.push(Order::CancelTreaty { treaty: t.id });
            }
            d.records.push(record(
                format!("keep guarantee to {} (bad assessments: {})", client.code, *pressure),
                DecisionKind::StayInAlliance,
                Some(client.id),
                &s,
                !drop,
                precedents(view, client.id, 3),
            ));
        }
        // Patron re-evaluation: every stream we fund is re-scored; two bad
        // assessments in a row and it is cut (clients get dropped when the
        // budget tightens or the purpose fades).
        let contain = threat_target;
        let pressed = has(Goal::RestoreStability) || view.own.debt_ratio() > 0.9;
        let streams: Vec<_> = view.streams.iter().filter(|s| s.from == me).copied().collect();
        self.stream_pressure
            .retain(|k, _| streams.iter().any(|s| (s.to, s.kind as u8) == *k));
        for st in streams {
            let Some(client) = foreign(view, st.to) else { continue };
            let s = if st.sale {
                evaluate::sell_arms(view, client, st.amount, st.covert, true)
            } else {
                evaluate::keep_stream(
                    view,
                    client,
                    &st,
                    contain,
                    pressed,
                    has(Goal::DevelopEconomy) && !has(Goal::ProtectClients),
                )
            };
            let pressure = self.stream_pressure.entry((st.to, st.kind as u8)).or_insert(0);
            *pressure = if s.total() < EXIT_THRESHOLD { *pressure + 1 } else { 0 };
            let cut = *pressure >= EXIT_ASSESSMENTS;
            if cut {
                d.orders.push(match st.kind {
                    sim_core::StreamKind::Aid => Order::StopStream { to: st.to },
                    sim_core::StreamKind::Arms => Order::StopArmsStream { to: st.to },
                });
            }
            d.records.push(record(
                format!(
                    "keep {} to {} (bad assessments: {})",
                    if st.sale { "arms sales".to_string() } else { format!("{:?} stream", st.kind) },
                    client.code,
                    *pressure
                ),
                DecisionKind::StreamStop,
                Some(client.id),
                &s,
                !cut,
                precedents(view, client.id, 3),
            ));
        }

        // Arms to a threatened friend (peacetime patronage, DESIGN §7.2).
        if (contain.is_some() || has(Goal::ProtectClients) || has(Goal::LeadBloc)) && !pressed && budget.0 >= 1 {
            let best = view
                .others
                .iter()
                .filter(|f| f.tier != sim_core::Tier::Playable || f.military.value < view.own.power())
                .filter(|f| {
                    !wa::arming(view, f.id)
                        && (f.our_opinion_of_them >= 20.0 || crate::inputs::affinity_with(view, f) >= 1.0)
                })
                .filter(|f| goals::danger(view, f.id) >= 0.4)
                .filter(|f| !self.proposed_recently(f.id, 4, view.turn))
                .map(|f| (f, evaluate::arm_client(view, f, contain)))
                .max_by(|a, b| a.1.total().total_cmp(&b.1.total()).then(b.0.id.cmp(&a.0.id)));
            if let Some((f, s)) = best {
                let go = s.total() > 0.0 && wa::arms_route(view, f.id) && budget.take(1);
                if go {
                    d.orders.push(Order::StartArmsStream {
                        to: f.id,
                        amount: wa::gift_amount(view, f.id),
                        covert: false,
                    });
                    self.note_proposal(f.id, 4, view.turn);
                }
                d.records.push(record(
                    format!("arm {}", f.code),
                    DecisionKind::StreamStart,
                    Some(f.id),
                    &s,
                    go,
                    precedents(view, f.id, 3),
                ));
            }
        }

        // Finance the war against our main threat (issue 23): when the
        // state we are containing is fighting someone else, that war does
        // our containment for us; a pledge of money each assessment while
        // it lasts (it ends with the war, and every pledge is on the record).
        if let (Some(x), false) = (contain, pressed) {
            let revenue = view.own.gdp * view.own.tax_rate;
            let amount = evaluate::WAR_FINANCE_SHARE * revenue;
            let best = view
                .wars
                .iter()
                .filter(|w| w.side_of(me).is_none())
                .filter_map(|w| {
                    let side = w.side_of(x)?;
                    let friend = foreign(view, w.leader(side.other()))?;
                    let enemy = foreign(view, x)?;
                    let at_war_with_us = view.wars.iter().any(|v| {
                        v.side_of(me).is_some() && v.side_of(friend.id).is_some_and(|s| Some(s) != v.side_of(me))
                    });
                    (!at_war_with_us).then(|| (friend, evaluate::fund_war(view, w, friend, enemy, amount)))
                })
                .max_by(|a, b| a.1.total().total_cmp(&b.1.total()).then(b.0.id.cmp(&a.0.id)));
            if let Some((f, s)) = best {
                let go = s.total() > 0.0 && budget.take(1);
                if go {
                    d.orders.push(Order::Aid {
                        to: f.id,
                        amount: amount * ASSESSMENT_INTERVAL as f64,
                    });
                }
                d.records.push(record(
                    format!("finance {}'s war against {}", f.code, code(view, x)),
                    DecisionKind::StreamStart,
                    Some(f.id),
                    &s,
                    go,
                    precedents(view, f.id, 3),
                ));
            }
        }

        // Arms we buy (D58): re-scored; two bad assessments and we cancel.
        let purchases: Vec<_> = view.streams.iter().filter(|s| s.to == me && s.sale).copied().collect();
        self.purchase_pressure
            .retain(|k, _| purchases.iter().any(|s| s.from == *k));
        for st in purchases {
            let Some(seller) = foreign(view, st.from) else { continue };
            let s = evaluate::keep_purchase(view, seller, &st);
            let pressure = self.purchase_pressure.entry(st.from).or_insert(0);
            *pressure = if s.total() < EXIT_THRESHOLD { *pressure + 1 } else { 0 };
            let cut = *pressure >= EXIT_ASSESSMENTS;
            if cut {
                d.orders.push(Order::CancelArmsPurchase { from: st.from });
            }
            d.records.push(record(
                format!("keep buying arms from {} (bad assessments: {})", seller.code, *pressure),
                DecisionKind::SellArms,
                Some(seller.id),
                &s,
                !cut,
                precedents(view, seller.id, 3),
            ));
        }

        // Arms for sale (D58): an arms industry looks for customers who need
        // arms and can pay, within its order book.
        if view.own.arms_industry > 0.0 && budget.0 >= 1 {
            let unit = |to| sim_core::diplomacy::arms_value(sim_core::StreamKind::Arms, 1.0) * evaluate::market_price(view, to);
            let booked: f64 = view
                .streams
                .iter()
                .filter(|s| s.from == me && s.sale)
                .map(|s| s.amount * unit(s.to))
                .sum();
            let room = sim_core::economy::export_capacity(&view.own) - booked;
            let mut best: Option<(&sim_core::view::ForeignView, f64, bool, crate::Score)> = None;
            for f in view.others.iter() {
                let at_war = view.wars.iter().any(|w| w.side_of(f.id).is_some());
                let sanctioned = view.sanctions.iter().any(|s| s.target == f.id);
                let need = goals::danger(view, f.id).max(if at_war { 1.0 } else { 0.0 });
                // Countries import what they can't build: we sell only kit
                // better than theirs, or equal kit to a country with no
                // industry of its own.
                let better = view.own.military_tech > f.military_tech
                    || (view.own.military_tech == f.military_tech && f.arms_industry == 0.0);
                if room <= 0.0
                    || !better
                    || wa::arming(view, f.id)
                    || (need < 0.35 && !sanctioned)
                    || !wa::arms_route(view, f.id)
                    || self.proposed_recently(f.id, 5, view.turn)
                {
                    continue;
                }
                // What they could spend (a share of their budget, scaled by
                // need), less what other suppliers already sell them.
                let spend = f.gdp * 0.25 * sim_core::economy::MAX_PURCHASE_SHARE * need.max(0.5);
                let supplied: f64 = view
                    .streams
                    .iter()
                    .filter(|s| s.to == f.id && s.sale)
                    .map(|s| s.amount * unit(f.id))
                    .sum();
                let open = (spend - supplied).min(room);
                if open < 0.25 * spend {
                    continue;
                }
                let amount = open / unit(f.id);
                for covert in [false, true] {
                    let s = evaluate::sell_arms(view, f, amount, covert, false);
                    if best.as_ref().is_none_or(|b| s.total() > b.3.total()) {
                        best = Some((f, amount, covert, s));
                    }
                }
            }
            if let Some((f, amount, covert, s)) = best {
                let go = s.total() > 0.0 && budget.take(1);
                if go {
                    d.orders.push(Order::SellArms { to: f.id, amount, covert });
                    self.note_proposal(f.id, 5, view.turn);
                }
                d.records.push(record(
                    format!("sell arms to {}{}", f.code, if covert { " (covertly)" } else { "" }),
                    DecisionKind::SellArms,
                    Some(f.id),
                    &s,
                    go,
                    precedents(view, f.id, 3),
                ));
            }
        }

        // Sanction the main rival.
        if let Some(t) = threat_target {
            let already = view.sanctions.iter().any(|s| s.by == me && s.target == t)
                || d.orders
                    .iter()
                    .any(|o| matches!(o, Order::Sanction { target } if *target == t));
            if let (Some(target), false, true) = (foreign(view, t), already, self.may_resanction(view, t)) {
                let s = evaluate::impose_sanction(view, target, true);
                let go = s.total() > 0.0 && budget.take(1);
                if go {
                    d.orders.push(Order::Sanction { target: t });
                }
                d.records.push(record(
                    format!("sanction {}", target.code),
                    DecisionKind::ImposeSanction,
                    Some(t),
                    &s,
                    go,
                    precedents(view, t, 3),
                ));
            }
        }
    }

    /// Review each sanction we impose (assessment turns): keep it while the
    /// case for it holds, lift it after [`EXIT_ASSESSMENTS`] bad
    /// assessments in a row once it has stood [`sanctions::MIN_SANCTION_TURNS`].
    /// Lifting is free (a standing setting).
    fn review_sanctions(&mut self, view: &ObserverView, d: &mut Decision) {
        let me = view.observer;
        let ours: Vec<sim_core::Sanction> = view.sanctions.iter().filter(|s| s.by == me).copied().collect();
        self.sanction_pressure
            .retain(|t, _| ours.iter().any(|s| s.target == *t));
        // Forget a lift once the target has given a new cause.
        self.lifted_at
            .retain(|&t, &mut at| !sanctions::new_hostile_act(view, t, at));
        for s in ours {
            if view.turn < s.since + sanctions::MIN_SANCTION_TURNS {
                continue;
            }
            let Some(target) = foreign(view, s.target) else {
                continue;
            };
            let score = sanctions::keep_sanction(view, &s, goals::contains_target(&self.goals, s.target));
            let pressure = self.sanction_pressure.entry(s.target).or_insert(0);
            *pressure = if score.total() < 0.0 { *pressure + 1 } else { 0 };
            let bad = *pressure;
            let lift = bad >= EXIT_ASSESSMENTS;
            if lift {
                d.orders.push(Order::LiftSanction { target: s.target });
                self.sanction_pressure.remove(&s.target);
                self.lifted_at.insert(s.target, view.turn);
            }
            d.records.push(record(
                format!("keep sanctions on {} (bad assessments: {bad})", target.code),
                DecisionKind::KeepSanction,
                Some(s.target),
                &score,
                !lift,
                precedents(view, s.target, 3),
            ));
        }
    }

    /// Choose an involvement band for `side` in `w` and emit the orders that
    /// move us there from where we are. The record's value is the margin of
    /// the chosen option over the best alternative (so a chosen decision is
    /// always positive and its lines still sum to it).
    fn involvement(
        &mut self,
        view: &ObserverView,
        w: &War,
        side: Side,
        fresh: bool,
        budget: &mut Budget,
        orders: &mut Vec<Order>,
    ) -> Option<DecisionRecord> {
        let current = wa::current_band(view, w, side);
        let max_band = if view.own.tier == sim_core::Tier::Minor { 1 } else { 4 };
        let mut scored: Vec<(BandOption, Score)> = BandOption::ALL
            .iter()
            .filter(|o| o.band <= max_band)
            .map(|&o| (o, wa::band(view, w, side, o)))
            .collect();
        // Once involved, staying put gets the hysteresis bonus (a fresh
        // crisis already has the status-quo term).
        for (o, s) in &mut scored {
            if *o == current && !fresh {
                s.add("no change", BAND_HYSTERESIS);
            }
        }
        scored.sort_by(|a, b| b.1.total().total_cmp(&a.1.total()).then(a.0.band.cmp(&b.0.band)));
        let (best, best_score) = scored[0].clone();
        let (alt, alt_score) = scored[1].clone();
        let friend = w.leader(side);
        let enemy = w.leader(side.other());

        let mut emitted = Vec::new();
        if best != current {
            // Step down from what we're doing now.
            if current.band >= 3 && best.band < 3 {
                emitted.push(Order::LeaveWar { war: w.id });
            }
            if current.band == 2 && best.band != 2 {
                emitted.push(Order::StopArmsStream { to: friend });
            }
            // Sanctions are left alone here: they may serve other goals,
            // and lifting them on every re-score made them flicker. They
            // are lifted by `review_sanctions`, with hysteresis.
            let sanctioning = view
                .sanctions
                .iter()
                .any(|s| s.by == view.observer && s.target == enemy);
            let step_up = match best.band {
                1 if !sanctioning && self.may_resanction(view, enemy) => Some(Order::Sanction { target: enemy }),
                2 => Some(Order::StartArmsStream {
                    to: friend,
                    amount: wa::proxy_amount(view, friend),
                    covert: best.covert,
                }),
                b @ 3..=4 if w.side_of(view.observer).is_none() => Some(Order::JoinWar {
                    war: w.id,
                    side,
                    band: b,
                }),
                _ => None,
            };
            if let Some(order) = step_up {
                if budget.take(1) {
                    emitted.push(order);
                }
            }
        }
        let acted = !emitted.is_empty() || best == current;
        orders.extend(emitted);
        let mut score = best_score;
        let alt_total = alt_score.total();
        score.add_term(
            "best_alternative",
            format!("best alternative: {}", alt.label()),
            -alt_total,
        );
        Some(record(
            format!(
                "involvement in {}–{} war for {}: {} (was {})",
                code(view, w.attacker),
                code(view, w.defender),
                code(view, friend),
                best.label(),
                current.label()
            ),
            DecisionKind::Band(best.band),
            Some(friend),
            &score,
            acted && score.total() > 0.0,
            precedents(view, friend, 3),
        ))
    }
}
