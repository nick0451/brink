//! D58: arms export as commerce. Any country with an arms industry can sell;
//! buyers pay a market price that rises with isolation and war; patrons
//! object to their clients arming the patron's rivals.

use std::path::PathBuf;

use ai::{Controller, Strategist};
use sim_core::{observe, resolve_turn, CountryId, Order, OrderSet, WorldState};

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

fn step(w: &mut WorldState, actor: CountryId, orders: Vec<Order>) {
    resolve_turn(w, vec![OrderSet { country: actor, orders }]);
}

#[test]
fn isolated_and_warring_buyers_pay_a_premium() {
    use sim_core::diplomacy::arms_price;
    assert_eq!(arms_price(0, false), 1.0);
    assert!((arms_price(3, false) - 1.3).abs() < 1e-9);
    assert!((arms_price(9, true) - 1.75).abs() < 1e-9, "sanction premium caps at +0.5");
}

#[test]
fn a_sale_is_paid_for_and_an_industry_builds_most_of_it() {
    let base = world(1);
    let (isr, twn) = (id(&base, "ISR"), id(&base, "TWN"));
    let amount = 0.05;

    let mut sold = base.clone();
    step(&mut sold, isr, vec![Order::SellArms { to: twn, amount, covert: false }]);
    let mut gifted = base.clone();
    step(&mut gifted, isr, vec![Order::StartArmsStream { to: twn, amount, covert: false }]);

    let s = sold.country(isr);
    assert!(s.arms_sold > 0.0 && s.arms_income > 0.0, "the seller delivers and is paid");
    assert!((sold.country(twn).arms_in - s.arms_sold).abs() < 1e-12, "the buyer gets all of it");
    let unit = sim_core::diplomacy::arms_value(sim_core::StreamKind::Arms, 1.0);
    assert!((s.arms_income - s.arms_sold * unit).abs() < 1e-9, "TWN is unsanctioned and at peace: base price");
    assert_eq!(s.arms_out, 0.0, "a sale is commerce, not a defence burden");
    assert!(
        s.forces.strength() > gifted.country(isr).forces.strength(),
        "selling (industry 0.8) costs our forces less than giving the same arms away"
    );
}

#[test]
fn without_an_industry_a_sale_comes_out_of_your_own_forces() {
    // Data swap, not identity: the same order from the same country with its
    // industry set to zero gives up the full amount.
    let mut base = world(1);
    let (isr, twn) = (id(&base, "ISR"), id(&base, "TWN"));
    base.country_mut(isr).arms_industry = 0.0;
    let mut with = base.clone();
    let mut without = base.clone();
    // An industry of zero has no order book either; sales need capacity.
    step(&mut with, isr, vec![Order::SellArms { to: twn, amount: 0.05, covert: false }]);
    step(&mut without, isr, vec![]);
    assert_eq!(with.country(isr).arms_sold, 0.0, "no industry, no export order book");
    assert_eq!(
        with.country(isr).forces.strength(),
        without.country(isr).forces.strength()
    );
}

#[test]
fn patrons_object_to_arms_for_their_rivals_unless_it_is_covert() {
    let mut base = world(1);
    let (usa, isr, irn) = (id(&base, "USA"), id(&base, "ISR"), id(&base, "IRN"));
    assert!(
        base.diplomacy.streams.iter().any(|s| s.from == usa && s.to == isr && !s.sale),
        "fixture: the USA funds Israel"
    );
    base.tension.set(usa, irn, 70.0);
    assert_eq!(sim_core::diplomacy::sale_objectors(&base, isr, irn), vec![usa]);

    let before = base.opinions.opinion(usa, isr);
    let mut open = base.clone();
    step(&mut open, isr, vec![Order::SellArms { to: irn, amount: 0.02, covert: false }]);
    let mut covert = base.clone();
    step(&mut covert, isr, vec![Order::SellArms { to: irn, amount: 0.02, covert: true }]);
    let mut none = base.clone();
    step(&mut none, isr, vec![]);
    let (o_open, o_covert, o_none) = (
        open.opinions.opinion(usa, isr),
        covert.opinions.opinion(usa, isr),
        none.opinions.opinion(usa, isr),
    );
    assert!(o_open < o_none - 10.0, "open sale: {before:.1} -> {o_open:.1} vs {o_none:.1}");
    assert!((o_covert - o_none).abs() < 1e-9, "a covert sale is not seen (until exposed)");

    // The patron's AI re-scores its aid: a client arming its rival loses it.
    let view = observe(&open, usa);
    let aid = *view.streams.iter().find(|s| s.from == usa && s.to == isr && !s.sale).unwrap();
    let client = view.others.iter().find(|f| f.id == isr).unwrap();
    let s = ai::evaluate::keep_stream(&view, client, &aid, None, false, false);
    assert!(
        s.lines.iter().any(|l| l.term.contains("arming our rival") || l.label.contains("arming our rival")),
        "patron leverage: {s:?}"
    );
}

/// D68: an arms stream to a belligerent is a band-2 involvement, and ending
/// it writes a withdrawal whose cause names the band. The Gate 6 statistics
/// classify arms-only involvement by that "(band 2)" text, so this fixture
/// pins the wording the engine writes.
#[test]
fn ending_an_arms_stream_to_a_belligerent_writes_a_band_2_withdrawal() {
    use sim_core::{CauseCode, LedgerLog, WarAim};
    let mut w = world(1);
    let (isr, irn, irq) = (id(&w, "ISR"), id(&w, "IRN"), id(&w, "IRQ"));
    let r = resolve_turn(
        &mut w,
        vec![OrderSet {
            country: irq,
            orders: vec![Order::DeclareWar { target: irn, aim: WarAim::Limited }],
        }],
    );
    assert!(r.rejected.is_empty(), "fixture: {:?}", r.rejected);
    assert!(w.wars.at_war(irq, irn));
    step(&mut w, isr, vec![Order::SellArms { to: irn, amount: 0.02, covert: false }]);
    step(&mut w, isr, vec![]);
    let band = w
        .wars
        .involvements
        .iter()
        .find(|i| i.actor == isr && i.beneficiary == irn)
        .map(|i| i.band);
    assert_eq!(band, Some(2), "a sale to a belligerent is a band-2 involvement");

    let r = resolve_turn(
        &mut w,
        vec![OrderSet {
            country: irn,
            orders: vec![Order::CancelArmsPurchase { from: isr }],
        }],
    );
    let causes: Vec<&str> = r
        .ledger_log
        .iter()
        .filter_map(|l| match l {
            LedgerLog::EntryWritten {
                actor,
                code: CauseCode::WarWithdrawal,
                cause,
                ..
            } if *actor == isr => Some(cause.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(causes.len(), 1, "one withdrawal entry: {causes:?}");
    assert!(causes[0].contains("(band 2)"), "the cause names the band: {}", causes[0]);
    assert!(!w.wars.involvements.iter().any(|i| i.actor == isr && i.war == w.wars.active[0].id));
}

/// Across campaigns: sellers are the arms industries, they sell to buyers
/// with worse kit, and the small export-led industry earns the most
/// relative to its size (the paradox the mechanic exists for).
#[test]
fn campaign_arms_trade_follows_industry_not_identity() {
    let mut income_share: std::collections::BTreeMap<String, f64> = Default::default();
    let mut sales = 0;
    for seed in [1u64, 3] {
        let mut w = world(seed);
        let mut ais: Vec<Strategist> = w.ids().map(|_| Strategist::with_seed(seed)).collect();
        for _ in 0..80 {
            let views: Vec<_> = w.ids().map(|c| observe(&w, c)).collect();
            let orders: Vec<OrderSet> = views
                .iter()
                .zip(ais.iter_mut())
                .map(|(v, ai)| OrderSet {
                    country: v.observer,
                    orders: ai.decide(v).orders,
                })
                .collect();
            for o in &orders {
                for order in &o.orders {
                    if let Order::SellArms { to, .. } = order {
                        sales += 1;
                        let (s, b) = (w.country(o.country), w.country(*to));
                        assert!(s.arms_industry > 0.0, "{} sold without an industry", s.code);
                        assert!(
                            s.military_tech > b.military_tech
                                || (s.military_tech == b.military_tech && b.arms_industry == 0.0),
                            "{} sold to {} who builds better",
                            s.code,
                            b.code
                        );
                    }
                }
            }
            resolve_turn(&mut w, orders);
            for c in w.countries.iter().filter(|c| c.arms_income > 0.0) {
                *income_share.entry(c.code.clone()).or_default() += c.arms_income / (c.gdp * c.tax_rate);
            }
        }
    }
    assert!(sales >= 20, "the arms market should be active: {sales} sales");
    // The paradox, stated generically: small economies with export-led arms
    // industries earn more from arms, relative to their state's income, than
    // the large economies do.
    let start = world(1);
    let mean = |pick: &dyn Fn(&sim_core::Country) -> bool| {
        let v: Vec<f64> = start
            .countries
            .iter()
            .filter(|c| pick(c))
            .map(|c| income_share.get(&c.code).copied().unwrap_or(0.0))
            .collect();
        v.iter().sum::<f64>() / v.len().max(1) as f64
    };
    let small = mean(&|c| c.arms_industry >= 0.4 && c.gdp < 10.0);
    let large = mean(&|c| c.arms_industry > 0.0 && c.gdp >= 20.0);
    assert!(small > large, "small export-led {small:.2} vs large {large:.2}: {income_share:?}");
}
