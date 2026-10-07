//! v0.2 P10: covert programmes and declaration (scenario-1980 P10).

use std::path::PathBuf;

use sim_core::{observe, resolve_turn, CountryId, DiplomaticEvent, Order, OrderSet, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world(seed: u64) -> WorldState {
    scenario::build(
        &scenario::load(root().join("data/scenarios/1980.ron")).unwrap(),
        Some(seed),
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

#[test]
fn an_undeclared_arsenal_is_hidden_from_observers_without_coverage() {
    let w = world(1);
    let (isr, usa, cub) = (id(&w, "ISR"), id(&w, "USA"), id(&w, "CUB"));
    assert_eq!(w.country(isr).arsenal, 1);
    assert!(!w.country(isr).arsenal_declared);
    let seen_by = |o: CountryId| observe(&w, o).others.iter().find(|f| f.id == isr).unwrap().arsenal;
    assert_eq!(seen_by(usa), 1, "high coverage sees it");
    assert_eq!(seen_by(cub), 0, "low coverage doesn't");
}

#[test]
fn programmes_progress_by_capability_and_complete_as_undeclared_arsenals() {
    let mut w = world(1);
    let zaf = id(&w, "ZAF");
    let mut done = None;
    for t in 0..30 {
        let r = step(&mut w, vec![]);
        if r.events
            .iter()
            .any(|e| matches!(e, DiplomaticEvent::ProgrammeCompleted { country } if *country == zaf))
        {
            done = Some(t);
            break;
        }
    }
    assert!(done.is_some(), "South Africa's late-stage programme completes");
    let c = w.country(zaf);
    assert_eq!(c.arsenal, 1);
    // New arsenals are undeclared unless exposure already made them public.
    assert!(!c.arsenal_declared);
    // Capability: a tiny economy progresses far slower than a capable one.
    let cub = w.country(id(&w, "CUB"));
    assert!(sim_core::programme::capability(cub) < 0.5 * sim_core::programme::capability(w.country(id(&w, "IND"))));
}

#[test]
fn exposure_is_a_public_proliferation_violation() {
    // North Korea: a hostile observer with high coverage (the United States)
    // can see its programme.
    let mut w = world(1);
    let ind = id(&w, "PRK");
    let mut exposed = false;
    for _ in 0..60 {
        let r = step(&mut w, vec![]);
        if r.events
            .iter()
            .any(|e| matches!(e, DiplomaticEvent::ProgrammeExposed { country, .. } if *country == ind))
        {
            exposed = true;
            break;
        }
    }
    assert!(exposed);
    assert!(w.country(ind).programme_exposed);
    assert!(w
        .ledger
        .entries
        .iter()
        .any(|e| e.actor == ind && e.cause_code == sim_core::CauseCode::ProgrammeExposed));
}

#[test]
fn declaring_and_dismantling() {
    let mut w = world(1);
    let (isr, cub, zaf) = (id(&w, "ISR"), id(&w, "CUB"), id(&w, "ZAF"));
    let before = w.opinions.opinion(cub, isr);
    let r = step(&mut w, vec![(isr, vec![Order::DeclareArsenal])]);
    assert!(r
        .events
        .iter()
        .any(|e| matches!(e, DiplomaticEvent::ArsenalDeclared { .. })));
    assert!(w.country(isr).arsenal_declared);
    assert!(w.opinions.opinion(cub, isr) < before);
    let seen = observe(&w, cub).others.iter().find(|f| f.id == isr).unwrap().arsenal;
    assert_eq!(seen, 1, "declared arsenals are public");

    let r = step(&mut w, vec![(zaf, vec![Order::DiscloseAndDismantle])]);
    assert!(r
        .events
        .iter()
        .any(|e| matches!(e, DiplomaticEvent::ArsenalDismantled { .. })));
    assert!(w.country(zaf).programme.is_none() && w.country(zaf).arsenal == 0);
}

#[test]
fn a_punitive_strike_sets_a_programme_back_and_claims_the_norm() {
    let mut w = world(1);
    let (isr, irq) = (id(&w, "ISR"), id(&w, "IRQ"));
    let p0 = w.country(irq).programme.unwrap();
    step(
        &mut w,
        vec![(
            isr,
            vec![Order::DeclareWar {
                target: irq,
                aim: sim_core::WarAim::Punitive,
            }],
        )],
    );
    step(&mut w, vec![]);
    assert!(w.country(irq).programme.unwrap_or(0.0) < p0, "Osirak-style setback");
    assert!(w
        .ledger
        .norms
        .iter()
        .any(|n| n.actor == isr && n.norm == sim_core::NormTag::Proliferation));
}

#[test]
fn ai_programme_decisions_explain_themselves() {
    let r = headless::run_campaign(&scenario::load(root().join("data/scenarios/1980.ron")).unwrap(), 4, 40).unwrap();
    let recs: Vec<_> = r
        .reasoning
        .iter()
        .filter(|e| {
            matches!(
                e.decision.kind,
                sim_core::DecisionKind::Programme | sim_core::DecisionKind::Dismantle
            )
        })
        .collect();
    assert!(!recs.is_empty());
    for e in recs {
        let sum: f64 = e.decision.lines.iter().map(|l| l.value).sum();
        assert!((sum - e.decision.score).abs() < 1e-9);
    }
}
