//! R1 acceptance tests: reflexes as data (design/behaviour-and-voice.md §1).

use std::path::PathBuf;

use ai::{Controller, DecisionRecord, Steady, Strategist};
use sim_core::{
    observe, resolve_turn, Condition, CountryId, DecisionKind, Order, OrderSet, ProposalFamily, Reflex, TreatyKind,
    WorldState,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn library() -> Vec<sim_core::ReflexSet> {
    scenario::load_reflex_library(root().join("data/reflexes/1980.ron")).expect("library loads and validates")
}

/// The tense four-actor fixture (the Neutral threatened by the Rival).
fn tense_def() -> scenario::ScenarioDef {
    let mut def = scenario::load(root().join("data/fixtures/four_actor.ron")).unwrap();
    def.opinions.retain(|o| !(o.from == "NEU" && o.to == "RIV"));
    for (from, to, value) in [("RIV", "NEU", -60.0), ("NEU", "RIV", -50.0)] {
        def.opinions.push(scenario::OpinionDef {
            from: from.into(),
            to: to.into(),
            value,
            decay: 0.0,
            source: "dispute".into(),
        });
    }
    def.tensions.push(scenario::TensionDef {
        a: "RIV".into(),
        b: "NEU".into(),
        value: 60.0,
    });
    def.reflex_sets = library();
    def
}

fn neutral(def: &mut scenario::ScenarioDef) -> &mut scenario::CountryDef {
    def.countries.iter_mut().find(|c| c.id == "NEU").unwrap()
}

/// The Gate 1 "kept support" world, then an alliance offer to the Neutral;
/// returns the Neutral's decision record on it.
fn alliance_decision(def: &scenario::ScenarioDef) -> DecisionRecord {
    let mut w: WorldState = scenario::build(def, Some(1)).unwrap();
    let (maj, neu) = (w.find("MAJ").unwrap(), w.find("NEU").unwrap());
    let mut controllers: Vec<Box<dyn Controller>> = w
        .countries
        .iter()
        .map(|c| -> Box<dyn Controller> {
            if c.code == "NEU" {
                Box::new(Strategist::new())
            } else {
                Box::new(Steady)
            }
        })
        .collect();
    let mut found = None;
    let script: Vec<Vec<(CountryId, Vec<Order>)>> = vec![
        vec![(maj, vec![Order::StartStream { to: neu, amount: 0.5 }])],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![(
            maj,
            vec![Order::ProposeTreaty {
                to: neu,
                kind: TreatyKind::DefensiveAlliance,
            }],
        )],
        vec![],
    ];
    for (turn, extra) in script.into_iter().enumerate() {
        if turn == 1 {
            w.country_mut(neu).stability = 20.0;
        }
        let views: Vec<_> = w.ids().map(|c| observe(&w, c)).collect();
        let mut orders = Vec::new();
        for (view, c) in views.iter().zip(controllers.iter_mut()) {
            let d = c.decide(view);
            if let Some(r) = d
                .records
                .iter()
                .find(|r| r.kind == DecisionKind::AcceptProposal(ProposalFamily::DefensiveAlliance))
            {
                found = Some(r.clone());
            }
            orders.push(OrderSet {
                country: view.observer,
                orders: d.orders,
            });
        }
        for (country, o) in extra {
            orders.push(OrderSet { country, orders: o });
        }
        resolve_turn(&mut w, orders);
    }
    found.expect("the Neutral evaluated the alliance")
}

#[test]
fn library_parses_validates_and_covers_the_roster() {
    let lib = library();
    let ids: Vec<_> = lib.iter().map(|s| s.id.as_str()).collect();
    for code in [
        "USA", "SOV", "CHN", "JPN", "FRG", "POL", "YUG", "IRN", "IRQ", "SAU", "ISR", "IND", "PAK", "PRK", "ZAF", "CUB",
    ] {
        assert!(ids.contains(&code), "missing reflex set {code}");
    }
    for set in &lib {
        let mut seen = std::collections::BTreeSet::new();
        for r in &set.reflexes {
            assert!(seen.insert(r.id.as_str()), "duplicate {} in {}", r.id, set.id);
        }
    }
}

#[test]
fn a_reflex_adds_a_labelled_term_and_can_flip_the_decision() {
    let base = alliance_decision(&tense_def());
    let mut def = tense_def();
    neutral(&mut def).reflex_sets = vec!["IND".into()];
    let with = alliance_decision(&def);
    assert!(base.chosen, "without reflexes the Neutral accepts: {base:#?}");
    assert!(!with.chosen, "a non-aligned habit refuses: {with:#?}");
    let line = with
        .lines
        .iter()
        .find(|l| l.term == "reflex:non_alignment")
        .expect("reflex line present");
    assert_eq!(line.label, "non-alignment is not neutrality");
    let sum: f64 = with.lines.iter().map(|l| l.value).sum();
    assert!((sum - with.score).abs() < 1e-9);
}

#[test]
fn a_changed_personality_stops_behaving_historically() {
    let mut def = tense_def();
    neutral(&mut def).reflex_sets = vec!["IND".into()];
    neutral(&mut def).personality.ideology = 0.2; // below the habit's gate
    let r = alliance_decision(&def);
    assert!(
        !r.lines.iter().any(|l| l.term.starts_with("reflex:")),
        "gate closed: {r:#?}"
    );
    assert!(r.chosen);
}

#[test]
fn reflex_total_is_capped() {
    let mut def = tense_def();
    neutral(&mut def).reflexes = (0..3)
        .map(|i| Reflex {
            id: format!("zeal_{i}"),
            decision: DecisionKind::AcceptProposal(ProposalFamily::DefensiveAlliance),
            when: vec![],
            weight: 30.0,
            label: "zeal".into(),
            active_if: vec![],
            ideological: false,
        })
        .collect();
    let r = alliance_decision(&def);
    let reflex_total: f64 = r
        .lines
        .iter()
        .filter(|l| l.term.starts_with("reflex:"))
        .map(|l| l.value)
        .sum();
    assert!(
        (reflex_total - sim_core::reflex::REFLEX_CAP).abs() < 1e-9,
        "{reflex_total}"
    );
}

#[test]
fn behaviour_follows_data_not_identity() {
    // The same country with two different habit tables decides differently;
    // the habit table, not the country, carries the behaviour.
    let mut as_india = tense_def();
    neutral(&mut as_india).reflex_sets = vec!["IND".into()];
    let mut as_pakistan = tense_def();
    neutral(&mut as_pakistan).reflex_sets = vec!["PAK".into()];
    assert!(!alliance_decision(&as_india).chosen);
    assert!(alliance_decision(&as_pakistan).chosen);
}

#[test]
fn reflex_conditions_respect_fog() {
    let make = |blind: bool| {
        let mut def = tense_def();
        neutral(&mut def).reflexes = vec![Reflex {
            id: "watching".into(),
            decision: DecisionKind::AcceptProposal(ProposalFamily::DefensiveAlliance),
            when: vec![Condition::CounterpartStabilityBelow(100.0)],
            weight: 10.0,
            label: "we can see their stability".into(),
            active_if: vec![],
            ideological: false,
        }];
        if blind {
            neutral(&mut def).intel_tech = 1;
            let maj = def.countries.iter_mut().find(|c| c.id == "MAJ").unwrap();
            maj.openness = Some(0.0);
            maj.intel_capacity = Some(1e6);
        }
        alliance_decision(&def)
    };
    let fires = |r: &DecisionRecord| r.lines.iter().any(|l| l.term == "reflex:watching");
    assert!(fires(&make(false)), "visible stability band: the condition can hold");
    assert!(
        !fires(&make(true)),
        "below the coverage tier the condition cannot be evaluated"
    );
}
