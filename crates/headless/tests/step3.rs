//! Build step 3 acceptance tests: diplomacy and economics (implementation plan §3).

use std::path::PathBuf;

use sim_core::domestic::prosperity_terms;
use sim_core::{
    observe, resolve_turn, CountryId, DiplomaticEvent, Order, OrderSet, TreatyKind, TurnReport, WorldState,
};

fn fixture_def() -> scenario::ScenarioDef {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures/four_actor.ron");
    scenario::load(path).unwrap()
}

fn world() -> WorldState {
    scenario::build(&fixture_def(), Some(1)).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

/// Resolve a turn where only the listed countries act.
fn step(w: &mut WorldState, acts: Vec<(CountryId, Vec<Order>)>) -> TurnReport {
    let orders = acts
        .into_iter()
        .map(|(country, orders)| OrderSet { country, orders })
        .collect();
    resolve_turn(w, orders)
}

fn lost_trade(w: &WorldState, c: CountryId) -> f64 {
    prosperity_terms(w.country(c))
        .into_iter()
        .find(|t| t.0 == "lost trade")
        .unwrap()
        .1
}

#[test]
fn actions_cost_initiative_and_excess_is_rejected() {
    let mut w = world();
    let (maj, riv, neu) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "NEU"));
    let available = w.country(maj).initiative.available();
    let mut orders = vec![Order::SetDeficit(0.0)]; // standing setting: free
    orders.extend((0..available).map(|_| Order::Denounce { target: riv }));
    orders.push(Order::Aid { to: neu, amount: 0.5 });
    let report = step(&mut w, vec![(maj, orders)]);
    assert_eq!(report.rejected.len(), 1, "{:?}", report.rejected);
    assert!(matches!(report.rejected[0].1, Order::Aid { .. }));
    assert_eq!(report.rejected[0].2, "not enough Initiative");
}

#[test]
fn accepting_a_commitment_costs_initiative_but_trade_does_not() {
    for (kind, accept_costs) in [
        (TreatyKind::NonAggression, true),
        (TreatyKind::Trade { deep: true }, false),
    ] {
        let mut w = world();
        let (riv, neu, maj) = (id(&w, "RIV"), id(&w, "NEU"), id(&w, "MAJ"));
        let report = step(&mut w, vec![(riv, vec![Order::ProposeTreaty { to: neu, kind }])]);
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);

        let proposal = observe(&w, neu).incoming_proposals[0].id;
        let available = w.country(neu).initiative.available();
        let mut orders = vec![Order::Respond { proposal, accept: true }];
        orders.extend((0..available).map(|_| Order::Denounce { target: maj }));
        let report = step(&mut w, vec![(neu, orders)]);

        let expected_rejections = usize::from(accept_costs);
        assert_eq!(
            report.rejected.len(),
            expected_rejections,
            "{kind:?}: {:?}",
            report.rejected
        );
        assert!(w.diplomacy.find_treaty(kind, riv, neu).is_some_and(|t| t.kind == kind));
    }
}

#[test]
fn proposals_answerable_next_turn_only_then_lapse() {
    let mut w = world();
    let (riv, neu) = (id(&w, "RIV"), id(&w, "NEU"));
    step(
        &mut w,
        vec![(
            riv,
            vec![Order::ProposeTreaty {
                to: neu,
                kind: TreatyKind::NonAggression,
            }],
        )],
    );
    let proposal = w.diplomacy.proposals[0].id;

    // An answer in the same turn as the proposal is impossible by construction:
    // the proposal only becomes visible after that turn resolves.
    assert_eq!(observe(&w, neu).incoming_proposals.len(), 1);

    let report = step(&mut w, vec![]);
    assert!(report
        .events
        .iter()
        .any(|e| matches!(e, DiplomaticEvent::ProposalLapsed { proposal: p } if p.id == proposal)));
    assert!(w.diplomacy.proposals.is_empty());
    let late = step(&mut w, vec![(neu, vec![Order::Respond { proposal, accept: true }])]);
    assert_eq!(late.rejected.len(), 1);
}

#[test]
fn refusing_costs_only_opinion() {
    let mut w = world();
    let (riv, neu) = (id(&w, "RIV"), id(&w, "NEU"));
    step(
        &mut w,
        vec![(
            riv,
            vec![Order::ProposeTreaty {
                to: neu,
                kind: TreatyKind::DefensiveAlliance,
            }],
        )],
    );
    let proposal = w.diplomacy.proposals[0].id;
    let mut control = w.clone();
    let before = w.opinions.opinion(riv, neu);
    let report = step(
        &mut w,
        vec![(
            neu,
            vec![Order::Respond {
                proposal,
                accept: false,
            }],
        )],
    );
    step(&mut control, vec![]);
    assert!(report.rejected.is_empty());
    let refused = w.opinions.opinion(riv, neu);
    let ignored = control.opinions.opinion(riv, neu);
    assert!(refused < ignored && refused < before, "refusal should cost opinion");
    assert_eq!(
        w.country(neu).initiative,
        control.country(neu).initiative,
        "refusal is free"
    );
}

#[test]
fn sanctions_erode_through_substitution_and_adaptation_persists() {
    let mut w = world();
    let (maj, riv) = (id(&w, "MAJ"), id(&w, "RIV"));
    step(&mut w, vec![(maj, vec![Order::Sanction { target: riv }])]);
    let first_loss = w.country(riv).trade_lost;
    assert!(first_loss > 0.0);

    let mut losses = vec![first_loss];
    for _ in 0..4 {
        step(&mut w, vec![]);
        losses.push(w.country(riv).trade_lost);
    }
    assert!(
        losses.windows(2).all(|p| p[1] < p[0]),
        "target loss shrinks each turn: {losses:?}"
    );
    let adapted = w.country(riv).sanction_adaptation;
    assert!((adapted - 0.5).abs() < 1e-9, "~10% per turn, got {adapted}");

    step(&mut w, vec![(maj, vec![Order::LiftSanction { target: riv }])]);
    for _ in 0..3 {
        step(&mut w, vec![]);
    }
    assert_eq!(w.country(riv).trade_lost, 0.0);
    assert!(
        w.country(riv).sanction_adaptation >= adapted,
        "adaptation persists after lifting"
    );

    step(&mut w, vec![(maj, vec![Order::Sanction { target: riv }])]);
    let reimposed = w.country(riv).trade_lost;
    assert!(
        reimposed < first_loss * 0.6,
        "re-imposed sanction bites less: {reimposed} vs {first_loss}"
    );
}

#[test]
fn deeper_trade_makes_sanctioning_a_partner_costlier() {
    let mut costs = Vec::new();
    for depth in [None, Some(false), Some(true)] {
        let mut def = fixture_def();
        def.treaties
            .retain(|t| !matches!(t.kind, TreatyKind::Trade { .. }) || t.a != "MAJ");
        if let Some(deep) = depth {
            def.treaties.push(scenario::TreatyDef {
                kind: TreatyKind::Trade { deep },
                a: "MAJ".into(),
                b: "ALY".into(),
            });
        }
        let mut w = scenario::build(&def, Some(1)).unwrap();
        let (maj, aly) = (id(&w, "MAJ"), id(&w, "ALY"));
        step(&mut w, vec![(maj, vec![Order::Sanction { target: aly }])]);
        costs.push(w.country(maj).trade_lost);
    }
    assert!(
        costs[0] < costs[1] && costs[1] < costs[2],
        "none < shallow < deep: {costs:?}"
    );
}

#[test]
fn sanctioner_pays_a_visible_prosperity_cost() {
    let mut w = world();
    let mut control = w.clone();
    let (maj, riv) = (id(&w, "MAJ"), id(&w, "RIV"));
    step(&mut w, vec![(maj, vec![Order::Sanction { target: riv }])]);
    step(&mut control, vec![]);
    assert!(lost_trade(&w, maj) < 0.0, "sanctioner shows a lost-trade modifier");
    assert_eq!(lost_trade(&control, maj), 0.0);
    assert!(w.country(maj).prosperity < control.country(maj).prosperity);
}

#[test]
fn deep_integration_causes_a_temporary_adjustment_shock() {
    let mut w = world();
    let (maj, aly) = (id(&w, "MAJ"), id(&w, "ALY"));
    step(
        &mut w,
        vec![(
            maj,
            vec![Order::ProposeTreaty {
                to: aly,
                kind: TreatyKind::Trade { deep: true },
            }],
        )],
    );
    let proposal = w.diplomacy.proposals[0].id;
    step(&mut w, vec![(aly, vec![Order::Respond { proposal, accept: true }])]);
    let shock = |w: &WorldState, c| {
        prosperity_terms(w.country(c))
            .into_iter()
            .find(|t| t.0 == "integration adjustment")
            .unwrap()
            .1
    };
    assert!(shock(&w, maj) < 0.0 && shock(&w, aly) < 0.0, "both partners feel it");
    for _ in 0..8 {
        step(&mut w, vec![]);
    }
    assert_eq!(shock(&w, maj), 0.0, "adjustment is temporary");
    assert_eq!(
        w.diplomacy
            .treaties
            .iter()
            .filter(|t| matches!(t.kind, TreatyKind::Trade { .. }) && t.involves(maj))
            .count(),
        1,
        "deep replaced shallow"
    );
}

#[test]
fn support_streams_pay_every_turn_until_cut() {
    let mut w = world();
    let (maj, neu) = (id(&w, "MAJ"), id(&w, "NEU"));
    step(&mut w, vec![(maj, vec![Order::StartStream { to: neu, amount: 0.5 }])]);
    for _ in 0..3 {
        assert!((w.country(neu).aid_in - 0.5).abs() < 1e-9);
        assert!((w.country(maj).aid_out - 0.5).abs() < 1e-9);
        step(&mut w, vec![]);
    }
    step(&mut w, vec![(maj, vec![Order::StopStream { to: neu }])]);
    assert_eq!(w.country(neu).aid_in, 0.0);
    assert!(w.opinions.modifiers(neu, maj).iter().any(|m| m.source == "support cut"));
}

#[test]
fn tension_rises_with_hostility_and_global_tension_is_power_weighted() {
    let mut w = world();
    let (maj, riv, aly, neu) = (id(&w, "MAJ"), id(&w, "RIV"), id(&w, "ALY"), id(&w, "NEU"));
    let before = w.tension.get(maj, riv);
    step(&mut w, vec![(maj, vec![Order::Denounce { target: riv }])]);
    assert!(w.tension.get(maj, riv) > before + 3.0);

    let base = world();
    let mut strong = base.clone();
    let mut weak = base.clone();
    strong.tension.add(maj, riv, 40.0);
    weak.tension.add(aly, neu, 40.0);
    assert!(sim_core::tension::global(&strong) > sim_core::tension::global(&weak) * 1.5);
}

#[test]
fn allies_share_intelligence_coverage() {
    let with = world();
    let mut def = fixture_def();
    def.treaties.retain(|t| t.kind != TreatyKind::DefensiveAlliance);
    let without = scenario::build(&def, Some(1)).unwrap();
    let cov = |w: &WorldState| {
        let (aly, riv) = (id(w, "ALY"), id(w, "RIV"));
        observe(w, aly).others.iter().find(|f| f.id == riv).unwrap().coverage
    };
    assert!(cov(&with) > cov(&without) + 3.0, "{} vs {}", cov(&with), cov(&without));
}
