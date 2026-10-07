//! Issue 25 (D97): harmful hostile acts deepen a grudge.
//!
//! An act that harms its victim every turn it continues (oil flooding
//! against an indebted exporting neighbour; a war debt held against a
//! debtor in distress) deepens the victim's grudge toward the actor a step
//! per turn, creating one where none existed, down to a cap. Once the
//! campaign stops the grudge fades by the D81 rules. One-off acts that
//! carry their own opinion hit (sanction, denunciation, war) only renew.
//! Generic: the data-swap tests run each rule with the roles exchanged.

use std::path::PathBuf;

use sim_core::grudge::{DEEPEN_CAP, DEEPEN_PER_TURN, FADE_PER_YEAR, RENEWAL_TURNS, TURNS_PER_YEAR};
use sim_core::opinion::HostileAct;
use sim_core::orders::Order;
use sim_core::{resolve_turn, CountryId, OrderSet, ProductionPolicy, WorldState};

fn world() -> WorldState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let def = scenario::load(root.join("data/fixtures/four_actor.ron")).unwrap();
    scenario::build(&def, Some(1)).unwrap()
}

const STEP: f64 = FADE_PER_YEAR / TURNS_PER_YEAR as f64;

fn turn(w: &mut WorldState, orders: &[(CountryId, Order)]) -> sim_core::TurnReport {
    let sets = w
        .ids()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|id| OrderSet {
            country: id,
            orders: orders
                .iter()
                .filter(|(c, _)| *c == id)
                .map(|(_, o)| o.clone())
                .collect(),
        })
        .collect();
    resolve_turn(w, sets)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// `flooder` floods the market against `victim`, an indebted net exporter
/// in the same area; capacities as multiples of each one's own GDP. A glut
/// (flooder 2x, victim 3x) sinks the price below its base; small producers
/// (1x and 1.1x) leave the world short and the price above base.
fn flooding(flooder: &str, victim: &str, glut: bool) -> (WorldState, CountryId, CountryId) {
    let mut w = world();
    let (f, v) = (w.find(flooder).unwrap(), w.find(victim).unwrap());
    let (cf, cv) = if glut { (2.0, 3.0) } else { (1.0, 1.1) };
    let gdp_f = w.country(f).gdp;
    let x = w.country_mut(f);
    x.area = Some("gulf".into());
    x.energy_capacity = cf * gdp_f;
    x.energy_policy = ProductionPolicy::Flood;
    let x = w.country_mut(v);
    let gdp_v = x.gdp;
    x.area = Some("gulf".into());
    x.energy_capacity = cv * gdp_v;
    x.debt = 4.0 * gdp_v;
    (w, f, v)
}

/// A sustained flood into a depressed price deepens the victim's grudge
/// every turn (by at most a full step) down to the cap, whether or not the
/// pair had one (MAJ-ALY start with none: one is created); the flooder's
/// own grudge is not deepened. Fails without the fix (a renewal only
/// paused the fade; no grudge was ever created).
#[test]
fn a_sustained_harmful_campaign_deepens_the_victims_grudge_to_the_cap() {
    for (flooder, victim) in [("RIV", "MAJ"), ("MAJ", "RIV"), ("ALY", "MAJ"), ("MAJ", "ALY")] {
        let (mut w, f, v) = flooding(flooder, victim, true);
        let (mut prev, back) = (w.opinions.memory(v, f), w.opinions.memory(f, v));
        let mut turns = 0;
        while turns < 60 {
            turn(&mut w, &[]);
            turns += 1;
            let now = w.opinions.memory(v, f);
            assert!(
                now <= prev + 1e-9 && now >= prev - DEEPEN_PER_TURN - 1e-9,
                "{victim}: {prev} -> {now}"
            );
            prev = now;
        }
        assert!(w.energy.deviation() < 0.0, "fixture: a glut");
        assert!(close(prev, DEEPEN_CAP), "{victim}->{flooder} at the cap: {prev}");
        assert!(w.opinions.memory(f, v) >= back, "the flooder's grudge is not deepened");
        let m = w.opinions.modifiers(v, f).iter().find(|m| m.memory.is_some()).unwrap();
        let mem = m.memory.unwrap();
        assert_eq!(mem.deepened, Some(HostileAct::OilFlood));
        println!("{victim}->{flooder}: {m}");
        assert!(m.to_string().contains("deepened by OilFlood"));
    }
}

/// A victim on Flood itself resents nobody for flooding (D97 review).
#[test]
fn a_flooding_victim_does_not_resent_other_flooders() {
    for (flooder, victim) in [("RIV", "MAJ"), ("ALY", "MAJ"), ("MAJ", "ALY")] {
        let (mut w, f, v) = flooding(flooder, victim, true);
        w.country_mut(v).energy_policy = ProductionPolicy::Flood;
        let start = w.opinions.memory(v, f);
        for _ in 0..12 {
            turn(&mut w, &[]);
        }
        assert!(w.energy.deviation() < 0.0, "fixture: a glut");
        assert!(
            w.opinions.memory(v, f) >= start - 1e-9,
            "{victim}->{flooder} not deepened"
        );
    }
}

/// A flood into a high price harms nobody: no deepening, no renewal (the
/// historical grudge fades as usual).
#[test]
fn a_flood_into_a_high_price_deepens_nothing() {
    for (flooder, victim) in [("RIV", "MAJ"), ("MAJ", "RIV"), ("ALY", "MAJ")] {
        let (mut w, f, v) = flooding(flooder, victim, false);
        let start = w.opinions.memory(v, f);
        for _ in 0..12 {
            turn(&mut w, &[]);
            assert!(w.energy.deviation() >= 0.0, "fixture: price above base");
        }
        let after = w.opinions.memory(v, f);
        assert!(after >= start - 1e-9, "{victim}->{flooder}: {start} -> {after}");
        assert!(w
            .opinions
            .modifiers(v, f)
            .iter()
            .filter_map(|m| m.memory)
            .all(|m| m.deepened_turns == 0));
    }
}

/// Once the flood stops, the deepened grudge is paused for the renewal
/// window and then fades at the D81 rate (MAJ-ALY: same bloc, floor 0).
#[test]
fn the_deepened_grudge_fades_once_the_campaign_stops() {
    let (mut w, f, v) = flooding("ALY", "MAJ", true);
    for _ in 0..60 {
        turn(&mut w, &[]);
    }
    assert!(close(w.opinions.memory(v, f), DEEPEN_CAP));
    // The order lands this turn: no flood, so no deepening; the last
    // renewal was the turn before.
    turn(&mut w, &[(f, Order::SetEnergyPolicy(ProductionPolicy::Normal))]);
    assert!(
        close(w.opinions.memory(v, f), DEEPEN_CAP),
        "stopped: no further deepening"
    );
    // Paused for the rest of the renewal window, then 1.5/yr.
    for _ in 1..RENEWAL_TURNS - 1 {
        turn(&mut w, &[]);
    }
    assert!(
        close(w.opinions.memory(v, f), DEEPEN_CAP),
        "paused for the renewal window"
    );
    for k in 1..=8 {
        turn(&mut w, &[]);
        assert!(
            close(w.opinions.memory(v, f), DEEPEN_CAP + k as f64 * STEP),
            "fading, step {k}"
        );
    }
}

/// A held war debt deepens the debtor's grudge every turn the hold stands
/// and the debtor stays in distress (creating one between allies), and
/// replaces D96's one-off -10 opinion. Forgiving ends it.
#[test]
fn a_held_debt_deepens_while_it_stands() {
    for (creditor, debtor) in [("ALY", "MAJ"), ("MAJ", "ALY")] {
        let mut w = world();
        let (c, d) = (w.find(creditor).unwrap(), w.find(debtor).unwrap());
        let gdp = w.country(d).gdp;
        w.country_mut(d).debt = 0.8 * 4.0 * gdp;
        w.diplomacy.lend(c, d, 1.0, 0);
        let opinion = w.opinions.opinion(d, c);
        turn(&mut w, &[]);
        assert!(close(w.opinions.memory(d, c), 0.0), "an unheld claim deepens nothing");
        let r = turn(&mut w, &[(c, Order::HoldDebt { debtor: d })]);
        assert!(r.rejected.is_empty(), "{:?}", r.rejected);
        for _ in 0..3 {
            turn(&mut w, &[]);
        }
        assert!(w.opinions.memory(d, c) < -1.0, "the hold deepened a grudge");
        assert!(
            close(w.opinions.memory(d, c), -4.0 * DEEPEN_PER_TURN),
            "{}",
            w.opinions.memory(d, c)
        );
        assert!(
            close(w.opinions.opinion(d, c) - opinion, -4.0 * DEEPEN_PER_TURN),
            "no extra one-off hit"
        );
        // Out of distress, the hold lapses: no deepening, even back in
        // distress, until it is held again.
        let mut out = w.clone();
        out.country_mut(d).debt = 0.0;
        turn(&mut out, &[]);
        assert!(out.diplomacy.loans.iter().all(|l| !l.held), "hold lapsed");
        out.country_mut(d).debt = 0.8 * 4.0 * gdp;
        let m = out.opinions.memory(d, c);
        turn(&mut out, &[]);
        assert!(close(out.opinions.memory(d, c), m), "a lapsed hold deepens nothing");
        turn(&mut w, &[(c, Order::ForgiveDebt { debtor: d })]);
        let m = w.opinions.memory(d, c);
        turn(&mut w, &[]);
        assert!(
            close(w.opinions.memory(d, c), m),
            "forgiven: no further deepening (paused)"
        );
    }
}

/// One-off acts with their own opinion hit renew a grudge but never deepen
/// it, and create none where none existed: no double counting.
#[test]
fn one_off_acts_with_their_own_opinion_hit_do_not_deepen() {
    let mut w = world();
    let (maj, aly, riv, neu) = (
        w.find("MAJ").unwrap(),
        w.find("ALY").unwrap(),
        w.find("RIV").unwrap(),
        w.find("NEU").unwrap(),
    );
    let (mr, nr) = (w.opinions.memory(riv, maj), w.opinions.memory(neu, aly));
    assert!(
        mr < 0.0 && close(nr, 0.0),
        "fixture: RIV holds a grudge, NEU none against ALY"
    );
    turn(
        &mut w,
        &[
            (maj, Order::Sanction { target: riv }),
            (aly, Order::Denounce { target: neu }),
        ],
    );
    assert!(
        close(w.opinions.memory(riv, maj), mr),
        "sanction renews (paused), no deepening"
    );
    assert!(
        w.opinions.modifiers(neu, aly).iter().all(|m| m.memory.is_none()),
        "no grudge created"
    );
    let renewed = w.opinions.modifiers(riv, maj).iter().find_map(|m| m.memory).unwrap();
    assert_eq!(renewed.renewed.map(|(_, a)| a), Some(HostileAct::Sanction));
    assert_eq!(renewed.deepened_turns, 0);
}
