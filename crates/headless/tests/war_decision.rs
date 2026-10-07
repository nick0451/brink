//! The war decision (crisis pass 4, issue 14): victory-contingent gains are
//! worth their odds, a raid succeeds by getting through, a sanctioner weighs
//! what the target could bring against its whole defence, and a war can
//! remove armies but not an arsenal.

use std::path::PathBuf;

use ai::{evaluate, Score};
use sim_core::{observe, CountryId, Sanction, WarAim, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world(seed: u64) -> WorldState {
    scenario::build(
        &scenario::load(root().join("data/scenarios/1980.ron")).unwrap(),
        Some(seed),
    )
    .unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

fn term(s: &Score, label: &str) -> f64 {
    s.lines.iter().find(|l| l.label == label).map_or(0.0, |l| l.value)
}

/// The win (or, for a raid, get-through) chance behind a war score.
fn odds(s: &Score) -> f64 {
    term(s, "odds of success") / 40.0 + 0.5
}

fn war(w: &WorldState, from: &str, to: &str, aim: WarAim) -> Score {
    let view = observe(w, id(w, from));
    let target = ai::inputs::foreign(&view, id(w, to)).unwrap();
    ai::war::declare_war(&view, target, aim)
}

/// The Neutral under threat from the Rival (the Gate 3 crisis fixture),
/// each with a full-weight claim on the other.
fn claims_def() -> scenario::ScenarioDef {
    let mut def = scenario::load(root().join("data/fixtures/four_actor.ron")).unwrap();
    def.opinions.retain(|o| !(o.from == "NEU" && o.to == "RIV"));
    for (from, to, value) in [("RIV", "NEU", -60.0), ("NEU", "RIV", -50.0)] {
        def.opinions.push(scenario::OpinionDef {
            from: from.into(),
            to: to.into(),
            value,
            decay: 0.0,
            source: "border dispute".into(),
        });
    }
    def.tensions.push(scenario::TensionDef {
        a: "RIV".into(),
        b: "NEU".into(),
        value: 70.0,
    });
    for (by, against) in [("NEU", "RIV"), ("RIV", "NEU")] {
        def.claims.push(scenario::ClaimDef {
            by: by.into(),
            against: against.into(),
            weight: 1.0,
        });
    }
    def
}

/// (1) A claim is settled, a threat removed, only if the war is won (sim-core
/// settles a claim on AttackerWon alone), so every victory-contingent gain,
/// the claim included, is worth its odds: a weak claimant does not read
/// "our claim" as "attack now", a strong one keeps nearly all of it.
#[test]
fn victory_gains_are_worth_their_odds_including_the_claim() {
    let w = scenario::build(&claims_def(), Some(5)).unwrap();
    for aim in [WarAim::Limited, WarAim::Major] {
        let weak = war(&w, "NEU", "RIV", aim);
        let p = odds(&weak);
        assert!(p < 0.3, "a minnow attacking a giant: odds {p:.2}");
        let claim = term(&weak, "pressing our claim");
        println!("{aim:?}: NEU claim {claim:.1} at odds {p:.2}");
        assert!((claim - ai::war::CLAIM_WAR_VALUE * p).abs() < 1e-9);
        let view = observe(&w, id(&w, "NEU"));
        let riv = ai::inputs::foreign(&view, id(&w, "RIV")).unwrap();
        let scale = if aim == WarAim::Major { 1.5 } else { 1.0 };
        let threat = 0.3 * ai::inputs::conventional_threat(&view, riv) * scale;
        assert!(threat > 0.0);
        assert!((term(&weak, "removing a threat") - threat * p).abs() < 1e-9);

        let strong = war(&w, "RIV", "NEU", aim);
        let q = odds(&strong);
        assert!(q > 0.8, "a giant attacking a minnow: odds {q:.2}");
        assert!((term(&strong, "pressing our claim") - ai::war::CLAIM_WAR_VALUE * q).abs() < 1e-9);
    }
}

// (2) KNOWN GAP (review 14): the Punitive odds hunk ("a raid succeeds by
// getting through") was dropped at integration. Its stand-off factor is a
// loss-rate ratio the simulation never applies to a raid; in the sim the
// programme setback is certain and the target's ground-force ratio decides
// escalation. The follow-up: do not weight "stop their weapons programme"
// by odds, and cost the escalation into a Limited war (STATE.md D78).

/// (3) A sanctioner weighs what the target could bring against its whole
/// defence: a minnow beside a giant with no protector risks nearly all of
/// the retaliation weight, a protected neighbour of a superpower less, a
/// distant great power almost nothing. Kuwait did not embargo Iraq.
#[test]
fn a_minnow_beside_a_giant_risks_everything_by_sanctioning_it() {
    let w = world(1);
    let risk = |by: &str, target: &str| {
        let view = observe(&w, id(&w, by));
        let t = ai::inputs::foreign(&view, id(&w, target)).unwrap();
        let imposed = term(&evaluate::impose_sanction(&view, t, false), "retaliation risk");
        let joined = term(
            &evaluate::join_sanction(
                &view,
                &Sanction {
                    by: id(&w, "ISR"),
                    target: id(&w, target),
                    since: 0,
                },
            ),
            "retaliation risk",
        );
        assert!((imposed - joined).abs() < 1e-9);
        imposed
    };
    let kuwait = risk("KWT", "IRQ");
    let germany = risk("FRG", "SOV");
    let america = risk("USA", "IRQ");
    println!("retaliation risk: KWT on IRQ {kuwait:.1}, FRG on SOV {germany:.1}, USA on IRQ {america:.1}");
    assert!(kuwait < -0.75 * evaluate::RETALIATION, "{kuwait:.1}");
    assert!(germany > kuwait);
    assert!(america > -0.1 * evaluate::RETALIATION, "{america:.1}");
}

/// (5) A war removes armies, not an arsenal (D74; seed 2148 t72: Iraq
/// declared a Major war on Iran three turns after Iran's arsenal appeared):
/// the arsenal raises the threat the target poses, but the gain from
/// "removing a threat" is its conventional part only; the arsenal's weight
/// in the decision is the catastrophic-risk term.
#[test]
fn an_arsenal_is_not_a_removable_threat() {
    let mut w = world(1);
    let before = war(&w, "IRQ", "IRN", WarAim::Major);
    let irn = id(&w, "IRN");
    let threat_before = {
        let view = observe(&w, id(&w, "IRQ"));
        ai::inputs::military_threat(&view, ai::inputs::foreign(&view, irn).unwrap())
    };
    let c = w.country_mut(irn);
    c.arsenal = 1;
    c.arsenal_declared = true;
    c.strike_capacity = sim_core::nuclear::full_capacity(1);
    let after = war(&w, "IRQ", "IRN", WarAim::Major);
    let threat_after = {
        let view = observe(&w, id(&w, "IRQ"));
        ai::inputs::military_threat(&view, ai::inputs::foreign(&view, irn).unwrap())
    };
    println!(
        "threat {threat_before:.1} -> {threat_after:.1}; removing a threat {:.1} -> {:.1}; war {:.1} -> {:.1}",
        term(&before, "removing a threat"),
        term(&after, "removing a threat"),
        before.total(),
        after.total()
    );
    assert!(threat_after > threat_before + 5.0, "the arsenal is a threat");
    assert!((term(&after, "removing a threat") - term(&before, "removing a threat")).abs() < 1e-9);
    assert!(term(&after, "catastrophic risk (nuclear)") < 0.0);
    assert!(after.total() < before.total());
}
