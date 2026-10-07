//! The Security double count (crisis pass 3, issue 11, hunk B): a defender
//! hostile to us is paper, not shield; a friendly one counts as before.
//! F3 (issue 11 hunk A, applied by issue 14): a client that is itself the
//! aggressor, or presses a claim on the rival a pledge would face, is not
//! underwritten.

use std::path::PathBuf;

use ai::evaluate;
use sim_core::domestic::security_breakdown;
use sim_core::{
    observe, resolve_turn, CountryId, Order, OrderSet, Proposal, ProposalId, TreatyKind, WarAim, WorldState,
};

fn fixture_def() -> scenario::ScenarioDef {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures/four_actor.ron");
    scenario::load(path).unwrap()
}

/// The Neutral under threat from the Rival and guaranteed by the Major
/// Power (the Gate 3 crisis fixture).
fn crisis_def() -> scenario::ScenarioDef {
    let mut def = fixture_def();
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
    def.treaties.push(scenario::TreatyDef {
        kind: TreatyKind::Guarantee,
        a: "MAJ".into(),
        b: "NEU".into(),
    });
    def
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

fn step(w: &mut WorldState, acts: Vec<(CountryId, Vec<Order>)>) {
    let orders = acts
        .into_iter()
        .map(|(country, orders)| OrderSet { country, orders })
        .collect();
    resolve_turn(w, orders);
}

fn ally_term(w: &WorldState, me: &str, ally: &str) -> f64 {
    let b = security_breakdown(w, id(w, me));
    b.allies.iter().find(|a| a.0 == id(w, ally)).map_or(0.0, |a| a.1)
}

fn hostile_term(w: &WorldState, me: &str, h: &str) -> f64 {
    let b = security_breakdown(w, id(w, me));
    b.hostile.iter().find(|a| a.0 == id(w, h)).map_or(0.0, |a| a.1)
}

/// A friendly guarantor counts in full (the pledge discount is 1 at zero
/// hostility), so an unchanged relationship gives an unchanged Security.
#[test]
fn a_friendly_defender_counts_in_full() {
    let w = scenario::build(&crisis_def(), Some(1)).unwrap();
    let (maj, neu) = (id(&w, "MAJ"), id(&w, "NEU"));
    assert_eq!(sim_core::domestic::hostility(&w, maj, neu), 0.0);
    assert_eq!(sim_core::domestic::pledge_worth(0.0), 1.0);
    let expected = w.country(maj).power()
        * ai::inputs::ALLY_WEIGHT
        * sim_core::reputation::credibility(&w, neu, maj, sim_core::reputation::RepKind::Back)
        / 100.0
        * sim_core::domestic::reach(&w, maj, neu);
    assert!((ally_term(&w, "NEU", "MAJ") - expected).abs() < 1e-9);
    assert_eq!(hostile_term(&w, "NEU", "MAJ"), 0.0);
}

/// A guarantor hostile to its protégé is counted as pressure, not as
/// shield: the ally term shrinks with its hostility and vanishes at war,
/// while the hostile term stays. The AI's own defence estimate follows.
#[test]
fn a_hostile_defender_is_pressure_not_shield() {
    let friendly = scenario::build(&crisis_def(), Some(1)).unwrap();
    let shield = ally_term(&friendly, "NEU", "MAJ");
    assert!(shield > 0.0, "a friendly guarantor counts: {shield}");
    let own_defence = ai::inputs::defence(&observe(&friendly, id(&friendly, "NEU")));

    // The same guarantee from a power that has turned against us.
    let mut def = crisis_def();
    def.opinions.push(scenario::OpinionDef {
        from: "MAJ".into(),
        to: "NEU".into(),
        value: -80.0,
        decay: 0.0,
        source: "fallen out".into(),
    });
    let hostile = scenario::build(&def, Some(1)).unwrap();
    let paper = ally_term(&hostile, "NEU", "MAJ");
    let pressure = hostile_term(&hostile, "NEU", "MAJ");
    println!("ally term {shield:.1} -> {paper:.1}; hostile term {pressure:.1}");
    assert!(pressure > 0.0, "its hostility is pressure");
    assert!(
        paper < 0.3 * shield,
        "a hostile defender's pledge is paper: {paper:.1} vs {shield:.1}"
    );
    assert!(
        security_breakdown(&hostile, id(&hostile, "NEU")).value
            < security_breakdown(&friendly, id(&friendly, "NEU")).value
    );
    let seen = ai::inputs::defence(&observe(&hostile, id(&hostile, "NEU")));
    assert!(
        seen < own_defence,
        "the AI reads the same discount: {seen:.1} < {own_defence:.1}"
    );

    // At war with our own guarantor the pledge is worth nothing.
    let mut w = scenario::build(&crisis_def(), Some(1)).unwrap();
    let (maj, neu) = (id(&w, "MAJ"), id(&w, "NEU"));
    step(
        &mut w,
        vec![(
            maj,
            vec![Order::DeclareWar {
                target: neu,
                aim: WarAim::Limited,
            }],
        )],
    );
    assert!(w.wars.at_war(maj, neu));
    assert_eq!(ally_term(&w, "NEU", "MAJ"), 0.0);
    assert!(hostile_term(&w, "NEU", "MAJ") > 0.0);
}

fn guarantee_request(w: &WorldState, from: &str, to: &str) -> Proposal {
    Proposal {
        id: ProposalId(999),
        from: id(w, from),
        to: id(w, to),
        kind: TreatyKind::Guarantee,
        turn: w.turn,
    }
}

fn has_line(s: &ai::Score, label: &str) -> bool {
    s.lines.iter().any(|l| l.label == label && l.value < 0.0)
}

fn line(s: &ai::Score, label: &str) -> f64 {
    s.lines.iter().find(|l| l.label == label).map_or(0.0, |l| l.value)
}

/// (F3) An active aggressor is not underwritten: the attacker in a war is
/// refused a guarantee (asked or unasked) and its existing guarantee is
/// re-read against the war, while the state it attacked carries none of
/// these terms. After the war the record fades but is remembered.
#[test]
fn an_active_aggressor_is_not_offered_a_guarantee() {
    // No guarantee yet: the Major Power weighs both neighbours cold.
    let mut def = crisis_def();
    def.treaties.retain(|t| t.kind != TreatyKind::Guarantee);
    let mut w = scenario::build(&def, Some(5)).unwrap();
    let (maj, riv, neu) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "NEU"));
    let view = observe(&w, maj);
    let neu_v = ai::inputs::foreign(&view, neu).unwrap();
    let calm_offer = evaluate::offer_guarantee(&view, neu_v).total();
    let calm_accept = evaluate::proposal(&view, &guarantee_request(&w, "NEU", "MAJ")).total();

    // The Neutral starts the war.
    step(
        &mut w,
        vec![(
            neu,
            vec![Order::DeclareWar {
                target: riv,
                aim: WarAim::Limited,
            }],
        )],
    );
    let view = observe(&w, maj);
    let neu_v = ai::inputs::foreign(&view, neu).unwrap();
    let riv_v = ai::inputs::foreign(&view, riv).unwrap();
    assert!(ai::inputs::is_attacker(&view, neu));
    assert!(!ai::inputs::is_attacker(&view, riv));

    let offer = evaluate::offer_guarantee(&view, neu_v);
    assert!((line(&offer, "they are the attacker in a war") + evaluate::UNDERWRITING_ATTACKER).abs() < 1e-9);
    assert!(
        offer.total() < calm_offer,
        "unasked offer {:.1} vs {calm_offer:.1} before the war",
        offer.total()
    );
    let accept = evaluate::proposal(&view, &guarantee_request(&w, "NEU", "MAJ"));
    assert!((line(&accept, "they are the attacker in a war") + evaluate::UNDERWRITING_ATTACKER).abs() < 1e-9);
    assert!(
        accept.total() < calm_accept,
        "{:.1} vs {calm_accept:.1}",
        accept.total()
    );
    let keep = evaluate::keep_guarantee(&view, neu_v, false);
    assert!(has_line(&keep, "they are the attacker in a war"));
    let alliance = evaluate::proposal(
        &view,
        &Proposal {
            kind: TreatyKind::DefensiveAlliance,
            ..guarantee_request(&w, "NEU", "MAJ")
        },
    );
    assert!(has_line(&alliance, "they are the attacker in a war"));

    // The victim is not an aggressor: none of the terms apply to it.
    let for_victim = evaluate::offer_guarantee(&view, riv_v);
    assert!(!has_line(&for_victim, "they are the attacker in a war"));
    assert!(!has_line(&for_victim, "their record of starting wars"));

    // The record outlives the war and fades (the observer's own ledger).
    let record_now = ai::inputs::aggression_record(&view, neu);
    assert!(record_now > 0.9, "fresh: {record_now}");
    let mut later = view.clone();
    later.turn += ai::war::WAR_MEMORY_TURNS as u32 / 2;
    later.wars.clear();
    let record_later = ai::inputs::aggression_record(&later, neu);
    assert!(
        record_later > 0.0 && record_later < record_now,
        "fading: {record_later}"
    );
    let after = evaluate::offer_guarantee(&later, neu_v);
    assert!(has_line(&after, "their record of starting wars"));
    assert!(!has_line(&after, "they are the attacker in a war"));
}

/// (F3) A standing claim against the very rival the pledge would face is
/// moral hazard: the patron would underwrite the client's revisionism.
#[test]
fn a_client_pressing_a_claim_on_its_rival_is_not_underwritten() {
    let mut def = crisis_def();
    def.treaties.retain(|t| t.kind != TreatyKind::Guarantee);
    let plain = scenario::build(&def, Some(5)).unwrap();
    def.claims.push(scenario::ClaimDef {
        by: "NEU".into(),
        against: "RIV".into(),
        weight: 1.0,
    });
    let claimant = scenario::build(&def, Some(5)).unwrap();
    let score = |w: &WorldState| {
        let view = observe(w, id(w, "MAJ"));
        let neu = ai::inputs::foreign(&view, id(w, "NEU")).unwrap();
        (
            evaluate::offer_guarantee(&view, neu),
            evaluate::proposal(&view, &guarantee_request(w, "NEU", "MAJ")),
        )
    };
    let (offer0, accept0) = score(&plain);
    let (offer1, accept1) = score(&claimant);
    assert!(!has_line(&offer0, "underwriting their claim"));
    assert!(has_line(&offer1, "underwriting their claim"));
    assert!((offer0.total() - offer1.total() - evaluate::UNDERWRITING_CLAIM).abs() < 1e-9);
    assert!((accept0.total() - accept1.total() - evaluate::UNDERWRITING_CLAIM).abs() < 1e-9);
}

/// (F3, 1980) Iraq's invasion of Iran is not underwritten: once Iraq is the
/// attacker, a superpower's standing guarantee to it is re-read as moral
/// hazard and an unasked offer scores below zero.
#[test]
fn the_1980_invader_loses_its_guarantee() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let def = scenario::load(root.join("data/scenarios/1980.ron")).unwrap();
    let mut w = scenario::build(&def, Some(1)).unwrap();
    let (usa, irq, irn) = (id(&w, "USA"), id(&w, "IRQ"), id(&w, "IRN"));
    step(
        &mut w,
        vec![(
            irq,
            vec![Order::DeclareWar {
                target: irn,
                aim: WarAim::Major,
            }],
        )],
    );
    let view = observe(&w, usa);
    let irq_v = ai::inputs::foreign(&view, irq).unwrap();
    let offer = evaluate::offer_guarantee(&view, irq_v);
    println!("USA offer to IRQ after the invasion: {:.1}", offer.total());
    assert!(has_line(&offer, "they are the attacker in a war"));
    assert!(offer.total() < 0.0);
    let keep = evaluate::keep_guarantee(&view, irq_v, false);
    assert!(has_line(&keep, "they are the attacker in a war"));
}
