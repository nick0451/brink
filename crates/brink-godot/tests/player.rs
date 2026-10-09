//! V-2a player bridge (design/v2-plan.md §5): the player path has no side
//! effects, really controls its country, replays deterministically, checks
//! Initiative, stays inside the player's fog, and ends the game on the
//! player's regime collapse or coup. All without Godot.

use std::path::PathBuf;

use ai::{Controller, Decision, DecisionRecord, Strategist};
use brink_godot::player::{self, OrderSpec};
use brink_godot::session::Session;
use sim_core::{
    intel, observe, resolve_turn, CountryId, DecisionKind, Government, Order, OrderSet, Tier, TransitionKind,
    WorldState,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scenario_path() -> PathBuf {
    root().join("data/scenarios/1980.ron")
}

fn lines_path() -> PathBuf {
    root().join("data/voice/lines.ron")
}

fn world(seed: u64) -> WorldState {
    let def = scenario::load(scenario_path()).unwrap();
    scenario::build(&def, Some(seed)).unwrap()
}

fn json(w: &WorldState) -> String {
    serde_json::to_string(w).unwrap()
}

fn headless_final(seed: u64, turns: u32) -> String {
    let def = scenario::load(scenario_path()).unwrap();
    headless::run_campaign(&def, seed, turns).unwrap().final_state
}

const TURNS: u32 = 24;

#[test]
fn player_rs_never_names_the_canonical_world() {
    let src = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/player.rs")).unwrap();
    assert!(
        !src.contains("WorldState"),
        "player.rs must read only the player's ObserverView (principle 7)"
    );
    assert!(!src.contains("observe("), "player.rs must not build views itself");
}

#[test]
fn delegating_player_matches_headless() {
    // SOV is not country 0: the AIs before it in id order must not evict
    // the cabinet's cached decision (review finding 1).
    for (code, seed) in [("USA", 1u64), ("USA", 7), ("SOV", 1), ("SOV", 7)] {
        let mut s = Session::load(&scenario_path(), Some(&lines_path()), seed).unwrap();
        s.set_player(code).unwrap();
        s.set_delegate(true);
        for _ in 0..TURNS {
            // Advice is asked for mid-turn, as the client would: it must
            // not run the cabinet twice.
            let _ = s.advice();
            let out = s.step();
            assert!(out.game_over.is_none(), "{code} seed {seed}: {:?}", out.game_over);
        }
        assert_eq!(json(&s.world), headless_final(seed, TURNS), "{code} seed {seed}");
    }
}

/// A controller that does nothing (an idle human).
struct Idle;
impl Controller for Idle {
    fn decide(&mut self, _: &sim_core::ObserverView) -> Decision {
        Decision::default()
    }
}

#[test]
fn idle_player_controls_its_country() {
    let seed = 1;
    let mut s = Session::load(&scenario_path(), Some(&lines_path()), seed).unwrap();
    let p = s.set_player("SOV").unwrap();
    for _ in 0..TURNS {
        let _ = s.advice();
        assert!(s.step().game_over.is_none());
    }

    let mut w = world(seed);
    let mut cs: Vec<Box<dyn Controller>> = w
        .ids()
        .map(|c| {
            if c == p {
                Box::new(Idle) as Box<dyn Controller>
            } else {
                Box::new(Strategist::with_seed(seed))
            }
        })
        .collect();
    for _ in 0..TURNS {
        let views: Vec<_> = w.ids().map(|c| observe(&w, c)).collect();
        let orders = views
            .iter()
            .zip(cs.iter_mut())
            .map(|(v, c)| OrderSet {
                country: v.observer,
                orders: c.decide(v).orders,
            })
            .collect();
        resolve_turn(&mut w, orders);
    }
    assert_eq!(json(&s.world), json(&w), "idle player = do-nothing controller");
    assert_ne!(
        json(&s.world),
        headless_final(seed, TURNS),
        "the player really controls SOV"
    );
}

fn spec(kind: &str, target: Option<&str>) -> OrderSpec {
    OrderSpec {
        kind: kind.into(),
        target: target.map(Into::into),
        ..OrderSpec::default()
    }
}

/// Play a scripted campaign through `queue`; returns the final world and
/// the accepted order log.
fn scripted(seed: u64) -> (String, Vec<Vec<Order>>) {
    let mut s = Session::load(&scenario_path(), Some(&lines_path()), seed).unwrap();
    s.set_player("SOV").unwrap();
    let mut log = Vec::new();
    for t in 0..12u32 {
        let view = s.player_view().unwrap();
        let mut wish = Vec::new();
        if t == 0 {
            let mut b = spec("budget", None);
            b.shares = Some([0.35, 0.25, 0.32, 0.08]);
            wish.push(b);
            wish.push(spec("sanction", Some("CHN")));
        }
        if t == 2 {
            wish.push(spec("denounce", Some("USA")));
            let mut d = spec("deficit", None);
            d.amount = Some(0.1);
            wish.push(d);
        }
        if t == 5 {
            wish.push(spec("lift_sanction", Some("CHN")));
            let mut m = spec("mobilization", None);
            m.level = Some("Partial".into());
            wish.push(m);
        }
        for w in &wish {
            let o = player::parse_order(w, &view).unwrap();
            s.queue(o).unwrap_or_else(|e| panic!("turn {t}: {w:?}: {e}"));
        }
        // Proposals are answered: refuse everything (free).
        for p in player::proposals(&view) {
            let mut r = spec("respond", None);
            r.id = Some(p.id);
            r.flag = Some(false);
            s.queue(player::parse_order(&r, &view).unwrap()).unwrap();
        }
        log.push(s.pending().to_vec());
        let out = s.step();
        assert!(out.rejected.is_empty(), "turn {t}: {:?}", out.rejected);
    }
    (json(&s.world), log)
}

#[test]
fn replaying_the_order_log_reproduces_the_world() {
    let (a, log) = scripted(3);
    let (b, log2) = scripted(3);
    assert_eq!(a, b, "same seed + same orders → same world");
    assert_eq!(log, log2);
    // A replay from the saved log (no rehearsal) lands on the same world.
    let mut s = Session::load(&scenario_path(), None, 3).unwrap();
    s.set_player("SOV").unwrap();
    for orders in &log {
        for o in orders {
            s.queue_unchecked(o.clone());
        }
        s.step();
    }
    assert_eq!(json(&s.world), a, "replay from the order log");
    assert!(log.iter().map(Vec::len).sum::<usize>() >= 6);
}

#[test]
fn queue_checks_initiative_and_reports_the_sims_reasons() {
    let mut s = Session::load(&scenario_path(), None, 1).unwrap();
    s.set_player("USA").unwrap();
    let view = s.player_view().unwrap();
    let available = view.own.initiative.available();
    assert!(available >= 2);
    assert_eq!(s.initiative_left(), available);

    // Denounce one country after another until Initiative runs out.
    let targets: Vec<String> = view.others.iter().map(|f| f.code.clone()).collect();
    let mut accepted = 0u8;
    let mut refusal = None;
    for code in &targets {
        let o = player::parse_order(&spec("denounce", Some(code)), &view).unwrap();
        match s.queue(o) {
            Ok(cost) => {
                assert_eq!(cost, 1);
                accepted += 1;
                assert_eq!(s.initiative_left(), available - accepted);
            }
            Err(e) => {
                refusal = Some(e);
                break;
            }
        }
    }
    assert_eq!(accepted, available, "exactly the available Initiative is spendable");
    assert_eq!(refusal.as_deref(), Some("not enough Initiative"));
    // Free orders still queue when Initiative is gone.
    let mut d = spec("deficit", None);
    d.amount = Some(0.05);
    assert_eq!(s.queue(player::parse_order(&d, &view).unwrap()), Ok(0));
    // Unqueue frees the Initiative again.
    assert!(s.unqueue(0).is_some());
    assert_eq!(s.initiative_left(), 1);

    // An invalid order: queue refuses it with the simulation's own reason…
    let lift = player::parse_order(&spec("lift_sanction", Some("CHN")), &view).unwrap();
    assert_eq!(s.queue(lift.clone()), Err("no such sanction".to_string()));
    // …and the same order forced through (a replay) is rejected at
    // resolution with that same reason, reported to the player only.
    s.queue_unchecked(lift.clone());
    let out = s.step();
    assert_eq!(out.rejected.len(), 1, "{:?}", out.rejected);
    assert_eq!(out.rejected[0].order, lift);
    assert_eq!(out.rejected[0].reason, "no such sanction");
    assert!(out.rejected[0].text.contains("Lift sanctions"));

    // Unknown kinds and codes are refused by the parser.
    assert!(player::parse_order(&spec("nuke_everything", None), &view).is_err());
    assert!(player::parse_order(&spec("sanction", Some("XXX")), &view).is_err());
    assert!(
        player::parse_order(&spec("sanction", Some("USA")), &view).is_err(),
        "not ourselves"
    );
}

#[test]
fn every_order_kind_round_trips() {
    let s = Session::load(&scenario_path(), None, 1).unwrap();
    let w = &s.world;
    let view = observe(w, w.find("USA").unwrap());
    let mk = |kind: &str| {
        let mut x = spec(kind, Some("IRN"));
        x.aim = Some("Limited".into());
        x.amount = Some(1.5);
        x.flag = Some(true);
        x.id = Some(4);
        x.treaty = Some("NonAggression".into());
        x.side = Some("Defender".into());
        x.band = Some(4);
        x.shares = Some([0.3, 0.2, 0.4, 0.1]);
        x.level = Some(match kind {
            "mobilization" => "Full".into(),
            "energy_policy" => "Flood".into(),
            _ => "Loose".into(),
        });
        x
    };
    for kind in player::ORDER_KINDS {
        let order = player::parse_order(&mk(kind), &view).unwrap_or_else(|e| panic!("{kind}: {e}"));
        let back = player::spec_of(&order, &view).unwrap_or_else(|| panic!("{kind}: no spec"));
        assert_eq!(back.kind, *kind);
        assert_eq!(player::parse_order(&back, &view).unwrap(), order, "{kind}");
        assert!(!player::describe_order(&order, &view).is_empty());
    }
}

#[test]
fn foreign_data_respects_coverage() {
    // Everyone starts above domestic coverage in 1980, so blind one
    // observer: no reconnaissance, no intelligence service, closed targets.
    let mut found = None;
    let base = world(1);
    for id in base
        .countries
        .iter()
        .filter(|c| c.active && c.tier == Tier::Playable)
        .map(|c| c.id)
    {
        let mut w = base.clone();
        for x in &mut w.countries {
            x.openness = 0.0;
        }
        let me = w.country_mut(id);
        me.intel_tech = 0;
        me.intel_capacity = 0.0;
        let view = observe(&w, id);
        if let Some(f) = view.others.iter().find(|f| f.coverage < intel::TIER_DOMESTIC) {
            found = Some((view.clone(), f.code.clone()));
            break;
        }
    }
    let (view, code) = found.expect("some Playable observer has a low-coverage target");
    let f = player::foreign_info(&view, &code).unwrap();
    assert!(f.coverage < intel::TIER_DOMESTIC);
    assert_eq!(f.debt_ratio, None, "debt hidden");
    assert_eq!(f.stability_band, None, "stability hidden");
    assert_eq!(f.budget, None, "budget hidden");
    assert!(
        f.military[1] <= f.military[0] && f.military[0] <= f.military[2],
        "an estimate with a band"
    );
    assert!(f.military[2] > f.military[1], "low coverage: a real band");
    let row = player::map_rows(&view).into_iter().find(|r| r.code == code).unwrap();
    assert_eq!(row.stability, None);
    assert!(!row.own);
    let me = player::map_rows(&view).into_iter().find(|r| r.own).unwrap();
    assert_eq!(me.code, view.own.code);
    assert_eq!(me.stability, Some(view.own.stability), "own state is exact");
    // Own state is exact.
    let ps = player::player_state(&view);
    assert_eq!(ps.stability, view.own.stability);
    assert_eq!(ps.initiative_available, view.own.initiative.available());
}

/// A covert arms stream from `a` to `b`, narrated through a session whose
/// player is `player`. Returns the narrated facts.
fn narrate_covert_stream(w: &WorldState, a: CountryId, b: CountryId, player: Option<&str>) -> Vec<String> {
    let mut s = Session::from_world(w.clone(), Some(Vec::new()), 1);
    if let Some(p) = player {
        s.set_player(p).unwrap();
    }
    let pre = s.player_view();
    let orders = vec![OrderSet {
        country: a,
        orders: vec![Order::StartArmsStream {
            to: b,
            amount: 0.5,
            covert: true,
        }],
    }];
    let decisions = vec![(
        a,
        DecisionRecord {
            subject: "arm a client".into(),
            kind: DecisionKind::StreamStart,
            counterpart: Some(b),
            score: 1.0,
            chosen: true,
            lines: Vec::new(),
            precedents: Vec::new(),
        },
    )];
    let out = s.resolve(orders, &decisions, pre);
    assert!(
        s.world
            .diplomacy
            .streams
            .iter()
            .any(|x| x.from == a && x.to == b && x.covert),
        "the covert stream started"
    );
    out.narration.into_iter().map(|n| n.fact).collect()
}

#[test]
fn narration_keeps_covert_acts_out_of_the_players_sight() {
    let w = world(1);
    let playable: Vec<CountryId> = w
        .countries
        .iter()
        .filter(|c| c.active && c.tier == Tier::Playable)
        .map(|c| c.id)
        .collect();
    // A supplier `a`, a client `b` (playable) and a playable bystander `c`
    // who cannot see `a`'s covert acts.
    let mut pick = None;
    'outer: for &c in &playable {
        for &a in &playable {
            if a == c || sim_core::war::sees_covert(&w, c, a) {
                continue;
            }
            for &b in &playable {
                let streaming = w.diplomacy.streams.iter().any(|s| s.from == a && s.to == b);
                if b != a && b != c && !streaming {
                    pick = Some((a, b, c));
                    break 'outer;
                }
            }
        }
    }
    let (a, b, c) = pick.expect("a bystander blind to someone's covert acts");
    let code = |x: CountryId| w.country(x).code.clone();
    let armed = |facts: &[String]| facts.iter().any(|f| f.contains("began supplying"));

    let omniscient = narrate_covert_stream(&w, a, b, None);
    assert!(armed(&omniscient), "the spectator narrator voices it: {omniscient:?}");
    let bystander = narrate_covert_stream(&w, a, b, Some(&code(c)));
    assert!(!armed(&bystander), "{} cannot know: {bystander:?}", code(c));
    let client = narrate_covert_stream(&w, a, b, Some(&code(b)));
    assert!(armed(&client), "{} receives the arms: {client:?}", code(b));
}

fn fixture_session(code: &str, edit: impl Fn(&mut sim_core::Country)) -> Session {
    let mut w = world(1);
    let p = w.find(code).unwrap();
    edit(w.country_mut(p));
    let mut s = Session::from_world(w, None, 1);
    s.set_player(code).unwrap();
    s
}

#[test]
fn collapse_ends_the_game() {
    let mut s = fixture_session("USA", |c| {
        c.stability = 1.0;
        c.legitimacy = 1.0;
        c.prosperity = 1.0;
        c.security = 1.0;
        c.transition.quiet_until = 0;
    });
    let out = s.step();
    assert_eq!(out.game_over.as_deref(), Some("Collapse"));
    let turn = s.world.turn;
    let again = s.step();
    assert_eq!(again.game_over.as_deref(), Some("Collapse"));
    assert_eq!(s.world.turn, turn, "nothing resolves after game over");
    assert!(s.queue(Order::SetDeficit(0.0)).is_err());
    // A fallen regime stays fallen: no new seat, the game stays over.
    assert!(s.set_player("SOV").is_err());
    assert_eq!(s.game_over(), Some("Collapse"));
}

#[test]
fn a_coup_ends_the_game() {
    let code = "SOV";
    let mut s = fixture_session(code, |c| {
        c.stability = 25.0;
        c.legitimacy = 20.0;
        c.transition.crackdowns = 100;
        c.transition.quiet_until = 0;
    });
    assert_ne!(s.world.country(s.player().unwrap()).government, Government::Democracy);
    let out = s.step();
    assert_eq!(out.game_over.as_deref(), Some("Coup"));
}

#[test]
fn the_players_own_reform_does_not_end_the_game() {
    let mut s = fixture_session("SOV", |c| {
        c.transition.pending_since = Some(0);
    });
    let view = s.player_view().unwrap();
    assert!(player::player_state(&view).crisis_pending);
    s.queue(player::parse_order(&spec("reform", None), &view).unwrap())
        .unwrap();
    let before = s.world.country(s.player().unwrap()).transition.reformed_turn;
    let out = s.step();
    assert!(out.rejected.is_empty(), "{:?}", out.rejected);
    assert_eq!(out.game_over, None);
    let after = &s.world.country(s.player().unwrap()).transition;
    assert_ne!(after.reformed_turn, before, "the reform happened");
    assert!(player::game_over(
        s.player().unwrap(),
        &[sim_core::DiplomaticEvent::Transition {
            country: s.player().unwrap(),
            kind: TransitionKind::Reform
        }],
        true
    )
    .is_none());
}

#[test]
fn only_data_flagged_countries_can_be_chosen() {
    let mut s = Session::load(&scenario_path(), None, 1).unwrap();
    let all = s.playable();
    let chosen: Vec<String> = all
        .iter()
        .filter(|(_, ok)| *ok)
        .map(|(c, _)| s.world.country(*c).code.clone())
        .collect();
    assert_eq!(chosen, vec!["USA".to_string(), "SOV".to_string()]);
    assert!(all.len() > 2, "every Playable country is listed");
    assert!(s.set_player("IRN").is_err());
    assert!(s.set_player("CHN").is_err());
    assert!(s.set_player("KWT").is_err());
    assert!(s.set_player("SOV").is_ok());
    assert!(s.set_player("USA").is_ok());
}

/// Three playable countries with no streams between `a` and `b` in either
/// direction: `a` and `b` are parties, `c` a bystander.
fn trio(w: &WorldState) -> (CountryId, CountryId, CountryId) {
    let ids: Vec<CountryId> = w
        .countries
        .iter()
        .filter(|c| c.active && c.tier == Tier::Playable)
        .map(|c| c.id)
        .collect();
    for &a in &ids {
        for &b in &ids {
            let linked = w
                .diplomacy
                .streams
                .iter()
                .any(|s| (s.from == a && s.to == b) || (s.from == b && s.to == a));
            if a != b && !linked {
                let c = *ids.iter().find(|&&c| c != a && c != b).unwrap();
                return (a, b, c);
            }
        }
    }
    panic!("no unlinked pair");
}

#[test]
fn third_party_proposals_and_private_reasons_stay_private() {
    use sim_core::diplomacy::DiplomaticEvent as E;
    use voice::lines::Trigger;
    let w = world(1);
    let (a, b, c) = trio(&w);
    let (va, vc) = (observe(&w, a), observe(&w, c));
    let proposal = sim_core::Proposal {
        id: sim_core::ProposalId(999),
        from: a,
        to: b,
        kind: sim_core::TreatyKind::Guarantee,
        turn: 0,
    };
    for e in [
        E::Proposed { proposal },
        E::ProposalRefused { proposal },
        E::ProposalLapsed { proposal },
    ] {
        assert!(!player::event_visible(&vc, &vc, &e), "bystander sees {e:?}");
        assert!(player::event_visible(&va, &va, &e), "the proposer sees {e:?}");
    }
    // The refuser's private "decisive factor" and the coalition's motive.
    for t in [
        Trigger::GuaranteeRefused,
        Trigger::RepeatedRequestRefused,
        Trigger::StatusQuoRefusal,
        Trigger::CoalitionByDependence,
        Trigger::SanctionsHurtSender,
        Trigger::NormIgnoredByFriend,
    ] {
        assert!(!player::narration_visible(&vc, &vc, t, b, a), "bystander hears {t:?}");
        assert!(player::narration_visible(&va, &va, t, b, a), "the party hears {t:?}");
    }
    // "…which funds it": no funding stream the bystander can see.
    assert!(!player::narration_visible(&vc, &vc, Trigger::RefusedPatron, b, a));
}

#[test]
fn norm_tests_need_the_norm_to_have_been_seen() {
    use sim_core::{Grade, LedgerLog, NormTag, TurnReport};
    let w = world(1);
    let (a, v, c) = trio(&w);
    let vc = observe(&w, c);
    let seen = |n: NormTag| {
        vc.ledger
            .iter()
            .any(|e| e.actor == a && e.kind == sim_core::EntryKind::Norm(n))
    };
    let norm = [NormTag::Aggression, NormTag::Proliferation, NormTag::ChokepointClosure]
        .into_iter()
        .find(|&n| !seen(n))
        .expect("a norm the bystander never saw claimed");
    let report = TurnReport {
        turn: 0,
        events: Vec::new(),
        ledger_log: vec![LedgerLog::NormTest {
            norm,
            actor: a,
            violator: v,
            response: Grade::Abandoned,
            turn: 0,
            deltas: Vec::new(),
        }],
        rejected: Vec::new(),
    };
    assert!(player::visible_report(&vc, &vc, &report).ledger_log.is_empty());
    let va = observe(&w, a);
    assert_eq!(player::visible_report(&va, &va, &report).ledger_log.len(), 1);
}
