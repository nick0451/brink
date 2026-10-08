//! Issue 28: oil-state budgets with reserves. An exporter's spending is
//! committed to the revenue it was built on and follows the oil price only
//! slowly (`commodity::SPENDING_ADJUST`); a windfall above it fills
//! reserves, a slump below it draws them down, and the gap becomes money
//! pressure (`ai::evaluate::money_pressure`) only once the reserves run low.
//! Before the fix a slump cut spending at once and added no pressure: no
//! exporter ever came under pressure from the price, and the oil price
//! never swung by half after the first year (0 of 100 runs, D102). Every
//! test works from data and state, never from a country's name.

use std::path::PathBuf;

use sim_core::commodity::{self, ProductionPolicy};
use sim_core::{observe, CountryId, WorldState};

fn def1980() -> scenario::ScenarioDef {
    scenario::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/scenarios/1980.ron")).unwrap()
}

fn cleared(def: &scenario::ScenarioDef) -> WorldState {
    let mut w = scenario::build(def, Some(1)).unwrap();
    sim_core::resolve_turn(&mut w, vec![]);
    w
}

/// The largest exporter of the scenario (by net exports), found from state.
fn largest_exporter(w: &WorldState) -> CountryId {
    w.countries
        .iter()
        .max_by(|a, b| a.energy_net_exports.total_cmp(&b.energy_net_exports))
        .unwrap()
        .id
}

#[test]
fn reserves_fill_in_a_windfall_and_drain_in_a_slump() {
    let w = cleared(&def1980());
    let mut c = w.country(largest_exporter(&w)).clone();
    let revenue = c.gdp * c.tax_rate;
    c.oil_budget = Some(1.0);
    c.reserves = 2.0 * revenue;
    // Windfall: revenue at 1.3, spending held at the committed 1.0.
    let spent = commodity::oil_budget(&mut c, 1.3);
    assert!((spent - 1.0).abs() < 1e-9, "spent {spent}");
    assert!((c.reserves - 2.3 * revenue).abs() < 1e-9, "the windfall is saved");
    assert!(c.oil_budget.unwrap() > 1.0, "commitments creep up toward revenue");
    // Slump: revenue at 0.7; spending held up while reserves last.
    c.oil_budget = Some(1.0);
    let spent = commodity::oil_budget(&mut c, 0.7);
    assert!((spent - 1.0).abs() < 1e-9, "spending held up by reserves");
    assert!((c.reserves - 2.0 * revenue).abs() < 1e-9, "the gap is drawn");
    let mut last = spent;
    for _ in 0..40 {
        last = commodity::oil_budget(&mut c, 0.7);
    }
    assert_eq!(c.reserves, 0.0, "a long slump empties the account");
    assert!((last - 0.7).abs() < 1e-9, "then spending falls to revenue: {last}");
    // An importer spends what it gets: no reserves move.
    let mut m = w.country(w.find("JPN").unwrap()).clone();
    assert!(m.energy_net_exports < 0.0);
    let before = m.reserves;
    assert_eq!(commodity::oil_budget(&mut m, 0.9), 0.9);
    assert_eq!(m.reserves, before);
}

fn scores(w: &WorldState, id: CountryId) -> (f64, f64, ai::Score) {
    let v = observe(w, id);
    let n = ai::evaluate::energy_policy(&v, ProductionPolicy::Normal);
    let f = ai::evaluate::energy_policy(&v, ProductionPolicy::Flood);
    (n.total(), f.total(), f)
}

/// The failing fixture (data swap): the same exporter in the same slump
/// (price 20% below base, budget still committed to the 1980 price),
/// once with ample reserves and once with the account empty. With reserves
/// it waits and holds; without them the slump gap is money pressure and it
/// breaks ranks, saying why. Before the fix both held: a slump added no
/// pressure at all.
#[test]
fn a_slump_presses_an_exporter_only_once_its_reserves_run_low() {
    let mut w = cleared(&def1980());
    let id = largest_exporter(&w);
    w.energy.price = 0.8 * w.energy.base_price;
    let revenue = w.country(id).gdp * w.country(id).tax_rate;

    w.country_mut(id).reserves = 100.0 * revenue;
    let v = observe(&w, id);
    assert_eq!(ai::evaluate::money_pressure(&v), 0.0, "ample reserves: it can wait");
    let (normal, flood, _) = scores(&w, id);
    assert!(flood < normal, "ample: holds, flood {flood:+.1} vs normal {normal:+.1}");

    w.country_mut(id).reserves = 0.0;
    let v = observe(&w, id);
    let pressure = ai::evaluate::money_pressure(&v);
    assert!(pressure > 0.5, "empty account: pressure {pressure:.2}");
    let (normal, flood, fs) = scores(&w, id);
    assert!(
        fs.lines
            .iter()
            .any(|l| l.label.starts_with("reserves 0 quarters left") && l.value > 0.0),
        "the reason is in the log: {:?}",
        fs.lines
    );
    assert!(
        flood > normal,
        "depleted: cheats, flood {flood:+.1} vs normal {normal:+.1}"
    );
}

/// The campaign fixture (D101 a, Gate 6 target > 70%): the oil price swings
/// by half after the first year once rigid budgets drain reserves; with
/// every exporter's reserves made bottomless (data swap) no flood is ever
/// taken for want of reserves.
#[test]
fn rigid_budgets_drain_reserves_and_swing_the_oil_price() {
    let def = def1980();
    let swung = (1..=4u64)
        .filter(|&s| {
            let r = headless::run_campaign(&def, s, 80).unwrap();
            headless::stats::plausibility(&r).price_swing >= 1.5
        })
        .count();
    assert!(swung >= 3, "oil swing of 50%+ after year 1 in {swung}/4 runs");

    let mut ample = def.clone();
    for c in &mut ample.countries {
        c.reserves = 1000.0 * c.gdp;
    }
    for s in 1..=2u64 {
        let r = headless::run_campaign(&ample, s, 80).unwrap();
        let reserve_floods = r
            .reasoning
            .iter()
            .filter(|e| e.decision.subject == "energy policy: Flood")
            .filter(|e| e.decision.lines.iter().any(|l| l.label.starts_with("reserves")))
            .count();
        assert_eq!(reserve_floods, 0, "seed {s}");
    }
}
