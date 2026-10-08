//! Issue 26 (D99): flooding the oil market is a choice, not every
//! producer's default. A flood is weighed at the price left once the other
//! exporters still holding back answer it in kind; restraint invites no
//! answer. Every test works from data and state (opinions, tensions,
//! policies), never from a country's name.

use std::path::PathBuf;

use sim_core::commodity::ProductionPolicy;
use sim_core::{observe, CountryId, WorldState};

fn def1980() -> scenario::ScenarioDef {
    scenario::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/scenarios/1980.ron")).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

/// The 1980 world after one turn (the market has cleared once).
fn cleared(def: &scenario::ScenarioDef) -> WorldState {
    let mut w = scenario::build(def, Some(1)).unwrap();
    sim_core::resolve_turn(&mut w, vec![]);
    w
}

fn scores(w: &WorldState, code: &str) -> [(f64, ai::Score); 3] {
    let v = observe(w, id(w, code));
    [
        ProductionPolicy::Restrain,
        ProductionPolicy::Normal,
        ProductionPolicy::Flood,
    ]
    .map(|p| {
        let s = ai::evaluate::energy_policy(&v, p);
        println!("{code} {p:?} {:+.1}", s.total());
        for l in s.sorted() {
            println!("    {:+6.1}  {}", l.value, l.label);
        }
        (s.total(), s)
    })
}

fn line(s: &ai::Score, label: &str) -> f64 {
    s.lines.iter().filter(|l| l.label == label).map(|l| l.value).sum()
}

/// The defect: with demand this inelastic, a producer under a fifth of the
/// market gains by flooding if the others sit still, so every exporter
/// flooded in every run from the second year. A large producer whose own
/// barrels move the price (and a rentier for whom oil is everything) holds
/// at normal output when the others would answer.
#[test]
fn a_producer_holds_its_output_when_a_flood_would_start_a_price_war() {
    let w = cleared(&def1980());
    for code in ["SAU", "KWT"] {
        let [(_, _), (normal, _), (flood, fs)] = scores(&w, code);
        assert!(
            flood < normal,
            "{code} floods by default: flood {flood:+.1} vs normal {normal:+.1}"
        );
        assert!(
            line(&fs, "export revenue") < 0.0,
            "{code}: a price war costs it revenue"
        );
    }
    // And over a campaign: no peaceful exporter floods in the first five years.
    let def = def1980();
    let r = headless::run_campaign(&def, 1, 20).unwrap();
    let floods: Vec<_> = r
        .reasoning
        .iter()
        .filter(|e| e.decision.subject == "energy policy: Flood" && ["SAU", "KWT"].contains(&e.country.as_str()))
        .map(|e| (e.turn, e.country.clone()))
        .collect();
    assert!(floods.is_empty(), "flooded by default: {floods:?}");
}

/// No one left to provoke: once the other exporters are already flooding,
/// the price can't be defended alone, and the same producer joins.
#[test]
fn a_producer_joins_a_price_war_it_cannot_stop_alone() {
    let mut w = cleared(&def1980());
    let sau = id(&w, "SAU");
    let others: Vec<CountryId> = w
        .countries
        .iter()
        .filter(|c| c.id != sau && ai::evaluate::is_exporter(c.energy_capacity, c.energy_net_exports))
        .map(|c| c.id)
        .collect();
    assert!(others.len() >= 3);
    for c in others {
        w.country_mut(c).energy_policy = ProductionPolicy::Flood;
    }
    let [(_, _), (normal, _), (flood, _)] = scores(&w, "SAU");
    assert!(flood > normal, "flood {flood:+.1} vs normal {normal:+.1}");
}

/// A flood is still a weapon: a producer that doesn't live on its oil and
/// faces hostile, oil-dependent exporters floods to squeeze them (data
/// swap: the same producer, the same market, only the quarrels added).
#[test]
fn a_producer_floods_to_squeeze_hostile_exporters_when_the_data_gives_it_rivals() {
    let base = def1980();
    let w = cleared(&base);
    let [(_, _), (normal, _), (flood, _)] = scores(&w, "SOV");
    assert!(
        flood < normal,
        "no quarrel with the Gulf, no squeeze: {flood:+.1} vs {normal:+.1}"
    );
    let mut def = base.clone();
    for rival in ["SAU", "IRQ", "KWT", "IRN"] {
        def.opinions.retain(|o| !(o.from == "SOV" && o.to == rival));
        def.opinions.push(scenario::OpinionDef {
            from: "SOV".into(),
            to: rival.into(),
            value: -80.0,
            decay: 0.0,
            source: "test".into(),
        });
    }
    let w = cleared(&def);
    let [(_, _), (normal, _), (flood, fs)] = scores(&w, "SOV");
    assert!(line(&fs, "squeezing hostile producers and consumers") > 0.0);
    assert!(flood > normal, "squeeze: flood {flood:+.1} vs normal {normal:+.1}");
}

/// Data swap: the same producer with its debt raised to `ratio` × annual GDP.
fn indebted(code: &str, ratio: f64) -> scenario::ScenarioDef {
    let mut def = def1980();
    let c = def.countries.iter_mut().find(|c| c.id == code).unwrap();
    c.debt = ratio * 4.0 * c.gdp;
    def
}

/// Issue 27 (D101 b): quota cheating under money pressure. A producer whose
/// creditors take a large share of its revenue needs the cash now and
/// discounts the others' answer to its flood; the same producer without the
/// debt holds (D100). Before the fix debt played no part: both held.
#[test]
fn a_producer_under_money_pressure_breaks_ranks_at_peace() {
    let w = cleared(&def1980());
    let v = observe(&w, id(&w, "IRN"));
    assert_eq!(ai::evaluate::money_pressure(&v), 0.0);
    let [(_, _), (normal, _), (flood, _)] = scores(&w, "IRN");
    assert!(
        flood < normal,
        "solvent: holds, flood {flood:+.1} vs normal {normal:+.1}"
    );

    let w = cleared(&indebted("IRN", 1.2));
    let irn = id(&w, "IRN");
    assert!(!w.wars.is_belligerent(irn), "at peace");
    let v = observe(&w, irn);
    let pressure = ai::evaluate::money_pressure(&v);
    assert!(pressure > 0.3, "debt service eats its budget: pressure {pressure:.2}");
    let [(_, _), (normal, _), (flood, fs)] = scores(&w, "IRN");
    assert!(
        line(&fs, "we need the cash now (the price war can wait)") > 0.0,
        "the discount is visible in the reasoning"
    );
    assert!(
        flood > normal,
        "indebted: cheats, flood {flood:+.1} vs normal {normal:+.1}"
    );
}

/// No regression of D100: a producer the oil price pays for feels no
/// pressure and still holds. At turn 1 the price is well above base, so the
/// windfall covers even a 1.2 debt's service: this is the windfall case, not
/// immunity to debt (review 27). At the base price the same indebted Kuwait
/// does feel the pressure and breaks ranks; restraint is unaffected.
#[test]
fn a_producer_without_money_pressure_still_holds() {
    for code in ["SAU", "KWT"] {
        for ratio in [0.05, 1.2] {
            let w = cleared(&indebted(code, ratio));
            let v = observe(&w, id(&w, code));
            assert_eq!(ai::evaluate::money_pressure(&v), 0.0, "{code} at debt {ratio}");
            let [(restrain, rs), (normal, _), (flood, _)] = scores(&w, code);
            assert!(
                flood < normal,
                "{code} at debt {ratio}: flood {flood:+.1} vs normal {normal:+.1}"
            );
            assert!(restrain < normal);
            assert_eq!(line(&rs, "we need the cash now (the price war can wait)"), 0.0);
        }
    }
    // Without the windfall the same debt bites: indebted Kuwait at the base
    // price is under pressure and floods.
    let mut w = cleared(&indebted("KWT", 1.2));
    w.energy.price = w.energy.base_price;
    let v = observe(&w, id(&w, "KWT"));
    assert!(ai::evaluate::money_pressure(&v) > 0.5, "indebted KWT at base price");
    let [_, (normal, _), (flood, _)] = scores(&w, "KWT");
    assert!(flood > normal, "indebted KWT at base price: flood {flood:+.1} vs normal {normal:+.1}");
}
