//! The front-end session must resolve exactly as the headless runner does:
//! same seed, same turns → same canonical end state.

use std::path::PathBuf;

use brink_godot::session::Session;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn session_matches_headless_final_state() {
    let scenario = root().join("data/scenarios/1980.ron");
    let lines = root().join("data/voice/lines.ron");
    for seed in [1u64, 7] {
        let turns = 24;
        let mut s = Session::load(&scenario, Some(&lines), seed).unwrap();
        let mut narrated = 0;
        for _ in 0..turns {
            narrated += s.step().narration.len();
        }
        let def = scenario::load(&scenario).unwrap();
        let headless = headless::run_campaign(&def, seed, turns).unwrap();
        assert_eq!(serde_json::to_string(&s.world).unwrap(), headless.final_state, "seed {seed}");
        assert!(narrated > 0, "the session should narrate something in {turns} turns");
    }
}
