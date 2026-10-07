//! Issue 7 (crisis pass 2, follow-up F1): war weariness vs regime stability.
//!
//! Weariness used to subtract 1:1 from the stability target, so a regime
//! fell below the collapse line (10) at weariness ~15–25, long before the
//! war system's exhaustion level (60) could force peace: Iran, then Iraq,
//! collapsed mid-war, and exhaustion never ended a single war (0 of ~100 in
//! 100 runs). Weariness now spends the regime's margin above collapse in
//! proportion to how close the war is to exhaustion.

use std::path::PathBuf;

use sim_core::transition::COLLAPSE_STABILITY;
use sim_core::war::EXHAUSTION;
use sim_core::WorldState;

fn world_1980() -> WorldState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let def = scenario::load(root.join("data/scenarios/1980.ron")).unwrap();
    scenario::build(&def, Some(1)).unwrap()
}

/// Run the domestic phase for `turns` turns with every country's war
/// weariness held at `weariness`; returns each country's stability per turn.
fn stability_paths(weariness: f64, turns: usize) -> Vec<Vec<f64>> {
    let mut w = world_1980();
    let mut paths = vec![Vec::new(); w.countries.len()];
    for _ in 0..turns {
        for c in &mut w.countries {
            c.war_weariness = weariness;
        }
        sim_core::domestic::run(&mut w);
        for (p, c) in paths.iter_mut().zip(&w.countries) {
            p.push(c.stability);
        }
    }
    paths
}

/// War weariness short of exhaustion never by itself carries a regime past
/// the collapse line: a regime that would stand without the war still
/// stands, worn down, when the war system forces peace. Fails on the old
/// 1:1 rule (every 1980 regime at weariness 59 had a target of ~0).
#[test]
fn weariness_short_of_exhaustion_does_not_collapse_a_standing_regime() {
    let turns = 40;
    let calm = stability_paths(0.0, turns);
    let weary = stability_paths(0.98 * EXHAUSTION, turns);
    let w = world_1980();
    let mut checked = 0;
    for (i, c) in w.countries.iter().enumerate() {
        if !c.active || calm[i].iter().any(|&s| s < COLLAPSE_STABILITY + 1.0) {
            continue;
        }
        checked += 1;
        let low = weary[i].iter().copied().fold(f64::INFINITY, f64::min);
        assert!(
            low >= COLLAPSE_STABILITY,
            "{} would collapse at weariness {:.0} (< exhaustion {EXHAUSTION}): stability fell to {low:.1}",
            c.code,
            0.98 * EXHAUSTION
        );
        // Weariness still bites: near exhaustion the regime has spent almost
        // all of its margin above collapse.
        let (calm_end, weary_end) = (calm[i][turns - 1], weary[i][turns - 1]);
        assert!(
            weary_end - COLLAPSE_STABILITY < 0.25 * (calm_end - COLLAPSE_STABILITY),
            "{}: weariness near exhaustion barely hurt ({weary_end:.1} vs {calm_end:.1} at peace)",
            c.code
        );
    }
    assert!(checked >= 15, "only {checked} regimes checked");
}

/// Half-way to exhaustion costs half the margin: weariness is a real,
/// graded cost, not a cliff at the exhaustion line.
#[test]
fn weariness_cost_is_graded() {
    for base in [20.0, 45.0, 70.0] {
        let half = sim_core::domestic::stability_target(base, 0.5 * EXHAUSTION);
        let expected = base - 0.5 * (base - COLLAPSE_STABILITY);
        assert!((half - expected).abs() < 1e-9, "base {base}: {half} vs {expected}");
        assert_eq!(sim_core::domestic::stability_target(base, 0.0), base);
    }
    // A regime already below the collapse line gets no help from the rule.
    assert_eq!(sim_core::domestic::stability_target(6.0, 30.0), 6.0);
}
