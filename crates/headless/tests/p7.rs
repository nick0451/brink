//! v0.2 P7: world interest rate and the reserve holder's monetary stance.

use std::path::PathBuf;

use sim_core::money::{self, MonetaryStance};
use sim_core::{resolve_turn, CountryId, Order, OrderSet, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn def() -> scenario::ScenarioDef {
    scenario::load(root().join("data/scenarios/1980.ron")).unwrap()
}

fn world(seed: u64) -> WorldState {
    scenario::build(&def(), Some(seed)).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

fn step(w: &mut WorldState, actor: CountryId, orders: Vec<Order>) -> sim_core::TurnReport {
    resolve_turn(w, vec![OrderSet { country: actor, orders }])
}

#[test]
fn the_reserve_holder_is_data_not_identity() {
    let mut w = world(1);
    let (usa, jpn) = (id(&w, "USA"), id(&w, "JPN"));
    assert_eq!(money::holder(&w), Some(usa));
    // Anyone else's stance order is rejected.
    let r = step(&mut w, jpn, vec![Order::SetMonetaryStance(MonetaryStance::Loose)]);
    assert_eq!(r.rejected.len(), 1);
    assert_eq!(w.money.stance, MonetaryStance::Tight, "1980 opens tight (data)");
    // Swap the financial weight and the other country holds the currency.
    let mut w = world(1);
    w.country_mut(jpn).financial_weight = 0.8;
    assert_eq!(money::holder(&w), Some(jpn));
    step(&mut w, jpn, vec![Order::SetMonetaryStance(MonetaryStance::Loose)]);
    assert_eq!(w.money.stance, MonetaryStance::Loose);
}

#[test]
fn tight_money_raises_debtors_bills_and_costs_them_prosperity() {
    let mut tight = world(1);
    let mut loose = world(1);
    let usa = id(&tight, "USA");
    let pol = id(&tight, "POL");
    step(&mut loose, usa, vec![Order::SetMonetaryStance(MonetaryStance::Loose)]);
    step(&mut tight, usa, vec![]);
    for _ in 0..8 {
        step(&mut tight, usa, vec![]);
        step(&mut loose, usa, vec![]);
    }
    assert!(tight.money.rate > 2.0 * loose.money.rate, "{} vs {}", tight.money.rate, loose.money.rate);
    let (pt, pl) = (tight.country(pol), loose.country(pol));
    assert!(pt.debt_service > 2.0 * pl.debt_service);
    assert!(pt.debt_service_hit > pl.debt_service_hit + 5.0, "{} vs {}", pt.debt_service_hit, pl.debt_service_hit);
    // The reserve currency's privilege: the holder pays less on the same debt.
    let us = tight.country(usa);
    assert!(us.debt_service < us.debt * tight.money.rate);
}

#[test]
fn an_inflation_spiral_persists_until_tight_money_breaks_it() {
    let dev = 0.0;
    let mut neutral = 1.0;
    let mut tight = 1.0;
    let mut loose = 1.0;
    for _ in 0..16 {
        neutral += (money::inflation_target(dev, MonetaryStance::Neutral, neutral) - neutral) * money::INFLATION_ADJUST;
        tight += (money::inflation_target(dev, MonetaryStance::Tight, tight) - tight) * money::INFLATION_ADJUST;
        loose += (money::inflation_target(dev, MonetaryStance::Loose, loose) - loose) * money::INFLATION_ADJUST;
    }
    assert!(tight < 0.2, "four years of tight money: {tight}");
    assert!(neutral > 0.3 && neutral < 1.0, "neutral lets it fade slowly: {neutral}");
    assert!(loose > 1.0 && loose <= money::MAX_INFLATION, "easy money feeds it: {loose}");
}

/// The 1980 campaign: the AI holds tight money while inflation is high,
/// then eases; the world rate peaks early (a Volcker shape, not a script).
#[test]
fn the_volcker_shape_emerges_in_the_1980_campaign() {
    let runs = headless::run_batch(&def(), &(1..=10).collect::<Vec<_>>(), 40, 8).unwrap();
    let mut eased = 0;
    for r in &runs {
        let early = r.samples[1].interest_rate;
        let late = r.samples[40].interest_rate;
        assert!(early > 10.0, "rates open high: {early}");
        if late < early - 3.0 {
            eased += 1;
        }
    }
    assert!(eased >= 8, "the holder eases once inflation is broken in most runs ({eased}/10)");
}
