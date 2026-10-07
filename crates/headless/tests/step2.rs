//! Build step 2 acceptance tests: fog / observer state (implementation plan §3).

use std::path::PathBuf;

use sim_core::{intel, observe, resolve_turn, OrderSet};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_world(seed: u64) -> sim_core::WorldState {
    let def = scenario::load(workspace_root().join("data/fixtures/four_actor.ron")).unwrap();
    scenario::build(&def, Some(seed)).unwrap()
}

fn idle_orders(state: &sim_core::WorldState) -> Vec<OrderSet> {
    state
        .ids()
        .map(|id| OrderSet {
            country: id,
            orders: Vec::new(),
        })
        .collect()
}

/// Mean relative error of military estimates, per observer→target pair,
/// over an 80-turn campaign, paired with the mean coverage.
fn pair_errors(seed: u64) -> Vec<(f64, f64)> {
    let mut state = fixture_world(seed);
    let n = state.countries.len();
    let mut sums = vec![(0.0, 0.0); n * n];
    let turns = 80;
    for _ in 0..turns {
        for observer in state.ids().collect::<Vec<_>>() {
            for f in observe(&state, observer).others {
                let truth = state.country(f.id).power();
                let cell = &mut sums[observer.index() * n + f.id.index()];
                cell.0 += f.coverage;
                cell.1 += (f.military.value / truth - 1.0).abs();
            }
        }
        let orders = idle_orders(&state);
        resolve_turn(&mut state, orders);
    }
    sums.into_iter()
        .filter(|(cov, _)| *cov > 0.0)
        .map(|(cov, err)| (cov / turns as f64, err / turns as f64))
        .collect()
}

#[test]
fn estimate_error_shrinks_with_coverage() {
    let mut pairs: Vec<(f64, f64)> = (0..10).flat_map(pair_errors).collect();
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    let third = pairs.len() / 3;
    let mean = |s: &[(f64, f64)]| s.iter().map(|p| p.1).sum::<f64>() / s.len() as f64;
    let low = mean(&pairs[..third]);
    let high = mean(&pairs[pairs.len() - third..]);
    assert!(
        high < low * 0.75,
        "high-coverage error {high:.3} should be well below low-coverage error {low:.3}"
    );
}

#[test]
fn estimates_bracket_truth_and_tiers_hide_detail() {
    let mut state = fixture_world(5);
    let mut saw_hidden = false;
    let mut saw_budget = false;
    for _ in 0..40 {
        for observer in state.ids().collect::<Vec<_>>() {
            for f in observe(&state, observer).others {
                let target = state.country(f.id);
                assert!(f.military.low <= target.power() && target.power() <= f.military.high);
                assert_eq!(f.stability_band.is_some(), f.coverage >= intel::TIER_DOMESTIC);
                assert_eq!(f.debt_ratio.is_some(), f.coverage >= intel::TIER_DOMESTIC);
                assert_eq!(f.budget.is_some(), f.coverage >= intel::TIER_BUDGET);
                saw_hidden |= f.budget.is_none();
                saw_budget |= f.budget.is_some();
            }
        }
        let orders = idle_orders(&state);
        resolve_turn(&mut state, orders);
    }
    assert!(
        saw_hidden && saw_budget,
        "fixture should exercise both visibility tiers"
    );
}

#[test]
fn observing_does_not_change_the_simulation() {
    let mut watched = fixture_world(9);
    let mut unwatched = fixture_world(9);
    for _ in 0..20 {
        for id in watched.ids().collect::<Vec<_>>() {
            let _ = observe(&watched, id);
        }
        let o = idle_orders(&watched);
        resolve_turn(&mut watched, o);
        let o = idle_orders(&unwatched);
        resolve_turn(&mut unwatched, o);
    }
    assert_eq!(
        serde_json::to_string(&watched).unwrap(),
        serde_json::to_string(&unwatched).unwrap()
    );
}

/// The AI crate must consume only observer-visible state (DESIGN §14.1).
#[test]
fn ai_crate_never_references_canonical_state() {
    let dir = workspace_root().join("crates/ai/src");
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        for forbidden in ["WorldState", "resolve_turn"] {
            for (n, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                assert!(
                    !code.contains(forbidden),
                    "{}:{} references {forbidden}",
                    path.display(),
                    n + 1
                );
            }
        }
    }
}
