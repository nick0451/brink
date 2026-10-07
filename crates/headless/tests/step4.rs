//! Build step 4 acceptance tests: Event Ledger and reputation (DESIGN §21).

use std::path::PathBuf;

use sim_core::ledger::{self, LedgerLog};
use sim_core::reputation::{credibility, expectation, trust};
use sim_core::{
    intel, observe, resolve_turn, CountryId, EntryKind, Grade, NormTag, Order, OrderSet, RepKind, TreatyKind,
    TurnReport, Visibility, WorldState,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_def() -> scenario::ScenarioDef {
    scenario::load(workspace_root().join("data/fixtures/four_actor.ron")).unwrap()
}

fn world() -> WorldState {
    scenario::build(&fixture_def(), Some(1)).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

fn step(w: &mut WorldState, acts: Vec<(CountryId, Vec<Order>)>) -> TurnReport {
    let orders = acts
        .into_iter()
        .map(|(country, orders)| OrderSet { country, orders })
        .collect();
    resolve_turn(w, orders)
}

#[test]
fn observers_diverge_from_the_same_history() {
    let mut w = world();
    let (maj, aly, riv, neu) = (id(&w, "MAJ"), id(&w, "ALY"), id(&w, "RIV"), id(&w, "NEU"));
    step(
        &mut w,
        vec![(
            maj,
            vec![
                Order::StartStream { to: aly, amount: 0.5 },
                Order::StartStream { to: neu, amount: 0.5 },
            ],
        )],
    );
    // Both clients fall into crisis; the patron keeps one and cuts the other.
    w.country_mut(aly).stability = 20.0;
    w.country_mut(neu).stability = 20.0;
    step(&mut w, vec![]);
    let report = step(&mut w, vec![(maj, vec![Order::StopStream { to: neu }])]);
    assert!(report.ledger_log.iter().any(|l| matches!(l,
        LedgerLog::EntryWritten { kind: EntryKind::Back, grade: Some(Grade::Abandoned), counterpart, .. } if *counterpart == neu)));
    for _ in 0..4 {
        step(&mut w, vec![]);
    }
    assert!(w
        .ledger
        .entries
        .iter()
        .any(|e| e.counterpart == aly && e.grade == Some(Grade::Honoured)));

    let read = |o| credibility(&w, o, maj, RepKind::Back);
    let (by_kept, by_rival, by_cut) = (read(aly), read(riv), read(neu));
    assert!(
        by_kept > by_rival && by_rival > by_cut,
        "kept client {by_kept:.1} > bystander {by_rival:.1} > cut client {by_cut:.1}"
    );
    // The view carries the observer's own reading, not a global one.
    let view_value = observe(&w, aly)
        .others
        .iter()
        .find(|f| f.id == maj)
        .unwrap()
        .credibility_back;
    assert!((view_value - by_kept).abs() < 1e-9);
}

#[test]
fn covert_entries_respect_fog_until_exposed() {
    let mut w = world();
    let (maj, neu) = (id(&w, "MAJ"), id(&w, "NEU"));
    let blind = w
        .ids()
        .find(|&o| o != maj && o != neu && intel::coverage(w.country(o), w.country(maj)) < w.params.covert_visibility)
        .expect("fixture has an observer below the covert-visibility threshold");
    let before = expectation(&w, blind, maj, RepKind::Back);
    let mut log = Vec::new();
    let entry = ledger::record(
        &mut w,
        maj,
        neu,
        EntryKind::Back,
        Some(Grade::Abandoned),
        0.0,
        1.0,
        false,
        Visibility::Covert,
        sim_core::CauseCode::SupportCut,
        "secretly dropped a client",
        &mut log,
    );
    assert_eq!(
        expectation(&w, blind, maj, RepKind::Back),
        before,
        "unseen covert act changes nothing"
    );
    assert!(expectation(&w, neu, maj, RepKind::Back) < before, "the victim knows");
    ledger::expose(&mut w, entry);
    assert!(
        expectation(&w, blind, maj, RepKind::Back) < before,
        "exposure reveals it"
    );
}

/// Runs a norm scenario: `maj` enforces against `riv` (creating an
/// Aggression norm), then `neu` violates it; `respond` decides whether `maj`
/// answers with sanctions. Returns (logs, final world).
fn norm_scenario(respond: bool) -> (Vec<LedgerLog>, WorldState) {
    let mut w = world();
    let (maj, riv, neu) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "NEU"));
    let mut log = Vec::new();
    ledger::create_norm(&mut w, maj, NormTag::Aggression, riv, "PUNITIVE_ACTION", &mut log);
    ledger::report_violation(&mut w, neu, NormTag::Aggression, &mut log);
    let orders = if respond {
        vec![(maj, vec![Order::Sanction { target: neu }])]
    } else {
        vec![]
    };
    log.extend(step(&mut w, orders).ledger_log);
    for _ in 0..2 {
        log.extend(step(&mut w, vec![]).ledger_log);
    }
    (log, w)
}

#[test]
fn implied_norms_are_instrumented_and_graded() {
    let (log, w) = norm_scenario(true);
    let (maj, neu) = (id(&w, "MAJ"), id(&w, "NEU"));
    assert!(log.iter().any(|l| matches!(l,
        LedgerLog::NormCreated { norm: NormTag::Aggression, actor, cause, seen_by, .. }
            if *actor == maj && cause == "PUNITIVE_ACTION" && !seen_by.is_empty())));
    let test = log
        .iter()
        .find_map(|l| match l {
            LedgerLog::NormTest {
                actor,
                violator,
                response,
                deltas,
                ..
            } if *actor == maj && *violator == neu => Some((*response, deltas.clone())),
            _ => None,
        })
        .expect("norm test logged");
    assert_eq!(test.0, Grade::Partial, "sanctions alone are a partial response");
    assert!(!test.1.is_empty(), "the log records reputation deltas");

    let (_, ignored) = norm_scenario(false);
    let riv = id(&w, "RIV");
    assert!(
        expectation(&ignored, riv, maj, RepKind::Norm) < expectation(&w, riv, maj, RepKind::Norm),
        "ignoring a violation costs more than a partial response"
    );
}

#[test]
fn withdrawing_before_an_attack_counts_as_abandonment() {
    let calm = {
        let mut w = world();
        let (maj, aly) = (id(&w, "MAJ"), id(&w, "ALY"));
        let alliance = w
            .diplomacy
            .find_treaty(TreatyKind::DefensiveAlliance, maj, aly)
            .unwrap()
            .id;
        step(&mut w, vec![(maj, vec![Order::CancelTreaty { treaty: alliance }])]);
        w
    };
    let (maj, aly, riv) = (id(&calm, "MAJ"), id(&calm, "ALY"), id(&calm, "RIV"));
    let baseline = world();
    assert!(
        trust(&calm, aly, maj) < trust(&baseline, aly, maj),
        "the abandoned ally remembers"
    );
    assert_eq!(
        expectation(&calm, riv, maj, RepKind::Back),
        expectation(&baseline, riv, maj, RepKind::Back),
        "a calm withdrawal is local"
    );

    let mut attacked = calm.clone();
    let mut log = Vec::new();
    ledger::report_attack(&mut attacked, aly, &mut log);
    assert!(log.iter().any(|l| matches!(l, LedgerLog::ShadowTriggered { .. })));
    assert!(
        expectation(&attacked, riv, maj, RepKind::Back) < expectation(&calm, riv, maj, RepKind::Back),
        "an attack inside the shadow window makes it a public abandonment"
    );

    let mut late = calm.clone();
    for _ in 0..ledger::SHADOW_TURNS + 1 {
        step(&mut late, vec![]);
    }
    let mut log = Vec::new();
    ledger::report_attack(&mut late, aly, &mut log);
    assert!(log.is_empty(), "the shadow expires");
}

#[test]
fn third_parties_react_by_their_own_relationships() {
    let mut w = world();
    let mut control = w.clone();
    let (maj, riv, aly, neu) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "ALY"), id(&w, "NEU"));
    step(&mut w, vec![(maj, vec![Order::Sanction { target: riv }])]);
    step(&mut control, vec![]);
    let delta = |o| w.opinions.opinion(o, maj) - control.opinions.opinion(o, maj);
    assert!(delta(aly) > 0.0, "the rival's enemy approves: {}", delta(aly));
    assert!(delta(neu) < 0.0, "the rival's friend disapproves: {}", delta(neu));
}

#[test]
fn sanctions_leave_ungraded_coercion_entries() {
    let mut w = world();
    let (maj, riv, aly) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "ALY"));
    let before = credibility(&w, aly, maj, RepKind::Back);
    step(&mut w, vec![(maj, vec![Order::Sanction { target: riv }])]);
    let entry = w.ledger.entries.iter().find(|e| e.kind == EntryKind::Coercion).unwrap();
    assert_eq!((entry.actor, entry.counterpart, entry.grade), (maj, riv, None));
    assert_eq!(credibility(&w, aly, maj, RepKind::Back), before);
}

#[test]
fn seeded_history_shapes_starting_reputation() {
    let mut def = fixture_def();
    def.history.push(scenario::HistoryDef {
        actor: "MAJ".into(),
        counterpart: "NEU".into(),
        kind: EntryKind::Back,
        grade: Some(Grade::Abandoned),
        turns_ago: 4,
        cost_paid: 0.0,
        weight: 1.0,
        local: false,
        cause: "abandoned a client before the scenario".into(),
    });
    let with = scenario::build(&def, Some(1)).unwrap();
    let without = world();
    let (maj, neu) = (id(&with, "MAJ"), id(&with, "NEU"));
    assert!(credibility(&with, neu, maj, RepKind::Back) < credibility(&without, neu, maj, RepKind::Back));
}

/// DESIGN §21.3: the display-only global credibility is never an AI input.
#[test]
fn ai_never_reads_global_credibility() {
    for entry in std::fs::read_dir(workspace_root().join("crates/ai/src")).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(!text.contains("global_credibility"));
    }
}
