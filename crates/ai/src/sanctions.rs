//! Sanction exit (issue 15): should a sanction we impose stay in force?
//!
//! On assessment turns each of our sanctions is re-justified by **the same
//! decisions that start sanctions** (`impose_sanction`, `join_sanction` for
//! every other sanction on the target), evaluated on a counterfactual view
//! without the sanction regime's own echo: the opinion and tension our
//! sanction (and any counter-sanction that answered it) put into the
//! relationship are consequences of sanctioning, not reasons to keep doing
//! it. Without that, every sanction justified itself forever (D75 follow-up
//! (a): 0 lifts in 80-turn runs).
//!
//! Hysteresis: the counterfactual case must fall below zero by the
//! **residual** the impose route would still see right after the lift (the
//! tension the sanctions added decays only 5% a turn; an answering
//! counter-sanction stands until its owner lifts it), plus [`LIFT_MARGIN`];
//! on [`crate::strategist::EXIT_ASSESSMENTS`] assessments in a row, and not
//! before [`MIN_SANCTION_TURNS`]. That keeps the impose route from
//! re-imposing at once.
//!
//! It is **not** flicker-proof by construction: the exit weighs only the
//! sanctions that came before ours (a coalition is not held by its
//! followers), while the strategist's join loop weighs every sanction on the
//! target. The flicker guard for that gap is `Strategist::lifted_at`: a
//! sanction we lifted is re-imposed or re-joined only after the target
//! commits a new hostile act ([`new_hostile_act`]). A sanction lifted
//! because its cause passed returns only for a new cause; grievance that
//! predates the lift was weighed in it and does not count again (issue 19;
//! issue 15's narrower guard, barring only the sanctions in force at the
//! lift, let each newly arriving sanctioner re-open it).

use sim_core::view::{ForeignView, StabilityBand};
use sim_core::{CountryId, DecisionKind, Government, ObserverView, Sanction};

use crate::inputs::foreign;
use crate::reflexes::{self, Parties};
use crate::Score;

/// Margin on top of the computed residual: one assessment of drift in the
/// inputs the counterfactual does not touch (threat estimates, decaying
/// event modifiers).
pub const LIFT_MARGIN: f64 = 2.5;
/// No sanction is reviewed for lifting before it has stood this long (one
/// year: the 1980 grain embargo lasted fifteen months).
pub const MIN_SANCTION_TURNS: u32 = 4;
/// Leverage for reform: an autocracy in visible crisis counts each
/// democracy's sanction as a reason to reform (`evaluate::reform`, 6 per
/// democratic sanctioner); lifting it then throws that away. South Africa
/// 1986-91: sanctions held through the crisis until reform. Keep-only, so
/// it can delay a lift but never cause a re-imposition.
///
/// Added because the new lifts broke `p3::chronic_crises_end_in_reform_often_enough`
/// (review 15, round 2 ablation: off, PRK reforms 6/20; on, 9/20). Entry has
/// no matching term, though the same logic argues for imposing on a
/// wobbling regime (South Africa 1985-86): an open follow-up (D80). In
/// practice it holds Western sanctions on PRK almost permanently (+5.4 of 6
/// on average), because PRK spends most turns in Unrest or worse.
const REFORM_LEVERAGE: f64 = 6.0;
/// "Their war goes on": a sanction on a belligerent we stand against is our
/// band-1 involvement in that war; the crisis system re-imposes it while
/// the war lasts, so it is not lifted before the war ends.
const WAR_SANCTION: f64 = 30.0;

/// The observer's view on the turn after lifting: our sanction gone and the
/// target's standing −25 replaced by the −10 memory (what sim-core does),
/// everything else, tension included, as it is now.
pub fn post_lift(view: &ObserverView, sanction: &Sanction) -> ObserverView {
    let me = view.observer;
    let mut v = view.clone();
    v.sanctions.retain(|s| !(s.by == me && s.target == sanction.target));
    let step = sim_core::diplomacy::PAST_SANCTION_OPINION - sim_core::diplomacy::SANCTION_OPINION;
    if let Some(f) = v.others.iter_mut().find(|f| f.id == sanction.target) {
        f.their_opinion_of_us = (f.their_opinion_of_us + step).min(100.0);
    }
    v
}

/// The best case for starting `sanction` on `on` by the strategist's routes:
/// imposing on the main rival (a Contain/Balance goal), or joining a
/// sanction that came before ours (a coalition we lead is not held by its
/// followers). With neither route open, an impose without the goal, which
/// only a grave grievance carries.
fn best_entry(on: &ObserverView, view: &ObserverView, sanction: &Sanction, goal: bool) -> (&'static str, Score) {
    let Some(target) = foreign(on, sanction.target) else {
        return ("impose", Score::new());
    };
    let mut best: Option<(&str, Score)> = goal.then(|| ("impose", crate::evaluate::impose_sanction(on, target, true)));
    for other in preceding(view, sanction) {
        let mut j = crate::evaluate::join_sanction(on, other);
        if goal {
            // As the strategist scores joining.
            j.add("goal: pressure our main rival", 10.0);
        }
        if best.as_ref().is_none_or(|b| j.total() > b.1.total()) {
            best = Some(("join", j));
        }
    }
    best.unwrap_or_else(|| ("impose", crate::evaluate::impose_sanction(on, target, false)))
}

/// The observer's view as it would read without our sanction on
/// `sanction.target` and without the echo of the sanction regime:
/// - our sanction is gone; the target's opinion of us carries the fading
///   "past sanctions" memory instead of the standing penalty (what sim-core
///   does on a lift);
/// - a counter-sanction the target imposed after ours (an answer to it) is
///   gone too, with its penalty on our opinion of them; one that came first
///   is a cause and stays;
/// - bilateral tension loses the baseline the sanctions added: +10 while
///   either side sanctions, and the hostility of each penalty that goes
///   (ours: −25 replaced by the −10 memory; their answer: −25).
pub fn counterfactual(view: &ObserverView, sanction: &Sanction) -> ObserverView {
    let me = view.observer;
    let mut cf = post_lift(view, sanction);
    let theirs = view
        .sanctions
        .iter()
        .find(|s| s.by == sanction.target && s.target == me)
        .copied();
    let answer = theirs.is_some_and(|t| t.since >= sanction.since);
    let penalty = -sim_core::diplomacy::SANCTION_OPINION;
    let past = -sim_core::diplomacy::PAST_SANCTION_OPINION;
    // Tension baseline the regime adds: the sanction term while any side
    // sanctions (kept if their earlier sanction still stands) and the
    // hostility of each penalty that goes away.
    let mut tension_echo = sim_core::tension::HOSTILITY_TO_TENSION * (penalty - past) / 2.0;
    if answer {
        tension_echo += sim_core::tension::HOSTILITY_TO_TENSION * penalty / 2.0;
    }
    if theirs.is_none() || answer {
        tension_echo += sim_core::tension::SANCTION_BASELINE;
    }
    if answer {
        cf.sanctions.retain(|s| !(s.by == sanction.target && s.target == me));
    }
    if let Some(f) = cf.others.iter_mut().find(|f| f.id == sanction.target) {
        if answer {
            f.our_opinion_of_them = (f.our_opinion_of_them + penalty).min(100.0);
        }
        f.tension = (f.tension - tension_echo).max(0.0);
    }
    cf
}

/// Value of keeping `sanction` (ours) in force: the best case for starting
/// it today on the counterfactual view, plus the residual echo and
/// [`LIFT_MARGIN`]. Negative = the
/// reasons have passed; lift it. `goal` is true when containing or
/// balancing the target is an active goal.
pub fn keep_sanction(view: &ObserverView, sanction: &Sanction, goal: bool) -> Score {
    let mut s = Score::new();
    let Some(target) = foreign(view, sanction.target) else {
        return s;
    };
    let cf = counterfactual(view, sanction);
    let (how, best) = best_entry(&cf, view, sanction, goal);
    // The residual the lift must clear: how much more a route that stays
    // open after the lift would score the turn after it than the
    // counterfactual does (tension still decaying at 5% a turn, an
    // answering counter-sanction still standing). Only the impose route on
    // our main rival is weighed here; after the lift every route is barred
    // until the target gives a new cause (`Strategist::lifted_at`).
    let residual = if goal {
        let after = post_lift(view, sanction);
        foreign(&after, sanction.target).map_or(0.0, |t| {
            (crate::evaluate::impose_sanction(&after, t, true).total() - best.total()).max(0.0)
        })
    } else {
        0.0
    };
    for l in &best.lines {
        s.add_term(l.term.clone(), format!("{how} today: {}", l.label), l.value);
    }
    let in_crisis = matches!(
        target.stability_band,
        Some(StabilityBand::Collapse | StabilityBand::Crisis | StabilityBand::Unrest)
    );
    if view.own.government == Government::Democracy && target.government != Government::Democracy && in_crisis {
        s.add("leverage: their regime is in crisis", REFORM_LEVERAGE);
    }
    if at_war_against(view, target) {
        s.add("their war goes on", WAR_SANCTION);
    }
    s.add("case to re-impose the turn after a lift (residual)", residual);
    s.add("margin", LIFT_MARGIN);
    reflexes::apply(
        view,
        DecisionKind::KeepSanction,
        Parties {
            counterpart: Some(target),
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Has `target` given us a new cause, by a hostile act we know of on or
/// after turn `since` (the turn of our lift: acts that turn were not
/// weighed in it)? Every hostile act the simulation records is a public or
/// exposed Coercion entry in the Event Ledger with the aggressor as actor.
/// A cause is an act against us (a sanction, a denunciation, an exposed
/// covert arms stream, a war or strike), or an act against the order every
/// state has a stake in, whoever its victim: a war or punitive strike,
/// an exposed weapons programme, nuclear use (Kuwait 1990 brought back
/// sanctions on Iraq from states it never touched). Their quarrel with a
/// third party (a sanction or a denunciation) is not our cause. The
/// observer's ledger is fog-filtered, so only acts we have seen count.
pub fn new_hostile_act(view: &ObserverView, target: CountryId, since: u32) -> bool {
    view.ledger.iter().any(|e| {
        e.actor == target
            && e.kind == sim_core::EntryKind::Coercion
            && e.turn >= since as i32
            && is_cause_for(e.counterpart, e.cause_code, view.observer)
    })
}

/// Is a hostile (Coercion) act against `counterpart` of kind `code` a cause
/// for `observer`: aimed at it, or an act against the common order?
pub fn is_cause_for(counterpart: CountryId, code: sim_core::CauseCode, observer: CountryId) -> bool {
    use sim_core::CauseCode::{NuclearStrike, ProgrammeExposed, PunitiveStrike, WarDeclared};
    counterpart == observer || matches!(code, WarDeclared | PunitiveStrike | ProgrammeExposed | NuclearStrike)
}

/// Other countries' sanctions on the same target imposed before `ours`
/// (sim-core keeps sanctions in the order they were imposed).
pub fn preceding<'a>(view: &'a ObserverView, ours: &Sanction) -> impl Iterator<Item = &'a Sanction> {
    let pos = view
        .sanctions
        .iter()
        .position(|s| s == ours)
        .unwrap_or(view.sanctions.len());
    let ours = *ours;
    view.sanctions[..pos]
        .iter()
        .filter(move |x| x.target == ours.target && x.by != ours.by)
}

/// Does `target` lead a side, in a war we stay out of, against the side we
/// favour?
fn at_war_against(view: &ObserverView, target: &ForeignView) -> bool {
    view.wars.iter().any(|w| {
        w.side_of(view.observer).is_none()
            && w.is_leader(target.id)
            && w.side_of(target.id)
                .is_some_and(|side| crate::war::favoured_side(view, w).is_some_and(|(favoured, _)| favoured != side))
    })
}
