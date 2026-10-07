//! Issue 21: wars are financed by debt (D90). The economic-warfare grudge
//! renewal is held until wars run long enough to make it fire (D83, D90).
//! Generic: the rules read mobilization, belligerency, area, energy and
//! debt state, never identity; the data-swap tests run each rule with the
//! roles exchanged.

use std::path::PathBuf;

use sim_core::country::Mobilization;
use sim_core::economy::war_borrowing_rate;
use sim_core::orders::Order;
use sim_core::war::WarAim;
use sim_core::{resolve_turn, CountryId, OrderSet, WorldState};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world() -> WorldState {
    let def = scenario::load(workspace_root().join("data/fixtures/four_actor.ron")).unwrap();
    scenario::build(&def, Some(1)).unwrap()
}

fn turn(w: &mut WorldState, orders: &[(CountryId, Order)]) {
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
    resolve_turn(w, sets);
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// `attacker` declares war on `defender` and both sit at `level`; returns
/// (debt gained at war, debt gained by the same country in a peaceful twin).
fn war_debt(attacker: &str, defender: &str, level: Mobilization, turns: u32) -> (f64, f64, f64) {
    let mut war = world();
    let mut calm = world();
    let (a, d) = (war.find(attacker).unwrap(), war.find(defender).unwrap());
    for w in [&mut war, &mut calm] {
        w.country_mut(a).forces.mobilization = level;
        w.country_mut(d).forces.mobilization = level;
    }
    turn(
        &mut war,
        &[(
            a,
            Order::DeclareWar {
                target: d,
                aim: WarAim::Limited,
            },
        )],
    );
    turn(&mut calm, &[]);
    assert!(war.wars.enemies_of(a).contains(&d), "the war must be on");
    let (w0, c0) = (war.country(a).debt, calm.country(a).debt);
    let mut expected = 0.0;
    for _ in 0..turns {
        for w in [&mut war, &mut calm] {
            w.country_mut(a).forces.mobilization = level;
            w.country_mut(d).forces.mobilization = level;
        }
        turn(&mut war, &[]);
        turn(&mut calm, &[]);
        let c = war.country(a);
        // Recorded for the "because": this turn's borrowing is GDP x rate.
        assert!(
            c.war_borrowing > 0.0,
            "{attacker}: no borrowing on war turn; at war {}",
            war.wars.is_belligerent(a)
        );
        expected += c.war_borrowing;
    }
    (war.country(a).debt - w0, calm.country(a).debt - c0, expected)
}

#[test]
fn a_mobilized_belligerent_borrows_its_war_and_a_peaceful_twin_does_not() {
    // Data swap: the same rule with the roles exchanged (a smaller high-tax
    // rival attacking a big low-tax democracy, then the reverse).
    for (attacker, defender) in [("RIV", "MAJ"), ("MAJ", "RIV")] {
        let (at_war, at_peace, borrowed) = war_debt(attacker, defender, Mobilization::Full, 4);
        let gdp = world().country(world().find(attacker).unwrap()).gdp;
        // Four quarters of Full war borrow about 4 x 12% of quarterly GDP.
        assert!(
            borrowed > 0.6 * 4.0 * war_borrowing_rate(Mobilization::Full) * gdp,
            "{attacker}: borrowed {borrowed}"
        );
        assert!(
            at_war - at_peace > 0.8 * borrowed,
            "{attacker}: war debt {at_war} vs peace {at_peace} (borrowed {borrowed})"
        );
    }
    // The bill scales with the mobilization level.
    let (_, _, partial) = war_debt("RIV", "MAJ", Mobilization::Partial, 4);
    let (_, _, full) = war_debt("RIV", "MAJ", Mobilization::Full, 4);
    assert!(full > 3.0 * partial, "full {full} vs partial {partial}");
}

#[test]
fn peace_stops_the_borrowing_and_a_war_fought_unmobilized_borrows_nothing() {
    let mut w = world();
    let (riv, neu) = (w.find("RIV").unwrap(), w.find("NEU").unwrap());
    w.country_mut(riv).forces.mobilization = Mobilization::Full;
    turn(
        &mut w,
        &[(
            riv,
            Order::DeclareWar {
                target: neu,
                aim: WarAim::Punitive,
            },
        )],
    );
    let war = w.wars.active[0].id;
    turn(&mut w, &[]);
    assert!(w.country(riv).war_borrowing > 0.0, "borrowing while at war");
    turn(
        &mut w,
        &[(neu, Order::OfferPeace { war }), (riv, Order::OfferPeace { war })],
    );
    assert!(w.wars.active.is_empty(), "peace made");
    // Still mobilized, but at peace: no war bill.
    w.country_mut(riv).forces.mobilization = Mobilization::Full;
    for _ in 0..3 {
        turn(&mut w, &[]);
        assert!(close(w.country(riv).war_borrowing, 0.0));
    }
    assert!(close(war_borrowing_rate(Mobilization::Peacetime), 0.0));
}
