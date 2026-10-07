//! Build step 1 acceptance tests (implementation plan §3).

use std::path::PathBuf;

use headless::{run_batch, run_campaign};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> scenario::ScenarioDef {
    scenario::load(workspace_root().join("data/fixtures/four_actor.ron")).expect("fixture loads")
}

#[test]
fn eighty_turns_run_and_respect_invariants() {
    let r = run_campaign(&fixture(), 1980, 80).unwrap();
    assert_eq!(r.samples.len(), 81);
    let state: serde_json::Value = serde_json::from_str(&r.final_state).unwrap();
    assert_eq!(state["turn"], 80);
    for s in &r.samples {
        for c in &s.countries {
            assert!(c.gdp > 0.0 && c.gdp.is_finite(), "{} gdp {}", c.code, c.gdp);
            assert!(
                (0.0..=100.0).contains(&c.stability),
                "{} stability {}",
                c.code,
                c.stability
            );
            assert!(c.debt_ratio >= 0.0 && c.debt_ratio.is_finite());
            assert!(c.military >= 0.0 && c.military.is_finite());
            assert!((0.0..=100.0).contains(&c.security));
            assert!(c.initiative <= 3 + 1 + 2, "initiative above allowance + bank cap");
        }
    }
    for c in state["countries"].as_array().unwrap() {
        let b = &c["budget"];
        let sum: f64 = ["military", "development", "welfare", "intelligence"]
            .iter()
            .map(|k| b[k].as_f64().unwrap())
            .sum();
        assert!((sum - 1.0).abs() < 1e-9, "budget shares sum to {sum}");
    }
}

#[test]
fn same_seed_gives_identical_state() {
    let def = fixture();
    let a = run_campaign(&def, 42, 80).unwrap();
    let b = run_campaign(&def, 42, 80).unwrap();
    assert_eq!(a.final_state, b.final_state);
    assert_eq!(a, b);
}

#[test]
fn different_seeds_diverge() {
    let def = fixture();
    let a = run_campaign(&def, 1, 80).unwrap();
    let b = run_campaign(&def, 2, 80).unwrap();
    assert_ne!(a.final_state, b.final_state);
}

#[test]
fn batch_is_identical_across_thread_counts() {
    let def = fixture();
    let seeds: Vec<u64> = (0..16).collect();
    let one = run_batch(&def, &seeds, 80, 1).unwrap();
    let many = run_batch(&def, &seeds, 80, 8).unwrap();
    assert_eq!(one, many);
}

/// DESIGN §14.1 rule 5: engine and AI code must not branch on country identity.
/// Flags any 2–4 letter all-caps string literal (a country-code shape) in
/// sim-core or ai sources. Test-only code should use neutral lowercase ids.
#[test]
fn no_country_identity_literals_in_engine_or_ai() {
    let mut offenders = Vec::new();
    for crate_dir in ["crates/sim-core/src", "crates/ai/src"] {
        for entry in std::fs::read_dir(workspace_root().join(crate_dir)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for (n, line) in text.lines().enumerate() {
                let bytes = line.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'"' {
                        let start = i + 1;
                        let mut j = start;
                        while j < bytes.len() && bytes[j].is_ascii_uppercase() {
                            j += 1;
                        }
                        let len = j - start;
                        if j < bytes.len() && bytes[j] == b'"' && (2..=4).contains(&len) {
                            offenders.push(format!("{}:{}: {}", path.display(), n + 1, line.trim()));
                        }
                        i = j.max(start);
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "country-code literals found:\n{}",
        offenders.join("\n")
    );
}
