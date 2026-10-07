//! Build step 7 acceptance tests: war-lite → Gate 3 "escalation works: no
//! involvement band dominates" (DESIGN §6.3–6.6, §11.2, §22.1).

use std::path::PathBuf;

use ai::{Controller, Steady, Strategist};
use sim_core::reputation::{self, RepKind};
use sim_core::{
    observe, resolve_turn, CountryId, DiplomaticEvent, EntryKind, Grade, LedgerLog, Mobilization, Order, OrderSet,
    PeaceResult, Side, TreatyKind, Visibility, WarAim, WorldState,
};

fn fixture_def() -> scenario::ScenarioDef {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures/four_actor.ron");
    scenario::load(path).unwrap()
}

/// The Rival and the Neutral at daggers drawn; the Major Power has
/// guaranteed the Neutral.
fn crisis_def() -> scenario::ScenarioDef {
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
        value: 70.0,
    });
    def.treaties.push(scenario::TreatyDef {
        kind: TreatyKind::Guarantee,
        a: "MAJ".into(),
        b: "NEU".into(),
    });
    // A regional power, not a pushover: the war can last long enough for
    // outside help to matter.
    let neu = def.countries.iter_mut().find(|c| c.id == "NEU").unwrap();
    neu.military = 30.0;
    neu.gdp = 10.0;
    neu.budget.military = 0.35;
    def
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

/// Scripted orders for some countries; Strategist AI for `ai_for`; the rest
/// passive.
struct Harness {
    world: WorldState,
    controllers: Vec<Box<dyn Controller>>,
    events: Vec<(u32, DiplomaticEvent)>,
    ledger: Vec<(u32, LedgerLog)>,
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
            events: Vec::new(),
            ledger: Vec::new(),
        }
    }

    fn step(&mut self, scripted: Vec<(CountryId, Vec<Order>)>) {
        let views: Vec<_> = self.world.ids().map(|c| observe(&self.world, c)).collect();
        let mut orders = Vec::new();
        for (view, controller) in views.iter().zip(self.controllers.iter_mut()) {
            orders.push(OrderSet {
                country: view.observer,
                orders: controller.decide(view).orders,
            });
        }
        for (country, extra) in scripted {
            orders.push(OrderSet { country, orders: extra });
        }
        let turn = self.world.turn;
        let report = resolve_turn(&mut self.world, orders);
        assert!(report.rejected.is_empty(), "rejected: {:?}", report.rejected);
        self.events.extend(report.events.into_iter().map(|e| (turn, e)));
        self.ledger.extend(report.ledger_log.into_iter().map(|l| (turn, l)));
    }
}

#[test]
fn a_declared_war_fights_on_one_front_and_ends() {
    let world = scenario::build(&crisis_def(), Some(5)).unwrap();
    let (riv, neu) = (id(&world, "RIV"), id(&world, "NEU"));
    let mut h = Harness::new(world, &["RIV", "NEU"]);
    h.step(vec![(
        riv,
        vec![Order::DeclareWar {
            target: neu,
            aim: WarAim::Limited,
        }],
    )]);
    let w = &h.world.wars.active[0];
    assert_eq!(w.fronts.len(), 1, "v0.1: one front per war");
    assert_eq!(w.side_of(riv), Some(Side::Attacker));
    assert!(h.world.wars.at_war(riv, neu));
    let before = h.world.country(neu).forces.strength();
    for _ in 0..30 {
        h.step(vec![]);
        if h.world.wars.active.is_empty() {
            break;
        }
    }
    let peace = h
        .events
        .iter()
        .find_map(|(t, e)| match e {
            DiplomaticEvent::PeaceMade {
                result,
                reason,
                progress,
                ..
            } => Some((*t, *result, *reason, *progress)),
            _ => None,
        })
        .expect("wars end");
    println!("peace {peace:?}");
    assert!(
        h.world.country(neu).forces.strength() < before,
        "fighting costs strength"
    );
    // Enemies don't trade while at war; tension went to the war baseline.
    assert!(h
        .events
        .iter()
        .any(|(_, e)| matches!(e, DiplomaticEvent::FrontReport { .. })));
}

#[test]
fn punitive_strike_ends_if_accepted_or_becomes_a_limited_war() {
    // Passive target: never accepts, so the strike escalates.
    let mut w = scenario::build(&crisis_def(), Some(5)).unwrap();
    let (riv, neu) = (id(&w, "RIV"), id(&w, "NEU"));
    resolve_turn(
        &mut w,
        vec![OrderSet {
            country: riv,
            orders: vec![Order::DeclareWar {
                target: neu,
                aim: WarAim::Punitive,
            }],
        }],
    );
    let striker_air_before = w.country(riv).forces.land.strength;
    let mut escalated = false;
    for _ in 0..3 {
        let r = resolve_turn(&mut w, vec![]);
        escalated |= r.events.iter().any(|e| {
            matches!(
                e,
                DiplomaticEvent::WarEscalated {
                    aim: WarAim::Limited,
                    ..
                }
            )
        });
    }
    assert!(escalated, "refused strike becomes a limited war");
    assert!(
        w.ledger
            .entries
            .iter()
            .any(|e| e.kind == EntryKind::Coercion && e.actor == riv && e.counterpart == neu),
        "a punitive strike leaves a Coercion entry"
    );
    // Strikes use air and naval power: land forces weren't touched in the strike turns.
    let _ = striker_air_before;

    // Accepting target: peace after the strike.
    let mut w = scenario::build(&crisis_def(), Some(5)).unwrap();
    resolve_turn(
        &mut w,
        vec![OrderSet {
            country: riv,
            orders: vec![Order::DeclareWar {
                target: neu,
                aim: WarAim::Punitive,
            }],
        }],
    );
    let war = w.wars.active[0].id;
    let r = resolve_turn(
        &mut w,
        vec![OrderSet {
            country: neu,
            orders: vec![Order::OfferPeace { war }],
        }],
    );
    assert!(r.events.iter().any(|e| matches!(
        e,
        DiplomaticEvent::PeaceMade {
            result: PeaceResult::AttackerWon,
            aim: WarAim::Punitive,
            ..
        }
    )));
    assert!(w.wars.active.is_empty());
}

/// One branch of the Gate 3 crisis: the Rival attacks the guaranteed
/// Neutral; the Major Power responds at a band; everyone else is AI.
fn branch(orders: Vec<Order>, seed: u64) -> (Harness, [f64; 9]) {
    let world = scenario::build(&crisis_def(), Some(seed)).unwrap();
    let (maj, riv, neu, aly) = (
        id(&world, "MAJ"),
        id(&world, "RIV"),
        id(&world, "NEU"),
        id(&world, "ALY"),
    );
    let mut h = Harness::new(world, &["RIV", "NEU", "ALY"]);
    h.step(vec![(
        riv,
        vec![Order::DeclareWar {
            target: neu,
            aim: WarAim::Limited,
        }],
    )]);
    h.step(vec![(maj, orders)]);
    for _ in 0..10 {
        h.step(vec![]);
    }
    let w = &h.world;
    // How the Neutral fared: final progress for the defender, or the peace result.
    let outcome = h
        .events
        .iter()
        .find_map(|(_, e)| match e {
            DiplomaticEvent::PeaceMade { result, .. } => Some(match result {
                PeaceResult::DefenderWon => 100.0,
                PeaceResult::WhitePeace => 0.0,
                PeaceResult::AttackerWon => -100.0,
            }),
            _ => None,
        })
        .unwrap_or_else(|| w.wars.active.first().map_or(0.0, |war| -war.progress()));
    let metrics = [
        w.country(maj).forces.strength(),
        w.country(maj).gdp,
        -w.country(maj).war_weariness,
        -w.tension.get(maj, riv),
        outcome,
        -w.country(riv).forces.strength(),
        w.country(neu).forces.strength(),
        w.opinions.opinion(neu, maj),
        reputation::credibility(w, aly, maj, RepKind::Back),
    ];
    (h, metrics)
}

const METRICS: [&str; 9] = [
    "MAJ strength",
    "MAJ GDP",
    "MAJ war weariness (low)",
    "MAJ-RIV tension (low)",
    "NEU war outcome",
    "RIV strength (low)",
    "NEU strength",
    "NEU opinion of MAJ",
    "ALY's Credibility(Back) of MAJ",
];

fn options(w: &WorldState) -> Vec<(&'static str, Vec<Order>)> {
    let (riv, neu) = (id(w, "RIV"), id(w, "NEU"));
    let war = sim_core::WarId(0);
    vec![
        ("0 none", vec![]),
        (
            "1 coerce",
            vec![Order::Sanction { target: riv }, Order::Denounce { target: riv }],
        ),
        (
            "2 proxy",
            vec![Order::StartArmsStream {
                to: neu,
                amount: 2.4,
                covert: false,
            }],
        ),
        (
            "3 limited",
            vec![Order::JoinWar {
                war,
                side: Side::Defender,
                band: 3,
            }],
        ),
        (
            "4 major",
            vec![
                Order::JoinWar {
                    war,
                    side: Side::Defender,
                    band: 4,
                },
                Order::SetMobilization(Mobilization::Full),
            ],
        ),
    ]
}

/// GATE 3 — escalation works: at a fixture crisis no involvement band is
/// best on every metric, and every band is best on at least one
/// (forced-branch Pareto test, DESIGN §22.1).
#[test]
fn gate3_no_involvement_band_dominates() {
    let base = scenario::build(&crisis_def(), Some(5)).unwrap();
    let seeds: Vec<u64> = (1..=12).collect();
    let mut table = Vec::new();
    for (name, orders) in options(&base) {
        // Mean over seeds: combat variance and estimates differ per seed.
        let mut mean = [0.0; METRICS.len()];
        for &seed in &seeds {
            let (_, m) = branch(orders.clone(), seed);
            for k in 0..METRICS.len() {
                mean[k] += m[k] / seeds.len() as f64;
            }
        }
        table.push((name, mean));
    }
    println!("{:<12} {}", "band", METRICS.join(" | "));
    for (name, m) in &table {
        println!(
            "{name:<12} {}",
            m.iter().map(|v| format!("{v:8.1}")).collect::<Vec<_>>().join(" ")
        );
    }
    let best = |k: usize| table.iter().map(|r| r.1[k]).fold(f64::NEG_INFINITY, f64::max);
    for (name, row) in &table {
        let wins: Vec<&str> = (0..METRICS.len())
            .filter(|&k| row[k] >= best(k) - 1e-9)
            .map(|k| METRICS[k])
            .collect();
        println!("{name}: best at {wins:?}");
        assert!(!wins.is_empty(), "{name} is best at nothing");
        assert!(wins.len() < METRICS.len(), "{name} dominates everything");
        // Stronger: no other band is at least as good everywhere and better
        // somewhere (every band is Pareto-optimal).
        for (other, o) in &table {
            let dominated =
                (0..METRICS.len()).all(|k| o[k] >= row[k] - 1e-9) && (0..METRICS.len()).any(|k| o[k] > row[k] + 1e-9);
            assert!(!dominated, "{name} is dominated by {other}");
        }
    }
}

/// The guarantor arms the Neutral during the war, publicly or covertly, then
/// cuts it off while the Neutral is losing. Returns the world and the
/// withdrawal entry.
fn proxy_exit(covert: bool) -> (WorldState, sim_core::LedgerEntry) {
    let mut w = scenario::build(&crisis_def(), Some(5)).unwrap();
    let (maj, riv, neu) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "NEU"));
    // Make the Neutral weak enough to be losing when the cut comes.
    w.country_mut(neu).forces = sim_core::Forces::new(6.0, Default::default(), 5.0);
    let step = |w: &mut WorldState, acts: Vec<(CountryId, Vec<Order>)>| {
        resolve_turn(
            w,
            acts.into_iter()
                .map(|(country, orders)| OrderSet { country, orders })
                .collect(),
        )
    };
    step(
        &mut w,
        vec![(
            riv,
            vec![Order::DeclareWar {
                target: neu,
                aim: WarAim::Limited,
            }],
        )],
    );
    step(
        &mut w,
        vec![(
            maj,
            vec![Order::StartArmsStream {
                to: neu,
                amount: 1.0,
                covert,
            }],
        )],
    );
    step(&mut w, vec![]);
    assert!(w.wars.active[0].progress() > 0.0, "the Neutral is losing");
    step(&mut w, vec![(maj, vec![Order::StopArmsStream { to: neu }])]);
    let entry = w
        .ledger
        .entries
        .iter()
        .find(|e| e.actor == maj && e.cause_code == sim_core::CauseCode::WarWithdrawal)
        .cloned()
        .expect("withdrawal recorded");
    (w, entry)
}

#[test]
fn covert_exit_is_cheaper_than_public_exit() {
    let (pw, public) = proxy_exit(false);
    let (cw, covert) = proxy_exit(true);
    let maj = id(&pw, "MAJ");
    let n = pw.countries.len();
    assert_eq!(public.grade, Some(Grade::Abandoned));
    assert_eq!(covert.visibility, Visibility::Covert);
    let seen = |e: &sim_core::LedgerEntry| e.seen_by.iter(n).count();
    // Reputation damage summed over every observer except the actor.
    let damage = |w: &WorldState| -> f64 {
        w.ids()
            .filter(|&o| o != maj)
            .map(|o| 100.0 - reputation::credibility(w, o, maj, RepKind::Back))
            .sum()
    };
    println!(
        "public exit seen by {} (damage {:.1}); covert exit seen by {} (damage {:.1})",
        seen(&public),
        damage(&pw),
        seen(&covert),
        damage(&cw)
    );
    assert!(seen(&covert) < seen(&public));
    assert!(damage(&cw) < damage(&pw));
}

#[test]
fn token_forces_grade_partial_real_forces_honour() {
    let grade_for = |band: u8| {
        let mut w = scenario::build(&crisis_def(), Some(5)).unwrap();
        let (maj, riv, neu) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "NEU"));
        resolve_turn(
            &mut w,
            vec![OrderSet {
                country: riv,
                orders: vec![Order::DeclareWar {
                    target: neu,
                    aim: WarAim::Limited,
                }],
            }],
        );
        let war = w.wars.active[0].id;
        resolve_turn(
            &mut w,
            vec![OrderSet {
                country: maj,
                orders: vec![Order::JoinWar {
                    war,
                    side: Side::Defender,
                    band,
                }],
            }],
        );
        for _ in 0..2 {
            resolve_turn(&mut w, vec![]);
        }
        w.ledger
            .entries
            .iter()
            .find(|e| e.actor == maj && e.cause_code == sim_core::CauseCode::AllyAttacked)
            .and_then(|e| e.grade)
            .expect("guarantee test graded")
    };
    // Band 3 commits 30% of 60 = 18 against an attacker committing 45:
    // a declaration without enough forces is fake honour.
    assert_eq!(grade_for(3), Grade::Partial);
    assert_eq!(grade_for(4), Grade::Honoured);
}

#[test]
fn arming_both_sides_is_on_the_record() {
    let mut w = scenario::build(&crisis_def(), Some(5)).unwrap();
    let (maj, riv, neu, aly) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "NEU"), id(&w, "ALY"));
    let _ = maj;
    resolve_turn(
        &mut w,
        vec![OrderSet {
            country: riv,
            orders: vec![Order::DeclareWar {
                target: neu,
                aim: WarAim::Limited,
            }],
        }],
    );
    let arm = |to| Order::StartArmsStream {
        to,
        amount: 0.5,
        covert: false,
    };
    let r = resolve_turn(
        &mut w,
        vec![OrderSet {
            country: aly,
            orders: vec![arm(riv), arm(neu)],
        }],
    );
    let partials = r
        .ledger_log
        .iter()
        .filter(|l| {
            matches!(l, LedgerLog::EntryWritten { actor, grade: Some(Grade::Partial), code: sim_core::CauseCode::ArmedBothSides, .. } if *actor == aly)
        })
        .count();
    assert_eq!(partials, 2, "{:?}", r.ledger_log);
}

#[test]
fn proxy_supply_needs_a_route() {
    let mut w = scenario::build(&crisis_def(), Some(5)).unwrap();
    let (aly, riv, neu) = (id(&w, "ALY"), id(&w, "RIV"), id(&w, "NEU"));
    resolve_turn(
        &mut w,
        vec![OrderSet {
            country: riv,
            orders: vec![Order::DeclareWar {
                target: neu,
                aim: WarAim::Limited,
            }],
        }],
    );
    // The Ally's navy is far smaller than the Rival's and it has no bases.
    assert!(!sim_core::war::arms_route(&w, aly, neu));
    let before = w.country(neu).arms_in;
    resolve_turn(
        &mut w,
        vec![OrderSet {
            country: aly,
            orders: vec![Order::ArmsTransfer { to: neu, amount: 1.0 }],
        }],
    );
    assert_eq!(w.country(neu).arms_in, before, "no route, no delivery");
    // A consenting conduit opens one.
    let maj = id(&w, "MAJ");
    sim_core::diplomacy::sign(&mut w, TreatyKind::Basing, aly, maj);
    assert!(sim_core::war::arms_route(&w, aly, neu));
}

#[test]
fn an_ai_guarantor_picks_a_band_and_explains_it() {
    let world = scenario::build(&crisis_def(), Some(5)).unwrap();
    let (riv, neu) = (id(&world, "RIV"), id(&world, "NEU"));
    let mut h = Harness::new(world, &["MAJ", "RIV", "NEU", "ALY"]);
    // Record decisions by observing what the Major Power's controller does.
    h.step(vec![(
        riv,
        vec![Order::DeclareWar {
            target: neu,
            aim: WarAim::Limited,
        }],
    )]);
    let view = observe(&h.world, id(&h.world, "MAJ"));
    let mut s = Strategist::new();
    let d = s.decide(&view);
    let r = d
        .records
        .iter()
        .find(|r| r.subject.starts_with("involvement in"))
        .expect("the guarantor decides on involvement");
    for l in &r.lines {
        println!("  {:+6.1} {}", l.value, l.label);
    }
    println!("{} -> {:.1} chosen {}", r.subject, r.score, r.chosen);
    let sum: f64 = r.lines.iter().map(|l| l.value).sum();
    assert!((sum - r.score).abs() < 1e-9);
    assert!(r.chosen && r.score > 0.0);
}

#[test]
#[ignore]
fn debug_band_table() {
    let world = scenario::build(&crisis_def(), Some(5)).unwrap();
    let (riv, neu) = (id(&world, "RIV"), id(&world, "NEU"));
    let mut h = Harness::new(world, &["RIV", "NEU", "ALY"]);
    h.step(vec![(
        riv,
        vec![Order::DeclareWar {
            target: neu,
            aim: WarAim::Limited,
        }],
    )]);
    for who in ["MAJ", "ALY"] {
        let view = observe(&h.world, id(&h.world, who));
        let w = &view.wars[0];
        let (side, st) = ai::war::favoured_side(&view, w).unwrap();
        println!("{who} favours {side:?} stake {st:.2}");
        for o in ai::war::BandOption::ALL {
            let s = ai::war::band(&view, w, side, o);
            println!("  {:<32} {:+6.1}", o.label(), s.total());
            for l in s.sorted() {
                println!("        {:+6.1} {}", l.value, l.label);
            }
        }
    }
}

#[test]
fn wars_are_deterministic_per_seed() {
    let base = scenario::build(&crisis_def(), Some(9)).unwrap();
    for (_, orders) in options(&base) {
        let (a, ma) = branch(orders.clone(), 9);
        let (b, mb) = branch(orders, 9);
        assert_eq!(ma, mb);
        assert_eq!(a.events.len(), b.events.len());
        assert_eq!(
            serde_json::to_string(&a.world.wars).unwrap(),
            serde_json::to_string(&b.world.wars).unwrap()
        );
    }
}

/// A guarantee is a contingency order against an attack on the protégé, not
/// a promise to join its own war of choice: the ledger opens a Back test only
/// for the defender's allies, so the guarantor's band choice must not count
/// "our word as an ally" (or a full formal stake) when its protégé attacks.
/// (Crisis pass 2: the US guaranteed Iraq on turn 1 and then co-invaded Iran
/// in 77/77 Iran-Iraq wars, collapsing the Iranian regime in 31/40 runs.)
#[test]
fn a_guarantee_does_not_pull_the_guarantor_into_its_proteges_offensive() {
    let world = scenario::build(&crisis_def(), Some(5)).unwrap();
    let (riv, neu) = (id(&world, "RIV"), id(&world, "NEU"));
    // The guaranteed Neutral starts the war.
    let mut h = Harness::new(world, &["RIV", "NEU", "ALY"]);
    h.step(vec![(
        neu,
        vec![Order::DeclareWar {
            target: riv,
            aim: WarAim::Limited,
        }],
    )]);
    let view = observe(&h.world, id(&h.world, "MAJ"));
    let w = &view.wars[0];
    assert_eq!(w.attacker, neu);
    assert!(ai::war::stake(&view, w, Side::Attacker) < 1.0);
    for o in ai::war::BandOption::ALL {
        let s = ai::war::band(&view, w, Side::Attacker, o);
        assert!(
            s.sorted().iter().all(|l| l.label != "our word as an ally"),
            "{} counted our word for an attacking protégé",
            o.label()
        );
    }

    // The same guarantee is engaged when the protégé is the one attacked.
    let world = scenario::build(&crisis_def(), Some(5)).unwrap();
    let mut h = Harness::new(world, &["RIV", "NEU", "ALY"]);
    h.step(vec![(
        riv,
        vec![Order::DeclareWar {
            target: neu,
            aim: WarAim::Limited,
        }],
    )]);
    let view = observe(&h.world, id(&h.world, "MAJ"));
    let w = &view.wars[0];
    assert!((ai::war::stake(&view, w, Side::Defender) - 1.0).abs() < 1e-9);
    // Staying out of a protégé's defence costs our word.
    let s = ai::war::band(&view, w, Side::Defender, ai::war::BandOption::ALL[0]);
    assert!(s
        .sorted()
        .iter()
        .any(|l| l.label == "our word as an ally" && l.value < 0.0));
}
