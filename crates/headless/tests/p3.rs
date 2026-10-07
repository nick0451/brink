//! v0.2 P3: regime transition (scenario-1980 P3; approval item 5).

use std::path::PathBuf;

use ai::{Controller, Strategist};
use sim_core::{
    observe, resolve_turn, CountryId, DiplomaticEvent, Government, Order, OrderSet, TransitionKind, WorldState,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world() -> WorldState {
    scenario::build(
        &scenario::load(root().join("data/scenarios/1980.ron")).unwrap(),
        Some(1),
    )
    .unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

fn step(w: &mut WorldState, acts: Vec<(CountryId, Vec<Order>)>) -> sim_core::TurnReport {
    resolve_turn(
        w,
        acts.into_iter()
            .map(|(country, orders)| OrderSet { country, orders })
            .collect(),
    )
}

/// Hold a country in crisis conditions until the offer comes.
fn into_crisis(w: &mut WorldState, c: CountryId) -> u32 {
    for t in 0..20 {
        let x = w.country_mut(c);
        x.stability = 25.0;
        x.legitimacy = 25.0;
        let r = step(w, vec![]);
        if r.events
            .iter()
            .any(|e| matches!(e, DiplomaticEvent::TransitionCrisis { country } if *country == c))
        {
            return t;
        }
    }
    panic!("no transition crisis");
}

#[test]
fn a_sustained_crisis_forces_a_choice_and_reform_changes_the_regime() {
    let mut w = world();
    let sov = id(&w, "SOV");
    let t = into_crisis(&mut w, sov);
    assert!(t + 1 >= sim_core::transition::CRISIS_TURNS, "crisis needs time: {t}");
    assert!(w.country(sov).transition.pending_since.is_some());
    let before = w.country(sov).personality;
    let before_norm = w.country(sov).military_norm;
    let r = step(&mut w, vec![(sov, vec![Order::Reform])]);
    assert!(r.events.iter().any(|e| matches!(
        e,
        DiplomaticEvent::Transition {
            kind: TransitionKind::Reform,
            ..
        }
    )));
    let c = w.country(sov);
    assert_eq!(c.government, Government::Democracy);
    assert!(c.personality.ideology < before.ideology, "the B.18 reformer vector");
    assert_eq!(c.alignment, None, "the reformed regime leaves its bloc");
    assert_eq!(c.command_drag, 0.0, "market reform ends the command economy's drag");
    assert!(
        (c.military_norm - before_norm * sim_core::transition::REFORM_MILITARY_NORM).abs() < 1e-12,
        "the new regime habitually spends less on the army"
    );
}

#[test]
fn crackdown_buys_time_and_breeds_coups() {
    let mut w = world();
    let vnm = id(&w, "VNM");
    into_crisis(&mut w, vnm);
    let stab = w.country(vnm).stability;
    step(&mut w, vec![(vnm, vec![Order::Crackdown])]);
    let c = w.country(vnm);
    assert!(c.stability > stab);
    assert_eq!(c.transition.crackdowns, 1);
    assert!(c.transition.quiet_until > w.turn, "a reprieve");
    // Unanswered crises default to crackdown.
    let mut w = world();
    let ddr = id(&w, "DDR");
    into_crisis(&mut w, ddr);
    let mut defaulted = false;
    for _ in 0..3 {
        let r = step(&mut w, vec![]);
        defaulted |= r.events.iter().any(|e| {
            matches!(
                e,
                DiplomaticEvent::Transition {
                    kind: TransitionKind::Crackdown,
                    ..
                }
            )
        });
    }
    assert!(defaulted);
}

#[test]
fn collapse_is_a_disorderly_transition() {
    let mut w = world();
    let irn = id(&w, "IRN");
    // A catastrophe, not a bad quarter: the stability target itself is gone.
    w.country_mut(irn).stability = 5.0;
    w.country_mut(irn).war_weariness = 100.0;
    let r = step(&mut w, vec![]);
    assert!(r.events.iter().any(|e| matches!(
        e,
        DiplomaticEvent::Transition {
            kind: TransitionKind::Collapse,
            ..
        }
    )));
    assert!(w.country(irn).stability >= 35.0);
    // The reprieve: no second collapse straight away.
    w.country_mut(irn).stability = 5.0;
    w.country_mut(irn).war_weariness = 100.0;
    let r = step(&mut w, vec![]);
    assert!(!r.events.iter().any(|e| matches!(e, DiplomaticEvent::Transition { .. })));
}

#[test]
fn ai_answers_the_crisis_with_reasons() {
    let mut w = world();
    let sov = id(&w, "SOV");
    into_crisis(&mut w, sov);
    let d = Strategist::with_seed(1).decide(&observe(&w, sov));
    let r = d
        .records
        .iter()
        .find(|r| r.subject == "reform the regime" || r.subject == "crack down")
        .expect("answered");
    let sum: f64 = r.lines.iter().map(|l| l.value).sum();
    assert!((sum - r.score).abs() < 1e-9 && r.chosen && r.score > 0.0);
    assert!(d.orders.iter().any(|o| matches!(o, Order::Reform | Order::Crackdown)));
}

/// Regression target (2026-10-04, replacing "a transition somewhere in > 90%
/// of runs", which a model where one country transitions every run and
/// nobody else ever does could pass): crisis pressure is spread across
/// regimes, no single regime dominates, and every crisis has an
/// identifiable stressor at the moment it fires.
#[test]
fn crises_are_spread_and_each_has_an_identifiable_stressor() {
    let def = scenario::load(root().join("data/scenarios/1980.ron")).unwrap();
    let runs = headless::run_batch(&def, &(1..=40).collect::<Vec<_>>(), 80, 8).unwrap();
    let mut by_country: std::collections::BTreeMap<String, u32> = Default::default();
    let mut unexplained = Vec::new();
    for r in &runs {
        for (turn, e) in &r.events {
            match e {
                DiplomaticEvent::Transition { country, .. } => {
                    *by_country.entry(def.countries[country.index()].id.clone()).or_default() += 1;
                }
                DiplomaticEvent::TransitionCrisis { country } => {
                    let s = &r.samples[(*turn as usize).min(r.samples.len() - 1)].countries[country.index()];
                    let stressors = [
                        ("prosperity", s.prosperity < 50.0),
                        ("insecurity", s.security < 30.0),
                        ("military burden", s.burden > 3.0),
                        ("falling behind", s.behind > 3.0),
                        ("war weariness", s.war_weariness > 10.0),
                        ("contraction", s.growth < 0.0),
                        ("debt service", s.debt_service_hit > 3.0),
                        ("inflation", s.inflation_hit > 3.0),
                    ];
                    if !stressors.iter().any(|x| x.1) {
                        unexplained.push(format!("{} t{turn}", s.code));
                    }
                }
                _ => {}
            }
        }
    }
    let total: u32 = by_country.values().sum();
    println!("transitions by regime: {by_country:?}");
    assert!(by_country.len() >= 3, "crisis pressure reaches several regimes: {by_country:?}");
    let top = by_country.values().copied().max().unwrap_or(0);
    assert!(
        (top as f64) <= 0.5 * total as f64,
        "no single regime dominates transitions: {by_country:?}"
    );
    assert!(unexplained.is_empty(), "crises without a stressor: {unexplained:?}");
}

/// Issue 2 (2026-10-04): a coup changes the rulers but not the era's record
/// of failed repression; the record clears only after a long calm.
#[test]
fn a_coup_keeps_the_record_of_failed_repression_until_calm() {
    use sim_core::transition::{CALM_RESET, REPRIEVE};
    let mut w = world();
    let vnm = id(&w, "VNM");
    into_crisis(&mut w, vnm);
    step(&mut w, vec![(vnm, vec![Order::Crackdown])]);
    let first = w.country(vnm).transition.quiet_until - w.turn;
    // Sit out the reprieve, then the crisis returns.
    for _ in 0..first {
        step(&mut w, vec![]);
    }
    into_crisis(&mut w, vnm);
    step(&mut w, vec![(vnm, vec![Order::Crackdown])]);
    let t = &w.country(vnm).transition;
    let second = t.quiet_until - w.turn;
    assert_eq!(t.repressions, 2);
    assert_eq!(first, second, "every crackdown buys the same reprieve");
    assert!(first + 1 >= REPRIEVE);
    // Hold the crisis, unanswered, until hardliners strike.
    let mut couped = false;
    for _ in 0..80 {
        let x = w.country_mut(vnm);
        x.stability = 25.0;
        x.legitimacy = 25.0;
        let r = step(&mut w, vec![]);
        if r.events.iter().any(|e| {
            matches!(e, DiplomaticEvent::Transition { country, kind: TransitionKind::Coup } if *country == vnm)
        }) {
            couped = true;
            break;
        }
    }
    assert!(couped, "a coup in a long crisis with repeated crackdowns");
    let t = &w.country(vnm).transition;
    assert_eq!(t.crackdowns, 0, "the new junta's coup-risk count starts fresh");
    assert!(t.repressions >= 2, "but the era's record of failed repression survives the coup");
    // A long calm starts a new era (counted once the coup's reprieve ends).
    for _ in 0..=REPRIEVE + CALM_RESET {
        let x = w.country_mut(vnm);
        x.stability = 70.0;
        x.legitimacy = 70.0;
        step(&mut w, vec![]);
    }
    assert_eq!(w.country(vnm).transition.repressions, 0, "the record clears after a long calm");
}

/// The regime weighs that record: failed repression pushes rulers without a
/// mission ideology toward reform far more than true believers.
#[test]
fn failed_repression_pushes_pragmatic_rulers_toward_reform() {
    let mut w = world();
    let vnm = id(&w, "VNM");
    let gain = |w: &mut WorldState, ideology: f64| {
        w.country_mut(vnm).personality.ideology = ideology;
        w.country_mut(vnm).transition.repressions = 0;
        let a = ai::evaluate::reform(&observe(w, vnm)).total();
        w.country_mut(vnm).transition.repressions = 3;
        let b = ai::evaluate::reform(&observe(w, vnm)).total();
        b - a
    };
    let pragmatic = gain(&mut w, 0.3);
    let believer = gain(&mut w, 0.9);
    assert!(pragmatic > 20.0, "three failed crackdowns weigh heavily: {pragmatic}");
    assert!(believer < 0.3 * pragmatic, "true believers barely learn: {believer} vs {pragmatic}");
}

/// Campaign regression (issue 2): regimes stuck in a chronic crisis must be
/// able to exit. At these seeds (1..=40) the pre-fix engine gave 17 reforms
/// to 58 coups (23%): coups reset the regime and it never learned. With the
/// "repression has already failed" term it gives 29 reforms to 49 coups
/// (37%). The bar sits at 30%, a margin above the old baseline and below the
/// fixed result.
#[test]
fn chronic_crises_end_in_reform_often_enough() {
    let def = scenario::load(root().join("data/scenarios/1980.ron")).unwrap();
    let runs = headless::run_batch(&def, &(1..=40).collect::<Vec<_>>(), 80, 4).unwrap();
    let (mut reforms, mut coups) = (0u32, 0u32);
    for r in &runs {
        for (_, e) in &r.events {
            match e {
                DiplomaticEvent::Transition { kind: TransitionKind::Reform, .. } => reforms += 1,
                DiplomaticEvent::Transition { kind: TransitionKind::Coup, .. } => coups += 1,
                _ => {}
            }
        }
    }
    println!("reforms {reforms}, coups {coups}");
    assert!(
        reforms as f64 >= 0.30 * (reforms + coups) as f64,
        "reform is a real exit from chronic crisis: {reforms} reforms vs {coups} coups"
    );
}
