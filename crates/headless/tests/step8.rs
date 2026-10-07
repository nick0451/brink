//! Build step 8 acceptance tests: strategic reassessment → Gate 5
//! "post-rival world" (DESIGN §22.1, us-mechanics-report §7).
//!
//! The rival is "deleted" by ordinary state edits (its forces and economy
//! collapse, its bloc dissolves, old grudges fade). No separate ruleset
//! exists: everything after that is the same simulation and the same AI.

use std::collections::BTreeMap;
use std::path::PathBuf;

use ai::goals::Goal;
use ai::{Controller, Strategist};
use sim_core::{observe, resolve_turn, CountryId, DecisionKind, OrderSet, TreatyKind, WorldState};

fn fixture() -> scenario::ScenarioDef {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures/post_rival.ron");
    scenario::load(path).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

/// The rival collapses: 90% of its forces and half its economy gone, its
/// bloc tag dissolved, its historical grudges dropped.
fn delete_rival(w: &mut WorldState) {
    let riv = id(w, "RIV");
    let c = w.country_mut(riv);
    let strength = c.forces.strength();
    let mix = c.force_mix;
    let quality = c.military_tech as f64;
    c.forces = sim_core::Forces::new(strength * 0.1, mix, quality);
    c.gdp *= 0.5;
    c.alignment = None;
    c.budget_target.military = 0.10;
    let ids: Vec<CountryId> = w.ids().collect();
    for o in ids {
        w.opinions.remove_source(o, riv, "historical relations");
        w.opinions.remove_source(riv, o, "historical relations");
    }
    sim_core::tension::initialize(w);
}

/// The control arm keeps a live rivalry. Since D81 a scenario's starting
/// hostility is a fading memory (1.5 points a year), so without this the
/// "with rival" arm would itself drift toward detente and the test would
/// measure the fade, not the peace dividend from deleting the rival. Here
/// the rival stays a standing enemy: its hostility toward the Hegemon's
/// bloc is held as an ordinary (non-fading) opinion, as before D81.
fn keep_rivalry_live(w: &mut WorldState) {
    let riv = id(w, "RIV");
    let ids: Vec<CountryId> = w.ids().collect();
    for o in ids {
        for (a, b) in [(o, riv), (riv, o)] {
            for m in w.opinions.modifiers_mut(a, b) {
                m.memory = None;
            }
        }
    }
}

/// What the Hegemon did over a campaign.
#[derive(Debug, Default)]
struct Run {
    goals: BTreeMap<String, u32>,
    mil_start: f64,
    mil_end: f64,
    commitments_start: usize,
    commitments_end: usize,
    deep_trade_start: usize,
    deep_trade_end: usize,
    sanctions: usize,
    involvements: usize,
    alliance_warnings: u32,
    stability_end: f64,
}

fn commitments(w: &WorldState, c: CountryId) -> usize {
    w.diplomacy
        .treaties
        .iter()
        .filter(|t| match t.kind {
            TreatyKind::DefensiveAlliance => t.involves(c),
            TreatyKind::Guarantee => t.a == c,
            _ => false,
        })
        .count()
        + w.diplomacy.streams.iter().filter(|s| s.from == c).count()
}

fn deep_trade(w: &WorldState, c: CountryId) -> usize {
    w.diplomacy
        .treaties
        .iter()
        .filter(|t| t.kind == (TreatyKind::Trade { deep: true }) && t.involves(c))
        .count()
}

fn campaign(seed: u64, delete: bool, turns: u32) -> Run {
    let mut w = scenario::build(&fixture(), Some(seed)).unwrap();
    if delete {
        delete_rival(&mut w);
    } else {
        keep_rivalry_live(&mut w);
    }
    let heg = id(&w, "HEG");
    let mut ais: Vec<Strategist> = w.ids().map(|_| Strategist::with_seed(seed)).collect();
    let mut run = Run {
        mil_start: w.country(heg).budget_target.military,
        commitments_start: commitments(&w, heg),
        deep_trade_start: deep_trade(&w, heg),
        ..Default::default()
    };
    for _ in 0..turns {
        let views: Vec<_> = w.ids().map(|c| observe(&w, c)).collect();
        let mut orders = Vec::new();
        for (view, ai) in views.iter().zip(ais.iter_mut()) {
            let d = ai.decide(view);
            if view.observer == heg {
                for r in &d.records {
                    if r.kind == DecisionKind::StayInAlliance && r.subject.contains("bad assessments: 1") {
                        run.alliance_warnings += 1;
                    }
                }
            }
            orders.push(OrderSet {
                country: view.observer,
                orders: d.orders,
            });
        }
        let report = resolve_turn(&mut w, orders);
        for e in &report.events {
            match e {
                sim_core::DiplomaticEvent::SanctionImposed { by, .. } if *by == heg => run.sanctions += 1,
                sim_core::DiplomaticEvent::JoinedWar { country, .. } if *country == heg => run.involvements += 1,
                sim_core::DiplomaticEvent::StreamStarted { stream }
                    if stream.from == heg && stream.kind == sim_core::StreamKind::Arms =>
                {
                    run.involvements += 1;
                }
                _ => {}
            }
        }
        for g in &ais[heg.index()].goals {
            let key = match g {
                Goal::Contain(_) => "contain",
                Goal::ProtectClients => "protect clients",
                Goal::DevelopEconomy => "develop",
                Goal::LeadBloc => "lead bloc",
                Goal::Primacy => "primacy",
                Goal::BalanceHegemon(_) => "balance",
                Goal::RestoreStability => "stability",
                Goal::Exploit(_) => "exploit",
            };
            *run.goals.entry(key.to_string()).or_insert(0) += 1;
        }
    }
    run.mil_end = w.country(heg).budget_target.military;
    run.commitments_end = commitments(&w, heg);
    run.deep_trade_end = deep_trade(&w, heg);
    run.stability_end = w.country(heg).stability;
    run
}

/// The Hegemon's strategy, read from behaviour (us-mechanics-report §7).
fn strategy(r: &Run) -> &'static str {
    let mil = r.mil_end / r.mil_start;
    let more_commitments = r.commitments_end > r.commitments_start;
    let fewer_commitments = r.commitments_end < r.commitments_start;
    let integrated = r.deep_trade_end > r.deep_trade_start;
    if r.involvements > 0 || r.sanctions > 0 {
        "selective intervention"
    } else if more_commitments {
        "coalition leadership"
    } else if r.mil_end >= 0.2 || mil >= 0.9 {
        // Keeps a large military with no threat that justifies it.
        "primacy"
    } else if fewer_commitments {
        "retrenchment"
    } else if integrated {
        "integration"
    } else {
        "retrenchment"
    }
}

fn goal_share(runs: &[Run], key: &str) -> f64 {
    let total: u32 = runs.iter().flat_map(|r| r.goals.values()).sum();
    let k: u32 = runs.iter().map(|r| r.goals.get(key).copied().unwrap_or(0)).sum();
    k as f64 / total.max(1) as f64
}

/// GATE 5 (part met) — no separate ruleset: when the rival is deleted the
/// Hegemon's goal mix shifts away from containment and defence-burden
/// pressure cuts its military share, through ordinary terms only.
#[test]
fn gate5_goal_mix_shifts_and_peace_dividend_emerges() {
    let seeds: Vec<u64> = (1..=60).collect();
    let with: Vec<Run> = seeds.iter().map(|&s| campaign(s, false, 40)).collect();
    let without: Vec<Run> = seeds.iter().map(|&s| campaign(s, true, 40)).collect();
    let mean = |runs: &[Run], f: &dyn Fn(&Run) -> f64| runs.iter().map(f).sum::<f64>() / runs.len() as f64;
    println!(
        "contain {:.2} -> {:.2}; develop {:.2} -> {:.2}; HEG military share {:.3} -> {:.3}",
        goal_share(&with, "contain"),
        goal_share(&without, "contain"),
        goal_share(&with, "develop"),
        goal_share(&without, "develop"),
        mean(&with, &|r| r.mil_end),
        mean(&without, &|r| r.mil_end)
    );
    assert!(goal_share(&without, "contain") < goal_share(&with, "contain") - 0.2);
    assert!(goal_share(&without, "develop") > goal_share(&with, "develop") + 0.2);
    assert!(mean(&without, &|r| r.mil_end) < mean(&with, &|r| r.mil_end) - 0.1);
}

/// GATE 5 (KNOWN GAP, 2026-10-04) — strategy variety and alliance warning
/// windows. In this 6-actor fixture deterrence always holds, no crisis ever
/// forces a choice, and the Hegemon's post-rival behaviour has two clusters
/// (integration ~92%, primacy ~8%); alliance scores soften to −6..+4 but
/// never cross the −10 exit threshold. Re-run on the 16-country 1980
/// scenario (step 9), where crises can come from many sources.
#[test]
#[ignore]
fn gate5_post_rival_world() {
    let seeds: Vec<u64> = (1..=200).collect();
    let with: Vec<Run> = seeds.iter().map(|&s| campaign(s, false, 40)).collect();
    let without: Vec<Run> = seeds.iter().map(|&s| campaign(s, true, 40)).collect();

    println!("goal share       with rival   rival deleted");
    for key in [
        "contain",
        "protect clients",
        "develop",
        "lead bloc",
        "primacy",
        "balance",
        "stability",
        "exploit",
    ] {
        println!(
            "  {key:<16} {:>8.1}%   {:>8.1}%",
            100.0 * goal_share(&with, key),
            100.0 * goal_share(&without, key)
        );
    }
    let mean = |runs: &[Run], f: &dyn Fn(&Run) -> f64| runs.iter().map(f).sum::<f64>() / runs.len() as f64;
    println!(
        "HEG military share end: with {:.3}  deleted {:.3}",
        mean(&with, &|r| r.mil_end),
        mean(&without, &|r| r.mil_end)
    );
    let mut dist: BTreeMap<&str, usize> = BTreeMap::new();
    for r in &without {
        *dist.entry(strategy(r)).or_insert(0) += 1;
    }
    let mut dist_with: BTreeMap<&str, usize> = BTreeMap::new();
    for r in &with {
        *dist_with.entry(strategy(r)).or_insert(0) += 1;
    }
    println!("strategies with rival: {dist_with:?}");
    println!("strategies after deletion: {dist:?}");
    let mut clusters: BTreeMap<String, usize> = BTreeMap::new();
    for r in &without {
        let key = format!(
            "mil {:.2} commit {:+} deep {:+} primacy-goal {}",
            (r.mil_end * 50.0).round() / 50.0,
            r.commitments_end as i64 - r.commitments_start as i64,
            r.deep_trade_end as i64 - r.deep_trade_start as i64,
            r.goals.get("primacy").copied().unwrap_or(0) > 10
        );
        *clusters.entry(key).or_insert(0) += 1;
    }
    for (k, v) in &clusters {
        println!("  {v:4}  {k}");
    }
    let warned = without.iter().filter(|r| r.alliance_warnings > 0).count();
    println!("runs with an alliance warning window: {warned}/{}", without.len());

    // The goal mix shifts away from containing the rival.
    assert!(goal_share(&without, "contain") < goal_share(&with, "contain") - 0.1);
    // Peace-dividend pressure: the military share falls without any event.
    assert!(mean(&without, &|r| r.mil_end) < mean(&with, &|r| r.mil_end) - 0.02);
    // At least 3 strategies above 10%, none above 60%.
    let n = without.len() as f64;
    let common = dist.values().filter(|&&k| k as f64 / n > 0.10).count();
    assert!(common >= 3, "{dist:?}");
    assert!(dist.values().all(|&k| k as f64 / n <= 0.60), "{dist:?}");
    // Alliances wobble, with a visible warning window, in some runs.
    assert!(warned > 0);
}

#[test]
#[ignore]
fn debug_post_rival_run() {
    let seed = 3;
    let mut w = scenario::build(&fixture(), Some(seed)).unwrap();
    delete_rival(&mut w);
    let mut ais: Vec<Strategist> = w.ids().map(|_| Strategist::with_seed(seed)).collect();
    for turn in 0..40 {
        let views: Vec<_> = w.ids().map(|c| observe(&w, c)).collect();
        let mut orders = Vec::new();
        for (view, ai) in views.iter().zip(ais.iter_mut()) {
            let d = ai.decide(view);
            for r in &d.records {
                let interesting = r.chosen || r.subject.starts_with("declare") || r.subject.starts_with("strategic");
                if interesting && !r.subject.starts_with("stay in") && !r.subject.starts_with("keep guarantee") {
                    println!("t{turn:02} {} {} ({:+.1})", view.own.code, r.subject, r.score);
                    if r.subject.starts_with("declare") {
                        for l in &r.lines {
                            println!("        {:+6.1} {}", l.value, l.label);
                        }
                    }
                }
            }
            orders.push(OrderSet {
                country: view.observer,
                orders: d.orders,
            });
        }
        resolve_turn(&mut w, orders);
    }
    for c in &w.countries {
        println!(
            "{} mil {:.1} budget {:.2} stab {:.1}",
            c.code,
            c.forces.strength(),
            c.budget.military,
            c.stability
        );
    }
    for t in &w.diplomacy.treaties {
        println!("treaty {:?} {} {}", t.kind, w.country(t.a).code, w.country(t.b).code);
    }
    for s in &w.diplomacy.streams {
        println!(
            "stream {} -> {} {:?}",
            w.country(s.from).code,
            w.country(s.to).code,
            s.kind
        );
    }
}

#[test]
#[ignore]
fn debug_goal_table() {
    for seed in 1..=6 {
        let mut w = scenario::build(&fixture(), Some(seed)).unwrap();
        delete_rival(&mut w);
        let heg = id(&w, "HEG");
        let view = observe(&w, heg);
        print!("seed {seed}:");
        for (g, s) in ai::goals::assess(&view) {
            print!("  {} {:.1}", g.label(&view), s.total());
        }
        println!("   pers {:?}", view.own.personality);
    }
}

#[test]
#[ignore]
fn debug_alliance_scores() {
    let seed = 3;
    let mut w = scenario::build(&fixture(), Some(seed)).unwrap();
    delete_rival(&mut w);
    let mut ais: Vec<Strategist> = w.ids().map(|_| Strategist::with_seed(seed)).collect();
    for turn in 0..12 {
        let views: Vec<_> = w.ids().map(|c| observe(&w, c)).collect();
        let mut orders = Vec::new();
        for (view, ai) in views.iter().zip(ais.iter_mut()) {
            let d = ai.decide(view);
            for r in d.records.iter().filter(|r| r.subject.starts_with("stay in")) {
                println!("t{turn} {} {} {:+.1}", view.own.code, r.subject, r.score);
                for l in &r.lines {
                    println!("      {:+6.1} {}", l.value, l.label);
                }
            }
            orders.push(OrderSet {
                country: view.observer,
                orders: d.orders,
            });
        }
        resolve_turn(&mut w, orders);
    }
}
