//! Nuclear weapons, v0.1 trim (DESIGN §11.4, D14): arsenal, deterrence,
//! one use action, the penalty package, AI restraint, voice silence.

use std::path::PathBuf;

use ai::{Controller, Strategist};
use sim_core::{observe, resolve_turn, CountryId, DiplomaticEvent, Order, OrderSet, WarAim, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn def() -> scenario::ScenarioDef {
    scenario::load(root().join("data/scenarios/1980.ron")).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

fn step(w: &mut WorldState, acts: Vec<(CountryId, Vec<Order>)>) -> sim_core::TurnReport {
    resolve_turn(
        w,
        acts.into_iter()
            .map(|(country, orders)| OrderSet { country, orders })
            .collect(),
    )
}

#[test]
fn arsenals_load_with_full_capacity() {
    let w = scenario::build(&def(), Some(1)).unwrap();
    let usa = w.country(id(&w, "USA"));
    assert_eq!(usa.arsenal, 3);
    assert_eq!(usa.strike_capacity, sim_core::nuclear::full_capacity(3));
    assert_eq!(w.country(id(&w, "IRQ")).arsenal, 0);
}

fn catastrophic_line(s: &ai::Score) -> f64 {
    s.lines
        .iter()
        .find(|l| l.term.contains("catastrophic"))
        .map_or(0.0, |l| l.value)
}

/// D49, re-justified under D68 #5: the full catastrophic-risk term belongs to
/// a Major war (the aim that threatens the regime). India vs China (arsenal
/// 2) Major stays below -50; Limited is still negative and still carries at
/// least the floor share of the Major risk; and a state under a nuclear
/// ally's cover (Vietnam under the USSR) is still deterred.
#[test]
fn deterrence_makes_a_major_war_on_an_arsenal_state_far_worse() {
    let w = scenario::build(&def(), Some(1)).unwrap();
    let (chn, vnm, ind) = (id(&w, "CHN"), id(&w, "VNM"), id(&w, "IND"));
    let view = observe(&w, ind);
    let china = view.others.iter().find(|f| f.id == chn).unwrap();
    let major = catastrophic_line(&ai::war::declare_war(&view, china, WarAim::Major));
    let limited = catastrophic_line(&ai::war::declare_war(&view, china, WarAim::Limited));
    let punitive = catastrophic_line(&ai::war::declare_war(&view, china, WarAim::Punitive));
    assert!(major < -50.0, "major {major}");
    assert!(limited < 0.0 && punitive < 0.0, "limited {limited} punitive {punitive}");
    assert!(major < limited && limited <= punitive, "{major} {limited} {punitive}");
    // Reduced, not removed: the floor share of the Major risk always remains.
    assert!(limited <= ai::war::NUCLEAR_RISK_FLOOR * major, "limited {limited} vs major {major}");
    assert!(punitive <= ai::war::NUCLEAR_RISK_FLOOR * major + 1e-9, "punitive {punitive} vs major {major}");
    let view_c = observe(&w, chn);
    let vs_vnm = ai::war::declare_war(
        &view_c,
        view_c.others.iter().find(|f| f.id == vnm).unwrap(),
        WarAim::Limited,
    );
    assert!(
        catastrophic_line(&vs_vnm) < 0.0,
        "Vietnam sits under a nuclear ally's cover: {:?}",
        vs_vnm.lines
    );
}

/// D68 #5 (the Falklands shape): the same aggressor against the same nuclear
/// target scores a Limited aim above a Major one, and the Major aim stays
/// strongly negative. The war itself still does not happen at 1980 values
/// (the protector and status-quo terms are larger than the nuclear one);
/// this fixture pins the ordering, not the outcome.
#[test]
fn a_limited_aim_against_a_nuclear_state_is_less_deterred_than_a_major_one() {
    let w = scenario::build(&def(), Some(1)).unwrap();
    let (arg, gbr) = (id(&w, "ARG"), id(&w, "GBR"));
    let view = observe(&w, arg);
    let britain = view.others.iter().find(|f| f.id == gbr).unwrap();
    assert!(britain.arsenal > 0);
    let limited = ai::war::declare_war(&view, britain, WarAim::Limited);
    let major = ai::war::declare_war(&view, britain, WarAim::Major);
    println!(
        "ARG -> GBR limited {:.1} {:?}\nARG -> GBR major {:.1} {:?}",
        limited.total(),
        limited.lines.iter().map(|l| (&l.label, l.value)).collect::<Vec<_>>(),
        major.total(),
        major.lines.iter().map(|l| (&l.label, l.value)).collect::<Vec<_>>()
    );
    assert!(limited.total() > major.total());
    assert!(major.total() < -50.0, "major war on a nuclear state: {}", major.total());
    let (lim_nuc, maj_nuc) = (catastrophic_line(&limited), catastrophic_line(&major));
    assert!(lim_nuc < 0.0 && lim_nuc > maj_nuc, "{lim_nuc} vs {maj_nuc}");
    // A target visibly in crisis reads any war as existential: the Limited
    // aim then carries the full risk.
    let mut w2 = scenario::build(&def(), Some(1)).unwrap();
    w2.country_mut(gbr).stability = 15.0;
    w2.country_mut(arg).intel_tech = 9;
    let view2 = observe(&w2, arg);
    let britain2 = view2.others.iter().find(|f| f.id == gbr).unwrap();
    assert!(
        matches!(britain2.stability_band, Some(sim_core::view::StabilityBand::Crisis)),
        "{:?}",
        britain2.stability_band
    );
    let cornered = catastrophic_line(&ai::war::declare_war(&view2, britain2, WarAim::Limited));
    let cornered_major = catastrophic_line(&ai::war::declare_war(&view2, britain2, WarAim::Major));
    assert!((cornered - cornered_major).abs() < 1e-9, "{cornered} vs {cornered_major}");
}

#[test]
fn a_strike_breaks_the_front_and_triggers_the_penalty_package() {
    let mut w = scenario::build(&def(), Some(2)).unwrap();
    let (chn, vnm, usa) = (id(&w, "CHN"), id(&w, "VNM"), id(&w, "USA"));
    step(
        &mut w,
        vec![(
            chn,
            vec![Order::DeclareWar {
                target: vnm,
                aim: WarAim::Limited,
            }],
        )],
    );
    let before = w.country(vnm).forces.strength();
    let legit = w.country(chn).legitimacy;
    let price = w.energy.price;
    let r = step(&mut w, vec![(chn, vec![Order::NuclearStrike { target: vnm }])]);
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, DiplomaticEvent::NuclearStrike { .. })),
        "{:?}",
        r.rejected
    );
    assert!(w.country(vnm).forces.strength() < 0.6 * before);
    assert!(w.country(chn).legitimacy < legit - 10.0);
    assert!(w.opinions.opinion(usa, chn) < -60.0, "pariah");
    assert!(w.energy.price > price, "markets panic");
    assert!(sim_core::nuclear::has_used(&w, chn));
    assert!(w.country(chn).strike_capacity < sim_core::nuclear::full_capacity(2));
    // No war, no strike: the one use action needs a war with the target.
    let r = step(&mut w, vec![(chn, vec![Order::NuclearStrike { target: usa }])]);
    assert!(!r.rejected.is_empty());
}

#[test]
fn others_turn_on_a_nuclear_user() {
    let mut w = scenario::build(&def(), Some(2)).unwrap();
    let (chn, vnm, jpn) = (id(&w, "CHN"), id(&w, "VNM"), id(&w, "JPN"));
    step(
        &mut w,
        vec![(
            chn,
            vec![Order::DeclareWar {
                target: vnm,
                aim: WarAim::Limited,
            }],
        )],
    );
    step(&mut w, vec![(chn, vec![Order::NuclearStrike { target: vnm }])]);
    let view = observe(&w, jpn);
    let s = ai::evaluate::impose_sanction(&view, view.others.iter().find(|f| f.id == chn).unwrap(), false);
    assert!(s.lines.iter().any(|l| l.label == "they used nuclear weapons"));
}

/// Nuclear restraint over 20 campaigns, in two halves (D49, D68 #5):
/// - use: never more than once per campaign ("rare but not never");
/// - a Major war declared on a state holding an arsenal: never (hard);
/// - Limited or Punitive wars on arsenal states: rare but possible. Counted
///   and printed only (KNOWN GAP form of `step9`): none occur today because
///   `declare_war` never prefers a Limited aim (review 12), so a bound would
///   be room for noise, not a measured property. A jump is visible here
///   before anyone decides whether it is a failure.
#[test]
fn ai_never_uses_nuclear_weapons_in_calm_campaigns() {
    let d = def();
    let mut limited_runs = 0;
    let mut limited: Vec<String> = Vec::new();
    for seed in 1..=20 {
        let r = headless::run_campaign(&d, seed, 80).unwrap();
        let used = r
            .events
            .iter()
            .filter(|(_, e)| matches!(e, DiplomaticEvent::NuclearStrike { .. }))
            .count();
        // Rare but not never: never more than once, and only inside a war.
        assert!(used <= 1, "seed {seed}: {used} uses");
        let p = headless::stats::plausibility(&r);
        assert!(
            p.wars_on_arsenal.get("Major").is_none_or(|v| v.is_empty()),
            "seed {seed}: Major war on an arsenal state {:?}",
            p.wars_on_arsenal
        );
        let small: Vec<String> = ["Limited", "Punitive"]
            .iter()
            .filter_map(|aim| p.wars_on_arsenal.get(*aim).map(|v| (aim, v)))
            .flat_map(|(aim, v)| v.iter().map(move |(a, b)| format!("seed {seed} {aim} {a}-{b}")))
            .collect();
        if !small.is_empty() {
            limited_runs += 1;
            limited.extend(small);
        }
    }
    println!("limited/punitive wars on arsenal states in {limited_runs}/20 runs: {limited:?}");
    if limited_runs == 0 {
        println!("KNOWN GAP: no limited war on a nuclear state; the AI never prefers a Limited aim (see STATE.md D74)");
    }
}

#[test]
fn an_arsenal_state_losing_badly_weighs_a_strike_and_explains_it() {
    let mut w = scenario::build(&def(), Some(3)).unwrap();
    let (chn, vnm) = (id(&w, "CHN"), id(&w, "VNM"));
    step(
        &mut w,
        vec![(
            vnm,
            vec![Order::DeclareWar {
                target: chn,
                aim: WarAim::Limited,
            }],
        )],
    );
    for f in &mut w.wars.active[0].fronts {
        f.progress = 80.0; // China is losing badly
    }
    let view = observe(&w, chn);
    let mut s = Strategist::with_seed(3);
    let d = s.decide(&view);
    let r = d
        .records
        .iter()
        .find(|r| r.subject.starts_with("nuclear strike"))
        .expect("considered");
    let sum: f64 = r.lines.iter().map(|l| l.value).sum();
    assert!((sum - r.score).abs() < 1e-9);
    println!(
        "{} {:.1}: {:?}",
        r.subject,
        r.score,
        r.lines.iter().map(|l| (&l.label, l.value)).collect::<Vec<_>>()
    );
}

#[test]
fn nuclear_use_silences_the_voice() {
    let l = voice::lines::load_lines(root().join("data/voice/lines.ron")).unwrap();
    assert!(!l.iter().any(|x| x.trigger == voice::lines::Trigger::NuclearUse));
    let bad = r#"[(id: "x", trigger: NuclearUse, surface: InternalMemo, priority: 1, intensity: Black, max_gravity: 5, text: "x")]"#;
    assert!(
        voice::lines::parse_lines(bad).is_err(),
        "no line may exist for nuclear use"
    );

    let mut w = scenario::build(&def(), Some(2)).unwrap();
    let (chn, vnm) = (id(&w, "CHN"), id(&w, "VNM"));
    let mut n = voice::narrator::Narrator::new(l, voice::Intensity::Black, 2);
    let mut out = Vec::new();
    for acts in [
        vec![(
            chn,
            vec![Order::DeclareWar {
                target: vnm,
                aim: WarAim::Limited,
            }],
        )],
        vec![(chn, vec![Order::NuclearStrike { target: vnm }])],
        vec![],
    ] {
        n.observe_before(&w);
        let report = step(&mut w, acts);
        out.extend(n.narrate(voice::narrator::TurnInput {
            state: &w,
            report: &report,
            decisions: &[],
        }));
    }
    let strike_turn = out
        .iter()
        .find(|x| x.trigger == voice::lines::Trigger::NuclearUse)
        .expect("plain fact")
        .turn;
    assert!(
        out.iter().filter(|x| x.turn >= strike_turn).all(|x| x.line.is_none()),
        "silence"
    );
    let _ = Strategist::new().decide(&observe(&w, chn));
}
