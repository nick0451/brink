//! Issue 22: war length.
//!
//! Wars ended by negotiated white peace after a median 6 turns (gate6 100:
//! 6, p90 8, max 9; 0 of 76 by exhaustion). The defender offered peace
//! first in 67 of 76 wars, at its first assessment: `ai::war::continue_war`
//! gave the invaded side no war aim at all, only "defending our territory",
//! so it was ready for the status quo ante from the outset and peace came
//! as soon as the invader's weariness overtook its own aim. Iran in 1982,
//! with the front carried onto Iraqi soil, refused the ceasefires Iraq
//! offered for six years.

use std::path::PathBuf;

use sim_core::{
    observe, resolve_turn, CountryId, DiplomaticEvent, Order, OrderSet, PeaceReason, TransitionKind, WarAim, WorldState,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world(path: &str) -> WorldState {
    let def = scenario::load(root().join(path)).unwrap();
    scenario::build(&def, Some(3)).unwrap()
}

fn declare(w: &mut WorldState, attacker: CountryId, defender: CountryId) {
    let orders = vec![OrderSet {
        country: attacker,
        orders: vec![Order::DeclareWar {
            target: defender,
            aim: WarAim::Major,
        }],
    }];
    resolve_turn(w, orders);
    assert!(!w.wars.active.is_empty(), "the war started");
}

/// The defender's `continue_war` with the front at `progress` and both
/// leaders at weariness 15: (total, "punishing the aggressor" line, the
/// attacker's "war aim" line at the same moment).
fn defender_view(progress: f64) -> (f64, f64, f64) {
    let mut w = world("data/scenarios/1980.ron");
    let (riv, neu) = (w.find("IRQ").unwrap(), w.find("IRN").unwrap());
    declare(&mut w, riv, neu);
    for f in &mut w.wars.active[0].fronts {
        f.progress = progress;
    }
    w.country_mut(riv).war_weariness = 15.0;
    w.country_mut(neu).war_weariness = 15.0;
    let war = w.wars.active[0].clone();
    let line = |s: &ai::Score, label: &str| {
        s.lines
            .iter()
            .filter(|l| l.label == label)
            .map(|l| l.value)
            .sum::<f64>()
    };
    let d = ai::war::continue_war(&observe(&w, neu), &war);
    let a = ai::war::continue_war(&observe(&w, riv), &war);
    (d.total(), line(&d, "punishing the aggressor"), line(&a, "war aim"))
}

/// A defender that has carried the war onto the aggressor's soil takes up
/// the aggressor's war aim, by the share of the fighting on that soil; on
/// its own soil it fights only to defend it. Fails without the fix (no such
/// term: Iran at -20 scored -14.2, below its -4.1 at the border).
#[test]
fn a_defender_holding_the_aggressors_ground_fights_to_punish_it() {
    let (home, home_punish, _) = defender_view(10.0);
    let (border, border_punish, _) = defender_view(0.0);
    let (deep, deep_punish, aim) = defender_view(-20.0);
    println!("defender total: own soil {home:.1}, border {border:.1}, aggressor's soil {deep:.1}; punish {home_punish:.1}/{border_punish:.1}/{deep_punish:.1}; attacker aim {aim:.1}");
    assert_eq!(home_punish, 0.0, "no punishment aim while the war is on our own soil");
    assert_eq!(border_punish, 0.0, "nor at the border");
    // home_share(attacker) at -20 = 20 / WIN_MARGIN (25) = 0.8 of the aim
    // (the aim scales by each leader's own aggression, as the attacker's).
    assert!(deep_punish > 0.0, "the aim appears once the tide has turned");
    // Without the aim, carrying the war over the border made a white peace
    // more attractive (-14.2 against -4.1 at the border: home ground and
    // the defence term are lost); now it hardens the defender's terms.
    assert!(
        deep > border,
        "carrying the war over the border hardens the defender's terms ({deep:.1} vs {border:.1})"
    );
}

/// When neither side offers peace, the war ends by exhaustion, and before
/// either leader's regime collapses (D71: weariness spends the regime's
/// margin, reaching the collapse line only at EXHAUSTION).
#[test]
fn a_war_both_sides_fight_on_ends_by_exhaustion_before_a_regime_falls() {
    let mut w = world("data/scenarios/1980.ron");
    let (irq, irn) = (w.find("IRQ").unwrap(), w.find("IRN").unwrap());
    declare(&mut w, irq, irn);
    let mut ended = None;
    let mut collapsed = Vec::new();
    for _ in 0..120 {
        let report = resolve_turn(&mut w, Vec::new());
        for e in &report.events {
            match e {
                DiplomaticEvent::PeaceMade { reason, .. } => ended = Some((*reason, w.turn)),
                DiplomaticEvent::Transition {
                    country,
                    kind: TransitionKind::Collapse,
                } if ended.is_none() => collapsed.push(*country),
                _ => {}
            }
        }
        if ended.is_some() {
            break;
        }
    }
    println!(
        "ended {ended:?}; weariness IRQ {:.1} IRN {:.1}; collapses {collapsed:?}",
        w.country(irq).war_weariness,
        w.country(irn).war_weariness
    );
    assert!(collapsed.is_empty(), "no regime falls before the war ends");
    assert!(
        matches!(ended, Some((PeaceReason::Exhaustion | PeaceReason::Decisive, _))),
        "the war ends by exhaustion (or decision), not never: {ended:?}"
    );
}
