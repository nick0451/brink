//! "Why not?" instrumentation for the crisis system (`brink diagnose`).
//!
//! Before changing any value: for every regime, how close it came to a
//! transition crisis and which drivers held it up; for every war the AI
//! considered, the terms that pushed for and against it. Diagnosis, not
//! knob-tuning.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::RunResult;

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Transition pressure per non-democratic regime, over every sampled turn.
pub fn transitions(runs: &[RunResult]) -> String {
    use sim_core::transition::{CRISIS_LEGITIMACY, CRISIS_STABILITY};
    #[derive(Default)]
    struct Acc {
        turns: usize,
        crisis: usize,
        stab: Vec<f64>,
        legit: Vec<f64>,
        prosperity: Vec<f64>,
        security: Vec<f64>,
        burden: Vec<f64>,
        weariness: Vec<f64>,
        growth: Vec<f64>,
        behind: Vec<f64>,
        min_stab: Vec<f64>,
        min_legit: Vec<f64>,
    }
    let mut acc: BTreeMap<String, Acc> = BTreeMap::new();
    for r in runs {
        let mut mins: BTreeMap<String, (f64, f64)> = BTreeMap::new();
        for s in &r.samples {
            for c in s.countries.iter().filter(|c| c.active && !c.democracy && c.gdp > 0.0) {
                let a = acc.entry(c.code.clone()).or_default();
                a.turns += 1;
                if c.stability < CRISIS_STABILITY && c.legitimacy < CRISIS_LEGITIMACY {
                    a.crisis += 1;
                }
                a.stab.push(c.stability);
                a.legit.push(c.legitimacy);
                a.prosperity.push(c.prosperity);
                a.security.push(c.security);
                a.burden.push(c.burden);
                a.weariness.push(c.war_weariness);
                a.growth.push(400.0 * c.growth);
                a.behind.push(c.behind);
                let m = mins.entry(c.code.clone()).or_insert((f64::MAX, f64::MAX));
                m.0 = m.0.min(c.stability);
                m.1 = m.1.min(c.legitimacy);
            }
        }
        for (code, (s, l)) in mins {
            let a = acc.entry(code).or_default();
            a.min_stab.push(s);
            a.min_legit.push(l);
        }
    }
    let mut rows: Vec<_> = acc.into_iter().collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.1.crisis * 1000 / r.1.turns.max(1)));
    let mut out = String::new();
    let _ = writeln!(
        out,
        "TRANSITION PRESSURE (non-democracies; crisis = stability < {CRISIS_STABILITY} and legitimacy < {CRISIS_LEGITIMACY})"
    );
    let _ = writeln!(
        out,
        "{:4} {:>7} | {:>5} {:>5} | {:>6} {:>6} | {:>5} {:>5} {:>6} {:>6} {:>5} {:>6} | legitimacy target = .65 prosp + .35 sec - burden - behind (+ rally)",
        "", "crisis%", "stab", "legit", "minSt", "minLg", "prosp", "sec", "burden", "behind", "weary", "grow%"
    );
    for (code, a) in rows {
        let (p, sec, b, bh) = (
            median(a.prosperity),
            median(a.security),
            median(a.burden),
            median(a.behind),
        );
        let k = sim_core::domestic::SECURITY_LEGITIMACY;
        let _ = writeln!(
            out,
            "{code:4} {:>6.1}% | {:>5.1} {:>5.1} | {:>6.1} {:>6.1} | {:>5.1} {:>5.1} {:>6.1} {:>6.1} {:>5.1} {:>6.1} | {:.1} = {:.1} + {:.1} - {:.1} - {:.1}",
            100.0 * a.crisis as f64 / a.turns.max(1) as f64,
            median(a.stab),
            median(a.legit),
            median(a.min_stab),
            median(a.min_legit),
            p,
            sec,
            b,
            bh,
            median(a.weariness),
            median(a.growth),
            (1.0 - k) * p + k * sec - b - bh,
            (1.0 - k) * p,
            k * sec,
            b,
            bh
        );
    }
    out
}

/// Every war the AI weighed: how often, how often chosen, and the mean of
/// each reasoning term (the strongest for and against).
pub fn wars(runs: &[RunResult], top: usize) -> String {
    #[derive(Default)]
    struct Acc {
        n: usize,
        chosen: usize,
        score: f64,
        terms: BTreeMap<String, f64>,
        /// Which aim won the comparison each time (the record keeps the best).
        aims: BTreeMap<String, usize>,
    }
    let mut acc: BTreeMap<String, Acc> = BTreeMap::new();
    for r in runs {
        for e in &r.reasoning {
            let d = &e.decision;
            // "declare {aim} war on {code}"; "declare our arsenal" is not a war.
            let Some(rest) = d.subject.strip_prefix("declare ").filter(|r| r.contains(" war on ")) else {
                continue;
            };
            let target = rest.rsplit(' ').next().unwrap_or("?");
            let aim = rest.split(' ').next().unwrap_or("?").to_string();
            let a = acc.entry(format!("{} -> {target}", e.country)).or_default();
            a.n += 1;
            a.chosen += usize::from(d.chosen);
            a.score += d.score;
            *a.aims.entry(aim).or_default() += 1;
            for l in &d.lines {
                *a.terms.entry(l.label.clone()).or_default() += l.value;
            }
        }
    }
    let mut rows: Vec<_> = acc.into_iter().collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.1.n));
    let mut out = String::new();
    let _ = writeln!(out, "WARS CONSIDERED (best target per assessment; mean term values)");
    for (pair, a) in rows.into_iter().take(top) {
        let n = a.n as f64;
        let mut terms: Vec<(String, f64)> = a.terms.into_iter().map(|(k, v)| (k, v / n)).collect();
        terms.sort_by(|x, y| y.1.total_cmp(&x.1));
        let pro: Vec<String> = terms
            .iter()
            .filter(|t| t.1 > 0.0)
            .take(3)
            .map(|t| format!("{} {:+.1}", t.0, t.1))
            .collect();
        let con: Vec<String> = terms
            .iter()
            .rev()
            .filter(|t| t.1 < 0.0)
            .take(3)
            .map(|t| format!("{} {:+.1}", t.0, t.1))
            .collect();
        let aims: Vec<String> = a.aims.iter().map(|(k, v)| format!("{k} {v}")).collect();
        let _ = writeln!(
            out,
            "{pair:11} considered {:4} chosen {:3} mean {:+6.1} best aim {{{}}} | for: {} | against: {}",
            a.n,
            a.chosen,
            a.score / n,
            aims.join(", "),
            pro.join("; "),
            con.join("; ")
        );
    }
    out
}

/// Transition events by country and kind, and in how many runs each country
/// had any: spread matters more than the total (one state transitioning in
/// every run while nobody else ever does is a broken model that passes a
/// coarse "transition somewhere" test).
pub fn transition_events(runs: &[RunResult], codes: &[String]) -> BTreeMap<String, (BTreeMap<String, u32>, u32)> {
    let mut out: BTreeMap<String, (BTreeMap<String, u32>, u32)> = BTreeMap::new();
    for r in runs {
        let mut seen = std::collections::BTreeSet::new();
        for (_, e) in &r.events {
            if let sim_core::DiplomaticEvent::Transition { country, kind, .. } = e {
                let code = codes[country.index()].clone();
                let entry = out.entry(code.clone()).or_default();
                *entry.0.entry(format!("{kind:?}")).or_default() += 1;
                seen.insert(code);
            }
        }
        for code in seen {
            out.entry(code).or_default().1 += 1;
        }
    }
    out
}

pub fn transitions_report(runs: &[RunResult], codes: &[String]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "TRANSITIONS (events by kind; runs with any)");
    for (code, (kinds, runs_with)) in transition_events(runs, codes) {
        let _ = writeln!(out, "{code:4} in {runs_with:3}/{} runs: {kinds:?}", runs.len());
    }
    out
}

/// Transition sequences per country (`K` crackdown, `C` coup, `R` reform,
/// `X` collapse, each with its turn), a few sample runs, and the mean terms
/// of the regime's answers (reform score; positive favours reform). Repeated
/// identical cycles (K K C K K C ...) are the failure this exposes.
pub fn transition_sequences(runs: &[RunResult], codes: &[String], samples: usize) -> String {
    let mut seqs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut cycles: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for r in runs {
        let mut per: BTreeMap<String, Vec<(u32, char)>> = BTreeMap::new();
        for (t, e) in &r.events {
            if let sim_core::DiplomaticEvent::Transition { country, kind, .. } = e {
                let k = match kind {
                    sim_core::TransitionKind::Crackdown => 'K',
                    sim_core::TransitionKind::Coup => 'C',
                    sim_core::TransitionKind::Reform => 'R',
                    sim_core::TransitionKind::Collapse => 'X',
                };
                per.entry(codes[country.index()].clone()).or_default().push((*t, k));
            }
        }
        for (code, v) in per {
            let coups = v.iter().filter(|x| x.1 == 'C').count() as u32;
            let e = cycles.entry(code.clone()).or_default();
            // A "repeat coup cycle": more than one coup in the same campaign.
            e.0 += u32::from(coups >= 2);
            e.1 += 1;
            let s: Vec<String> = v.iter().map(|(t, k)| format!("{k}{t}")).collect();
            seqs.entry(code).or_default().push(s.join(" "));
        }
    }
    let mut terms: BTreeMap<String, (usize, BTreeMap<String, f64>)> = BTreeMap::new();
    for r in runs {
        for e in &r.reasoning {
            let d = &e.decision;
            if d.subject != "reform the regime" && d.subject != "crack down" {
                continue;
            }
            let sign = if d.subject == "crack down" { -1.0 } else { 1.0 };
            let a = terms.entry(e.country.clone()).or_default();
            a.0 += 1;
            for l in &d.lines {
                *a.1.entry(l.label.clone()).or_default() += sign * l.value;
            }
        }
    }
    let mut out = String::new();
    let _ = writeln!(
        out,
        "TRANSITION SEQUENCES (K crackdown, C coup, R reform, X collapse; turn)"
    );
    for (code, v) in &seqs {
        let (rep, n) = cycles[code];
        let _ = writeln!(out, "{code:4} runs with 2+ coups: {rep}/{n}");
        for s in v.iter().take(samples) {
            let _ = writeln!(out, "     {s}");
        }
        if let Some((k, t)) = terms.get(code) {
            let n = *k as f64;
            let line: Vec<String> = t.iter().map(|(l, v)| format!("{l} {:+.1}", v / n)).collect();
            let _ = writeln!(out, "     reform score terms over {k} answers: {}", line.join("; "));
        }
    }
    out
}

/// Every war that ended: who started it, its aim, who won, why, and how
/// long it ran (turns), aggregated per (attacker -> defender).
/// Turns after a peace within which a new war between the same pair counts
/// as a re-declaration (the same quarrel resumed, not a new one).
pub const REDECLARE_WINDOW: u32 = 20;

pub fn war_outcomes(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::DiplomaticEvent as E;
    // Per pair: wars ended, outcome counts, durations, sign changes of the
    // front per war (border chatter, D65), re-declarations within
    // REDECLARE_WINDOW turns of a peace.
    #[derive(Default)]
    struct Outcomes {
        ended: u32,
        kinds: BTreeMap<String, u32>,
        durations: Vec<f64>,
        crossings: Vec<f64>,
        redeclared: u32,
        /// Any later war between the same pair in the same run.
        resumed: u32,
    }
    let mut acc: BTreeMap<String, Outcomes> = BTreeMap::new();
    let mut joins: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
    for r in runs {
        let mut started: BTreeMap<u32, u32> = BTreeMap::new();
        let mut pair: BTreeMap<u32, String> = BTreeMap::new();
        let mut progress: BTreeMap<u32, Vec<f64>> = BTreeMap::new();
        // Unordered pair -> turn of the last peace between them.
        let mut last_peace: BTreeMap<(usize, usize), u32> = BTreeMap::new();
        let unordered = |a: &sim_core::CountryId, b: &sim_core::CountryId| {
            let (x, y) = (a.index(), b.index());
            if x < y {
                (x, y)
            } else {
                (y, x)
            }
        };
        for (t, e) in &r.events {
            match e {
                E::WarDeclared {
                    war,
                    attacker,
                    defender,
                    ..
                } => {
                    started.insert(war.0, *t);
                    let key = format!("{} -> {}", codes[attacker.index()], codes[defender.index()]);
                    if let Some(p) = last_peace.get(&unordered(attacker, defender)) {
                        let a = acc.entry(key.clone()).or_default();
                        a.resumed += 1;
                        if *t <= p + REDECLARE_WINDOW {
                            a.redeclared += 1;
                        }
                    }
                    pair.insert(war.0, key);
                }
                E::FrontReport { war, progress: p, .. } => {
                    progress.entry(war.0).or_default().push(*p);
                }
                E::JoinedWar { war, country, side, .. } => {
                    let key = pair.get(&war.0).cloned().unwrap_or_default();
                    *joins
                        .entry(key)
                        .or_default()
                        .entry(format!("{} {side:?}", codes[country.index()]))
                        .or_default() += 1;
                }
                E::PeaceMade {
                    war,
                    attacker,
                    defender,
                    aim,
                    result,
                    reason,
                    ..
                } => {
                    let key = format!("{} -> {}", codes[attacker.index()], codes[defender.index()]);
                    let a = acc.entry(key).or_default();
                    a.ended += 1;
                    *a.kinds.entry(format!("{aim:?} {result:?} {reason:?}")).or_default() += 1;
                    if let Some(s) = started.get(&war.0) {
                        a.durations.push(f64::from(t - s));
                    }
                    let path = progress.remove(&war.0).unwrap_or_default();
                    a.crossings
                        .push(path.windows(2).filter(|w| (w[0] < 0.0) != (w[1] < 0.0)).count() as f64);
                    last_peace.insert(unordered(attacker, defender), *t);
                }
                _ => {}
            }
        }
    }
    let mut out = String::new();
    let _ = writeln!(
        out,
        "WAR OUTCOMES (ended wars; aim result reason: count; median turns; front sign changes per war: total, median, max; re-declarations within {REDECLARE_WINDOW} turns of a peace)"
    );
    for (pair, a) in acc {
        let _ = writeln!(
            out,
            "{pair:11} {:3} ended, median {:>4.1} turns, crossings {:3} total {:>3.1} median {:>3.0} max, redeclared {:2} ({:2} resumed ever): {:?}",
            a.ended,
            median(a.durations),
            a.crossings.iter().sum::<f64>(),
            median(a.crossings.clone()),
            a.crossings.iter().copied().fold(0.0, f64::max),
            a.redeclared,
            a.resumed,
            a.kinds
        );
        if let Some(j) = joins.get(&pair) {
            let _ = writeln!(out, "{:11} joined: {j:?}", "");
        }
    }
    out
}

/// Every disorderly collapse: was the country at war, and what drove its
/// Stability down (median of the drivers on the turn before).
pub fn collapses(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::transition::TransitionKind;
    use sim_core::DiplomaticEvent as E;
    #[derive(Default)]
    struct Acc {
        n: u32,
        at_war: u32,
        prosperity: Vec<f64>,
        security: Vec<f64>,
        legitimacy: Vec<f64>,
        weariness: Vec<f64>,
        democracy: u32,
    }
    let mut acc: BTreeMap<String, Acc> = BTreeMap::new();
    for r in runs {
        let mut members: Vec<(u32, usize)> = Vec::new();
        for (t, e) in &r.events {
            match e {
                E::WarDeclared {
                    war,
                    attacker,
                    defender,
                    ..
                } => {
                    members.push((war.0, attacker.index()));
                    members.push((war.0, defender.index()));
                }
                E::JoinedWar { war, country, .. } => members.push((war.0, country.index())),
                E::PeaceMade { war, .. } => members.retain(|m| m.0 != war.0),
                E::Transition {
                    country,
                    kind: TransitionKind::Collapse,
                } => {
                    let a = acc.entry(codes[country.index()].clone()).or_default();
                    a.n += 1;
                    a.at_war += u32::from(members.iter().any(|m| m.1 == country.index()));
                    if let Some(s) = r.samples.iter().rev().find(|s| s.turn < *t) {
                        let c = &s.countries[country.index()];
                        a.prosperity.push(c.prosperity);
                        a.security.push(c.security);
                        a.legitimacy.push(c.legitimacy);
                        a.weariness.push(c.war_weariness);
                        a.democracy += u32::from(c.democracy);
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = String::new();
    let _ = writeln!(out, "COLLAPSES (stability < 10; medians on the turn before)");
    for (code, a) in acc {
        let _ = writeln!(
            out,
            "{code:4} {:3} collapses, {:3} at war, {:3} as democracy | prosp {:5.1} sec {:5.1} legit {:5.1} weariness {:5.1}",
            a.n,
            a.at_war,
            a.democracy,
            median(a.prosperity),
            median(a.security),
            median(a.legitimacy),
            median(a.weariness)
        );
    }
    out
}

/// Wars actually declared: the mean of every reasoning term behind the
/// choice (the declarer's own fog estimate of its odds included), then what
/// the battlefield said: true power on the turn before, and the front's
/// attack ÷ defence ratio over the first turns. A war chosen at good
/// estimated odds that then loses at bad true odds is a fog problem; one
/// chosen at bad estimated odds is a motive problem.
pub fn chosen_wars(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::DiplomaticEvent as E;
    #[derive(Default)]
    struct Acc {
        n: usize,
        aims: BTreeMap<String, u32>,
        terms: BTreeMap<String, f64>,
        own: Vec<f64>,
        target: Vec<f64>,
        ratio: Vec<f64>,
        late: Vec<f64>,
        turn: Vec<f64>,
    }
    let mut acc: BTreeMap<String, Acc> = BTreeMap::new();
    for r in runs {
        for e in &r.reasoning {
            let d = &e.decision;
            if !d.chosen {
                continue;
            }
            let Some(rest) = d.subject.strip_prefix("declare ").filter(|r| r.contains(" war on ")) else {
                continue;
            };
            let target = rest.rsplit(' ').next().unwrap_or("?");
            let aim = rest.split(' ').next().unwrap_or("?").to_string();
            let a = acc.entry(format!("{} -> {target}", e.country)).or_default();
            a.n += 1;
            *a.aims.entry(aim).or_default() += 1;
            for l in &d.lines {
                *a.terms.entry(l.label.clone()).or_default() += l.value;
            }
        }
        let mut pair: BTreeMap<u32, (String, u32)> = BTreeMap::new();
        for (t, e) in &r.events {
            match e {
                E::WarDeclared {
                    war,
                    attacker,
                    defender,
                    ..
                } => {
                    let key = format!("{} -> {}", codes[attacker.index()], codes[defender.index()]);
                    if let Some(s) = r.samples.iter().rev().find(|s| s.turn < *t) {
                        let a = acc.entry(key.clone()).or_default();
                        a.own.push(s.countries[attacker.index()].military);
                        a.target.push(s.countries[defender.index()].military);
                    }
                    acc.entry(key.clone()).or_default().turn.push(*t as f64);
                    pair.insert(war.0, (key, *t));
                }
                E::FrontReport { war, ratio, .. } => {
                    if let Some((key, start)) = pair.get(&war.0) {
                        if t - start <= 3 {
                            acc.entry(key.clone()).or_default().ratio.push(*ratio);
                        } else if t - start <= 11 {
                            acc.entry(key.clone()).or_default().late.push(*ratio);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = String::new();
    let _ = writeln!(
        out,
        "WARS CHOSEN (mean of every term; true power before; front ratio, first 3 turns, then turns 4-11; declaration turn)"
    );
    for (pair, a) in acc.into_iter().filter(|x| x.1.n > 0) {
        let n = a.n as f64;
        let mut terms: Vec<(String, f64)> = a.terms.into_iter().map(|(k, v)| (k, v / n)).collect();
        terms.sort_by(|x, y| y.1.total_cmp(&x.1));
        let t: Vec<String> = terms.iter().map(|t| format!("{} {:+.1}", t.0, t.1)).collect();
        let _ = writeln!(
            out,
            "{pair:11} chosen {:3} {:?} | true power {:.1} vs {:.1} | front ratio median {:.2}, later {:.2} | declared turn median {:.0}\n            {}",
            a.n,
            a.aims,
            median(a.own),
            median(a.target),
            median(a.ratio),
            median(a.late),
            median(a.turn),
            t.join("; ")
        );
    }
    out
}

/// Protection of the smaller states (non-Playable tiers and any regime with
/// a median Security below 25): how often they had a protector, their
/// Security and crisis rate with and without one, and the guarantee
/// requests they sent and how those were answered.
pub fn protection(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::transition::{CRISIS_LEGITIMACY, CRISIS_STABILITY};
    #[derive(Default)]
    struct Acc {
        turns: [usize; 2],
        crisis: [usize; 2],
        security: [Vec<f64>; 2],
        by: BTreeMap<String, usize>,
        sent: BTreeMap<String, usize>,
        accepted: BTreeMap<String, usize>,
    }
    let mut acc: BTreeMap<String, Acc> = BTreeMap::new();
    for r in runs {
        for s in &r.samples {
            for c in s.countries.iter().filter(|c| c.active && c.gdp > 0.0) {
                let a = acc.entry(c.code.clone()).or_default();
                let k = usize::from(!c.protectors.is_empty());
                a.turns[k] += 1;
                if !c.democracy && c.stability < CRISIS_STABILITY && c.legitimacy < CRISIS_LEGITIMACY {
                    a.crisis[k] += 1;
                }
                a.security[k].push(c.security);
                for p in &c.protectors {
                    *a.by.entry(p.clone()).or_default() += 1;
                }
            }
        }
        let mut open: BTreeMap<u32, (String, String)> = BTreeMap::new();
        for (_, e) in &r.events {
            match e {
                sim_core::DiplomaticEvent::Proposed { proposal: p } if p.kind == sim_core::TreatyKind::Guarantee => {
                    let (from, to) = (codes[p.from.index()].clone(), codes[p.to.index()].clone());
                    *acc.entry(from.clone()).or_default().sent.entry(to.clone()).or_default() += 1;
                    open.insert(p.id.0, (from, to));
                }
                sim_core::DiplomaticEvent::TreatySigned { treaty: t } if t.kind == sim_core::TreatyKind::Guarantee => {
                    let (patron, client) = (codes[t.a.index()].clone(), codes[t.b.index()].clone());
                    if let Some(k) = open
                        .iter()
                        .find(|(_, v)| v.0 == client && v.1 == patron)
                        .map(|(k, _)| *k)
                    {
                        open.remove(&k);
                        *acc.entry(client).or_default().accepted.entry(patron).or_default() += 1;
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = String::new();
    let _ = writeln!(
        out,
        "PROTECTION (states with median Security < 25 or any guarantee request; crisis% non-democracies only)"
    );
    for (code, a) in acc {
        let all: Vec<f64> = a.security[0].iter().chain(a.security[1].iter()).copied().collect();
        if median(all) >= 25.0 && a.sent.is_empty() {
            continue;
        }
        let total = (a.turns[0] + a.turns[1]).max(1);
        let pct = |n: usize, d: usize| 100.0 * n as f64 / d.max(1) as f64;
        let mut by: Vec<_> = a.by.into_iter().collect();
        by.sort_by_key(|x| std::cmp::Reverse(x.1));
        let by: Vec<String> = by
            .iter()
            .take(4)
            .map(|(c, n)| format!("{c} {:.0}%", pct(*n, total)))
            .collect();
        let _ = writeln!(
            out,
            "{code:4} protected {:>5.1}% of turns (by {}) | security {:>5.1} protected / {:>5.1} not | crisis {:>5.1}% / {:>5.1}% | requests {:?} accepted {:?}",
            pct(a.turns[1], total),
            by.join(", "),
            median(a.security[1].clone()),
            median(a.security[0].clone()),
            pct(a.crisis[1], a.turns[1]),
            pct(a.crisis[0], a.turns[0]),
            a.sent,
            a.accepted
        );
    }
    out
}

/// Interventions weighed by the Gate 6 powers (the AI's "weigh
/// intervention" records): per power, war and side, the mean stake, the
/// mean margin of the best band 2–4 option and its largest terms. Answers
/// "why didn't they intervene?" — whether the stake is missing or the
/// costs outweigh it.
pub fn interventions(runs: &[RunResult], powers: &[String]) -> String {
    #[derive(Default)]
    struct Acc {
        n: usize,
        stake: f64,
        score: f64,
        bands: BTreeMap<String, usize>,
        terms: BTreeMap<String, f64>,
    }
    let mut acc: BTreeMap<String, Acc> = BTreeMap::new();
    for r in runs {
        for e in &r.reasoning {
            let d = &e.decision;
            let Some(rest) = d.subject.strip_prefix("weigh intervention in ") else {
                continue;
            };
            if !powers.contains(&e.country) {
                continue;
            }
            let Some((head, band)) = rest.split_once("): ") else {
                continue;
            };
            let Some((war, stake)) = head.split_once(" (stake ") else {
                continue;
            };
            let force = if band.starts_with("band 2") { "proxy" } else { "force" };
            let a = acc
                .entry(format!("{} {} {force}", e.country, war.trim_end_matches(" war")))
                .or_default();
            a.n += 1;
            a.stake += stake.parse::<f64>().unwrap_or(0.0);
            a.score += d.score;
            *a.bands
                .entry(band.split(' ').take(2).collect::<Vec<_>>().join(" "))
                .or_default() += 1;
            for l in &d.lines {
                *a.terms.entry(l.label.clone()).or_default() += l.value;
            }
        }
    }
    let mut out = String::new();
    let _ = writeln!(
        out,
        "INTERVENTIONS WEIGHED (Gate 6 powers; best proxy (band 2) and force (band 3-4) option per side, at the outbreak and each assessment; stay out = +5)"
    );
    for (key, a) in acc {
        let n = a.n as f64;
        let mut terms: Vec<(String, f64)> = a.terms.into_iter().map(|(k, v)| (k, v / n)).collect();
        terms.sort_by(|x, y| y.1.abs().total_cmp(&x.1.abs()));
        let t: Vec<String> = terms.iter().take(5).map(|t| format!("{} {:+.1}", t.0, t.1)).collect();
        let best = a.bands.iter().max_by_key(|b| b.1).map_or("?", |b| b.0.as_str());
        let _ = writeln!(
            out,
            "{key:34} n {:4} stake {:.2} best {best} {:+6.1} | {}",
            a.n,
            a.stake / n,
            a.score / n,
            t.join("; ")
        );
    }
    out
}

/// Commitments of the Gate 6 powers: wars they were involved in as the
/// engine records them (joined, or arms flowing to a belligerent: band 2),
/// how those involvements ended, support cuts by grade, and sanctions
/// (runs with each, median first turn, mean decision terms).
pub fn commitments(runs: &[RunResult], codes: &[String], powers: &[String]) -> String {
    use sim_core::{CauseCode, DecisionKind, DiplomaticEvent, LedgerLog};
    let is_power = |c: &String| powers.contains(c);
    let mut involved: BTreeMap<String, usize> = BTreeMap::new();
    let mut exits: BTreeMap<String, usize> = BTreeMap::new();
    let mut sanctions: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    let mut terms: BTreeMap<String, (usize, BTreeMap<String, f64>)> = BTreeMap::new();
    for r in runs {
        let mut seen = std::collections::BTreeSet::new();
        let mut first: BTreeMap<String, u32> = BTreeMap::new();
        for (_, l) in &r.ledger_log {
            let LedgerLog::EntryWritten {
                actor,
                counterpart,
                grade,
                code,
                cause,
                ..
            } = l
            else {
                continue;
            };
            let a = &codes[actor.index()];
            if !is_power(a) {
                continue;
            }
            let whom = &codes[counterpart.index()];
            match code {
                CauseCode::WarInvolvementKept | CauseCode::WarWithdrawal => {
                    let band = cause
                        .split("band ")
                        .nth(1)
                        .and_then(|b| b.chars().next())
                        .unwrap_or('?');
                    seen.insert(format!("{a} band {band} for {whom}"));
                    if *code == CauseCode::WarWithdrawal {
                        *exits
                            .entry(format!("{a} withdrew {grade:?} from {whom}'s war"))
                            .or_default() += 1;
                    }
                }
                CauseCode::SupportCut | CauseCode::GuaranteeCancelled => {
                    *exits.entry(format!("{a} {code:?} {grade:?}")).or_default() += 1;
                }
                _ => {}
            }
        }
        for k in seen {
            *involved.entry(k).or_default() += 1;
        }
        for (t, e) in &r.events {
            if let DiplomaticEvent::SanctionImposed { by, target } = e {
                let a = &codes[by.index()];
                if is_power(a) {
                    first.entry(format!("{a} -> {}", codes[target.index()])).or_insert(*t);
                }
            }
        }
        for (k, t) in first {
            sanctions.entry(k).or_default().push(t);
        }
        for e in &r.reasoning {
            let d = &e.decision;
            if d.chosen
                && is_power(&e.country)
                && matches!(d.kind, DecisionKind::JoinSanction | DecisionKind::ImposeSanction)
            {
                let x = terms.entry(format!("{} {:?}", e.country, d.kind)).or_default();
                x.0 += 1;
                for l in &d.lines {
                    *x.1.entry(l.label.clone()).or_default() += l.value;
                }
            }
        }
    }
    let n = runs.len();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "WAR INVOLVEMENTS (engine-recorded; runs out of {n}; ended involvements only)"
    );
    for (k, v) in involved {
        let _ = writeln!(out, "  {k:30} {v}");
    }
    let _ = writeln!(out, "COMMITMENT EXITS (events)");
    for (k, v) in exits {
        let _ = writeln!(out, "  {k:40} {v}");
    }
    let _ = writeln!(out, "SANCTIONS (runs out of {n}; median first turn)");
    for (k, mut ts) in sanctions {
        ts.sort_unstable();
        let _ = writeln!(out, "  {k:12} {:3} runs, turn {}", ts.len(), ts[ts.len() / 2]);
    }
    for (k, (c, t)) in terms {
        let mut t: Vec<(String, f64)> = t.into_iter().map(|(l, v)| (l, v / c as f64)).collect();
        t.sort_by(|x, y| y.1.abs().total_cmp(&x.1.abs()));
        let t: Vec<String> = t.iter().take(6).map(|(l, v)| format!("{l} {v:+.1}")).collect();
        let _ = writeln!(out, "  {k:22} n {c:4} | {}", t.join("; "));
    }
    out
}

/// Legend for [`security_line`] (`brink security`).
pub const SECURITY_LEGEND: &str = "allies = treaty defenders that back this regime, power x belief x reach; \
hostile = every country with hostility > 0 (tension, negative opinion or war), power x reach x hostility \
(sim-core security_breakdown). A defender hostile toward us is paper: its ally term is scaled by \
(1 - hostility) while its hostility counts in full; '*' marks it in the hostile list. Hostile terms below \
0.05 are summed as 'negligible'.";

/// Hostile terms below this are summed into one "negligible" figure.
const SECURITY_NOISE: f64 = 0.05;

/// One line of the Security driver breakdown for `me`, classified exactly
/// as sim-core's `security_breakdown` does (D68 display fix: a defender
/// that is also tense with us appears in both lists; the sim scales its
/// ally term by `pledge_worth` and counts its hostility in full, and the
/// line marks it instead of printing "hostile USA 0.0" next to "allies USA
/// 40.8").
pub fn security_line(state: &sim_core::WorldState, me: sim_core::CountryId) -> String {
    let b = sim_core::domestic::security_breakdown(state, me);
    let name = |x: sim_core::CountryId| state.country(x).code.clone();
    let mut allies = b.allies.clone();
    allies.sort_by(|x, y| y.1.total_cmp(&x.1));
    let mut hostile = b.hostile.clone();
    hostile.sort_by(|x, y| y.1.total_cmp(&x.1));
    let is_ally = |x: sim_core::CountryId| b.allies.iter().any(|a| a.0 == x);
    let defence = b.own + b.allies.iter().map(|a| a.1).sum::<f64>();
    let pressure: f64 = b.hostile.iter().map(|h| h.1).sum();
    let allies: Vec<String> = allies.iter().map(|a| format!("{} {:.1}", name(a.0), a.1)).collect();
    let (shown, noise): (Vec<&(sim_core::CountryId, f64)>, Vec<_>) =
        hostile.iter().partition(|h| h.1 >= SECURITY_NOISE);
    let mut hostile: Vec<String> = shown
        .iter()
        .map(|h| format!("{}{} {:.1}", name(h.0), if is_ally(h.0) { "*" } else { "" }, h.1))
        .collect();
    if !noise.is_empty() {
        let both = noise.iter().filter(|h| is_ally(h.0)).count();
        let note = if both > 0 {
            format!(", {both} also allies")
        } else {
            String::new()
        };
        hostile.push(format!(
            "+{} negligible {:.2}{note}",
            noise.len(),
            noise.iter().map(|h| h.1).sum::<f64>()
        ));
    }
    format!(
        "{:4} sec {:5.1} own {:6.1} defence {:6.1} pressure {:6.1} | allies {} | hostile {}",
        state.country(me).code,
        b.value,
        b.own,
        defence,
        pressure,
        allies.join(", "),
        hostile.join(", ")
    )
}

/// Sanction lifecycle (issue 15): impositions, lifts, durations and
/// re-impositions per pair, plus tension/opinion trajectories for the
/// pairs that carry the Cold War.
pub fn sanction_lifecycle(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::DiplomaticEvent;
    /// A lift followed by a re-imposition within this many turns is flicker.
    const FLICKER: u32 = 8;
    let n = runs.len().max(1);
    let mut imposed: BTreeMap<String, u32> = BTreeMap::new();
    let mut lifted: BTreeMap<String, u32> = BTreeMap::new();
    let mut durations: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let (mut total_imposed, mut total_lifted, mut flicker, mut reimposed) = (0u32, 0u32, 0u32, 0u32);
    let mut all_durations = Vec::new();
    let mut active_end = 0usize;
    let mut flicker_pairs: BTreeMap<String, u32> = BTreeMap::new();
    for r in runs {
        let mut open: BTreeMap<(usize, usize), u32> = BTreeMap::new();
        let mut last_lift: BTreeMap<(usize, usize), u32> = BTreeMap::new();
        for (t, e) in &r.events {
            match e {
                DiplomaticEvent::SanctionImposed { by, target } => {
                    let k = (by.index(), target.index());
                    let name = format!("{} -> {}", codes[k.0], codes[k.1]);
                    *imposed.entry(name).or_default() += 1;
                    total_imposed += 1;
                    if let Some(l) = last_lift.get(&k) {
                        reimposed += 1;
                        if t - l <= FLICKER {
                            flicker += 1;
                            *flicker_pairs
                                .entry(format!("{} -> {}", codes[k.0], codes[k.1]))
                                .or_default() += 1;
                        }
                    }
                    open.insert(k, *t);
                }
                DiplomaticEvent::SanctionLifted { by, target } => {
                    let k = (by.index(), target.index());
                    let name = format!("{} -> {}", codes[k.0], codes[k.1]);
                    *lifted.entry(name.clone()).or_default() += 1;
                    total_lifted += 1;
                    // Not opened in this run and never lifted: a scenario-start sanction.
                    let start = open.remove(&k).or_else(|| (!last_lift.contains_key(&k)).then_some(0));
                    if let Some(s) = start {
                        durations.entry(name).or_default().push(f64::from(t - s));
                        all_durations.push(f64::from(t - s));
                    }
                    last_lift.insert(k, *t);
                }
                _ => {}
            }
        }
        active_end += open.len();
    }
    let per_run = |x: f64| x / n as f64;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "SANCTION LIFECYCLE ({n} runs; imposed counts scenario-start sanctions only if re-imposed): imposed {:.2}/run, lifted {:.2}/run, \
         run-time impositions still open at end {:.2}/run, re-imposed after a lift {reimposed}, within {FLICKER} turns (flicker) {flicker}, \
         median duration of lifted {:.0} turns",
        per_run(f64::from(total_imposed)),
        per_run(f64::from(total_lifted)),
        per_run(active_end as f64),
        median(all_durations),
    );
    if !flicker_pairs.is_empty() {
        let f: Vec<String> = flicker_pairs.iter().map(|(k, v)| format!("{k} {v}")).collect();
        let _ = writeln!(out, "  flicker by pair: {}", f.join(", "));
    }
    let mut keys: Vec<String> = imposed.keys().chain(lifted.keys()).cloned().collect();
    keys.sort();
    keys.dedup();
    keys.sort_by(|a, b| {
        let x = imposed.get(a).copied().unwrap_or(0) + lifted.get(a).copied().unwrap_or(0);
        let y = imposed.get(b).copied().unwrap_or(0) + lifted.get(b).copied().unwrap_or(0);
        y.cmp(&x).then(a.cmp(b))
    });
    for k in keys.iter().take(70) {
        let i = imposed.get(k).copied().unwrap_or(0);
        let l = lifted.get(k).copied().unwrap_or(0);
        let d = durations.get(k).cloned().unwrap_or_default();
        let _ = writeln!(
            out,
            "  {k:12} imposed {:5.2}/run  lifted {:5.2}/run  median duration {}",
            per_run(f64::from(i)),
            per_run(f64::from(l)),
            if d.is_empty() {
                "-".to_string()
            } else {
                format!("{:.0}", median(d))
            },
        );
    }
    // Keep-sanction reviews: mean score and terms per pair (issue 15).
    let mut keeps: BTreeMap<String, (usize, f64, BTreeMap<String, f64>)> = BTreeMap::new();
    for r in runs {
        for e in &r.reasoning {
            if e.decision.kind != sim_core::DecisionKind::KeepSanction {
                continue;
            }
            let whom = e.decision.counterpart.map_or("?", |c| codes[c.index()].as_str());
            let x = keeps.entry(format!("{} -> {whom}", e.country)).or_default();
            x.0 += 1;
            x.1 += e.decision.score;
            for l in &e.decision.lines {
                *x.2.entry(l.label.clone()).or_default() += l.value;
            }
        }
    }
    let mut keep_rows: Vec<_> = keeps.into_iter().collect();
    keep_rows.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then(a.0.cmp(&b.0)));
    let _ = writeln!(
        out,
        "KEEP-SANCTION REVIEWS (top pairs by reviews: mean score | mean terms)"
    );
    for (k, (c, total, terms)) in keep_rows.into_iter().take(60) {
        let mut t: Vec<(String, f64)> = terms.into_iter().map(|(l, v)| (l, v / c as f64)).collect();
        t.sort_by(|x, y| y.1.abs().total_cmp(&x.1.abs()));
        let t: Vec<String> = t.iter().take(6).map(|(l, v)| format!("{l} {v:+.1}")).collect();
        let _ = writeln!(
            out,
            "  {k:12} n {c:5} mean {:+6.1} | {}",
            total / c as f64,
            t.join("; ")
        );
    }
    // Trajectories: mean tension and mean opinion each way, every 12 turns.
    let idx = |c: &str| codes.iter().position(|x| x == c);
    let pairs = [
        ("USA", "SOV"),
        ("GBR", "SOV"),
        ("FRG", "SOV"),
        ("FRA", "SOV"),
        ("JPN", "SOV"),
        ("TUR", "SOV"),
        ("USA", "CUB"),
        ("USA", "IRN"),
        ("GBR", "ARG"),
    ];
    let _ = writeln!(out, "RELATION TRAJECTORIES (mean over runs: tension|opinion a->b/b->a)");
    let turns: Vec<u32> = runs
        .first()
        .map(|r| {
            r.relations
                .iter()
                .map(|s| s.turn)
                .filter(|t| t % 12 == 0 || *t == r.turns)
                .collect()
        })
        .unwrap_or_default();
    let header: String = turns.iter().map(|t| format!("{:>14}", format!("t{t}"))).collect();
    let _ = writeln!(out, "  {:9}{header}", "pair");
    let nc = codes.len();
    for (a, b) in pairs {
        let (Some(ia), Some(ib)) = (idx(a), idx(b)) else {
            continue;
        };
        let mut line = format!("  {:9}", format!("{a}-{b}"));
        for &t in &turns {
            let (mut te, mut oab, mut oba, mut c) = (0.0, 0.0, 0.0, 0.0);
            for r in runs {
                if let Some(s) = r.relations.iter().find(|s| s.turn == t) {
                    te += s.tension[ia * nc + ib];
                    oab += s.opinion[ia * nc + ib];
                    oba += s.opinion[ib * nc + ia];
                    c += 1.0;
                }
            }
            if c > 0.0 {
                line += &format!("{:>14}", format!("{:.0}|{:.0}/{:.0}", te / c, oab / c, oba / c));
            }
        }
        let _ = writeln!(out, "{line}");
    }
    out
}

/// Historical grudges (D81): per pair, the mean memory each way at t20/40/
/// 60/80, mean renewals (both directions), and the thaw turn (first sample
/// with both directions' memory above -20) per run, split by whether either
/// side had a regime transition (reform, coup, collapse) in the run.
pub fn grudges(runs: &[RunResult], codes: &[String]) -> String {
    let mut out = String::new();
    let idx = |c: &str| codes.iter().position(|x| x == c);
    let pairs = [
        ("USA", "SOV"),
        ("SOV", "FRG"),
        ("SOV", "GBR"),
        ("SOV", "JPN"),
        ("SOV", "TUR"),
        ("CHN", "SOV"),
        ("KOR", "PRK"),
        ("IRN", "IRQ"),
        ("ISR", "SYR"),
        ("IRQ", "KWT"),
        ("SAU", "IRN"),
    ];
    let nc = codes.len();
    let _ = writeln!(
        out,
        "HISTORICAL GRUDGES (D81; mean memory a->b/b->a; renewals = mean per run, both directions; thaw = both above -20)"
    );
    let _ = writeln!(
        out,
        "  {:9}{:>12}{:>12}{:>12}{:>12} {:>10} {:>6} | thawed  median  min  max | thawed w/ transition | w/o",
        "pair", "t20", "t40", "t60", "t80", "t80 range", "renew"
    );
    for (a, b) in pairs {
        let (Some(ia), Some(ib)) = (idx(a), idx(b)) else {
            continue;
        };
        let mut line = format!("  {:9}", format!("{a}-{b}"));
        for t in [20, 40, 60, 80] {
            let (mut ab, mut ba, mut c) = (0.0, 0.0, 0.0);
            for r in runs {
                if let Some(s) = r.relations.iter().find(|s| s.turn == t) {
                    ab += s.memory[ia * nc + ib];
                    ba += s.memory[ib * nc + ia];
                    c += 1.0;
                }
            }
            let cell = if c > 0.0 {
                format!("{:.1}/{:.1}", ab / c, ba / c)
            } else {
                "-".into()
            };
            line += &format!("{cell:>12}");
        }
        let end: Vec<f64> = runs
            .iter()
            .filter_map(|r| r.relations.iter().find(|s| s.turn == 80))
            .map(|s| s.memory[ia * nc + ib].min(s.memory[ib * nc + ia]))
            .collect();
        let lo = end.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = end.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        line += &format!(" [{lo:.0}..{hi:.0}]");
        let mut renew = 0.0;
        let mut thaws = Vec::new();
        let (mut with_t, mut with_n, mut wo_t, mut wo_n) = (0, 0, 0, 0);
        for r in runs {
            if let Some(last) = r.relations.last() {
                renew += (last.renewals[ia * nc + ib] + last.renewals[ib * nc + ia]) as f64 / 2.0;
            }
            let thaw = r
                .relations
                .iter()
                .find(|s| s.memory[ia * nc + ib] > -20.0 && s.memory[ib * nc + ia] > -20.0)
                .map(|s| s.turn);
            let transition = r.events.iter().any(|(_, e)| {
                matches!(e, sim_core::DiplomaticEvent::Transition { country, kind }
                    if (country.index() == ia || country.index() == ib)
                        && *kind != sim_core::transition::TransitionKind::Crackdown)
            });
            if transition {
                with_n += 1;
                with_t += thaw.is_some() as u32;
            } else {
                wo_n += 1;
                wo_t += thaw.is_some() as u32;
            }
            if let Some(t) = thaw {
                thaws.push(t as f64);
            }
        }
        let n = runs.len().max(1) as f64;
        let (med, min, max) = if thaws.is_empty() {
            (f64::NAN, f64::NAN, f64::NAN)
        } else {
            let lo = thaws.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = thaws.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (median(thaws.clone()), lo, hi)
        };
        let _ = writeln!(
            out,
            "{line} {:>6.1} | {:>3}/{:<3} {:>6.0} {:>4.0} {:>4.0} | {with_t:>3}/{with_n:<3}            | {wo_t}/{wo_n}",
            renew / n,
            thaws.len(),
            runs.len(),
            med,
            min,
            max
        );
    }
    out
}

/// War length and endings (issue 22): duration distribution, how wars end,
/// who offered peace first and why, and the belligerents' state at the end.
pub fn war_lengths(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::DiplomaticEvent as E;
    let mut durations = Vec::new();
    let mut reasons: BTreeMap<String, u32> = BTreeMap::new();
    let mut first_offer: BTreeMap<String, u32> = BTreeMap::new();
    let mut offer_turn: Vec<f64> = Vec::new();
    let mut end_state: Vec<String> = Vec::new();
    let mut peak_weary = Vec::new();
    let mut debt_end: Vec<(String, f64)> = Vec::new();
    let mut offer_terms: BTreeMap<String, (f64, u32)> = BTreeMap::new();
    let mut close_terms: BTreeMap<String, (f64, u32)> = BTreeMap::new();
    let mut offers = 0u32;
    let mut closes = 0u32;
    for r in runs {
        let mut started: BTreeMap<u32, u32> = BTreeMap::new();
        for (t, e) in &r.events {
            match e {
                E::WarDeclared { war, .. } => {
                    started.insert(war.0, *t);
                }
                E::PeaceMade {
                    war,
                    attacker,
                    defender,
                    result,
                    reason,
                    progress,
                    ..
                } => {
                    let Some(&s) = started.get(&war.0) else { continue };
                    durations.push(f64::from(t - s));
                    *reasons.entry(format!("{result:?} {reason:?}")).or_default() += 1;
                    let (a, d) = (codes[attacker.index()].clone(), codes[defender.index()].clone());
                    let at = |c: &str, turn: u32| {
                        r.samples
                            .iter()
                            .find(|x| x.turn == turn)
                            .and_then(|x| x.countries.iter().find(|y| y.code == c))
                            .cloned()
                    };
                    let mut pw: f64 = 0.0;
                    for x in r.samples.iter().filter(|x| x.turn >= s && x.turn <= *t) {
                        for y in x.countries.iter().filter(|y| y.code == a || y.code == d) {
                            pw = pw.max(y.war_weariness);
                        }
                    }
                    peak_weary.push(pw);
                    if let (Some(ca), Some(cd)) = (at(&a, *t), at(&d, *t)) {
                        debt_end.push((a.clone(), ca.debt_ratio));
                        debt_end.push((d.clone(), cd.debt_ratio));
                        if end_state.len() < 30 {
                            end_state.push(format!(
                                "  seed {} {a}->{d} t{s}-{t} ({} turns) {result:?} {reason:?} front {progress:+.0} | {a} weary {:.0} stab {:.0} debt {:.2} | {d} weary {:.0} stab {:.0} debt {:.2}",
                                r.seed,
                                t - s,
                                ca.war_weariness,
                                ca.stability,
                                ca.debt_ratio,
                                cd.war_weariness,
                                cd.stability,
                                cd.debt_ratio
                            ));
                        }
                    }
                    let recs: Vec<_> = r
                        .reasoning
                        .iter()
                        .filter(|e| {
                            e.turn >= s
                                && e.turn <= *t
                                && e.decision.subject.starts_with("continue the war with")
                                && (e.country == a || e.country == d)
                        })
                        .collect();
                    if let Some(first) = recs.iter().find(|e| !e.decision.chosen) {
                        let side = if first.country == a { "attacker" } else { "defender" };
                        *first_offer.entry(side.into()).or_default() += 1;
                        offer_turn.push(f64::from(first.turn - s));
                        offers += 1;
                        for l in &first.decision.lines {
                            let x = offer_terms.entry(format!("{side}: {}", l.label)).or_default();
                            x.0 += l.value;
                            x.1 += 1;
                        }
                    }
                    if let Some(last) = recs.iter().rev().find(|e| !e.decision.chosen) {
                        let side = if last.country == a { "attacker" } else { "defender" };
                        closes += 1;
                        for l in &last.decision.lines {
                            let x = close_terms.entry(format!("{side}: {}", l.label)).or_default();
                            x.0 += l.value;
                            x.1 += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let pct = |v: &[f64], q: f64| {
        let mut v = v.to_vec();
        if v.is_empty() {
            return f64::NAN;
        }
        v.sort_by(f64::total_cmp);
        v[((v.len() as f64 - 1.0) * q).round() as usize]
    };
    let mut out = String::new();
    let _ = writeln!(
        out,
        "WAR LENGTH (issue 22; {} ended wars): turns median {:.0}, p90 {:.0}, max {:.0}; peak weariness median {:.1}, p90 {:.1}; ended by {reasons:?}",
        durations.len(),
        pct(&durations, 0.5),
        pct(&durations, 0.9),
        pct(&durations, 1.0),
        pct(&peak_weary, 0.5),
        pct(&peak_weary, 0.9),
    );
    let mut by: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (c, d) in &debt_end {
        by.entry(c.clone()).or_default().push(*d);
    }
    for (c, v) in by {
        let _ = writeln!(
            out,
            "  debt ratio at war's end {c}: median {:.2}, max {:.2} (n {})",
            pct(&v, 0.5),
            pct(&v, 1.0),
            v.len()
        );
    }
    let _ = writeln!(
        out,
        "  first offer by {first_offer:?}, median turn into the war {:.0}",
        pct(&offer_turn, 0.5)
    );
    let fmt = |m: &BTreeMap<String, (f64, u32)>, n: u32| {
        m.iter()
            .map(|(k, (v, c))| format!("{k} {:+.1} ({c})", v / f64::from((*c).max(1))))
            .collect::<Vec<_>>()
            .join("; ")
            + &format!(" [n {n}]")
    };
    let _ = writeln!(
        out,
        "  first offer terms (mean when present): {}",
        fmt(&offer_terms, offers)
    );
    let _ = writeln!(out, "  closing offer terms: {}", fmt(&close_terms, closes));
    for l in end_state {
        let _ = writeln!(out, "{l}");
    }
    let _ = write!(out, "{}", post_war_debt(runs, codes));
    out
}

/// After each war: does a belligerent's debt cross the debt-drag line
/// (`DEBT_DRAG_THRESHOLD`), and is another producer flooding the oil market
/// while it does (the held D90 economic-warfare renewal's condition, minus
/// the same-area test).
fn post_war_debt(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::DiplomaticEvent as E;
    let line = sim_core::economy::DEBT_DRAG_THRESHOLD;
    let mut peak: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut over: BTreeMap<String, (u32, u32, u32)> = BTreeMap::new();
    let mut flooders: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
    for r in runs {
        // Energy policy per country, turn by turn, from the change records.
        let mut policy_changes: Vec<(u32, String, bool)> = Vec::new();
        for e in &r.reasoning {
            if let Some(p) = e.decision.subject.strip_prefix("energy policy: ") {
                policy_changes.push((e.turn, e.country.clone(), p == "Flood"));
            }
        }
        let mut ended: BTreeMap<String, u32> = BTreeMap::new();
        for (t, e) in &r.events {
            if let E::PeaceMade { attacker, defender, .. } = e {
                for c in [attacker, defender] {
                    ended.entry(codes[c.index()].clone()).or_insert(*t);
                }
            }
        }
        for (c, end) in ended {
            let mut pk: f64 = 0.0;
            let (mut turns_over, mut turns_flooded) = (0u32, 0u32);
            for s in r.samples.iter().filter(|s| s.turn >= end) {
                let Some(x) = s.countries.iter().find(|y| y.code == c) else {
                    continue;
                };
                pk = pk.max(x.debt_ratio);
                if x.debt_ratio > line {
                    turns_over += 1;
                    let mut flooding: BTreeMap<String, bool> = BTreeMap::new();
                    for (_, who, f) in policy_changes.iter().filter(|p| p.0 <= s.turn) {
                        flooding.insert(who.clone(), *f);
                    }
                    let f: Vec<String> = flooding
                        .into_iter()
                        .filter(|(w, f)| *f && *w != c)
                        .map(|(w, _)| w)
                        .collect();
                    if !f.is_empty() {
                        turns_flooded += 1;
                    }
                    for w in f {
                        *flooders.entry(c.clone()).or_default().entry(w).or_default() += 1;
                    }
                }
            }
            peak.entry(c.clone()).or_default().push(pk);
            let o = over.entry(c).or_default();
            o.0 += u32::from(turns_over > 0);
            o.1 += turns_over;
            o.2 += turns_flooded;
        }
    }
    // Who financed a belligerent while it fought (creditor candidates):
    // aid pledged, arms transferred and streams started to it mid-war.
    let mut financed: BTreeMap<String, BTreeMap<String, (u32, f64)>> = BTreeMap::new();
    for r in runs {
        let mut at_war: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
        for (_, e) in &r.events {
            match e {
                E::WarDeclared {
                    war,
                    attacker,
                    defender,
                    ..
                } => {
                    at_war.insert(war.0, (attacker.0.into(), defender.0.into()));
                }
                E::PeaceMade { war, .. } => {
                    at_war.remove(&war.0);
                }
                E::AidPledged { from, to, amount } | E::ArmsTransferred { from, to, amount } => {
                    if at_war
                        .values()
                        .any(|(a, d)| *a == u32::from(to.0) || *d == u32::from(to.0))
                    {
                        let x = financed
                            .entry(codes[to.index()].clone())
                            .or_default()
                            .entry(format!(
                                "{} {}",
                                codes[from.index()],
                                if matches!(e, E::AidPledged { .. }) {
                                    "aid"
                                } else {
                                    "arms"
                                }
                            ))
                            .or_default();
                        x.0 += 1;
                        x.1 += amount;
                    }
                }
                E::StreamStarted { stream } => {
                    let to = u32::from(stream.to.0);
                    if at_war.values().any(|(a, d)| *a == to || *d == to) {
                        let x = financed
                            .entry(codes[stream.to.index()].clone())
                            .or_default()
                            .entry(format!("{} {:?} stream", codes[stream.from.index()], stream.kind))
                            .or_default();
                        x.0 += 1;
                        x.1 += stream.amount;
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = String::new();
    for (to, m) in &financed {
        let _ = writeln!(
            out,
            "  financed mid-war {to} ({} runs): {}",
            runs.len(),
            m.iter()
                .map(|(k, (n, a))| format!("{k} x{n} ({a:.2})"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    for (c, v) in peak {
        let (runs_over, t_over, t_flood) = over.get(&c).copied().unwrap_or_default();
        let _ = writeln!(
            out,
            "  post-war debt {c}: peak median {:.2}, max {:.2}; above {line} in {runs_over}/{} runs ({t_over} turns; {t_flood} with another producer on Flood: {:?})",
            median(v.clone()),
            v.iter().copied().fold(0.0, f64::max),
            v.len(),
            flooders.get(&c).cloned().unwrap_or_default()
        );
    }
    out
}

/// Grudges deepened by a harmful campaign (D97), every directed pair
/// (victim -> actor) deepened in any run: runs, mean quarters of harm by the
/// end, the deepest memory sampled (mean over those runs, and the worst),
/// runs that went past -40, and the acts that last deepened it.
pub fn deepened_grudges(runs: &[RunResult], codes: &[String]) -> String {
    let nc = codes.len();
    let mut out = format!(
        "DEEPENED GRUDGES (D97; victim->actor; {} runs; deepest = min sampled memory)\n",
        runs.len()
    );
    let _ = writeln!(
        out,
        "  {:9} {:>5} {:>9} {:>13} {:>7} {:>6}  acts",
        "pair", "runs", "quarters", "deepest mean", "worst", "<-40"
    );
    let mut rows = Vec::new();
    for i in 0..nc * nc {
        let mut acts: BTreeMap<String, u32> = BTreeMap::new();
        let (mut n, mut q, mut deep, mut worst, mut past) = (0u32, 0u32, 0.0, 0.0f64, 0u32);
        for r in runs {
            let Some(last) = r.relations.last() else { continue };
            if last.deepened[i] == 0 {
                continue;
            }
            n += 1;
            q += last.deepened[i];
            let min = r.relations.iter().map(|s| s.memory[i]).fold(0.0, f64::min);
            deep += min;
            worst = worst.min(min);
            past += (min < -40.0) as u32;
            if let Some(a) = last.deepened_by[i] {
                *acts.entry(format!("{a:?}")).or_default() += 1;
            }
        }
        if n > 0 {
            rows.push((n, i, q, deep, worst, past, acts));
        }
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (n, i, q, deep, worst, past, acts) in rows {
        let pair = format!("{}-{}", codes[i / nc], codes[i % nc]);
        let acts: Vec<String> = acts.iter().map(|(k, v)| format!("{k} {v}")).collect();
        let _ = writeln!(
            out,
            "  {pair:9} {n:>5} {:>9.1} {:>13.1} {worst:>7.0} {past:>6}  {}",
            q as f64 / n as f64,
            deep / n as f64,
            acts.join(", ")
        );
    }
    out
}

/// War loans and their settlement (issue 24): claims per (creditor,
/// debtor) pair, the forgive-or-hold decisions and their terms, and the
/// holds (public hostile acts that renew a grudge where one exists).
pub fn creditors(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::DiplomaticEvent as E;
    #[derive(Default)]
    struct Pair {
        runs: u32,
        lent: f64,
        outstanding: f64,
        forgiven: (u32, f64),
        held: u32,
        first_hold: Vec<f64>,
    }
    let mut pairs: BTreeMap<(String, String), Pair> = BTreeMap::new();
    let mut terms: BTreeMap<(String, bool), (u32, BTreeMap<String, f64>)> = BTreeMap::new();
    for r in runs {
        let mut lent: BTreeMap<(usize, usize), f64> = BTreeMap::new();
        let mut first: BTreeMap<(usize, usize), u32> = BTreeMap::new();
        let state: serde_json::Value = serde_json::from_str(&r.final_state).unwrap_or_default();
        for l in state["diplomacy"]["loans"].as_array().into_iter().flatten() {
            let c = l["creditor"].as_u64().unwrap_or(0) as usize;
            let d = l["debtor"].as_u64().unwrap_or(0) as usize;
            let a = l["amount"].as_f64().unwrap_or(0.0);
            *lent.entry((c, d)).or_default() += a;
            pairs.entry((codes[c].clone(), codes[d].clone())).or_default().outstanding += a;
        }
        for (t, e) in &r.events {
            match *e {
                E::DebtForgiven { creditor, debtor, amount } => {
                    *lent.entry((creditor.index(), debtor.index())).or_default() += amount;
                    let p = pairs
                        .entry((codes[creditor.index()].clone(), codes[debtor.index()].clone()))
                        .or_default();
                    p.forgiven.0 += 1;
                    p.forgiven.1 += amount;
                }
                E::DebtHeld { creditor, debtor, .. } => {
                    pairs
                        .entry((codes[creditor.index()].clone(), codes[debtor.index()].clone()))
                        .or_default()
                        .held += 1;
                    first.entry((creditor.index(), debtor.index())).or_insert(*t);
                }
                _ => {}
            }
        }
        for ((c, d), a) in lent {
            let p = pairs.entry((codes[c].clone(), codes[d].clone())).or_default();
            p.runs += 1;
            p.lent += a;
        }
        for ((c, d), t) in first {
            pairs
                .entry((codes[c].clone(), codes[d].clone()))
                .or_default()
                .first_hold
                .push(t as f64);
        }
        for e in &r.reasoning {
            if e.decision.kind != sim_core::DecisionKind::ForgiveDebt {
                continue;
            }
            let who = format!(
                "{} -> {}",
                e.country,
                e.decision.counterpart.map_or("?".to_string(), |c| codes[c.index()].clone())
            );
            let x = terms.entry((who, e.decision.chosen)).or_default();
            x.0 += 1;
            for l in &e.decision.lines {
                *x.1.entry(l.label.clone()).or_default() += l.value;
            }
        }
    }
    let mut out = format!("CREDITORS (issue 24; war loans over {} runs)\n", runs.len());
    for ((c, d), p) in &pairs {
        let _ = writeln!(
            out,
            "  {c} -> {d}: lent in {} runs, mean {:.2} per run with a loan; forgiven {} times ({:.2}); held {} assessments; outstanding at end {:.2} total; first hold turn median {:.0}",
            p.runs,
            p.lent / p.runs.max(1) as f64,
            p.forgiven.0,
            p.forgiven.1,
            p.held,
            p.outstanding,
            median(p.first_hold.clone())
        );
    }
    for ((who, forgive), (n, t)) in &terms {
        let mut v: Vec<_> = t.iter().map(|(k, x)| (k.clone(), x / *n as f64)).collect();
        v.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
        let _ = writeln!(
            out,
            "  {who} {} x{n}: {}",
            if *forgive { "FORGIVE" } else { "HOLD" },
            v.iter().map(|(k, x)| format!("{k} {x:+.1}")).collect::<Vec<_>>().join("; ")
        );
    }
    out
}

/// Turn-by-turn trace of every war in the given runs (issue 22): front,
/// both leaders' weariness, stability, debt and readiness, and their
/// `continue_war` score with its terms.
pub fn war_trace(runs: &[RunResult], codes: &[String]) -> String {
    use sim_core::DiplomaticEvent as E;
    let mut out = String::new();
    for r in runs {
        let mut wars: BTreeMap<u32, (u32, String, String)> = BTreeMap::new();
        let mut front: BTreeMap<(u32, u32), (f64, f64)> = BTreeMap::new();
        let mut ends: BTreeMap<u32, (u32, String)> = BTreeMap::new();
        let mut mobil: Vec<(u32, String, String)> = Vec::new();
        for (t, e) in &r.events {
            match e {
                E::WarDeclared {
                    war,
                    attacker,
                    defender,
                    ..
                } => {
                    wars.insert(
                        war.0,
                        (*t, codes[attacker.index()].clone(), codes[defender.index()].clone()),
                    );
                }
                E::FrontReport {
                    war, progress, ratio, ..
                } => {
                    front.insert((war.0, *t), (*progress, *ratio));
                }
                E::PeaceMade {
                    war, result, reason, ..
                } => {
                    ends.insert(war.0, (*t, format!("{result:?} {reason:?}")));
                }
                E::Mobilized { country, level } => {
                    mobil.push((*t, codes[country.index()].clone(), format!("{level:?}")));
                }
                _ => {}
            }
        }
        for (id, (s, a, d)) in &wars {
            let end = ends.get(id).map(|e| e.0).unwrap_or(r.turns);
            let _ = writeln!(
                out,
                "== seed {} war {id}: {a} -> {d}, turns {s}..{end} {}",
                r.seed,
                ends.get(id).map(|e| e.1.clone()).unwrap_or_default()
            );
            for t in *s..=end {
                for (_, c, l) in mobil.iter().filter(|m| m.0 == t && (m.1 == *a || m.1 == *d)) {
                    let _ = writeln!(out, "      {c} mobilizes: {l}");
                }
                let Some(smp) = r.samples.iter().find(|x| x.turn == t) else {
                    continue;
                };
                let c = |code: &str| smp.countries.iter().find(|y| y.code == code).cloned();
                let (Some(ca), Some(cd)) = (c(a), c(d)) else { continue };
                let (p, ratio) = front.get(&(*id, t)).copied().unwrap_or((f64::NAN, f64::NAN));
                let _ = writeln!(
                    out,
                    " t{t:<3} front {p:+5.0} ratio {ratio:4.2} | {a} wy {:4.1} st {:4.1} debt {:.2} rd {:.2} | {d} wy {:4.1} st {:4.1} debt {:.2} rd {:.2}",
                    ca.war_weariness,
                    ca.stability,
                    ca.debt_ratio,
                    ca.readiness,
                    cd.war_weariness,
                    cd.stability,
                    cd.debt_ratio,
                    cd.readiness
                );
                for e in r.reasoning.iter().filter(|e| {
                    e.turn == t
                        && e.decision.subject.starts_with("continue the war with")
                        && (e.country == *a || e.country == *d)
                }) {
                    let terms: Vec<String> = e
                        .decision
                        .lines
                        .iter()
                        .map(|l| format!("{} {:+.1}", l.label, l.value))
                        .collect();
                    let _ = writeln!(
                        out,
                        "      {} {} {:+.1}: {}",
                        e.country,
                        if e.decision.chosen { "FIGHT" } else { "OFFER" },
                        e.decision.score,
                        terms.join("; ")
                    );
                }
            }
        }
    }
    out
}
