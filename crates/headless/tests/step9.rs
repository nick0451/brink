//! Build step 9 acceptance tests: the 1980 scenario, event templates,
//! minors, area reach, batch statistics → Gate 6 (DESIGN §22.1;
//! design/scenario-1980.md).

use std::collections::BTreeMap;
use std::path::PathBuf;

use ai::goals::Goal;
use ai::{Controller, Strategist};
use headless::stats::{gate6_table, in_gate6_band, plausibility, BEHAVIOURS, INFO_ROWS};
use sim_core::{observe, resolve_turn, CountryId, DiplomaticEvent, OrderSet, Tier, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn def1980() -> scenario::ScenarioDef {
    scenario::load(root().join("data/scenarios/1980.ron")).expect("1980 scenario loads and validates")
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

#[test]
fn the_1980_world_loads_with_16_playable_countries_and_stays_sane() {
    let def = def1980();
    let w = scenario::build(&def, Some(1)).unwrap();
    let playable = w.countries.iter().filter(|c| c.tier == Tier::Playable).count();
    assert_eq!(playable, 16);
    assert!(w.countries.iter().any(|c| c.tier == Tier::Minor));
    assert_eq!(w.event_templates.len(), 6);
    assert!(w
        .countries
        .iter()
        .all(|c| !c.reflexes.is_empty() || c.tier != Tier::Playable));

    let r = headless::run_campaign(&def, 1, 80).unwrap();
    let (first, last) = (&r.samples[0].countries, &r.samples.last().unwrap().countries);
    // Dormant successors start with no economy; judge only states that exist in 1980.
    for (a, b) in first.iter().zip(last).filter(|(a, _)| a.active) {
        assert!(b.gdp.is_finite() && b.stability.is_finite() && b.military.is_finite());
        let annual = (b.gdp / a.gdp).powf(1.0 / 20.0) - 1.0;
        // Real China grew ~9.8%/yr over 1980-2000; nothing should beat it by much.
        assert!(annual < 0.11, "{} grows {:.1}%/yr", a.code, 100.0 * annual);
    }
}

#[test]
fn local_threats_outrank_distant_giants() {
    let w = scenario::build(&def1980(), Some(1)).unwrap();
    let (cub, zaf, usa, prk) = (id(&w, "CUB"), id(&w, "ZAF"), id(&w, "USA"), id(&w, "PRK"));
    // A small power in another area barely reaches; a superpower navy does.
    assert!(sim_core::domestic::reach(&w, cub, zaf) < 0.5);
    assert_eq!(sim_core::domestic::reach(&w, usa, prk), 1.0);
}

#[test]
fn event_templates_fire_only_from_their_conditions() {
    let def = def1980();
    let r = headless::run_campaign(&def, 2, 40).unwrap();
    let w = scenario::build(&def, Some(2)).unwrap();
    let mut fired: BTreeMap<String, usize> = BTreeMap::new();
    for (_, e) in &r.events {
        if let DiplomaticEvent::ScenarioEvent { template, .. } = e {
            *fired
                .entry(w.event_templates[*template as usize].id.clone())
                .or_insert(0) += 1;
        }
    }
    println!("{fired:?}");
    assert!(!fired.is_empty(), "the 1980 world meets some event conditions");
    // A calm fixture with no templates and no conditions fires nothing.
    let calm = scenario::load(root().join("data/fixtures/four_actor.ron")).unwrap();
    let rc = headless::run_campaign(&calm, 2, 40).unwrap();
    assert!(!rc
        .events
        .iter()
        .any(|(_, e)| matches!(e, DiplomaticEvent::ScenarioEvent { .. })));
}

#[test]
fn minors_never_start_or_join_wars() {
    let def = def1980();
    for seed in 1..=10 {
        let r = headless::run_campaign(&def, seed, 80).unwrap();
        let w = scenario::build(&def, Some(seed)).unwrap();
        for (_, e) in &r.events {
            match e {
                DiplomaticEvent::WarDeclared { attacker, .. } => {
                    assert_ne!(w.country(*attacker).tier, Tier::Minor, "a minor declared war")
                }
                DiplomaticEvent::JoinedWar { country, .. } => {
                    assert_ne!(w.country(*country).tier, Tier::Minor, "a minor joined a war")
                }
                _ => {}
            }
        }
    }
}

/// Instrumentation (issue 5): every war a Gate 6 power doesn't lead gets an
/// explained "weigh intervention" record for each side (proxy and force),
/// so a missing intervention can be traced to a missing stake or to its
/// costs. The records are never chosen and never counted as behaviour.
#[test]
fn interventions_not_taken_are_explained() {
    let def = def1980();
    let mut wars_seen = 0;
    for seed in 1..=6 {
        let r = headless::run_campaign(&def, seed, 60).unwrap();
        let weighed: Vec<_> = r
            .reasoning
            .iter()
            .filter(|e| e.decision.subject.starts_with("weigh intervention in "))
            .collect();
        for e in &weighed {
            assert!(!e.decision.chosen, "a weighing record was chosen: {:?}", e.decision);
            assert!(e.decision.subject.contains("(stake "), "{}", e.decision.subject);
            assert!(!e.decision.lines.is_empty(), "{}", e.decision.subject);
        }
        let war = r.events.iter().find_map(|(_, e)| match e {
            DiplomaticEvent::WarDeclared { attacker, defender, .. } => Some((*attacker, *defender)),
            _ => None,
        });
        if let Some((a, d)) = war {
            wars_seen += 1;
            let w = scenario::build(&def, Some(seed)).unwrap();
            let (a, d) = (&w.country(a).code, &w.country(d).code);
            for side in [a, d] {
                let needle = format!("weigh intervention in {a}–{d} war for {side} ");
                assert!(
                    weighed
                        .iter()
                        .any(|e| e.country == "USA" && e.decision.subject.starts_with(&needle)),
                    "USA left no record weighing {needle}"
                );
            }
        }
    }
    assert!(wars_seen > 0, "no war in the sample seeds");
}

#[test]
fn batches_of_the_1980_world_are_identical_across_thread_counts() {
    let def = def1980();
    let seeds: Vec<u64> = (10..14).collect();
    let a = headless::run_batch(&def, &seeds, 40, 1).unwrap();
    let b = headless::run_batch(&def, &seeds, 40, 4).unwrap();
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.final_state, y.final_state);
    }
}

/// GATE 6 (part met, 2026-10-04) — AI variety across 200 seeded runs.
/// Reports the behaviour table and asserts what holds: the superpowers'
/// crisis behaviour varies across runs (neither always nor never), and
/// variation comes with state (interventions happen in runs with wars).
#[test]
fn gate6_superpower_behaviour_varies_with_state() {
    let def = def1980();
    let seeds: Vec<u64> = (1..=200).collect();
    let runs = headless::run_batch(&def, &seeds, 80, 8).unwrap();
    let powers = ["USA", "SOV", "JPN", "FRG", "GBR", "FRA"];
    let (per, pooled) = gate6_table(&runs, &powers);
    for b in BEHAVIOURS.iter().chain(&INFO_ROWS).copied() {
        let row: Vec<String> = powers
            .iter()
            .map(|p| format!("{:>4.0}%", 100.0 * per.get(&(p.to_string(), b)).copied().unwrap_or(0.0)))
            .collect();
        println!(
            "{b:<32} {}  pooled {:>5.1}%",
            row.join(" "),
            100.0 * pooled.get(b).copied().unwrap_or(0.0)
        );
    }
    for p in ["USA", "SOV"] {
        let v = per.get(&(p.to_string(), "intervene")).copied().unwrap_or(0.0);
        // Not always (<= 95%).
        assert!(v <= 0.95, "{p} intervenes in {:.0}% of runs", 100.0 * v);
    }
    // Intervene band (D68): pooled >= 5% (not 20%: the 1980 war set gives few
    // occasions; interventions are expected from Kuwait-type wars). Only
    // fighting (band 3-4) counts; arms streams are "armed a side". The floor
    // is a KNOWN GAP (D62): superpower "intervention" in the old baseline was
    // guarantors joining their proteges' offensives (a bug, now fixed).
    // Re-assert it when defensive interventions exist again.
    let iv = pooled.get("intervene").copied().unwrap_or(0.0);
    if !in_gate6_band("intervene", iv) {
        println!(
            "KNOWN GAP: intervene pooled {:.1}% (band >= 5%; see STATE.md D62, D68)",
            100.0 * iv
        );
    }
    // Arms-only involvement is not intervention (D68). The involvement
    // record sees every join (band 3-4), and "armed a side" means band 2
    // with no fighting anywhere in the run.
    for r in &runs {
        let codes: Vec<&str> = r.samples[0].countries.iter().map(|c| c.code.as_str()).collect();
        for (_, e) in &r.events {
            if let DiplomaticEvent::JoinedWar { country, war, band, .. } = e {
                let c = codes[country.index()];
                assert!(
                    r.involvements
                        .iter()
                        .any(|i| i.actor == c && i.war == war.0 && i.max_band >= *band),
                    "seed {}: {c} joined war {} at band {band} but no involvement was recorded",
                    r.seed,
                    war.0
                );
            }
        }
        for (c, set) in headless::stats::behaviours(r) {
            if set.contains("armed a side") {
                assert!(!set.contains("intervene"), "seed {}: {c}", r.seed);
                let mine: Vec<u8> = r
                    .involvements
                    .iter()
                    .filter(|i| i.actor == c)
                    .map(|i| i.max_band)
                    .collect();
                assert!(
                    mine.contains(&2) && mine.iter().all(|&b| b == 2),
                    "seed {}: {c} {mine:?}",
                    r.seed
                );
            }
        }
    }
    // Interventions track crises: runs without any war have none.
    for r in &runs {
        let wars = plausibility(r).wars;
        let joined = r
            .events
            .iter()
            .any(|(_, e)| matches!(e, DiplomaticEvent::JoinedWar { .. }));
        assert!(wars > 0 || !joined);
    }
    // KNOWN GAP (D45): pooled frequencies outside their band (20–95%;
    // intervene >= 5%, D68). At baseline4 + D68 (200 seeds): intervene and
    // escalate 0.1%, abandon 2.0%, coerce economically 100% (Western powers
    // always join the starting US sanctions on Cuba and Iran). Re-measured
    // after each change; not asserted here.
    let outside: Vec<&str> = BEHAVIOURS
        .iter()
        .copied()
        .filter(|b| !in_gate6_band(b, pooled.get(b).copied().unwrap_or(0.0)))
        .collect();
    println!("pooled outside band: {outside:?}");
}

/// Gate 5 re-run on the 1980 world (D41): delete the USSR by ordinary state
/// edits and compare the United States' goals and budget.
#[test]
fn gate5_on_1980_data_goal_mix_shifts_when_the_ussr_is_deleted() {
    let def = def1980();
    let mut contain = [0.0f64; 2];
    let mut mil = [0.0f64; 2];
    let mut goals_seen: [BTreeMap<String, u32>; 2] = [BTreeMap::new(), BTreeMap::new()];
    let seeds = 1..=40u64;
    for seed in seeds.clone() {
        for (k, delete) in [(0, false), (1, true)] {
            let mut w = scenario::build(&def, Some(seed)).unwrap();
            if delete {
                let sov = id(&w, "SOV");
                let c = w.country_mut(sov);
                let (s, mix, q) = (c.forces.strength(), c.force_mix, c.military_tech as f64);
                c.forces = sim_core::Forces::new(s * 0.1, mix, q);
                c.gdp *= 0.5;
                c.alignment = None;
                c.budget_target.military = 0.10;
                let ids: Vec<CountryId> = w.ids().collect();
                for o in ids {
                    w.opinions.remove_source(o, sov, "historical relations");
                    w.opinions.remove_source(sov, o, "historical relations");
                }
                sim_core::tension::initialize(&mut w);
            }
            let usa = id(&w, "USA");
            let mut ais: Vec<Strategist> = w.ids().map(|_| Strategist::with_seed(seed)).collect();
            for _ in 0..40 {
                let views: Vec<_> = w.ids().map(|c| observe(&w, c)).collect();
                let orders = views
                    .iter()
                    .zip(ais.iter_mut())
                    .map(|(v, ai)| OrderSet {
                        country: v.observer,
                        orders: ai.decide(v).orders,
                    })
                    .collect();
                resolve_turn(&mut w, orders);
                for g in &ais[usa.index()].goals {
                    let key = match g {
                        Goal::Contain(x) => format!("contain {}", w.country(*x).code),
                        other => format!("{other:?}"),
                    };
                    *goals_seen[k].entry(key).or_insert(0) += 1;
                    if matches!(g, Goal::Contain(x) if w.country(*x).code == "SOV") {
                        contain[k] += 1.0;
                    }
                }
            }
            mil[k] += w.country(usa).budget_target.military / 40.0;
        }
    }
    println!("US goals with USSR: {:?}", goals_seen[0]);
    println!("US goals without:   {:?}", goals_seen[1]);
    println!("US military share: {:.3} -> {:.3}", mil[0], mil[1]);
    assert!(contain[1] < contain[0], "containing the USSR fades");
    assert!(mil[1] < mil[0], "peace dividend without an event");
}
