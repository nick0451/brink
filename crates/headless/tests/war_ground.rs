//! Issue 6 (crisis pass 2, follow-ups F1/F2): home ground follows the front.
//!
//! - F1: the 1980 aggressor (Iraq) collapsed in 11/40 runs, mid-war, before
//!   war weariness reached exhaustion: a failed invasion was routed all the
//!   way to decisive defeat because the declared defender kept the home
//!   ground bonus and the home-front rally wherever the front stood.
//! - F2: Iraq declared war in ~80% of runs at estimated odds of ~0.45 and
//!   lost 111/111. Its estimate matched the true ratio (~0.88); the engine,
//!   not the estimate, turned a near-even war into a certain rout.

use std::path::PathBuf;

use sim_core::{resolve_turn, CountryId, DiplomaticEvent, Order, OrderSet, PeaceReason, Side, WarAim, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> WorldState {
    let def = scenario::load(root().join("data/fixtures/four_actor.ron")).unwrap();
    scenario::build(&def, Some(3)).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

/// Hold a country's forces at a given power (armies fixed, so the test
/// isolates the ground rule from production and upkeep).
fn pin(w: &mut WorldState, c: CountryId, power: f64) {
    let f = &mut w.country_mut(c).forces;
    f.readiness = 0.5;
    let k = power / f.power().max(1e-9);
    f.land.strength *= k;
    f.naval.strength *= k;
    f.air.strength *= k;
}

/// Fight a Major war RIV → NEU with both armies pinned, nobody offering
/// peace. Returns the front progress per turn and how the war ended.
fn fight(att: f64, def: f64, turns: u32) -> (Vec<f64>, Option<(PeaceReason, f64)>) {
    let mut w = fixture();
    let (riv, neu) = (id(&w, "RIV"), id(&w, "NEU"));
    let mut progress = Vec::new();
    let mut ended = None;
    for t in 0..turns {
        pin(&mut w, riv, att);
        pin(&mut w, neu, def);
        let orders = if t == 0 {
            vec![OrderSet {
                country: riv,
                orders: vec![Order::DeclareWar {
                    target: neu,
                    aim: WarAim::Major,
                }],
            }]
        } else {
            vec![]
        };
        let report = resolve_turn(&mut w, orders);
        for e in &report.events {
            if let DiplomaticEvent::PeaceMade {
                reason, progress: p, ..
            } = e
            {
                ended = Some((*reason, *p));
            }
        }
        if let Some(war) = w.wars.active.first() {
            progress.push(war.progress());
        }
        if ended.is_some() {
            break;
        }
    }
    (progress, ended)
}

#[test]
fn home_ground_and_the_home_front_rally_follow_the_front() {
    let mut w = fixture();
    let (riv, neu) = (id(&w, "RIV"), id(&w, "NEU"));
    resolve_turn(
        &mut w,
        vec![OrderSet {
            country: riv,
            orders: vec![Order::DeclareWar {
                target: neu,
                aim: WarAim::Limited,
            }],
        }],
    );
    let mut war = w.wars.active[0].clone();
    for (progress, home) in [(30.0, Side::Defender), (0.0, Side::Defender), (-30.0, Side::Attacker)] {
        war.fronts[0].progress = progress;
        assert_eq!(war.home_side(), home, "progress {progress}");
        assert!(war.ground_mult(home) > war.ground_mult(home.other()));
    }
}

/// The home-ground advantage builds with depth: it fades continuously
/// across the border (full to the defender at progress 0, full to the
/// thrown-back invader at −WIN_MARGIN) instead of flipping at 0, so a front
/// held near the border is a line, not a flip-flop (issue 13, D65 follow-up).
#[test]
fn the_ground_multiplier_is_continuous_across_the_border() {
    let mut w = fixture();
    let (riv, neu) = (id(&w, "RIV"), id(&w, "NEU"));
    resolve_turn(
        &mut w,
        vec![OrderSet {
            country: riv,
            orders: vec![Order::DeclareWar {
                target: neu,
                aim: WarAim::Limited,
            }],
        }],
    );
    let mut war = w.wars.active[0].clone();
    let at = |war: &mut sim_core::War, p: f64, side: Side| {
        war.fronts[0].progress = p;
        war.ground_mult(side)
    };
    // The steepest slope is DEFENCE_MULT − 1 over WIN_MARGIN progress, so
    // a step of 0.1 progress can move the multiplier by at most 0.002.
    let mut p = -60.0;
    while p < 60.0 {
        for side in [Side::Attacker, Side::Defender] {
            let jump = (at(&mut war, p + 0.1, side) - at(&mut war, p, side)).abs();
            assert!(jump < 0.0021, "ground_mult jumps {jump:.4} at progress {p:.1} for {side:?}");
            let m = at(&mut war, p, side);
            assert!((1.0..=sim_core::war::DEFENCE_MULT).contains(&m));
        }
        p += 0.1;
    }
    // The ends of the ramp are the old discrete rule.
    assert_eq!(at(&mut war, 0.0, Side::Defender), sim_core::war::DEFENCE_MULT);
    assert_eq!(at(&mut war, 0.0, Side::Attacker), 1.0);
    assert_eq!(at(&mut war, 40.0, Side::Defender), sim_core::war::DEFENCE_MULT);
    assert_eq!(at(&mut war, -25.0, Side::Attacker), sim_core::war::DEFENCE_MULT);
    assert_eq!(at(&mut war, -25.0, Side::Defender), 1.0);
    assert_eq!(at(&mut war, -80.0, Side::Attacker), sim_core::war::DEFENCE_MULT);
    // Half-way, the two sides are even: the fighting is across the border.
    assert!((at(&mut war, -12.5, Side::Attacker) - at(&mut war, -12.5, Side::Defender)).abs() < 1e-9);
}

/// A failed invasion held near the border no longer flips the bonus every
/// turn: the front settles on the line where the invader's growing home
/// advantage balances the counter-attacker's strength, instead of chattering
/// across progress 0 (issue 6 review counted 4 sign crossings per Iran–Iraq
/// war, median, max 9).
#[test]
fn a_failed_invasion_held_at_the_border_does_not_flip_the_bonus_every_turn() {
    let (progress, ended) = fight(50.0, 40.0, 30);
    println!("progress {progress:?} ended {ended:?}");
    let crossings = progress.windows(2).filter(|p| (p[0] < 0.0) != (p[1] < 0.0)).count();
    assert!(progress.len() >= 10, "the war runs long enough to measure ({})", progress.len());
    assert!(crossings <= 1, "the front crosses the border {crossings} times, not once");
    // Settled: after the first few turns the front stays within a narrow
    // band on the invader's side of the border.
    let tail = &progress[progress.len() / 2..];
    let (lo, hi) = tail.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &p| (lo.min(p), hi.max(p)));
    assert!(hi < 0.0, "the invader is held on its own soil (high {hi:.1})");
    assert!(hi - lo < 25.0, "the front holds a line (band {lo:.1}..{hi:.1})");
}

/// Neither side can carry the war onto the other's soil: once the invasion
/// fails, the front is held near the invader's own border (whoever is on
/// home ground holds) and the war ends without a rout. Before issue 6 the
/// declared defender kept its ×1.5 wherever the front stood and routed the
/// invader to a decisive defeat. The former KNOWN GAP (the ×1.5 flipping at
/// progress 0, so the front chattered across the border) is closed by the
/// continuous ramp above.
#[test]
fn a_failed_invasion_oscillates_near_the_border_without_a_rout() {
    // Attacker commits 0.8 × 50 = 40 against 40 × 1.5 on the defender's
    // soil (ratio 0.67), but the defender's 40 meets 40 × 1.5 on the
    // attacker's soil (also 0.67).
    let (progress, ended) = fight(50.0, 40.0, 30);
    println!("progress {progress:?} ended {ended:?}");
    assert!(
        progress.iter().any(|&p| p < 0.0),
        "the invasion fails: the front moves onto the attacker's soil"
    );
    assert!(
        !matches!(ended, Some((PeaceReason::Decisive, _))),
        "a failed invasion is not a rout: {ended:?}"
    );
    let low = progress.iter().copied().fold(0.0, f64::min);
    assert!(
        low > -50.0,
        "the front stays near the invader's border (lowest {low:.1})"
    );
}

/// A genuinely catastrophic defeat is still possible: an enemy that
/// outweighs even the home-ground bonus carries the war to a decision.
#[test]
fn a_rout_still_ends_in_decisive_defeat() {
    let (progress, ended) = fight(50.0, 150.0, 30);
    println!("progress {progress:?} ended {ended:?}");
    assert!(
        matches!(ended, Some((PeaceReason::Decisive, p)) if p <= -100.0),
        "the far stronger defender wins outright: {ended:?}"
    );
}

/// Regression over the 1980 campaign (40 runs). Before issue 6: 8 of 11
/// Iraqi collapses came mid-war (about 1 in 4 wars), and aggressors lost
/// 31 of the 35 wars they chose (every war on Iran, chosen at estimated odds
/// ~0.45). Wars should end, by negotiation or exhaustion, before the regimes
/// fighting them fall; and a near-even war should not be a certain loss.
#[test]
fn wars_end_before_regimes_fall_and_chosen_wars_are_not_mostly_lost() {
    use sim_core::{PeaceResult, TransitionKind};
    let def = scenario::load(root().join("data/scenarios/1980.ron")).unwrap();
    let runs = headless::run_batch(&def, &(1..=40).collect::<Vec<_>>(), 80, 4).unwrap();
    let (mut wars, mut lost, mut mid_war) = (0u32, 0u32, 0u32);
    for r in &runs {
        let mut leaders: Vec<(u32, CountryId)> = Vec::new();
        for (_, e) in &r.events {
            match e {
                DiplomaticEvent::WarDeclared {
                    war,
                    attacker,
                    defender,
                    ..
                } => {
                    wars += 1;
                    leaders.push((war.0, *attacker));
                    leaders.push((war.0, *defender));
                }
                DiplomaticEvent::PeaceMade { war, result, .. } => {
                    lost += u32::from(*result == PeaceResult::DefenderWon);
                    leaders.retain(|l| l.0 != war.0);
                }
                DiplomaticEvent::Transition {
                    country,
                    kind: TransitionKind::Collapse,
                } => mid_war += u32::from(leaders.iter().any(|l| l.1 == *country)),
                _ => {}
            }
        }
    }
    println!("wars {wars}, lost by the aggressor {lost}, war leaders collapsing mid-war {mid_war}");
    assert!(wars > 0, "the campaign still has wars");
    assert!(
        f64::from(mid_war) <= 0.1 * f64::from(wars),
        "regimes fall mid-war in at most 1 war in 10"
    );
    assert!(
        f64::from(lost) <= 0.5 * f64::from(wars),
        "wars chosen on the chooser's own estimate are not mostly lost"
    );
}
