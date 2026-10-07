//! Protection instrumentation and protector reach (issue 4).

use std::path::PathBuf;

use sim_core::observe;

fn world() -> sim_core::WorldState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    scenario::build(&scenario::load(root.join("data/scenarios/1980.ron")).unwrap(), Some(1)).unwrap()
}

/// The sampled protectors are exactly the simulation's defenders.
#[test]
fn sampled_protectors_match_the_security_driver() {
    let w = world();
    let s = headless::sample(&w);
    for c in &s.countries {
        let id = w.find(&c.code).unwrap();
        let expected: Vec<String> = sim_core::domestic::defenders(&w, id)
            .into_iter()
            .map(|d| w.country(d).code.clone())
            .collect();
        assert_eq!(c.protectors, expected, "{}", c.code);
    }
    assert!(
        s.countries.iter().any(|c| !c.protectors.is_empty()),
        "1980 starts with protected states"
    );
}

/// A would-be attacker discounts a distant protector by its reach into the
/// target's area, as the Security driver does; a local one counts in full.
#[test]
fn a_distant_protector_counts_for_its_reach_only() {
    let w = world();
    let view = observe(&w, w.find("IRQ").unwrap());
    let reaches: Vec<f64> = view
        .others
        .iter()
        .filter(|f| f.area.is_some())
        .map(|f| {
            assert_eq!(ai::inputs::reach_into(f, &f.area), 1.0, "{} at home", f.code);
            ai::inputs::reach_into(f, &Some("__elsewhere__".into()))
        })
        .collect();
    assert!(reaches
        .iter()
        .all(|r| (sim_core::domestic::REACH_FLOOR..=1.0).contains(r)));
    assert!(
        reaches.iter().any(|&r| r < 1.0),
        "some powers can't project fully: {reaches:?}"
    );
}
