//! Build step 6 acceptance tests: force pools, threat-driven Security,
//! permanent arms transfers → Gate 4 "blowback without scripting"
//! (DESIGN §6.1–6.2, §21.6, §22.1).

use std::path::PathBuf;

use ai::evaluate;
use ai::inputs::{foreign, threat};
use sim_core::domestic::security_breakdown;
use sim_core::economy::{ARMS_QUALITY_MARGIN, MAX_ARMS_SHARE, MILITARY_UPKEEP};
use sim_core::{
    observe, resolve_turn, BudgetShares, CountryId, Grade, LedgerLog, Order, OrderSet, PoolKind, Sanction, TurnReport,
    WorldState,
};

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
        value: 70.0,
    });
    def
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
fn starting_strength_splits_by_force_mix_at_tech_quality() {
    let w = world();
    let maj = w.country(id(&w, "MAJ"));
    let f = &maj.forces;
    assert!((f.strength() - 60.0).abs() < 1e-9);
    assert!((f.land.strength - 36.0).abs() < 1e-9);
    assert!((f.naval.strength - 12.0).abs() < 1e-9);
    assert!((f.air.strength - 12.0).abs() < 1e-9);
    assert_eq!(f.land.quality, maj.military_tech as f64);
    // Fully funded peacetime force at reference quality: power = strength.
    assert!((maj.power() - 60.0).abs() < 1e-9);
}

#[test]
fn cutting_military_spending_hollows_readiness_before_strength() {
    let mut w = world();
    let maj = id(&w, "MAJ");
    let start = w.country(maj).forces;
    let cut = BudgetShares {
        military: 0.02,
        development: 0.33,
        welfare: 0.55,
        intelligence: 0.10,
    };
    for _ in 0..6 {
        step(&mut w, vec![(maj, vec![Order::SetBudget(cut)])]);
    }
    let now = w.country(maj).forces;
    let strength_kept = now.strength() / start.strength();
    let power_kept = now.power() / start.power();
    println!(
        "after 6 turns: strength {:.1}% readiness {:.2} power {:.1}%",
        100.0 * strength_kept,
        now.readiness,
        100.0 * power_kept
    );
    assert!(now.readiness < 0.3, "readiness {}", now.readiness);
    assert!(strength_kept > 0.7, "strength wears slowly: {strength_kept}");
    assert!(power_kept < strength_kept - 0.1, "power falls faster than strength");
}

#[test]
fn arms_move_between_pools_capped_per_turn_and_conserved() {
    let mut armed = world();
    let mut control = world();
    let (maj, neu) = (id(&armed, "MAJ"), id(&armed, "NEU"));
    let cap = armed.country(maj).forces.strength() * MAX_ARMS_SHARE;
    let report = step(
        &mut armed,
        vec![(
            maj,
            vec![Order::ArmsTransfer {
                to: neu,
                amount: 1000.0,
            }],
        )],
    );
    step(&mut control, vec![]);
    assert!(report.rejected.is_empty(), "{:?}", report.rejected);

    let moved = armed.country(neu).arms_in;
    assert!((moved - cap).abs() < 1e-9, "transfer capped at {cap}, moved {moved}");
    assert!((armed.country(maj).arms_out - cap).abs() < 1e-9);
    assert!((armed.country(maj).arms_out_total - cap).abs() < 1e-9);
    // What the supplier lost the recipient gained (after this turn's wear).
    let lost = control.country(maj).forces.strength() - armed.country(maj).forces.strength();
    let gained = armed.country(neu).forces.strength() - control.country(neu).forces.strength();
    assert!((lost - gained).abs() < 1e-6, "lost {lost} gained {gained}");
    assert!((gained - cap * (1.0 - MILITARY_UPKEEP)).abs() < 1e-6);
    assert!((armed.country(neu).arms_from(maj) - cap * (1.0 - MILITARY_UPKEEP)).abs() < 1e-6);
}

#[test]
fn received_quality_is_capped_at_recipient_tech_plus_two() {
    let mut w = world();
    let (maj, neu) = (id(&w, "MAJ"), id(&w, "NEU"));
    w.country_mut(maj).military_tech = 9;
    for k in PoolKind::ALL {
        w.country_mut(maj).forces.pool_mut(k).quality = 9.0;
    }
    w.country_mut(neu).military_tech = 3;
    for k in PoolKind::ALL {
        w.country_mut(neu).forces.pool_mut(k).quality = 3.0;
    }
    let before = w.country(neu).forces.land.strength;
    let maj_land = w.country(maj).forces.land.strength;
    let maj_total = w.country(maj).forces.strength();
    sim_core::economy::transfer_arms(&mut w, maj, neu, 4.0);
    let incoming = 4.0 * maj_land / maj_total;
    let cap = 3.0 + ARMS_QUALITY_MARGIN;
    let expected = (before * 3.0 + incoming * cap) / (before + incoming);
    let land = w.country(neu).forces.land;
    assert!((land.quality - expected).abs() < 1e-9, "{} vs {expected}", land.quality);
    assert!(land.quality < cap);
    // The supplier's remaining forces keep their quality.
    assert_eq!(w.country(maj).forces.land.quality, 9.0);
}

#[test]
fn security_reads_hostile_power_and_believed_allies() {
    let mut calm = world();
    let mut tense = scenario::build(&tense_def(), Some(1)).unwrap();
    step(&mut calm, vec![]);
    step(&mut tense, vec![]);
    let neu = id(&calm, "NEU");
    let calm_sec = calm.country(neu).security;
    let tense_sec = tense.country(neu).security;
    println!("NEU security calm {calm_sec:.1} tense {tense_sec:.1}");
    assert!(calm_sec > 70.0, "nobody hostile: {calm_sec}");
    assert!(tense_sec < 30.0, "hostile rival far stronger: {tense_sec}");

    // A guarantee from the Major Power raises it, scaled by belief.
    let maj = id(&tense, "MAJ");
    let without = security_breakdown(&tense, neu);
    step(&mut tense, vec![(maj, vec![Order::IssueGuarantee { to: neu }])]);
    let with = security_breakdown(&tense, neu);
    println!("breakdown without {without:?}\nwith {with:?}");
    assert!(with.allies.iter().any(|a| a.0 == maj && a.1 > 0.0));
    assert!(with.value > without.value + 10.0);
}

/// Gate 4 world. The Major Power arms the Neutral (or, in the control world,
/// doesn't), then turns on it: cuts the supply, sanctions and denounces it.
/// Afterwards the Rival sanctions the Major Power and the Neutral weighs
/// joining. Returns the world and every report after the flip.
fn gate4_world(armed: bool) -> (WorldState, Vec<TurnReport>) {
    let mut w = world();
    let (maj, neu, riv) = (id(&w, "MAJ"), id(&w, "NEU"), id(&w, "RIV"));
    if armed {
        step(
            &mut w,
            vec![(
                maj,
                vec![Order::StartArmsStream {
                    to: neu,
                    amount: 3.0,
                    covert: false,
                }],
            )],
        );
    } else {
        step(&mut w, vec![]);
    }
    for _ in 1..16 {
        step(&mut w, vec![]);
    }
    let mut flip = vec![Order::Sanction { target: neu }, Order::Denounce { target: neu }];
    if armed {
        flip.insert(0, Order::StopArmsStream { to: neu });
    }
    let mut reports = vec![step(&mut w, vec![(maj, flip)])];
    reports.push(step(&mut w, vec![(riv, vec![Order::Sanction { target: maj }])]));
    for _ in 0..8 {
        reports.push(step(&mut w, vec![]));
    }
    (w, reports)
}

#[test]
fn gate4_former_client_stays_stronger_and_uses_it_without_any_special_event() {
    let (a, a_reports) = gate4_world(true);
    let (b, b_reports) = gate4_world(false);
    let (maj, neu, riv) = (id(&a, "MAJ"), id(&a, "NEU"), id(&a, "RIV"));

    // The flip went through the ledger: the supply cut is on record.
    let cut = a_reports[0].ledger_log.iter().any(|l| {
        matches!(l, LedgerLog::EntryWritten { actor, counterpart, grade: Some(g), .. }
            if *actor == maj && *counterpart == neu && matches!(g, Grade::Lapsed | Grade::Abandoned))
    });
    assert!(cut, "supply cut must be a ledger entry: {:?}", a_reports[0].ledger_log);
    let opinion = a.opinions.opinion(neu, maj);
    assert!(opinion < -20.0, "relations flipped: NEU→MAJ opinion {opinion}");

    // 1. The client keeps the strength; the supplier never gets it back.
    let (na, nb) = (a.country(neu), b.country(neu));
    println!(
        "NEU strength armed {:.1} control {:.1}; still from MAJ {:.1}; power {:.1} vs {:.1}",
        na.forces.strength(),
        nb.forces.strength(),
        na.arms_from(maj),
        na.power(),
        nb.power()
    );
    assert!(na.arms_from(maj) > 5.0, "transferred strength still in service");
    assert!(na.forces.strength() > nb.forces.strength() + 5.0);
    assert!(na.power() > 2.0 * nb.power());
    assert!(a.country(maj).forces.strength() < b.country(maj).forces.strength());
    assert!(a.diplomacy.streams.is_empty(), "no supply is flowing any more");

    // 2. It changes how the former client sees its former patron.
    let (va, vb) = (observe(&a, neu), observe(&b, neu));
    let (ta, tb) = (
        threat(&va, foreign(&va, maj).unwrap()),
        threat(&vb, foreign(&vb, maj).unwrap()),
    );
    println!("NEU threat from MAJ armed {ta:.1} control {tb:.1}");
    assert!(ta < tb, "a client holding the patron's weapons fears it less");
    assert!(na.security > nb.security, "{} vs {}", na.security, nb.security);

    // 3. ...and a decision against the former patron.
    let sanction = Sanction {
        by: riv,
        target: maj,
        since: a.turn,
    };
    let (sa, sb) = (
        evaluate::join_sanction(&va, &sanction),
        evaluate::join_sanction(&vb, &sanction),
    );
    let term = |s: &ai::Score, t: &str| s.lines.iter().find(|l| l.term == t).map_or(0.0, |l| l.value);
    println!(
        "NEU joins sanction on MAJ: armed {:+.1} (retaliation {:+.1}) control {:+.1} (retaliation {:+.1})",
        sa.total(),
        term(&sa, "retaliation_risk"),
        sb.total(),
        term(&sb, "retaliation_risk")
    );
    for l in sa.sorted() {
        println!("    armed   {:+6.1}  {}", l.value, l.label);
    }
    for l in sb.sorted() {
        println!("    control {:+6.1}  {}", l.value, l.label);
    }
    assert!(term(&sa, "retaliation_risk") > term(&sb, "retaliation_risk") + 1.0);
    assert!(sa.total() > sb.total());

    // 4. No special event: after the flip, the armed world produces exactly
    // the same kinds of events and ledger records as the control world.
    let kinds = |reports: &[TurnReport]| -> Vec<String> {
        let mut v: Vec<String> = reports
            .iter()
            .flat_map(|r| {
                r.events
                    .iter()
                    .map(|e| format!("{:?}", std::mem::discriminant(e)))
                    .chain(r.ledger_log.iter().map(|l| format!("{:?}", std::mem::discriminant(l))))
            })
            .collect();
        v.sort();
        v.dedup();
        v
    };
    assert_eq!(kinds(&a_reports[1..]), kinds(&b_reports[1..]));
}

#[test]
fn no_scripted_blowback_anywhere_in_engine_ai_or_voice() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for dir in [
        "crates/sim-core/src",
        "crates/ai/src",
        "crates/voice/src",
        "crates/scenario/src",
    ] {
        for entry in std::fs::read_dir(root.join(dir)).unwrap() {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            for (n, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap().to_lowercase();
                assert!(
                    !code.contains("blowback"),
                    "{}:{}: blowback must emerge from state, not code",
                    path.display(),
                    n + 1
                );
            }
        }
    }
}

#[test]
fn cutting_arms_to_a_client_in_crisis_is_an_abandonment_like_aid() {
    let mut w = scenario::build(&tense_def(), Some(1)).unwrap();
    let (maj, neu) = (id(&w, "MAJ"), id(&w, "NEU"));
    step(
        &mut w,
        vec![(
            maj,
            vec![Order::StartArmsStream {
                to: neu,
                amount: 1.0,
                covert: false,
            }],
        )],
    );
    w.country_mut(neu).stability = 20.0;
    step(&mut w, vec![]);
    let report = step(&mut w, vec![(maj, vec![Order::StopArmsStream { to: neu }])]);
    let abandoned = report.ledger_log.iter().any(|l| {
        matches!(l, LedgerLog::EntryWritten { actor, counterpart, grade: Some(Grade::Abandoned), .. }
            if *actor == maj && *counterpart == neu)
    });
    assert!(abandoned, "{:?}", report.ledger_log);
}
