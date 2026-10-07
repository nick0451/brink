//! Strategic assessment and goals (DESIGN §14.2–14.4, us-mechanics-report
//! §7). Every assessment the AI re-scores a small goal library from its own
//! view and keeps the top one or two. Goals persist (with hysteresis) and
//! shape budgets, trade, coalition building, retrenchment and balancing.
//!
//! There is no "post-Cold-War" mode: when a rival fades, threat-driven goals
//! lose fit, the defence burden starts to bite, and other goals take over.

use serde::{Deserialize, Serialize};
use sim_core::{BudgetShares, CountryId, EntryKind, ObserverView, TreatyKind};

use crate::inputs::{defence, foreign, pair_tension, threat, top_threat};
use crate::reflexes::power_share;
use crate::Score;

/// A goal score must reach this to become active.
pub const GOAL_THRESHOLD: f64 = 20.0;
/// Bonus for keeping a current goal (goals persist, DESIGN §14.1).
pub const GOAL_HYSTERESIS: f64 = 10.0;
pub const MAX_GOALS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Goal {
    /// Hold the main threat down: arms, sanctions, alliances against it.
    Contain(CountryId),
    /// Look after the countries we've promised to protect.
    ProtectClients,
    /// Spend on growth; trade; avoid entanglements.
    DevelopEconomy,
    /// Organise our bloc: alliances, guarantees, coordinated sanctions.
    LeadBloc,
    /// Stay far ahead militarily, threat or no threat.
    Primacy,
    /// Check a dominant power that coerces others.
    BalanceHegemon(CountryId),
    RestoreStability,
    /// A hated, weaker neighbour whose protection looks thin: an opening
    /// (DESIGN §14.3 opportunity score; "Exploit Crisis in [X]").
    Exploit(CountryId),
}

impl Goal {
    pub fn label(self, view: &ObserverView) -> String {
        let code = |c: CountryId| foreign(view, c).map_or_else(|| format!("#{}", c.0), |f| f.code.clone());
        match self {
            Goal::Contain(x) => format!("contain {}", code(x)),
            Goal::ProtectClients => "protect clients".into(),
            Goal::DevelopEconomy => "develop the economy".into(),
            Goal::LeadBloc => "lead the bloc".into(),
            Goal::Primacy => "primacy".into(),
            Goal::BalanceHegemon(x) => format!("balance against {}", code(x)),
            Goal::RestoreStability => "restore stability".into(),
            Goal::Exploit(x) => format!("exploit the opening against {}", code(x)),
        }
    }
}

/// Countries we've pledged to protect or that we fund.
pub fn clients(view: &ObserverView) -> Vec<CountryId> {
    let me = view.observer;
    let mut v: Vec<CountryId> = view
        .treaties
        .iter()
        .filter_map(|t| match t.kind {
            TreatyKind::Guarantee if t.a == me => Some(t.b),
            TreatyKind::DefensiveAlliance if t.involves(me) => t.other(me),
            _ => None,
        })
        .chain(view.streams.iter().filter(|s| s.from == me).map(|s| s.to))
        .collect();
    v.sort();
    v.dedup();
    v
}

/// How endangered a country looks: its highest public tension, 0–1.
pub fn danger(view: &ObserverView, c: CountryId) -> f64 {
    let ids = std::iter::once(view.observer).chain(view.others.iter().map(|f| f.id));
    ids.filter(|&x| x != c)
        .map(|x| pair_tension(view, c, x))
        .fold(0.0, f64::max)
        / 100.0
}

/// Rate of a country's coercion as the observer has seen it (IntentEst,
/// DESIGN §14.3), 0–1. Recent entries only.
pub fn intent(view: &ObserverView, c: CountryId) -> f64 {
    let recent = view
        .ledger
        .iter()
        .filter(|e| e.actor == c && e.kind == EntryKind::Coercion && e.turn + 20 >= view.turn as i32)
        .map(|e| e.weight)
        .sum::<f64>();
    (recent / 3.0).min(1.0)
}

/// Hostile pressure share H/(D+H) from the observer's estimates (mirrors
/// the simulation's Security driver).
pub fn pressure_share(view: &ObserverView) -> f64 {
    let d = defence(view);
    let h: f64 = view
        .others
        .iter()
        .map(|f| {
            let hostility = (f.tension / 100.0).max(-f.their_opinion_of_us / 100.0).clamp(0.0, 1.0);
            f.military.value * hostility * crate::inputs::reach(view, f)
        })
        .sum();
    if d + h > 1e-9 {
        h / (d + h)
    } else {
        0.0
    }
}

/// Own power share among all countries (estimated).
pub fn own_share(view: &ObserverView) -> f64 {
    let gdp_total: f64 = view.own.gdp + view.others.iter().map(|o| o.gdp).sum::<f64>();
    let mil_total: f64 = view.own.power() + view.others.iter().map(|o| o.military.value).sum::<f64>();
    0.5 * view.own.gdp / gdp_total.max(1e-9) + 0.5 * view.own.power() / mil_total.max(1e-9)
}

/// Own defence-burden pain, estimated from our own state: Legitimacy
/// points the unjustified share of military spending costs (mirrors N8).
pub fn burden_pain(view: &ObserverView) -> f64 {
    let justified = sim_core::domestic::justified_share(pressure_share(view));
    let excess = (view.own.budget.military - justified).max(0.0);
    sim_core::domestic::BURDEN_LEGITIMACY * excess * (0.5 + view.own.openness)
}

/// Score every goal (DESIGN §14.4: situational fit × personality weight).
/// Fit is 0–~1.5 from state; the personality weight (0.5 + trait) scales
/// it, so the same situation reads differently to different governments.
pub fn assess(view: &ObserverView) -> Vec<(Goal, Score)> {
    let p = view.own.personality;
    let mut out = Vec::new();
    let w = |t: f64| 0.5 + t;
    let pain = burden_pain(view);

    if let Some((x, t)) = top_threat(view) {
        let mut s = Score::new();
        let hostility = (x.tension / 100.0).max(-x.their_opinion_of_us / 100.0).clamp(0.0, 1.0);
        s.add("threat", t * w(p.paranoia));
        // A peer: a hostile power close to our own weight, not just big.
        let peer = (power_share(view, x) / own_share(view).max(1e-9)).min(1.0);
        s.add("a peer competitor", 100.0 * peer * hostility * w(p.paranoia));
        out.push((Goal::Contain(x.id), s));
    }

    let cl = clients(view);
    if !cl.is_empty() {
        let mut s = Score::new();
        // The worst-placed client we can actually reach (D79): an alliance
        // whose every member has a quarrel somewhere is not in danger
        // everywhere at once, and a client beyond our reach is not ours to
        // shield. Summing every client's worst quarrel saturated the goal for
        // any state with several allies, so it held a goal slot every turn.
        let d: f64 = cl
            .iter()
            .map(|&c| {
                let reach = view.others.iter().find(|f| f.id == c).map_or(1.0, |f| crate::inputs::reach_to(view, f));
                danger(view, c) * reach
            })
            .fold(0.0, f64::max);
        s.add("clients in danger", 70.0 * d * w(p.loyalty));
        out.push((Goal::ProtectClients, s));
    }

    {
        let mut s = Score::new();
        let t = top_threat(view).map_or(0.0, |x| x.1);
        s.add("calm surroundings", 0.35 * (100.0 - t) * w(p.greed));
        s.add("defence burden at home", pain);
        if view.own.debt_ratio() > 0.6 {
            s.add("debt", 15.0);
        }
        s.add("weak growth", (-800.0 * view.own.last_growth).clamp(0.0, 20.0));
        out.push((Goal::DevelopEconomy, s));
    }

    let share = own_share(view);
    if view.own.alignment.is_some()
        && view
            .others
            .iter()
            .any(|f| f.alignment.is_some() && f.alignment == view.own.alignment)
    {
        let mut s = Score::new();
        // A bloc matters when a rival bloc is hostile to us.
        let rival_bloc = view
            .others
            .iter()
            .filter(|f| matches!((&f.alignment, &view.own.alignment), (Some(a), Some(b)) if a != b))
            .map(|f| (f.tension / 100.0).max(-f.their_opinion_of_us / 100.0).clamp(0.0, 1.0))
            .fold(0.0, f64::max);
        s.add(
            "weight in the bloc",
            90.0 * share * w(p.ideology) * (0.4 + 0.6 * rival_bloc),
        );
        out.push((Goal::LeadBloc, s));
    }

    if share >= 0.25 {
        let mut s = Score::new();
        s.add("our lead", 120.0 * (share - 0.2) * w(p.aggression));
        s.add("defence burden at home", -pain);
        out.push((Goal::Primacy, s));
    }

    for f in &view.others {
        let fs = power_share(view, f);
        if fs >= 0.4 {
            // Balancing needs intent, not just strength (anti-dogpile,
            // DESIGN §14.9): dominance × their coercion record × paranoia.
            let mut s = Score::new();
            s.add(
                "their dominance × their coercion",
                300.0 * (fs - 0.4) * intent(view, f.id) * w(p.paranoia),
            );
            if is_ally(view, f.id) || clients_of(view, f.id).contains(&view.observer) {
                s.add("they protect us", -40.0);
            }
            out.push((Goal::BalanceHegemon(f.id), s));
        }
    }

    if view.own.stability < 45.0 {
        let mut s = Score::new();
        s.add("unrest", 3.0 * (45.0 - view.own.stability));
        out.push((Goal::RestoreStability, s));
    }

    // Opportunity: hostility × our edge over the target and whoever we
    // believe would protect it (our own Credibility(Back) reading of them).
    let best = view
        .others
        .iter()
        .map(|f| {
            let hostility = (f.tension / 100.0).max(-f.our_opinion_of_them / 100.0).clamp(0.0, 1.0);
            let protection: f64 = view
                .others
                .iter()
                .filter(|p| {
                    p.id != f.id
                        && view.treaties.iter().any(|t| match t.kind {
                            TreatyKind::DefensiveAlliance => t.involves(p.id) && t.involves(f.id),
                            TreatyKind::Guarantee => t.a == p.id && t.b == f.id,
                            _ => false,
                        })
                })
                .map(|p| p.military.value * 0.5 * p.credibility_back / 100.0)
                .sum();
            let odds = view.own.power() / (f.military.value + protection).max(1e-6);
            // An opening is a target in trouble (DESIGN §14.3: weakness ×
            // likely intervention), not merely a weaker one.
            let crisis = match f.stability_band {
                Some(sim_core::view::StabilityBand::Collapse | sim_core::view::StabilityBand::Crisis) => 1.0,
                Some(sim_core::view::StabilityBand::Unrest) => 0.7,
                Some(sim_core::view::StabilityBand::Normal) => 0.3,
                Some(sim_core::view::StabilityBand::Strong) => 0.1,
                None => 0.3,
            };
            (
                f,
                hostility * (odds - 0.8).clamp(0.0, 1.0) * crisis * crate::inputs::reach_to(view, f),
            )
        })
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.id.cmp(&a.0.id)));
    if let Some((f, fit)) = best.filter(|b| b.1 > 0.0) {
        let mut s = Score::new();
        s.add("an opening", 60.0 * fit * w(p.aggression) * w(p.opportunism));
        out.push((Goal::Exploit(f.id), s));
    }

    out.sort_by(|a, b| b.1.total().total_cmp(&a.1.total()).then(a.0.cmp(&b.0)));
    out
}

/// Goals within this many points of each other count as near-tied.
pub const NEAR_TIE: f64 = 8.0;
/// Softmax temperature among near-tied goals.
pub const TIE_TEMPERATURE: f64 = 4.0;

/// Countries `patron` protects or funds (public treaties and streams).
fn clients_of(view: &ObserverView, patron: CountryId) -> Vec<CountryId> {
    view.treaties
        .iter()
        .filter_map(|t| match t.kind {
            TreatyKind::Guarantee if t.a == patron => Some(t.b),
            _ => None,
        })
        .chain(view.streams.iter().filter(|s| s.from == patron).map(|s| s.to))
        .collect()
}

fn is_ally(view: &ObserverView, c: CountryId) -> bool {
    view.treaties
        .iter()
        .any(|t| t.kind == TreatyKind::DefensiveAlliance && t.involves(view.observer) && t.involves(c))
}

/// Pick up to [`MAX_GOALS`] goals above the threshold; current goals get a
/// hysteresis bonus. Among goals near-tied with the best remaining one, a
/// seeded softmax picks (DESIGN §14.9: variety from near-ties only; a
/// clearly better goal always wins).
pub fn choose(view: &ObserverView, current: &[Goal], seed: u64) -> Vec<(Goal, Score)> {
    let mut scored = assess(view);
    for (g, s) in &mut scored {
        if current.contains(g) {
            s.add("current goal", GOAL_HYSTERESIS);
        }
    }
    scored.retain(|(_, s)| s.total() >= GOAL_THRESHOLD);
    scored.sort_by(|a, b| b.1.total().total_cmp(&a.1.total()).then(a.0.cmp(&b.0)));
    let mut out = Vec::new();
    let mut slot = 0u64;
    while out.len() < MAX_GOALS && !scored.is_empty() {
        let best = scored[0].1.total();
        let ties: Vec<usize> = (0..scored.len())
            .filter(|&i| scored[i].1.total() >= best - NEAR_TIE)
            .collect();
        let weights: Vec<f64> = ties
            .iter()
            .map(|&i| ((scored[i].1.total() - best) / TIE_TEMPERATURE).exp())
            .collect();
        let total: f64 = weights.iter().sum();
        let mut r = unit(seed, view.observer.0 as u64, view.turn as u64 * 8 + slot) * total;
        let mut pick = ties[0];
        for (k, &i) in ties.iter().enumerate() {
            if r < weights[k] {
                pick = i;
                break;
            }
            r -= weights[k];
        }
        out.push(scored.remove(pick));
        slot += 1;
    }
    out
}

/// Deterministic value in [0, 1) for the AI's own seeded choices.
pub fn unit(seed: u64, a: u64, b: u64) -> f64 {
    let mix = |mut x: u64| {
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    };
    (mix(mix(mix(seed ^ 0x51_7cc1) ^ a) ^ b) >> 11) as f64 / (1u64 << 53) as f64
}

/// Desired military budget share and the terms behind it (in percentage
/// points of the budget).
pub fn military_target(view: &ObserverView, goals: &[Goal]) -> Score {
    let mut s = Score::new();
    // Doctrine: the share this state habitually spends, scaled by how
    // threatened it is now (0.6x in calm, 1.4x under full pressure) and by
    // war; goals nudge it. A peace dividend and a wartime surge both emerge.
    let base = if view.own.military_norm > 0.0 { view.own.military_norm } else { 0.15 };
    let growth = if view.own.norm_gdp > 0.0 { view.own.gdp / view.own.norm_gdp } else { 1.0 };
    // Only growth lowers the share; a shrinking economy can't buy back its
    // old army by spending a bigger slice of less.
    let norm = base / growth.max(1.0).sqrt();
    let pressure = pressure_share(view);
    s.add("our habitual share", 100.0 * base);
    s.add("a richer economy spends a smaller share", 100.0 * (norm - base));
    // 0.65x with no threat (a real peace dividend) to 1.35x under full pressure.
    s.add("the threat now", 100.0 * norm * (0.7 * pressure - 0.35));
    if view.wars.iter().any(|w| w.side_of(view.observer).is_some()) {
        s.add("at war", 100.0 * norm * 0.5);
    }
    let scale = norm / 0.2;
    for g in goals {
        let (label, v) = match g {
            Goal::Contain(_) => ("goal: containment", 3.0),
            Goal::Primacy => ("goal: primacy", 5.0),
            Goal::BalanceHegemon(_) => ("goal: balancing", 2.0),
            Goal::LeadBloc => ("goal: bloc leadership", 1.5),
            Goal::ProtectClients => ("goal: protecting clients", 1.5),
            Goal::DevelopEconomy => ("goal: development", -3.0),
            Goal::RestoreStability => ("goal: stability", -3.0),
            // Preparing a war of choice is a surge (Iraq roughly doubled its
            // military spending 1979-81), sized to the war: a great power
            // eyeing a small target needs no national mobilisation.
            Goal::Exploit(x) => {
                let size = foreign(view, *x).map_or(0.0, |f| f.military.value) / view.own.power().max(1e-9);
                ("goal: exploit an opening", 12.0 * (2.0 * size).min(1.0))
            }
        };
        s.add(label, v * scale);
    }
    s
}

/// New budget target: military at `military`, the freed (or needed) share
/// taken from or given to the other lines according to goals.
pub fn budget_for(current: BudgetShares, military: f64, goals: &[Goal]) -> BudgetShares {
    let military = military.clamp(0.03, 0.6);
    let freed = current.military - military;
    let (dev, wel) = if goals.contains(&Goal::DevelopEconomy) {
        (0.6, 0.3)
    } else if goals.contains(&Goal::RestoreStability) {
        (0.2, 0.7)
    } else {
        (0.4, 0.45)
    };
    BudgetShares {
        military,
        development: current.development + freed * dev,
        welfare: current.welfare + freed * wel,
        intelligence: current.intelligence + freed * (1.0 - dev - wel),
    }
    .normalized()
}

/// Is `x` the observer's main threat target among goals?
pub fn contains_target(goals: &[Goal], x: CountryId) -> bool {
    goals
        .iter()
        .any(|g| matches!(g, Goal::Contain(c) | Goal::BalanceHegemon(c) if *c == x))
}

/// Threat helper re-exported for strategist use.
pub fn threat_of(view: &ObserverView, c: CountryId) -> f64 {
    foreign(view, c).map_or(0.0, |f| threat(view, f))
}
