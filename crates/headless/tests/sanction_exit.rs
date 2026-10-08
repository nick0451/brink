//! Issue 15: sanction exit. A sanction whose cause has passed is lifted, one
//! against a continuing grievance is kept, a coalition's followers do not
//! hold its leader in, and lifting never flickers.

use std::collections::BTreeMap;
use std::path::PathBuf;

use ai::{Controller, Strategist};
use sim_core::opinion::OpinionModifier;
use sim_core::{observe, resolve_turn, CountryId, DecisionKind, DiplomaticEvent, Order, OrderSet, WorldState};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn world() -> WorldState {
    let def = scenario::load(workspace_root().join("data/fixtures/four_actor.ron")).unwrap();
    scenario::build(&def, Some(1)).unwrap()
}

fn grievance(w: &mut WorldState, from: CountryId, to: CountryId, value: f64, decay: f64) {
    w.opinions.add(
        from,
        to,
        OpinionModifier {
            source: "test grievance".into(),
            value,
            decay,
            memory: None,
        },
    );
}

/// Events by turn, and keep-sanction reviews (turn, country, kept).
type Played = (Vec<(u32, DiplomaticEvent)>, Vec<(u32, CountryId, bool)>);

/// Run `turns` turns with the given countries on the strategist and the
/// rest idle. `first` orders are added on the first turn. Returns every
/// event and every keep-sanction record (turn, country, chosen).
fn play(w: &mut WorldState, ai_countries: &[CountryId], first: Vec<(CountryId, Order)>, turns: u32) -> Played {
    let mut ais: BTreeMap<CountryId, Strategist> =
        ai_countries.iter().map(|&c| (c, Strategist::with_seed(1))).collect();
    let mut events = Vec::new();
    let mut keeps = Vec::new();
    for t in 0..turns {
        let mut sets = Vec::new();
        for id in w.ids().collect::<Vec<_>>() {
            let mut orders = Vec::new();
            if let Some(s) = ais.get_mut(&id) {
                let d = s.decide(&observe(w, id));
                keeps.extend(
                    d.records
                        .iter()
                        .filter(|r| r.kind == DecisionKind::KeepSanction)
                        .map(|r| (w.turn, id, r.chosen)),
                );
                orders = d.orders;
            }
            if t == 0 {
                orders.extend(first.iter().filter(|(c, _)| *c == id).map(|(_, o)| o.clone()));
            }
            sets.push(OrderSet { country: id, orders });
        }
        let report = resolve_turn(w, sets);
        events.extend(report.events.into_iter().map(|e| (report.turn, e)));
    }
    (events, keeps)
}

fn lifted(events: &[(u32, DiplomaticEvent)], by: CountryId, target: CountryId) -> Option<u32> {
    events.iter().find_map(|(t, e)| match e {
        DiplomaticEvent::SanctionLifted { by: b, target: x } if *b == by && *x == target => Some(*t),
        _ => None,
    })
}

#[test]
fn sanction_is_lifted_once_its_cause_has_passed_and_followers_do_not_hold_it() {
    let mut w = world();
    let (maj, aly, neu) = (w.find("MAJ").unwrap(), w.find("ALY").unwrap(), w.find("NEU").unwrap());
    // A sharp quarrel that fades (an incident, not a standing hostility).
    grievance(&mut w, maj, neu, -70.0, 2.0);
    grievance(&mut w, aly, neu, -40.0, 2.0);
    let first = vec![(maj, Order::Sanction { target: neu })];
    let (events, keeps) = play(&mut w, &[maj, aly], first, 48);

    // While the quarrel was fresh the sanction was kept.
    let first_review = keeps.iter().find(|k| k.1 == maj).expect("MAJ reviewed its sanction");
    assert!(first_review.2, "kept at the first review while the grievance was fresh");
    // Then lifted: by the leader, with its follower still in at the time.
    let maj_lift = lifted(&events, maj, neu).expect("MAJ lifts once the grievance has decayed");
    let aly_joined = events
        .iter()
        .any(|(_, e)| matches!(e, DiplomaticEvent::SanctionImposed { by, target } if *by == aly && *target == neu));
    assert!(aly_joined, "ALY followed MAJ into the sanction");
    let aly_lift = lifted(&events, aly, neu).expect("the follower lifts after the leader");
    assert!(aly_lift >= maj_lift, "follower {aly_lift} before leader {maj_lift}");
    assert!(
        !w.diplomacy.sanctions.iter().any(|s| s.target == neu),
        "nobody sanctions NEU at the end: {:?}",
        w.diplomacy.sanctions
    );
    // Opinion recovers: the standing penalty is gone and the memory fades.
    assert!(w.opinions.modifiers(neu, maj).iter().all(|m| m.source != "sanctions"));
    // No re-imposition after the lift.
    assert!(!events.iter().any(|(t, e)| *t > maj_lift
        && matches!(e, DiplomaticEvent::SanctionImposed { by, target } if *by == maj && *target == neu)));
}

#[test]
fn sanction_against_a_continuing_grievance_is_kept() {
    let mut w = world();
    let (maj, riv) = (w.find("MAJ").unwrap(), w.find("RIV").unwrap());
    // A standing grievance that does not fade.
    grievance(&mut w, maj, riv, -40.0, 0.0);
    let first = vec![(maj, Order::Sanction { target: riv })];
    let (events, keeps) = play(&mut w, &[maj], first, 48);
    assert!(keeps.iter().any(|k| k.1 == maj), "the sanction was reviewed");
    assert_eq!(lifted(&events, maj, riv), None, "kept while the grievance stands");
    assert!(w.diplomacy.is_sanctioned_by(riv, maj));
}

/// Over whole 1980 campaigns: lifts happen, and no lifted sanction is
/// re-imposed (or re-joined) by the same country unless the target has
/// committed a new hostile act since the lift (D85): a Coercion entry with
/// it as actor, on or after the lift turn and before the re-imposition,
/// that is a cause for the re-imposer by `ai::sanctions::is_cause_for`
/// (aimed at it, or a war, strike, exposed programme or nuclear use against
/// anyone). Legitimate new-cause re-impositions must still happen.
///
/// Issue 15 defined flicker as any re-imposition within 8 turns of a lift.
/// That window also counted re-impositions for a new cause (seed 7: Iraq's
/// programme is exposed at t71, and ISR and SYR, who lifted at t66 and t71,
/// re-impose at t74/75), while missing churn after 8 turns. The canonical
/// ledger log is a superset of what each observer saw, so this check is no
/// stricter than the AI's own rule.
#[test]
fn campaign_lifts_without_flicker() {
    use sim_core::ledger::LedgerLog;
    use sim_core::{CauseCode, EntryKind};
    let def = scenario::load(workspace_root().join("data/scenarios/1980.ron")).unwrap();
    let (mut lifts, mut reimposed, mut flicker) = (0, 0, Vec::new());
    // 16 campaigns: the "returned for a new cause" check rested on a single
    // re-imposition in 8 (issue 26 moved it to seed 9); the flicker check
    // gets twice the coverage.
    for seed in 1..=16 {
        let r = headless::run_campaign(&def, seed, def.turns).unwrap();
        let acts: Vec<(u32, CountryId, CountryId, CauseCode)> = r
            .ledger_log
            .iter()
            .filter_map(|(t, l)| match l {
                LedgerLog::EntryWritten {
                    kind: EntryKind::Coercion,
                    actor,
                    counterpart,
                    code,
                    ..
                } => Some((*t, *actor, *counterpart, *code)),
                _ => None,
            })
            .collect();
        let mut last_lift: BTreeMap<(CountryId, CountryId), u32> = BTreeMap::new();
        for (t, e) in &r.events {
            match e {
                DiplomaticEvent::SanctionLifted { by, target } => {
                    lifts += 1;
                    last_lift.insert((*by, *target), *t);
                }
                DiplomaticEvent::SanctionImposed { by, target } => {
                    if let Some(l) = last_lift.remove(&(*by, *target)) {
                        reimposed += 1;
                        // Decided on turn t from what was known by the end of t - 1.
                        let cause = acts.iter().any(|(at, a, c, code)| {
                            a == target && *at >= l && at < t && ai::sanctions::is_cause_for(*c, *code, *by)
                        });
                        if !cause {
                            flicker.push((seed, *by, *target, l, *t));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    println!("lifts {lifts}, re-impositions after a lift {reimposed}");
    assert!(lifts > 0, "no sanction lifted in 16 campaigns");
    assert!(flicker.is_empty(), "re-imposed with no new hostile act since the lift: {flicker:?}");
    assert!(reimposed > 0, "no lifted sanction ever returned, not even for a new cause");
}

/// The keep decision reads the relationship without the sanctions' own echo:
/// a counter-sanction that answered ours is not a reason to keep ours, but
/// one that came first is.
#[test]
fn counterfactual_removes_only_the_echo() {
    let mut w = world();
    let (maj, neu) = (w.find("MAJ").unwrap(), w.find("NEU").unwrap());
    let step = |w: &mut WorldState, c: CountryId, o: Order| {
        let sets = w
            .ids()
            .map(|id| OrderSet {
                country: id,
                orders: if id == c { vec![o.clone()] } else { Vec::new() },
            })
            .collect();
        resolve_turn(w, sets);
    };
    step(&mut w, maj, Order::Sanction { target: neu });
    step(&mut w, neu, Order::Sanction { target: maj });
    let view = observe(&w, maj);
    let ours = *view.sanctions.iter().find(|s| s.by == maj).unwrap();
    let cf = ai::sanctions::counterfactual(&view, &ours);
    let (real, alt) = (
        view.others.iter().find(|f| f.id == neu).unwrap(),
        cf.others.iter().find(|f| f.id == neu).unwrap(),
    );
    assert!(
        (alt.our_opinion_of_them - real.our_opinion_of_them - 25.0).abs() < 1e-9,
        "their answer removed"
    );
    assert!(alt.tension < real.tension, "the regime's tension removed");
    assert!(!cf.sanctions.iter().any(|s| s.by == maj && s.target == neu));
    assert!(
        !cf.sanctions.iter().any(|s| s.by == neu && s.target == maj),
        "their answer is out of the trade term too"
    );
    // Seen from NEU, MAJ's earlier sanction is a cause, not an echo.
    let view = observe(&w, neu);
    let theirs = *view.sanctions.iter().find(|s| s.by == neu).unwrap();
    let cf = ai::sanctions::counterfactual(&view, &theirs);
    let (real, alt) = (
        view.others.iter().find(|f| f.id == maj).unwrap(),
        cf.others.iter().find(|f| f.id == maj).unwrap(),
    );
    assert!(
        (alt.our_opinion_of_them - real.our_opinion_of_them).abs() < 1e-9,
        "the cause stays"
    );
}
