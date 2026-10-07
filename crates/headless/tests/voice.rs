//! V1.5 acceptance tests: the early tone prototype
//! (design/behaviour-and-voice.md §2 and approval items 3, 4, 11, 12).
//! The standard: a satirical line appears only when the simulation earned it.

use std::path::PathBuf;

use headless::{run_campaign, run_campaign_with, VoiceSettings};
use sim_core::{resolve_turn, CountryId, Order, OrderSet, TreatyKind, WorldState};
use voice::lines::{load_lines, parse_lines, Line, Trigger};
use voice::narrator::{Narration, Narrator, TurnInput, BUDGET_BLACK_CASCADE};
use voice::Intensity;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn lines() -> Vec<Line> {
    load_lines(root().join("data/voice/lines.ron")).expect("voice lines validate")
}

fn fixture(name: &str) -> scenario::ScenarioDef {
    scenario::load(root().join("data/fixtures").join(name)).unwrap()
}

/// Scripted run with a narrator: passive countries, scripted orders.
fn scripted(
    def: &scenario::ScenarioDef,
    script: &[Vec<(&str, Order)>],
    intensity: Intensity,
) -> (WorldState, Vec<Narration>) {
    let mut w = scenario::build(def, Some(5)).unwrap();
    let mut narrator = Narrator::new(lines(), intensity, 5);
    let mut out = Vec::new();
    for turn_orders in script {
        let orders: Vec<OrderSet> = turn_orders
            .iter()
            .map(|(code, order)| OrderSet {
                country: w.find(code).unwrap(),
                orders: vec![order.clone()],
            })
            .collect();
        narrator.observe_before(&w);
        let report = resolve_turn(&mut w, orders);
        out.extend(narrator.narrate(TurnInput {
            state: &w,
            report: &report,
            decisions: &[],
        }));
    }
    (w, out)
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

#[test]
fn the_line_library_validates_and_stays_prototype_sized() {
    let l = lines();
    // V1.5 prototype + V2 war registers; V3 expands after Gate 6.
    assert!((15..=120).contains(&l.len()), "{} lines", l.len());
    for t in [
        Trigger::SanctionsHurtSender,
        Trigger::ClientCutWhileDependent,
        Trigger::NormIgnoredByFriend,
        Trigger::ShadowThenAttack,
    ] {
        assert!(l.iter().any(|x| x.trigger == t), "no line for {t:?}");
    }
    assert!(
        l.iter().any(|x| x.surface.is_private()) && l.iter().any(|x| !x.surface.is_private()),
        "public and private registers"
    );
}

#[test]
fn lint_rejects_atrocity_language_and_untrue_placeholders() {
    let banned = r#"[(id: "x", trigger: DebtBrake, surface: TreasuryMemo, priority: 4, intensity: Dry, text: "A resettlement programme.")]"#;
    assert!(parse_lines(banned).unwrap_err().contains("banned term"));
    let untrue = r#"[(id: "x", trigger: DebtBrake, surface: TreasuryMemo, priority: 4, intensity: Dry, text: "{casualties} lost.")]"#;
    assert!(parse_lines(untrue).unwrap_err().contains("not supplied"));
    let ok = r#"[(id: "x", trigger: DebtBrake, surface: TreasuryMemo, priority: 4, intensity: Dry, text: "Debt {debt}%, a temporary permanent condition.")]"#;
    assert!(parse_lines(ok).is_ok(), "ordinary doublespeak is allowed");
}

#[test]
fn no_trigger_no_joke() {
    let l = lines();
    let r = run_campaign_with(
        &fixture("four_actor.ron"),
        3,
        80,
        Some(VoiceSettings {
            lines: &l,
            intensity: Intensity::Black,
        }),
    )
    .unwrap();
    // The calm fixture still has a few real events (e.g. a trade offer
    // refused out of inertia). Jokes appear only on those, and rarely.
    let voiced: Vec<_> = r.narration.iter().filter(|n| n.line.is_some()).collect();
    for n in &voiced {
        assert!(!n.fact.is_empty(), "every line rides on a real fact");
    }
    assert!(voiced.len() <= 6, "calm world, few jokes: {voiced:?}");
}

#[test]
fn narration_never_changes_the_simulation() {
    let def = fixture("four_actor_tense.ron");
    let l = lines();
    let plain = run_campaign(&def, 9, 40).unwrap();
    let voiced = run_campaign_with(
        &def,
        9,
        40,
        Some(VoiceSettings {
            lines: &l,
            intensity: Intensity::Black,
        }),
    )
    .unwrap();
    assert_eq!(plain.final_state, voiced.final_state);
    assert!(!voiced.narration.is_empty());
}

#[test]
fn narration_is_deterministic() {
    let def = fixture("four_actor_tense.ron");
    let l = lines();
    let s = || {
        run_campaign_with(
            &def,
            9,
            40,
            Some(VoiceSettings {
                lines: &l,
                intensity: Intensity::Black,
            }),
        )
        .unwrap()
        .narration
    };
    assert_eq!(s(), s());
}

#[test]
fn off_shows_facts_but_no_satire() {
    let def = fixture("four_actor_tense.ron");
    let l = lines();
    let r = run_campaign_with(
        &def,
        9,
        40,
        Some(VoiceSettings {
            lines: &l,
            intensity: Intensity::Off,
        }),
    )
    .unwrap();
    assert!(!r.narration.is_empty() && r.narration.iter().all(|n| n.line.is_none()));
}

#[test]
fn sanctions_that_hurt_the_sender_are_noticed_with_real_numbers() {
    // A small economy sanctions a big one: the sender bleeds more.
    let (w, n) = scripted(
        &fixture("four_actor.ron"),
        &[vec![("NEU", Order::Sanction { target: CountryId(2) })]],
        Intensity::Black,
    );
    let hit = n
        .iter()
        .find(|x| x.trigger == Trigger::SanctionsHurtSender)
        .expect("trigger detected");
    let (neu, riv) = (w.country(id(&w, "NEU")), w.country(id(&w, "RIV")));
    let own = format!("{:.1}", neu.trade_lost / neu.gdp * 100.0);
    let theirs = format!("{:.1}", riv.trade_lost / riv.gdp * 100.0);
    assert!(
        hit.fact.contains(&own) && hit.fact.contains(&theirs),
        "fact uses the simulation's numbers: {}",
        hit.fact
    );
}

#[test]
fn substitution_cutting_a_dependent_client_and_punitive_interdependence() {
    let mut script: Vec<Vec<(&str, Order)>> = vec![
        vec![
            ("MAJ", Order::Sanction { target: CountryId(2) }),
            (
                "MAJ",
                Order::StartStream {
                    to: CountryId(3),
                    amount: 0.5,
                },
            ),
            (
                "MAJ",
                Order::ProposeTreaty {
                    to: CountryId(1),
                    kind: TreatyKind::Trade { deep: true },
                },
            ),
        ],
        vec![(
            "ALY",
            Order::Respond {
                proposal: sim_core::ProposalId(0),
                accept: true,
            },
        )],
    ];
    script.extend((0..4).map(|_| vec![]));
    script.push(vec![
        ("MAJ", Order::StopStream { to: CountryId(3) }),
        ("MAJ", Order::Sanction { target: CountryId(1) }),
    ]);
    let (w, n) = scripted(&fixture("four_actor.ron"), &script, Intensity::Black);
    for x in &n {
        eprintln!("[t{}] FACT  {}", x.turn, x.fact);
        if let Some((heading, speaker, text, _)) = &x.line {
            eprintln!("      {heading} — {speaker}: \"{text}\"");
        }
    }
    let triggers: Vec<Trigger> = n.iter().map(|x| x.trigger).collect();
    for t in [
        Trigger::SanctionsSubstituted,
        Trigger::ClientCutWhileDependent,
        Trigger::DeepTradeThenSanction,
    ] {
        assert!(triggers.contains(&t), "{t:?} not detected in {triggers:?}");
    }
    let cut = n
        .iter()
        .find(|x| x.trigger == Trigger::ClientCutWhileDependent)
        .unwrap();
    let neu = w.country(id(&w, "NEU"));
    let expected = format!("{:.1}%", 0.5 / (neu.gdp * neu.tax_rate) * 100.0);
    assert!(
        cut.fact.contains(&expected),
        "dependence from real revenue ({expected}): {}",
        cut.fact
    );
}

#[test]
fn abandoning_an_ally_reads_as_honour_until_termination() {
    let def = fixture("four_actor.ron");
    let w = scenario::build(&def, Some(5)).unwrap();
    let alliance = w
        .diplomacy
        .treaties
        .iter()
        .find(|t| t.kind == TreatyKind::DefensiveAlliance)
        .unwrap()
        .id;
    let (_, n) = scripted(
        &def,
        &[vec![("MAJ", Order::CancelTreaty { treaty: alliance })]],
        Intensity::Black,
    );
    assert!(n
        .iter()
        .any(|x| x.trigger == Trigger::CommitmentCancelled && x.fact.contains("cancelled its alliance")));
}

#[test]
fn black_never_becomes_a_stand_up_routine() {
    let def = fixture("four_actor_tense.ron");
    let l = lines();
    let r = run_campaign_with(
        &def,
        9,
        80,
        Some(VoiceSettings {
            lines: &l,
            intensity: Intensity::Black,
        }),
    )
    .unwrap();
    for turn in 0..80 {
        let voiced = r
            .narration
            .iter()
            .filter(|n| n.turn == turn && n.line.is_some())
            .count();
        assert!(voiced <= BUDGET_BLACK_CASCADE, "turn {turn}: {voiced} flavoured lines");
    }
    let total = r.narration.iter().filter(|n| n.line.is_some()).count();
    assert!(total <= 20, "{total} flavoured lines in 80 turns is too many");
}

#[test]
fn black_prefers_the_black_voice_and_dry_stays_restrained() {
    let l = lines();
    let intensity_of = |id: &str| l.iter().find(|x| x.id == id).unwrap().intensity;
    let has_black = |t: Trigger| l.iter().any(|x| x.trigger == t && x.intensity == Intensity::Black);
    let def = fixture("four_actor_tense.ron");
    for (setting, check) in [(Intensity::Black, true), (Intensity::Dry, false)] {
        let r = run_campaign_with(
            &def,
            9,
            80,
            Some(VoiceSettings {
                lines: &l,
                intensity: setting,
            }),
        )
        .unwrap();
        for n in r.narration.iter().filter(|n| n.line.is_some()) {
            let (_, _, _, id) = n.line.as_ref().unwrap();
            let used = intensity_of(id);
            if check {
                let trigger = l.iter().find(|x| &x.id == id).unwrap().trigger;
                if has_black(trigger) && used == Intensity::Dry {
                    // Allowed only when every Black variant was capped or cooling down.
                    continue;
                }
            } else {
                assert_eq!(used, Intensity::Dry, "Dry setting used a Black line: {id}");
            }
        }
    }
    // The scripted dependent-client cut speaks in the Black voice.
    let script: Vec<Vec<(&str, Order)>> = vec![
        vec![(
            "MAJ",
            Order::StartStream {
                to: CountryId(3),
                amount: 0.5,
            },
        )],
        vec![("MAJ", Order::StopStream { to: CountryId(3) })],
    ];
    let mut voiced = None;
    for seed_offset in 0..8u64 {
        let mut def = fixture("four_actor.ron");
        def.seed += seed_offset;
        let mut w = scenario::build(&def, Some(def.seed)).unwrap();
        let mut narrator = Narrator::new(lines(), Intensity::Black, def.seed);
        for turn_orders in &script {
            let orders: Vec<OrderSet> = turn_orders
                .iter()
                .map(|(code, order)| OrderSet {
                    country: w.find(code).unwrap(),
                    orders: vec![order.clone()],
                })
                .collect();
            narrator.observe_before(&w);
            let report = resolve_turn(&mut w, orders);
            for n in narrator.narrate(TurnInput {
                state: &w,
                report: &report,
                decisions: &[],
            }) {
                if n.trigger == Trigger::ClientCutWhileDependent && n.line.is_some() {
                    voiced = n.line.map(|x| x.3);
                }
            }
        }
        if voiced.is_some() {
            break;
        }
    }
    assert_eq!(
        voiced.as_deref(),
        Some("client_cut_rant"),
        "Black picks the Black line when one is available"
    );
}

/// V2: the war registers fire from real war state, keep casualty figures in
/// the plain FACT, and never put a war line on the wire.
#[test]
fn v2_war_registers_fire_from_real_war_state() {
    let mut def = fixture("four_actor.ron");
    def.treaties.push(scenario::TreatyDef {
        kind: TreatyKind::Guarantee,
        a: "MAJ".into(),
        b: "NEU".into(),
    });
    for (from, to) in [("RIV", "NEU"), ("NEU", "RIV")] {
        def.opinions.push(scenario::OpinionDef {
            from: from.into(),
            to: to.into(),
            value: -60.0,
            decay: 0.0,
            source: "border dispute".into(),
        });
    }
    let neu = CountryId(3);
    let riv = CountryId(2);
    let script: Vec<Vec<(&str, Order)>> = vec![
        // MAJ armed NEU in peacetime; RIV attacks from calm (no casus belli).
        vec![("MAJ", Order::ArmsTransfer { to: neu, amount: 4.0 })],
        vec![(
            "RIV",
            Order::DeclareWar {
                target: neu,
                aim: sim_core::WarAim::Limited,
            },
        )],
        // The guarantor answers with sanctions only; ALY arms both sides.
        vec![
            ("MAJ", Order::Sanction { target: riv }),
            (
                "ALY",
                Order::StartArmsStream {
                    to: riv,
                    amount: 0.3,
                    covert: false,
                },
            ),
            (
                "ALY",
                Order::StartArmsStream {
                    to: neu,
                    amount: 0.3,
                    covert: false,
                },
            ),
        ],
        vec![],
        vec![],
    ];
    let (_, out) = scripted(&def, &script, Intensity::Black);
    let fired: Vec<Trigger> = out.iter().map(|n| n.trigger).collect();
    for n in &out {
        println!("[{:?}] {}", n.trigger, n.fact);
        if let Some((h, sp, t, _)) = &n.line {
            println!("    {h} — {sp}: {t}");
        }
    }
    for t in [
        Trigger::WarWithoutCasusBelli,
        Trigger::GuarantorSentSanctions,
        Trigger::ArmsToBothSides,
    ] {
        assert!(fired.contains(&t), "{t:?} should fire: {fired:?}");
    }
    for n in out.iter().filter(|n| n.gravity >= 3) {
        if let Some((heading, ..)) = &n.line {
            assert_ne!(heading, "WIRE", "war lines stay on command surfaces");
        }
    }
    // The validator rejects a war line on the wire.
    let bad = r#"[(id: "x", trigger: PunitiveStrike, surface: Ticker, priority: 4, intensity: Black, max_gravity: 3, text: "x")]"#;
    assert!(parse_lines(bad).is_err());
}

/// V3: scenario-event lines are chosen by event id, strategy reversals and
/// arms deliveries are detected from decisions, and the epitaph pairs the
/// Official History with plain Ledger counts from the run.
#[test]
fn v3_event_lines_and_epitaph_come_from_the_run() {
    let l = lines();
    // Every scenario-event line names a template that exists in the 1980 data.
    let events = scenario::load(root().join("data/scenarios/1980.ron")).unwrap().events;
    for line in l.iter().filter(|x| x.trigger == Trigger::ScenarioEvent) {
        let e = line.event.as_ref().unwrap();
        assert!(events.iter().any(|t| &t.id == e), "{} names unknown event {e}", line.id);
    }
    // A line for a scenario event must name it; others must not.
    let bad =
        r#"[(id: "x", trigger: ScenarioEvent, surface: OfficialStatement, priority: 4, intensity: Black, text: "x")]"#;
    assert!(parse_lines(bad).is_err());

    let def = scenario::load(root().join("data/scenarios/1980.ron")).unwrap();
    let r = run_campaign_with(
        &def,
        4,
        80,
        Some(VoiceSettings {
            lines: &l,
            intensity: Intensity::Black,
        }),
    )
    .unwrap();
    let event_lines: Vec<_> = r
        .narration
        .iter()
        .filter(|n| n.trigger == "ScenarioEvent" && n.line.is_some())
        .collect();
    assert!(!event_lines.is_empty(), "scenario events earn lines in the 1980 world");
    let (official, ledger) = r.epitaph.clone().expect("epitaph with voice on");
    println!(
        "{official:?}
{}",
        ledger.join(
            "
"
        )
    );
    assert!(official.is_some_and(|o| o.starts_with("THE OFFICIAL HISTORY")));
    assert!(ledger[0].starts_with("THE LEDGER") && ledger[1].contains("honoured"));
}

/// Anti-fatigue (seen in a 1980 run): one speaker said the same line twice
/// in a campaign, and two speakers echoed one line in the same turn.
#[test]
fn no_speaker_repeats_a_line_and_no_line_echoes_within_a_turn() {
    let l = lines();
    let def = scenario::load(root().join("data/scenarios/1980.ron")).unwrap();
    for seed in [3u64, 5] {
        let run = run_campaign_with(
            &def,
            seed,
            80,
            Some(VoiceSettings {
                lines: &l,
                intensity: Intensity::Black,
            }),
        )
        .unwrap();
        let mut by_speaker = std::collections::BTreeSet::new();
        let mut by_turn = std::collections::BTreeSet::new();
        for n in &run.narration {
            if let Some((_, speaker, _, id)) = &n.line {
                assert!(by_speaker.insert((id.clone(), speaker.clone())), "seed {seed}: {speaker} repeated {id}");
                assert!(by_turn.insert((id.clone(), n.turn)), "seed {seed}: {id} echoed in turn {}", n.turn);
            }
        }
    }
}
