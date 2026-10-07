//! `brink` — headless BRINK runner.
//!
//!   brink run   <scenario.ron> [--seed N] [--turns N] [--json out.json] [--voice]
//!   brink batch   <scenario.ron> [--seeds N] [--turns N] [--threads N]
//!   brink observe <scenario.ron> [--seed N] [--turns N]   (debug: views vs truth)
//!   brink diagnose <scenario.ron> [--seeds N]  (why-not report: transition pressure, wars weighed)
//!   brink military <scenario.ron> [--seeds N]  (calibration dump, JSON lines)

use std::process::ExitCode;

use headless::{run_batch, RunResult};

struct Args {
    command: String,
    scenario: String,
    seed: Option<u64>,
    seeds: u64,
    turns: Option<u32>,
    threads: usize,
    json: Option<String>,
    voice: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let command = it.next().ok_or("missing command (run | batch)")?;
    let scenario = it.next().ok_or("missing scenario path")?;
    let mut args = Args {
        command,
        scenario,
        seed: None,
        seeds: 100,
        turns: None,
        threads: 0,
        json: None,
        voice: false,
    };
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--seed" => args.seed = Some(value()?.parse().map_err(|e| format!("--seed: {e}"))?),
            "--seeds" => args.seeds = value()?.parse().map_err(|e| format!("--seeds: {e}"))?,
            "--turns" => args.turns = Some(value()?.parse().map_err(|e| format!("--turns: {e}"))?),
            "--threads" => args.threads = value()?.parse().map_err(|e| format!("--threads: {e}"))?,
            "--voice" => {
                args.voice = true;
                continue;
            }
            "--json" => args.json = Some(value()?),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(args)
}

fn print_run(r: &RunResult) {
    println!("{} seed={} turns={}", r.scenario, r.seed, r.turns);
    let step = (r.samples.len() / 5).max(1);
    let last = r.samples.len() - 1;
    let shown = (0..r.samples.len()).filter(|i| i % step == 0 || *i == last);
    for s in shown.map(|i| &r.samples[i]) {
        print!("  {:7.2}", s.year);
        for c in &s.countries {
            print!(
                " | {} gdp {:7.2} stab {:5.1} sec {:4.1} debt {:4.2} mil {:6.1} init {}",
                c.code, c.gdp, c.stability, c.security, c.debt_ratio, c.military, c.initiative
            );
        }
        println!();
    }
}

fn print_batch(results: &[RunResult]) {
    let Some(first) = results.first() else { return };
    let runs = results.len() as f64;
    let mean =
        |f: &dyn Fn(&headless::LedgerStats) -> usize| results.iter().map(|r| f(&r.ledger)).sum::<usize>() as f64 / runs;
    println!(
        "ledger per run: entries {:.1}, tests {:.1}, shadows {:.2}, norms created {:.2}, norm tests {:.2} ({:.2} with effect)",
        mean(&|l| l.entries),
        mean(&|l| l.tests_opened),
        mean(&|l| l.shadows_triggered),
        mean(&|l| l.norms_created),
        mean(&|l| l.norm_tests),
        mean(&|l| l.norm_tests_with_effect)
    );
    println!("{}: {} runs x {} turns", first.scenario, results.len(), first.turns);
    let n_countries = first.samples[0].countries.len();
    for i in 0..n_countries {
        let finals: Vec<_> = results
            .iter()
            .map(|r| &r.samples.last().unwrap().countries[i])
            .collect();
        let stat = |f: &dyn Fn(&headless::CountrySample) -> f64| {
            let v: Vec<f64> = finals.iter().map(|c| f(c)).collect();
            let mean = v.iter().sum::<f64>() / v.len() as f64;
            let min = v.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            format!("{mean:7.2} [{min:7.2}..{max:7.2}]")
        };
        println!(
            "  {}  gdp {}  stability {}  debt {}",
            finals[0].code,
            stat(&|c| c.gdp),
            stat(&|c| c.stability),
            stat(&|c| c.debt_ratio)
        );
    }
}

/// Plain-register narration of a campaign: chosen decisions and ledger events, turn by turn.
fn print_voice(r: &RunResult) {
    let codes: Vec<String> = r.samples[0].countries.iter().map(|c| c.code.clone()).collect();
    let names = |id: sim_core::CountryId| codes.get(id.index()).cloned().unwrap_or_else(|| format!("#{}", id.0));
    for turn in 0..r.turns {
        let decisions: Vec<_> = r
            .reasoning
            .iter()
            .filter(|e| {
                e.turn == turn
                    && (e.decision.chosen || matches!(e.decision.kind, sim_core::DecisionKind::AcceptProposal(_)))
                    && e.decision.kind != sim_core::DecisionKind::StayInAlliance
            })
            .collect();
        let ledger: Vec<_> = r.ledger_log.iter().filter(|(t, _)| *t == turn).collect();
        if decisions.is_empty() && ledger.is_empty() && !r.narration.iter().any(|n| n.turn == turn) {
            continue;
        }
        println!("-- turn {turn} --");
        for e in decisions {
            println!("{}", voice::plain_decision(&e.country, &e.decision, 4));
        }
        for (_, line) in ledger {
            if let Some(text) = voice::plain_ledger(&names, line) {
                println!("{text}");
            }
        }
        for n in r.narration.iter().filter(|n| n.turn == turn) {
            println!("  FACT  {}", n.fact);
            if let Some((heading, speaker, text, _)) = &n.line {
                println!("  {heading} — {speaker}: \"{text}\"");
            }
        }
    }
}

/// Debug inspection: every observer's view of every other country, with the
/// hidden truth alongside for comparison.
fn print_observe(state: &sim_core::WorldState) {
    println!(
        "turn {} (year {:.2})  global tension {:.1} (Readiness Condition {})  ledger entries {}",
        state.turn,
        state.year(),
        state.global_tension,
        sim_core::tension::readiness_condition(state.global_tension),
        state.ledger.entries.len()
    );
    for observer in state.ids() {
        let view = sim_core::observe(state, observer);
        println!("{} sees:", view.own.code);
        for f in &view.others {
            let truth = state.country(f.id);
            let band = f.stability_band.map_or("hidden".to_string(), |b| format!("{b:?}"));
            let budget = f.budget.map_or("hidden".to_string(), |b| {
                format!("mil {:.2} (true {:.2})", b.military, truth.budget.military)
            });
            println!(
                "  {}  coverage {:5.1}  military {:6.1} [{:6.1}..{:6.1}] (true {:6.1})  stability {}  budget {}",
                f.code,
                f.coverage,
                f.military.value,
                f.military.low,
                f.military.high,
                truth.power(),
                band,
                budget
            );
            println!(
                "        opinion of us {:6.1}  tension {:5.1}  credibility back {:5.1} threat {:5.1} norm {:5.1}  trust {:5.1}",
                f.their_opinion_of_us, f.tension, f.credibility_back, f.credibility_threat, f.credibility_norm, f.trust
            );
            // Opinion breakdown: what we hold against them and why (own
            // state, not fogged); historical grudges show their fade (D81).
            let ours: Vec<String> = state
                .opinions
                .modifiers(observer, f.id)
                .iter()
                .map(|m| m.to_string())
                .collect();
            if !ours.is_empty() {
                println!("        our opinion: {}", ours.join("; "));
            }
        }
    }
}

fn main() -> ExitCode {
    let result = (|| -> Result<(), String> {
        let args = parse_args()?;
        let def = scenario::load(&args.scenario)?;
        let turns = args.turns.unwrap_or(def.turns);
        match args.command.as_str() {
            "run" => {
                let lines = if args.voice {
                    voice::lines::load_lines("data/voice/lines.ron")?
                } else {
                    Vec::new()
                };
                let settings = args.voice.then_some(headless::VoiceSettings {
                    lines: &lines,
                    intensity: voice::Intensity::Black,
                });
                let r = headless::run_campaign_with(&def, args.seed.unwrap_or(def.seed), turns, settings)?;
                print_run(&r);
                if args.voice {
                    print_voice(&r);
                    if let Some((official, ledger)) = &r.epitaph {
                        println!();
                        if let Some(o) = official {
                            println!("{o}");
                        }
                        for l in ledger {
                            println!("{l}");
                        }
                    }
                }
                if let Some(path) = args.json {
                    let text = serde_json::to_string_pretty(&r).map_err(|e| e.to_string())?;
                    std::fs::write(&path, text).map_err(|e| format!("cannot write {path}: {e}"))?;
                    println!("wrote {path}");
                }
            }
            "batch" => {
                let base = args.seed.unwrap_or(def.seed);
                let seeds: Vec<u64> = (0..args.seeds).map(|i| base + i).collect();
                let threads = if args.threads == 0 {
                    rayon::current_num_threads()
                } else {
                    args.threads
                };
                print_batch(&run_batch(&def, &seeds, turns, threads)?);
            }
            "gate6" => {
                let base = args.seed.unwrap_or(def.seed);
                let seeds: Vec<u64> = (0..args.seeds).map(|i| base + i).collect();
                let threads = if args.threads == 0 {
                    rayon::current_num_threads()
                } else {
                    args.threads
                };
                print_gate6(&run_batch(&def, &seeds, turns, threads)?);
            }
            "diagnose" => {
                let base = args.seed.unwrap_or(def.seed);
                let seeds: Vec<u64> = (0..args.seeds).map(|i| base + i).collect();
                let threads = if args.threads == 0 {
                    rayon::current_num_threads()
                } else {
                    args.threads
                };
                let runs = run_batch(&def, &seeds, turns, threads)?;
                println!("{}", headless::diagnose::transitions(&runs));
                let codes: Vec<String> = def.countries.iter().map(|c| c.id.clone()).collect();
                println!("{}", headless::diagnose::transitions_report(&runs, &codes));
                println!("{}", headless::diagnose::transition_sequences(&runs, &codes, 4));
                println!("{}", headless::diagnose::protection(&runs, &codes));
                println!("{}", headless::diagnose::wars(&runs, 25));
                println!("{}", headless::diagnose::chosen_wars(&runs, &codes));
                println!("{}", headless::diagnose::war_outcomes(&runs, &codes));
                println!("{}", headless::diagnose::war_lengths(&runs, &codes));
                println!("{}", headless::diagnose::collapses(&runs, &codes));
                let powers = headless::stats::gate6_powers(&runs[0]);
                println!("{}", headless::diagnose::interventions(&runs, &powers));
                println!("{}", headless::diagnose::commitments(&runs, &codes, &powers));
                println!("{}", headless::diagnose::sanction_lifecycle(&runs, &codes));
                println!("{}", headless::diagnose::grudges(&runs, &codes));
            }
            "wartrace" => {
                // Turn-by-turn trace of every war (issue 22).
                let base = args.seed.unwrap_or(def.seed);
                let seeds: Vec<u64> = (0..args.seeds).map(|i| base + i).collect();
                let runs = run_batch(&def, &seeds, turns, args.threads.max(1))?;
                let codes: Vec<String> = def.countries.iter().map(|c| c.id.clone()).collect();
                println!("{}", headless::diagnose::war_trace(&runs, &codes));
            }
            "military" => {
                // Calibration dump: one JSON line per seed, samples every 20 turns.
                let base = args.seed.unwrap_or(def.seed);
                let seeds: Vec<u64> = (0..args.seeds).map(|i| base + i).collect();
                let threads = if args.threads == 0 {
                    rayon::current_num_threads()
                } else {
                    args.threads
                };
                for (seed, r) in seeds.iter().zip(run_batch(&def, &seeds, turns, threads)?) {
                    let picks: Vec<_> = r.samples.iter().filter(|s| s.turn % 20 == 0).collect();
                    let line = serde_json::json!({ "seed": seed, "samples": picks });
                    println!("{line}");
                }
            }
            "security" => {
                // Debug: Security driver breakdown for every country at the end of a run.
                let r = headless::run_campaign_with(&def, args.seed.unwrap_or(def.seed), turns, None)?;
                let state: sim_core::WorldState = serde_json::from_str(&r.final_state).map_err(|e| e.to_string())?;
                println!("{}", headless::diagnose::SECURITY_LEGEND);
                for id in state.ids() {
                    println!("{}", headless::diagnose::security_line(&state, id));
                }
            }
            "observe" => {
                let mut state = scenario::build(&def, args.seed)?;
                for _ in 0..args.turns.unwrap_or(0) {
                    let idle = state
                        .ids()
                        .map(|id| sim_core::OrderSet {
                            country: id,
                            orders: Vec::new(),
                        })
                        .collect();
                    sim_core::resolve_turn(&mut state, idle);
                }
                print_observe(&state);
            }
            other => return Err(format!("unknown command {other}")),
        }
        Ok(())
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Gate 6 report: behaviour frequencies for the major powers (power share
/// ≥ 5% at start) and the measurable plausibility targets.
fn print_gate6(runs: &[RunResult]) {
    use headless::stats::{gate6_band, gate6_table, in_gate6_band, plausibility, BEHAVIOURS, INFO_ROWS};
    let powers = headless::stats::gate6_powers(&runs[0]);
    let powers: Vec<&str> = powers.iter().map(String::as_str).collect();
    let (per, pooled) = gate6_table(runs, &powers);
    println!("Gate 6 over {} runs; powers: {}", runs.len(), powers.join(" "));
    print!("{:<32}", "behaviour");
    for p in &powers {
        print!("{p:>6}");
    }
    println!("  pooled");
    for b in BEHAVIOURS.iter().chain(&INFO_ROWS).copied() {
        print!("{b:<32}");
        for p in &powers {
            print!("{:>5.0}%", 100.0 * per.get(&(p.to_string(), b)).copied().unwrap_or(0.0));
        }
        let v = pooled.get(b).copied().unwrap_or(0.0);
        let flag = match gate6_band(b) {
            None => "  (info, not a gate criterion)".to_string(),
            Some((lo, hi)) if !in_gate6_band(b, v) => {
                format!("  <-- outside {:.0}-{:.0}%", 100.0 * lo, 100.0 * hi)
            }
            Some(_) => String::new(),
        };
        println!("  {:>5.1}%{flag}", 100.0 * v);
    }
    let n = runs.len() as f64;
    let ps: Vec<_> = runs.iter().map(plausibility).collect();
    let wars: f64 = ps.iter().map(|p| p.wars as f64).sum::<f64>() / n;
    let share = |pair: (&str, &str)| {
        let key = if pair.0 < pair.1 {
            (pair.0.to_string(), pair.1.to_string())
        } else {
            (pair.1.to_string(), pair.0.to_string())
        };
        ps.iter().filter(|p| p.war_pairs.contains(&key)).count() as f64 / n
    };
    let any_regime = ps.iter().filter(|p| p.regime_changes > 0).count() as f64 / n;
    let swing = ps.iter().filter(|p| p.price_swing >= 1.5).count() as f64 / n;
    println!(
        "oil price swing of 50%+ in {:.0}% of runs (target > 70%)",
        100.0 * swing
    );
    let nuclear = ps.iter().filter(|p| p.nuclear_uses > 0).count() as f64 / n;
    println!("any nuclear use in {:.1}% of runs (target 1-5%)", 100.0 * nuclear);
    // Wars declared on arsenal states, by aim (D68 #5): Major must stay
    // nearly unthinkable; Limited/Punitive rare but possible.
    let mut on_arsenal: std::collections::BTreeMap<&str, std::collections::BTreeMap<String, usize>> =
        Default::default();
    for p in &ps {
        for (aim, pairs) in &p.wars_on_arsenal {
            for (a, b) in pairs {
                *on_arsenal
                    .entry(aim.as_str())
                    .or_default()
                    .entry(format!("{a}-{b}"))
                    .or_insert(0) += 1;
            }
        }
    }
    for aim in ["Punitive", "Limited", "Major"] {
        let pairs = on_arsenal.get(aim).cloned().unwrap_or_default();
        let total: usize = pairs.values().sum();
        let detail: Vec<String> = pairs.iter().map(|(k, v)| format!("{k} {v}")).collect();
        println!(
            "wars on arsenal states, {aim}: {total} in {} runs ({:.1}% of runs){}",
            ps.iter()
                .filter(|p| p.wars_on_arsenal.get(aim).is_some_and(|v| !v.is_empty()))
                .count(),
            100.0
                * ps.iter()
                    .filter(|p| p.wars_on_arsenal.get(aim).is_some_and(|v| !v.is_empty()))
                    .count() as f64
                / n,
            if detail.is_empty() {
                String::new()
            } else {
                format!(": {}", detail.join(", "))
            }
        );
    }
    println!("wars per run {wars:.2}; Iran-Iraq war {:.0}% (target 50-80%); US-USSR war {:.0}% (target <10%); any regime change by war {:.0}%",
        100.0 * share(("IRN", "IRQ")), 100.0 * share(("USA", "SOV")), 100.0 * any_regime);
    // Directed declarations by aim (Osirak-type strikes are Punitive).
    let mut by_aim: std::collections::BTreeMap<String, usize> = Default::default();
    let mut directed: std::collections::BTreeMap<String, usize> = Default::default();
    for p in &ps {
        for (a, b, aim) in &p.declared {
            *by_aim.entry(aim.clone()).or_insert(0) += 1;
            *directed.entry(format!("{a}>{b} {}", &aim[..1])).or_insert(0) += 1;
        }
    }
    let aims: Vec<String> = by_aim
        .iter()
        .map(|(k, v)| format!("{k} {:.2}", *v as f64 / n))
        .collect();
    println!("declarations per run by aim: {}", aims.join(", "));
    let mut d: Vec<_> = directed.into_iter().collect();
    d.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    let d: Vec<String> = d.iter().take(14).map(|(k, v)| format!("{k} {v}")).collect();
    println!(
        "declarations (attacker>defender aim, total over runs): {}",
        d.join(", ")
    );
    let all: usize = ps.iter().map(|p| p.declared.len()).sum();
    let mut prot: std::collections::BTreeMap<String, usize> = Default::default();
    for p in &ps {
        for (a, by) in &p.protected_attackers {
            *prot.entry(format!("{a}({})", by.join("+"))).or_insert(0) += 1;
        }
    }
    let k: usize = prot.values().sum();
    let mut pv: Vec<_> = prot.into_iter().collect();
    pv.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    let pv: Vec<String> = pv.iter().take(8).map(|(a, v)| format!("{a} {v}")).collect();
    println!(
        "declarations by an attacker under a pledge (F3): {k} of {all}: {}",
        pv.join(", ")
    );
    let turns: usize = ps.iter().map(|p| p.sampled_turns).sum();
    let mut irq: std::collections::BTreeMap<String, usize> = Default::default();
    for p in &ps {
        for ((c, pr), v) in &p.protection_turns {
            if c == "IRQ" {
                *irq.entry(pr.clone()).or_insert(0) += v;
            }
        }
    }
    let irq: Vec<String> = irq
        .iter()
        .map(|(k, v)| format!("{k} {:.1}%", 100.0 * *v as f64 / turns.max(1) as f64))
        .collect();
    println!(
        "IRQ protected by (share of sampled turns): {}",
        if irq.is_empty() {
            "none".to_string()
        } else {
            irq.join(", ")
        }
    );
    let mut pairs: std::collections::BTreeMap<(String, String), usize> = Default::default();
    for p in &ps {
        for k in &p.war_pairs {
            *pairs.entry(k.clone()).or_insert(0) += 1;
        }
    }
    let mut v: Vec<_> = pairs.into_iter().collect();
    v.sort_by_key(|x| std::cmp::Reverse(x.1));
    for ((a, b), k) in v.into_iter().take(10) {
        println!("  war {a}-{b}: {:.0}% of runs", 100.0 * k as f64 / n);
    }
}
