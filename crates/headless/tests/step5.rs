//! Build step 5 acceptance tests: four-actor AI → Gates 1 and 2
//! (DESIGN §22.1, implementation plan §3).

use std::path::PathBuf;

use ai::{Controller, Decision, Steady, Strategist};
use sim_core::{observe, resolve_turn, CountryId, Order, OrderSet, TreatyKind, WorldState};

fn fixture_def() -> scenario::ScenarioDef {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures/four_actor.ron");
    scenario::load(path).unwrap()
}

/// The fixture with the Neutral under threat from the Rival.
fn tense_def() -> scenario::ScenarioDef {
    let mut def = fixture_def();
    def.opinions.retain(|o| !(o.from == "NEU" && o.to == "RIV"));
    for (from, to, value) in [("RIV", "NEU", -60.0), ("NEU", "RIV", -50.0)] {
        def.opinions.push(scenario::OpinionDef {
            from: from.into(),
            to: to.into(),
            value,
            decay: 0.0,
            source: "border dispute".into(),
        });
    }
    def.tensions.push(scenario::TensionDef {
        a: "RIV".into(),
        b: "NEU".into(),
        value: 60.0,
    });
    def
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

/// Mixed harness: scripted orders for some countries, controllers for the rest.
struct Harness {
    world: WorldState,
    controllers: Vec<Box<dyn Controller>>,
    decisions: Vec<(u32, CountryId, Decision)>,
}

impl Harness {
    fn new(world: WorldState, ai_for: &[&str]) -> Self {
        let controllers = world
            .countries
            .iter()
            .map(|c| -> Box<dyn Controller> {
                if ai_for.contains(&c.code.as_str()) {
                    Box::new(Strategist::new())
                } else {
                    Box::new(Steady)
                }
            })
            .collect();
        Harness {
            world,
            controllers,
            decisions: Vec::new(),
        }
    }

    fn step(&mut self, scripted: Vec<(CountryId, Vec<Order>)>) {
        let views: Vec<_> = self.world.ids().map(|c| observe(&self.world, c)).collect();
        let mut orders = Vec::new();
        for (view, controller) in views.iter().zip(self.controllers.iter_mut()) {
            let decision = controller.decide(view);
            orders.push(OrderSet {
                country: view.observer,
                orders: decision.orders.clone(),
            });
            self.decisions.push((view.turn, view.observer, decision));
        }
        for (country, extra) in scripted {
            orders.push(OrderSet { country, orders: extra });
        }
        resolve_turn(&mut self.world, orders);
    }
}

/// Gate 1 world: the Major Power supports the Neutral through a crisis and
/// either keeps (`keep = true`) or cuts the stream; then proposes an
/// alliance. Returns the harness after the Neutral has answered.
fn gate1_world(keep: bool) -> Harness {
    let world = scenario::build(&tense_def(), Some(1)).unwrap();
    let (maj, neu) = (id(&world, "MAJ"), id(&world, "NEU"));
    let mut h = Harness::new(world, &["NEU"]);
    h.step(vec![(maj, vec![Order::StartStream { to: neu, amount: 0.5 }])]);
    h.world.country_mut(neu).stability = 20.0;
    h.step(vec![]);
    h.step(if keep {
        vec![]
    } else {
        vec![(maj, vec![Order::StopStream { to: neu }])]
    });
    for _ in 0..3 {
        h.step(vec![]);
    }
    h.step(vec![(
        maj,
        vec![Order::ProposeTreaty {
            to: neu,
            kind: TreatyKind::DefensiveAlliance,
        }],
    )]);
    h.step(vec![]);
    h
}

fn alliance_answer(h: &Harness) -> (bool, ai::DecisionRecord) {
    let neu = id(&h.world, "NEU");
    let record = h
        .decisions
        .iter()
        .filter(|(_, c, _)| *c == neu)
        .flat_map(|(_, _, d)| d.records.iter())
        .find(|r| r.subject.starts_with("accept DefensiveAlliance"))
        .expect("the Neutral evaluated the alliance")
        .clone();
    (record.chosen, record)
}

/// GATE 1 — memory matters: identical worlds except for one past act; the
/// same later offer gets a different answer, and the reasons cite it.
#[test]
fn gate1_past_behaviour_changes_later_decisions() {
    let (kept_answer, kept) = alliance_answer(&gate1_world(true));
    let (cut_answer, cut) = alliance_answer(&gate1_world(false));
    for (name, r) in [("kept support", &kept), ("cut support", &cut)] {
        eprintln!("[{name}] {} -> score {:.1}, chosen {}", r.subject, r.score, r.chosen);
        for l in &r.lines {
            eprintln!("    {:+7.1}  {}", l.value, l.label);
        }
        eprintln!("    remembers: {:?}", r.precedents);
    }
    assert!(kept_answer, "kept-support world should accept: {kept:#?}");
    assert!(!cut_answer, "cut-support world should refuse: {cut:#?}");
    let line = |r: &ai::DecisionRecord, label: &str| r.lines.iter().find(|l| l.label == label).map_or(0.0, |l| l.value);
    assert!(line(&kept, "their reliability") > line(&cut, "their reliability") + 10.0);
    assert!(
        kept.precedents.iter().any(|p| p.contains("Honoured toward us")),
        "{:?}",
        kept.precedents
    );
    assert!(
        cut.precedents.iter().any(|p| p.contains("Abandoned toward us")),
        "{:?}",
        cut.precedents
    );
    let h = gate1_world(true);
    let (maj, neu) = (id(&h.world, "MAJ"), id(&h.world, "NEU"));
    assert!(h.world.diplomacy.has_treaty(TreatyKind::DefensiveAlliance, maj, neu));
}

/// GATE 2 — contradictory incentives: supporting a useful but disliked
/// partner. Each option must be best on at least one metric, none on all
/// (forced-branch Pareto test).
#[test]
fn gate2_no_option_dominates_supporting_a_contested_partner() {
    let mut def = tense_def();
    def.opinions.push(scenario::OpinionDef {
        from: "ALY".into(),
        to: "NEU".into(),
        value: -40.0,
        decay: 0.0,
        source: "old grievance".into(),
    });
    let base = scenario::build(&def, Some(3)).unwrap();
    let (maj, aly, neu) = (id(&base, "MAJ"), id(&base, "ALY"), id(&base, "NEU"));
    let options: Vec<(&str, Vec<Order>)> = vec![
        ("stay out", vec![]),
        ("support stream", vec![Order::StartStream { to: neu, amount: 0.6 }]),
        ("guarantee", vec![Order::IssueGuarantee { to: neu }]),
    ];
    let metric_names = [
        "MAJ military",
        "NEU opinion of MAJ",
        "NEU stability",
        "ALY opinion of MAJ",
        "fewer commitments",
    ];
    let mut table = Vec::new();
    for (name, orders) in &options {
        let mut w = base.clone();
        resolve_turn(
            &mut w,
            vec![OrderSet {
                country: maj,
                orders: orders.clone(),
            }],
        );
        for _ in 0..8 {
            resolve_turn(&mut w, vec![]);
        }
        let commitments = w
            .diplomacy
            .treaties
            .iter()
            .filter(|t| t.a == maj && t.kind == TreatyKind::Guarantee)
            .count();
        table.push((
            *name,
            [
                w.country(maj).power(),
                w.opinions.opinion(neu, maj),
                w.country(neu).stability,
                w.opinions.opinion(aly, maj),
                -(commitments as f64),
            ],
        ));
    }
    let best = |m: usize| table.iter().map(|r| r.1[m]).fold(f64::NEG_INFINITY, f64::max);
    for (name, row) in &table {
        let wins: Vec<_> = (0..metric_names.len()).filter(|&m| row[m] >= best(m) - 1e-9).collect();
        assert!(!wins.is_empty(), "{name} is best at nothing: {table:?}");
        assert!(
            wins.len() < metric_names.len(),
            "{name} dominates everything: {table:?}"
        );
    }
}

#[test]
fn explanation_lines_always_sum_to_the_score() {
    let def = tense_def();
    let r = headless::run_campaign(&def, 7, 40).unwrap();
    assert!(!r.reasoning.is_empty(), "the tense fixture produces decisions");
    for entry in &r.reasoning {
        let sum: f64 = entry.decision.lines.iter().map(|l| l.value).sum();
        assert!((sum - entry.decision.score).abs() < 1e-9, "{:?}", entry.decision);
        assert!(
            !(entry.decision.chosen && entry.decision.score <= 0.0),
            "chose a non-positive score: {:?}",
            entry.decision
        );
    }
}

#[test]
fn threatened_states_seek_protection_and_patrons_weigh_it() {
    let r = headless::run_campaign(&tense_def(), 7, 24).unwrap();
    let asked = r
        .reasoning
        .iter()
        .find(|e| e.country == "NEU" && e.decision.subject.starts_with("request guarantee") && e.decision.chosen)
        .expect("the threatened Neutral asks for a guarantee");
    let patron = asked
        .decision
        .subject
        .trim_start_matches("request guarantee from ")
        .to_string();
    assert!(
        r.reasoning
            .iter()
            .any(|e| e.country == patron && e.decision.subject.starts_with("accept Guarantee from NEU")),
        "the asked patron evaluates the request"
    );
}

#[test]
fn allies_join_sanctions_but_the_targets_friends_do_not() {
    let mut def = fixture_def();
    def.opinions.retain(|o| !(o.from == "NEU" && o.to == "RIV"));
    def.opinions.push(scenario::OpinionDef {
        from: "NEU".into(),
        to: "RIV".into(),
        value: 40.0,
        decay: 0.0,
        source: "friendship".into(),
    });
    let world = scenario::build(&def, Some(1)).unwrap();
    let (maj, riv, aly, neu) = (
        id(&world, "MAJ"),
        id(&world, "RIV"),
        id(&world, "ALY"),
        id(&world, "NEU"),
    );
    let mut h = Harness::new(world, &["ALY", "NEU"]);
    h.step(vec![(maj, vec![Order::Sanction { target: riv }])]);
    for _ in 0..4 {
        h.step(vec![]);
    }
    assert!(h.world.diplomacy.is_sanctioned_by(riv, aly), "the ally joins");
    assert!(
        !h.world.diplomacy.is_sanctioned_by(riv, neu),
        "the target's friend stays out"
    );
}

/// Issue 8 (D68): the Major Power sanctions the small Neutral. An ally with
/// no quarrel of its own with the Neutral does not pay to join out of
/// loyalty alone (Western Europe and the Cuba embargo); the same ally with
/// a real dispute with the Neutral does.
#[test]
fn allies_join_sanctions_only_when_they_share_the_grievance() {
    let run = |quarrel: bool| {
        let mut def = fixture_def();
        if quarrel {
            for (from, to) in [("ALY", "NEU"), ("NEU", "ALY")] {
                def.opinions.push(scenario::OpinionDef {
                    from: from.into(),
                    to: to.into(),
                    value: -30.0,
                    decay: 0.0,
                    source: "dispute".into(),
                });
            }
        }
        let world = scenario::build(&def, Some(1)).unwrap();
        let (maj, neu, aly) = (id(&world, "MAJ"), id(&world, "NEU"), id(&world, "ALY"));
        let mut h = Harness::new(world, &["ALY"]);
        h.step(vec![(maj, vec![Order::Sanction { target: neu }])]);
        let sanction = sim_core::Sanction {
            by: maj,
            target: neu,
            since: h.world.turn,
        };
        let score = ai::evaluate::join_sanction(&observe(&h.world, aly), &sanction);
        println!("quarrel {quarrel}: total {:+.1}", score.total());
        for l in score.sorted() {
            println!("    {:+6.1}  {}", l.value, l.label);
        }
        for _ in 0..4 {
            h.step(vec![]);
        }
        h.world.diplomacy.is_sanctioned_by(neu, aly)
    };
    assert!(!run(false), "an ally with no grievance of its own stays out");
    assert!(run(true), "an ally that shares the grievance joins");
}

#[test]
fn leaving_an_alliance_needs_two_bad_assessments() {
    let mut def = fixture_def();
    for c in &mut def.countries {
        if c.id == "ALY" {
            c.personality.loyalty = 0.0;
        }
    }
    def.opinions.push(scenario::OpinionDef {
        from: "ALY".into(),
        to: "MAJ".into(),
        value: -200.0, // opinion bottoms out at -100 despite alliance and trade bonuses
        decay: 0.0,
        source: "bitter dispute".into(),
    });
    let world = scenario::build(&def, Some(1)).unwrap();
    let (maj, aly) = (id(&world, "MAJ"), id(&world, "ALY"));
    let mut h = Harness::new(world, &["ALY"]);
    let mut exit_turn = None;
    for _ in 0..12 {
        h.step(vec![]);
        if exit_turn.is_none() && !h.world.diplomacy.has_treaty(TreatyKind::DefensiveAlliance, maj, aly) {
            exit_turn = Some(h.world.turn);
        }
    }
    let records: Vec<_> = h
        .decisions
        .iter()
        .filter(|(_, c, _)| *c == aly)
        .flat_map(|(t, _, d)| d.records.iter().map(move |r| (*t, r)))
        .filter(|(_, r)| r.subject.starts_with("stay in alliance"))
        .collect();
    assert!(records.len() >= 2, "{records:?}");
    assert!(records[0].1.chosen, "no exit on the first bad assessment");
    assert!(exit_turn.is_some(), "exits after the second");
}

/// No identity branching: renaming every country changes nothing numerically.
#[test]
fn renaming_countries_changes_nothing() {
    let def = tense_def();
    let rename = |s: &str| format!("Z{}", s.to_lowercase());
    let mut renamed = def.clone();
    for c in &mut renamed.countries {
        c.id = rename(&c.id);
        c.name = format!("Country {}", c.id);
    }
    for o in &mut renamed.opinions {
        o.from = rename(&o.from);
        o.to = rename(&o.to);
    }
    for t in &mut renamed.treaties {
        t.a = rename(&t.a);
        t.b = rename(&t.b);
    }
    for t in &mut renamed.tensions {
        t.a = rename(&t.a);
        t.b = rename(&t.b);
    }
    let a = headless::run_campaign(&def, 11, 40).unwrap();
    let b = headless::run_campaign(&renamed, 11, 40).unwrap();
    let numbers = |r: &headless::RunResult| {
        r.samples
            .iter()
            .flat_map(|s| s.countries.iter().map(|c| (c.gdp, c.stability, c.military)))
            .collect::<Vec<_>>()
    };
    assert_eq!(numbers(&a), numbers(&b));
    let choices = |r: &headless::RunResult| {
        r.reasoning
            .iter()
            .map(|e| (e.decision.chosen, e.decision.score))
            .collect::<Vec<_>>()
    };
    assert_eq!(choices(&a), choices(&b));
}
