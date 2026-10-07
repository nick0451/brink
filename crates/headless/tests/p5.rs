//! v0.2 P5: the world energy market and production policy (DESIGN §2.4;
//! scenario-1980 P5).

use std::path::PathBuf;

use sim_core::commodity::{revenue_factor, ProductionPolicy};
use sim_core::{resolve_turn, CountryId, Order, OrderSet, WarAim, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world1980(seed: u64) -> WorldState {
    scenario::build(
        &scenario::load(root().join("data/scenarios/1980.ron")).unwrap(),
        Some(seed),
    )
    .unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

fn step(w: &mut WorldState, acts: Vec<(CountryId, Vec<Order>)>) {
    resolve_turn(
        w,
        acts.into_iter()
            .map(|(country, orders)| OrderSet { country, orders })
            .collect(),
    );
}

#[test]
fn fixtures_without_energy_data_keep_the_market_off() {
    let mut w = scenario::build(
        &scenario::load(root().join("data/fixtures/four_actor.ron")).unwrap(),
        Some(1),
    )
    .unwrap();
    for _ in 0..10 {
        step(&mut w, vec![]);
    }
    assert_eq!(w.energy.price, 1.0);
    assert_eq!(w.energy.supply, 0.0);
}

#[test]
fn war_between_exporters_raises_the_price() {
    let mut calm = world1980(3);
    let mut war = world1980(3);
    let (irq, irn) = (id(&war, "IRQ"), id(&war, "IRN"));
    step(
        &mut war,
        vec![(
            irq,
            vec![Order::DeclareWar {
                target: irn,
                aim: WarAim::Limited,
            }],
        )],
    );
    step(&mut calm, vec![]);
    for _ in 0..3 {
        step(&mut war, vec![]);
        step(&mut calm, vec![]);
    }
    println!("price calm {:.3} war {:.3}", calm.energy.price, war.energy.price);
    assert!(war.energy.price > calm.energy.price);
}

#[test]
fn a_flood_lowers_the_price_and_squeezes_other_exporters() {
    let mut normal = world1980(5);
    let mut flood = world1980(5);
    let sau = id(&flood, "SAU");
    let sov = id(&flood, "SOV");
    step(
        &mut flood,
        vec![(sau, vec![Order::SetEnergyPolicy(ProductionPolicy::Flood)])],
    );
    step(&mut normal, vec![]);
    for _ in 0..4 {
        step(&mut flood, vec![]);
        step(&mut normal, vec![]);
    }
    println!(
        "price normal {:.3} flood {:.3}",
        normal.energy.price, flood.energy.price
    );
    assert!(flood.energy.price < normal.energy.price);
    let rf = |w: &WorldState, c: CountryId| {
        let x = w.country(c);
        revenue_factor(x.energy_net_exports, x.gdp, w.energy.deviation())
    };
    assert!(rf(&flood, sov) < rf(&normal, sov), "another exporter's revenue falls");
}

#[test]
fn high_prices_help_exporters_and_hurt_importers() {
    assert!(revenue_factor(10.0, 5.0, 0.5) > 1.0);
    assert!(revenue_factor(-10.0, 20.0, 0.5) < 1.0);
    assert_eq!(revenue_factor(10.0, 5.0, 0.0), 1.0);
    // Capped: an oil state's budget can swing hard but not without limit.
    assert!(revenue_factor(100.0, 1.0, 3.0) <= 1.6);
}

#[test]
fn ai_exporters_choose_policies_and_explain_them() {
    let r = headless::run_campaign(&scenario::load(root().join("data/scenarios/1980.ron")).unwrap(), 4, 40).unwrap();
    let records: Vec<_> = r
        .reasoning
        .iter()
        .filter(|e| e.decision.subject.starts_with("energy policy"))
        .collect();
    assert!(!records.is_empty(), "some exporter changes policy");
    for e in &records {
        let sum: f64 = e.decision.lines.iter().map(|l| l.value).sum();
        assert!((sum - e.decision.score).abs() < 1e-9 && e.decision.score > 0.0);
    }
    // Importers never set a production policy.
    let w = world1980(4);
    for e in &records {
        let c = w.country(w.find(&e.country).unwrap());
        assert!(c.energy_net_exports > 0.0, "{} is an importer", e.country);
    }
}
