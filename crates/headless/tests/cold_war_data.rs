//! Issue 10 (D72): Cold War hostility as scenario data. The 1980 file gives
//! the USSR and the Western alliance states on its perimeter a documented
//! quarrel each (NATO's central front, the Northern Territories, the
//! Straits, the divided peninsula). Everything the AI does with it (threat,
//! joining sanctions, oil policy, arms grants) follows the data, never the
//! country's name: each test swaps the data and watches the behaviour move.

use std::path::PathBuf;

use sim_core::commodity::ProductionPolicy;
use sim_core::{observe, CountryId, DiplomaticEvent, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn def1980() -> scenario::ScenarioDef {
    scenario::load(root().join("data/scenarios/1980.ron")).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

/// Drop every starting opinion and tension between `a` and `b`.
fn erase_pair(def: &mut scenario::ScenarioDef, a: &str, b: &str) {
    def.opinions
        .retain(|o| !((o.from == a && o.to == b) || (o.from == b && o.to == a)));
    def.tensions
        .retain(|t| !((t.a == a && t.b == b) || (t.a == b && t.b == a)));
}

fn threat_of(w: &WorldState, observer: &str, target: &str) -> f64 {
    let v = observe(w, id(w, observer));
    let f = v.others.iter().find(|f| f.code == target).unwrap();
    ai::inputs::threat(&v, f)
}

#[test]
fn west_germany_and_japan_perceive_a_real_soviet_threat_in_1980_because_the_data_says_so() {
    let w = scenario::build(&def1980(), Some(1)).unwrap();
    for obs in ["FRG", "JPN", "GBR", "TUR"] {
        let t = threat_of(&w, obs, "SOV");
        println!("{obs} threat from SOV: {t:.1}");
        assert!(t >= 15.0, "{obs} reads the USSR as a real threat (got {t:.1})");
    }
    // Data swap: without the pair's opinions and tension the same code reads
    // the same army as no threat at all (D69's finding, now a guard).
    let mut def = def1980();
    for obs in ["FRG", "JPN"] {
        erase_pair(&mut def, "SOV", obs);
    }
    let w = scenario::build(&def, Some(1)).unwrap();
    for obs in ["FRG", "JPN"] {
        let t = threat_of(&w, obs, "SOV");
        assert!(t < 5.0, "{obs} without the data: {t:.1}");
    }
    // Untouched pairs keep theirs.
    assert!(threat_of(&w, "GBR", "SOV") >= 15.0);
}

#[test]
fn europe_joins_the_measures_against_a_shared_adversary_but_not_against_cuba() {
    let def = def1980();
    let w = scenario::build(&def, Some(1)).unwrap();
    let usa = id(&w, "USA");
    let score = |obs: &str, target: &str| {
        let v = observe(&w, id(&w, obs));
        let s = ai::evaluate::join_sanction(
            &v,
            &sim_core::Sanction {
                by: usa,
                target: id(&w, target),
                since: 0,
            },
        );
        println!("{obs} joining the US sanctions on {target}: {:+.1}", s.total());
        for l in s.sorted() {
            println!("    {:+6.1}  {}", l.value, l.label);
        }
        s.total()
    };
    // The 1980 measures after Afghanistan: Britain and Japan fall in; nobody
    // in Western Europe embargoes Cuba (its quarrel is Washington's alone).
    for obs in ["GBR", "JPN"] {
        assert!(score(obs, "SOV") > 0.0, "{obs} shares the quarrel with Moscow");
    }
    for obs in ["FRA", "GBR", "FRG"] {
        assert!(score(obs, "CUB") < 0.0, "{obs} has no quarrel with Havana");
        assert!(score(obs, "SOV") > score(obs, "CUB"));
    }
    // Data swap: erase Britain's quarrel with Moscow and it stays out too.
    let mut swapped = def1980();
    erase_pair(&mut swapped, "SOV", "GBR");
    let w2 = scenario::build(&swapped, Some(1)).unwrap();
    let v = observe(&w2, id(&w2, "GBR"));
    let s = ai::evaluate::join_sanction(
        &v,
        &sim_core::Sanction {
            by: id(&w2, "USA"),
            target: id(&w2, "SOV"),
            since: 0,
        },
    );
    assert!(s.total() < 0.0, "no data, no join: {:+.1}", s.total());

    // And in play (8 turns, seed 1): the joins happen, the Cuba embargo
    // stays American (Japan's provisional-solidarity reflex is data, item f).
    let r = headless::run_campaign(&def, 1, 8).unwrap();
    let w = scenario::build(&def, Some(1)).unwrap();
    let sanctioned = |by: &str, target: &str| {
        r.events.iter().any(|(_, e)| {
            matches!(e, DiplomaticEvent::SanctionImposed { by: b, target: t } if *b == id(&w, by) && *t == id(&w, target))
        })
    };
    assert!(sanctioned("GBR", "SOV") && sanctioned("JPN", "SOV"));
    for obs in ["FRA", "GBR", "FRG"] {
        assert!(!sanctioned(obs, "CUB"), "{obs} embargoes Cuba");
    }
}

/// Oil hunk (issue 10 item 4): export revenue is earned on net exports, not
/// on gross output. A producer that burns most of its own oil loses half its
/// exports to a 15% cut, so the squeeze on hostile importers is an option it
/// pays dearly for, not a standing policy; a rentier barely notices.
#[test]
fn restraint_costs_a_self_consuming_producer_its_exports_so_the_squeeze_is_not_standing_policy() {
    // The market clears on the first turn; score policies after it.
    let cleared = |def: &scenario::ScenarioDef| {
        let mut w = scenario::build(def, Some(1)).unwrap();
        sim_core::resolve_turn(&mut w, vec![]);
        w
    };
    let def = def1980();
    let w = cleared(&def);
    let scores = |w: &WorldState, code: &str| {
        let v = observe(w, id(w, code));
        let r = ai::evaluate::energy_policy(&v, ProductionPolicy::Restrain);
        let n = ai::evaluate::energy_policy(&v, ProductionPolicy::Normal);
        let line = |label: &str| r.lines.iter().find(|l| l.label == label).map_or(0.0, |l| l.value);
        (
            r.total(),
            n.total(),
            line("export revenue"),
            line("squeezing hostile producers and consumers"),
        )
    };
    let (restrain, normal, revenue, squeeze) = scores(&w, "SOV");
    println!("1980 USSR: restrain {restrain:+.2} normal {normal:+.2} (revenue {revenue:+.2}, squeeze {squeeze:+.2})");
    assert!(squeeze > 0.0, "the hostile West would be squeezed by a higher price");
    assert!(revenue < -3.0, "but half the exports go with a 15% cut: {revenue:+.2}");
    assert!(restrain < normal, "no standing squeeze");
    let (_, _, sau_revenue, _) = scores(&w, "SAU");
    let sov = w.country(id(&w, "SOV"));
    let sau = w.country(id(&w, "SAU"));
    // Per unit of dependence, the rentier's cut is far cheaper than the
    // self-consumer's (its exports fall 15%, not 55%).
    let per_dep = |rev: f64, c: &sim_core::Country| rev / (c.energy_net_exports / c.gdp).clamp(0.0, 3.0);
    assert!(
        per_dep(sau_revenue, sau) > per_dep(revenue, sov),
        "SAU {sau_revenue:+.2} vs SOV {revenue:+.2} per unit of dependence"
    );
    // Data swap: give the USSR a rentier's capacity and the same cut costs
    // it little (exports dominate output), with the same code.
    let mut rentier = def1980();
    for c in &mut rentier.countries {
        if c.id == "SOV" {
            c.energy_capacity *= 3.0;
        }
    }
    let w2 = cleared(&rentier);
    let (_, _, revenue2, _) = scores(&w2, "SOV");
    assert!(revenue2 > revenue + 2.0, "as a rentier: {revenue:+.2} -> {revenue2:+.2}");
}

/// Arms-grant hunk (issue 10 item 3): a patron's grants go where the client
/// cannot pay; a rich ally buys. The line scales with the client's economy
/// against the patron's, so Japan's data makes it a buyer, not a ward.
#[test]
fn patrons_do_not_gift_arms_to_allies_who_can_pay() {
    let def = def1980();
    let w = scenario::build(&def, Some(1)).unwrap();
    let pay_line = |w: &WorldState, patron: &str, client: &str| {
        let v = observe(w, id(w, patron));
        let f = v.others.iter().find(|f| f.code == client).unwrap();
        let s = ai::evaluate::arm_client(&v, f, None);
        s.lines
            .iter()
            .find(|l| l.label == "they can pay for their own")
            .map_or(0.0, |l| l.value)
    };
    let jpn = pay_line(&w, "USA", "JPN");
    let tur = pay_line(&w, "USA", "TUR");
    println!("USA arming: JPN {jpn:+.1}, TUR {tur:+.1}");
    assert!(jpn < -5.0, "Japan can pay for its own: {jpn:+.1}");
    assert!(tur > -2.0 && tur <= 0.0, "Turkey cannot: {tur:+.1}");
    // Data swap: a poor Japan would get grants like anyone else.
    let mut poor = def1980();
    for c in &mut poor.countries {
        if c.id == "JPN" {
            c.gdp = 2.0;
        }
    }
    let w2 = scenario::build(&poor, Some(1)).unwrap();
    let jpn2 = pay_line(&w2, "USA", "JPN");
    assert!(jpn2 > -2.0, "a poor Japan: {jpn2:+.1}");
}
