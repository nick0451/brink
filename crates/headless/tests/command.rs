//! Issue 20: a regime in crisis, or one whose rulers just took power by
//! revolution or were installed by a conqueror, fights worse
//! (`sim_core::transition::command`; a coup purges no one, D88). Desertion
//! and broken command in a crisis, and the new rulers' purge of the officer
//! corps, cap the readiness the army can reach; the purge's effect recovers
//! over `PURGE_RECOVERY` turns. Others see it only through their own,
//! fog-filtered estimate of the regime's military power.

use std::path::PathBuf;

use sim_core::transition::{COLLAPSE_STABILITY, CRISIS_STABILITY, PURGE_RECOVERY};
use sim_core::{WarAim, WorldState};

fn world_1980() -> WorldState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let def = scenario::load(root.join("data/scenarios/1980.ron")).unwrap();
    let mut w = scenario::build(&def, Some(1)).unwrap();
    // Start from established regimes; each test applies its own swap.
    for c in &mut w.countries {
        c.transition.took_power = None;
    }
    w
}

/// Run the economy phase for `turns` turns with every country's stability
/// held at `stability`; `swap` is applied once before the first turn.
/// Returns the world afterwards.
fn run_economy(stability: f64, turns: u32, swap: impl Fn(&mut WorldState)) -> WorldState {
    let mut w = world_1980();
    swap(&mut w);
    for _ in 0..turns {
        for c in &mut w.countries {
            c.stability = stability;
        }
        sim_core::economy::run(&mut w);
        w.turn += 1;
    }
    w
}

/// Data swap: every regime, held in crisis (Stability 14, well below the
/// crisis line of 35), ends with a lower readiness and less military power than
/// the same state held stable. Fails without the fix (readiness ignored
/// the regime's condition).
#[test]
fn a_regime_in_crisis_fights_worse_than_the_same_state_stable() {
    let stable = run_economy(60.0, 8, |_| {});
    let crisis = run_economy(0.4 * CRISIS_STABILITY, 8, |_| {});
    let mut checked = 0;
    for (s, c) in stable.countries.iter().zip(&crisis.countries) {
        if !s.active {
            continue;
        }
        checked += 1;
        assert!(
            c.forces.readiness < 0.6 * s.forces.readiness,
            "{}: readiness in crisis {:.2} vs stable {:.2}",
            s.code,
            c.forces.readiness,
            s.forces.readiness
        );
        assert!(
            c.forces.power() < 0.9 * s.forces.power(),
            "{}: power in crisis {:.2} vs stable {:.2}",
            s.code,
            c.forces.power(),
            s.forces.power()
        );
    }
    assert!(checked >= 20, "only {checked} countries checked");
}

/// A revolution (a collapse) purges the officer corps: readiness falls at
/// first, then recovers as the new regime rebuilds command. Within
/// `PURGE_RECOVERY` turns plus the readiness lag it is back near the level
/// of a regime that never changed hands. The purge comes from the collapse
/// itself (the transition phase), not from setting the state by hand.
#[test]
fn the_purge_after_a_revolution_recovers() {
    let purge = |w: &mut WorldState| {
        for c in &mut w.countries {
            c.stability = 0.5 * COLLAPSE_STABILITY;
        }
        let mut events = Vec::new();
        sim_core::transition::run(w, &mut events);
        for c in w.countries.iter().filter(|c| c.active) {
            assert_eq!(c.transition.took_power, Some(0), "{}: a collapse did not start a purge", c.code);
        }
    };
    let early = PURGE_RECOVERY as u32 / 3;
    let late = PURGE_RECOVERY as u32 + 24;
    let (stable_early, purged_early) = (run_economy(60.0, early, |_| {}), run_economy(60.0, early, purge));
    let (stable_late, purged_late) = (run_economy(60.0, late, |_| {}), run_economy(60.0, late, purge));
    for i in 0..stable_early.countries.len() {
        let s = &stable_early.countries[i];
        if !s.active {
            continue;
        }
        let (se, pe) = (s.forces.readiness, purged_early.countries[i].forces.readiness);
        let (sl, pl) = (
            stable_late.countries[i].forces.readiness,
            purged_late.countries[i].forces.readiness,
        );
        assert!(pe < 0.7 * se, "{}: purge barely bit ({pe:.2} vs {se:.2})", s.code);
        assert!(
            pl > 0.95 * sl,
            "{}: no recovery after the purge ({pl:.2} vs {sl:.2})",
            s.code
        );
    }
}

/// An attacker's odds against a regime in revolutionary crisis (rulers who
/// just came to power by revolution, Stability in the crisis band) are higher than
/// against the same state stable, and the observer gets there only through
/// its own fog-filtered estimate of the target's power (the same estimate
/// noise in both worlds: same seed, turn and pair).
#[test]
fn odds_against_a_regime_in_revolutionary_crisis_are_higher_through_the_fog() {
    let w0 = world_1980();
    let (me, them) = (w0.find("IRQ").unwrap(), w0.find("IRN").unwrap());
    let stable = run_economy(60.0, 4, |_| {});
    let mut revolution = world_1980();
    revolution.country_mut(them).transition.took_power = Some(0);
    for _ in 0..4 {
        for c in &mut revolution.countries {
            c.stability = if c.id == them { 20.0 } else { 60.0 };
        }
        sim_core::economy::run(&mut revolution);
        revolution.turn += 1;
    }
    // The observer is unchanged between the worlds (up to trade feedback).
    let (a, b) = (stable.country(me).forces.power(), revolution.country(me).forces.power());
    assert!((a - b).abs() < 1e-3 * a, "observer's own power differs: {a} vs {b}");
    let odds = |w: &WorldState| {
        let view = sim_core::view::observe(w, me);
        let target = ai::inputs::foreign(&view, them).unwrap().clone();
        (target.military.value, ai::war::war_odds(&view, &target, WarAim::Major))
    };
    let (est_stable, odds_stable) = odds(&stable);
    let (est_crisis, odds_crisis) = odds(&revolution);
    assert!(
        est_crisis < 0.85 * est_stable,
        "estimated power {est_crisis:.2} in crisis vs {est_stable:.2} stable"
    );
    assert!(
        odds_crisis > odds_stable + 0.03,
        "odds {odds_crisis:.3} against the regime in crisis vs {odds_stable:.3} stable"
    );
}
