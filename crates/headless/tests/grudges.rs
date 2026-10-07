//! Issue 18 (D81): grudges fade unless renewed. A scenario's starting
//! hostility fades 1.5 points a year; a hostile act between the pair pauses
//! the fade for two years; opposing blocs hold it at -10 until a regime
//! leaves its bloc.

use std::path::PathBuf;

use sim_core::grudge::{BLOC_FLOOR, FADE_PER_YEAR, RENEWAL_TURNS, TURNS_PER_YEAR};
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

const STEP: f64 = FADE_PER_YEAR / TURNS_PER_YEAR as f64;

/// One turn; `orders` go to their countries, everyone else idles.
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

fn idle(w: &mut WorldState, turns: u32) {
    for _ in 0..turns {
        turn(w, &[]);
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn starting_hostility_is_a_memory_but_friendship_and_decaying_modifiers_are_not() {
    let w = world();
    let (maj, aly, riv, neu) = (
        w.find("MAJ").unwrap(),
        w.find("ALY").unwrap(),
        w.find("RIV").unwrap(),
        w.find("NEU").unwrap(),
    );
    assert!(close(w.opinions.memory(maj, riv), -40.0));
    assert!(w.opinions.modifiers(maj, aly).iter().all(|m| m.memory.is_none()));
    assert!(w.opinions.modifiers(neu, riv).iter().all(|m| m.memory.is_none()));
}

#[test]
fn idle_grudge_fades_at_the_stated_rate_stops_at_the_bloc_floor_and_ends_when_blocs_part() {
    let mut w = world();
    let (maj, riv) = (w.find("MAJ").unwrap(), w.find("RIV").unwrap());
    idle(&mut w, 8);
    // Two years without hostile acts: 3 points.
    assert!(
        close(w.opinions.memory(maj, riv), -40.0 + 8.0 * STEP),
        "{}",
        w.opinions.memory(maj, riv)
    );
    assert!(close(w.opinions.memory(riv, maj), -45.0 + 8.0 * STEP));
    // Near the floor: west and east still oppose, so it stops at -10.
    for m in w.opinions.modifiers_mut(maj, riv) {
        if m.memory.is_some() {
            m.value = -11.0;
        }
    }
    idle(&mut w, 12);
    assert!(close(w.opinions.memory(maj, riv), BLOC_FLOOR));
    // The rival's regime leaves its bloc: the floor lifts and the grudge ends.
    w.country_mut(riv).alignment = None;
    idle(&mut w, 30);
    assert_eq!(w.opinions.memory(maj, riv), 0.0);
    assert!(w.opinions.modifiers(maj, riv).iter().all(|m| m.memory.is_none()));
}

#[test]
fn a_hostile_act_pauses_the_fade_for_two_years_and_a_standing_sanction_renews_nothing() {
    let mut w = world();
    let (maj, riv) = (w.find("MAJ").unwrap(), w.find("RIV").unwrap());
    turn(&mut w, &[(maj, Order::Sanction { target: riv })]);
    idle(&mut w, RENEWAL_TURNS - 1);
    // Paused both ways for two years.
    assert!(close(w.opinions.memory(maj, riv), -40.0));
    assert!(close(w.opinions.memory(riv, maj), -45.0));
    idle(&mut w, 4);
    assert!(close(w.opinions.memory(maj, riv), -40.0 + 4.0 * STEP));
    let m = w
        .opinions
        .modifiers(maj, riv)
        .iter()
        .find(|m| m.memory.is_some())
        .unwrap();
    // The standing sanction renewed nothing after its imposition; the
    // breakdown says when and why it was last renewed.
    assert_eq!(m.memory.unwrap().renewals, 1);
    assert!(m.to_string().contains("renewed turn 0 by Sanction"), "{m}");
}

#[test]
fn same_pair_stays_hostile_while_it_keeps_sanctioning_and_thaws_without() {
    let turns = 60;
    let mut quiet = world();
    let mut feud = world();
    let (maj, riv) = (feud.find("MAJ").unwrap(), feud.find("RIV").unwrap());
    idle(&mut quiet, turns);
    for t in 0..turns {
        let orders = match t % RENEWAL_TURNS {
            0 => vec![(maj, Order::Sanction { target: riv })],
            4 => vec![(maj, Order::LiftSanction { target: riv })],
            _ => vec![],
        };
        turn(&mut feud, &orders);
    }
    let q = quiet.opinions.memory(maj, riv);
    let f = feud.opinions.memory(maj, riv);
    assert!(q > -20.0, "the quiet pair thaws: {q}");
    assert!(close(f, -40.0), "the feuding pair stays hostile: {f}");
}

#[test]
fn arming_an_enemy_renews_only_while_it_is_at_war() {
    let mut w = world();
    let (maj, aly, riv, neu) = (
        w.find("MAJ").unwrap(),
        w.find("ALY").unwrap(),
        w.find("RIV").unwrap(),
        w.find("NEU").unwrap(),
    );
    // Peacetime: MAJ arms its ally; RIV's grudge keeps fading.
    turn(&mut w, &[(maj, Order::ArmsTransfer { to: aly, amount: 1.0 })]);
    assert!(close(w.opinions.memory(riv, maj), -45.0 + STEP));
    // RIV attacks NEU; arming NEU now is arming RIV's enemy at war.
    turn(
        &mut w,
        &[(
            riv,
            Order::DeclareWar {
                target: neu,
                aim: WarAim::Limited,
            },
        )],
    );
    assert!(w.wars.enemies_of(riv).contains(&neu), "the war must be on");
    let before = w.opinions.memory(riv, maj);
    turn(&mut w, &[(maj, Order::ArmsTransfer { to: neu, amount: 1.0 })]);
    idle(&mut w, 2);
    assert!(
        close(w.opinions.memory(riv, maj), before),
        "paused by arming the enemy at war"
    );
}

#[test]
fn an_arms_race_renews_the_builders_grudge_with_its_top_threat_while_it_lasts() {
    let mut w = world();
    let (maj, aly, riv) = (w.find("MAJ").unwrap(), w.find("ALY").unwrap(), w.find("RIV").unwrap());
    // RIV spends half again its habitual share and is not cutting.
    let x = w.country_mut(riv);
    x.military_norm = x.budget.military / 1.5;
    idle(&mut w, 12);
    // Its top threat is MAJ: that grudge is renewed both ways...
    for (a, b, start) in [(maj, riv, -40.0), (riv, maj, -45.0)] {
        assert!(close(w.opinions.memory(a, b), start), "{a:?}->{b:?} {}", w.opinions.memory(a, b));
    }
    let m = w.opinions.modifiers(maj, riv).iter().find(|m| m.memory.is_some()).unwrap();
    assert!(m.to_string().contains("by ArmsBuildUp"), "{m}");
    // ...while its other historical rival's grudge keeps fading.
    assert!(close(w.opinions.memory(aly, riv), -30.0 + 12.0 * STEP));
    // It cuts back to its habit: the fade resumes. Last renewed on turn 11:
    // paused through turn 18, fading turns 19-23.
    let x = w.country_mut(riv);
    x.military_norm = x.budget.military;
    idle(&mut w, RENEWAL_TURNS + 4);
    assert!(close(w.opinions.memory(maj, riv), -40.0 + 5.0 * STEP), "{}", w.opinions.memory(maj, riv));
}
