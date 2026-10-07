//! Batch statistics for Gate 6 (AI variety) and the scenario plausibility
//! targets (DESIGN §22.1; scenario-1980 E.3). Everything is read from a
//! run's events, ledger log and reasoning records — the same output a
//! player-facing history panel would use.

use std::collections::{BTreeMap, BTreeSet};

use sim_core::{CauseCode, DecisionKind, DiplomaticEvent, Grade, LedgerLog, PeaceResult, StreamKind, TreatyKind};

use crate::RunResult;

/// The Gate 6 behaviours (DESIGN §22.1).
pub const BEHAVIOURS: [&str; 9] = [
    "intervene",
    "abstain",
    "coerce economically",
    "use proxies",
    "abandon peripheral commitment",
    "defend important commitment",
    "deepen trade",
    "reduce military burden",
    "escalate",
];

/// Informational rows printed with the Gate 6 table but not gate criteria
/// (D68). "armed a side": a power whose only war involvement in the run was
/// band 2, i.e. an arms stream to a belligerent (sim-core turns every arms
/// stream to a country at war into a band-2 involvement). Arms supply is
/// already counted under "use proxies"; it is not intervention.
pub const INFO_ROWS: [&str; 1] = ["armed a side"];

/// Pooled in-band range of a Gate 6 behaviour, `None` for informational
/// rows. All behaviours: 20–95% of (run, power) pairs, except intervene:
/// ≥5% (D68: the 1980 war set is Iran–Iraq, sometimes Iraq–Kuwait, rarely
/// India–Pakistan; nobody intervened in Iran–Iraq, Japan and West Germany
/// never did, so 20% would need an unhistorically eager AI).
pub fn gate6_band(behaviour: &str) -> Option<(f64, f64)> {
    match behaviour {
        "intervene" => Some((0.05, 0.95)),
        b if BEHAVIOURS.contains(&b) => Some((0.2, 0.95)),
        _ => None,
    }
}

/// Whether a pooled frequency is inside its Gate 6 band (informational
/// rows always are).
pub fn in_gate6_band(behaviour: &str, pooled: f64) -> bool {
    gate6_band(behaviour).is_none_or(|(lo, hi)| (lo..=hi).contains(&pooled))
}

/// Which Gate 6 behaviours each country showed in one run.
///
/// War involvement (D68): only fighting (band 3–4) is "intervene", and only
/// a fought involvement kept to the end or abandoned counts towards
/// "defend important commitment" / "abandon peripheral commitment". A band-2
/// involvement is an arms stream to a belligerent (often a pre-war sale the
/// war happened to catch), reported as "armed a side" and, via the stream
/// itself, "use proxies".
pub fn behaviours(r: &RunResult) -> BTreeMap<String, BTreeSet<&'static str>> {
    let codes: Vec<String> = r.samples[0].countries.iter().map(|c| c.code.clone()).collect();
    let code = |i: sim_core::CountryId| codes[i.index()].clone();
    let mut out: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    let mut mark = |c: String, b: &'static str| {
        out.entry(c).or_default().insert(b);
    };
    for (_, e) in &r.events {
        match e {
            DiplomaticEvent::JoinedWar { country, band, .. } => {
                mark(code(*country), "intervene");
                if *band >= 4 {
                    mark(code(*country), "escalate");
                }
            }
            DiplomaticEvent::WarDeclared { attacker, .. } => mark(code(*attacker), "escalate"),
            DiplomaticEvent::SanctionImposed { by, .. } => mark(code(*by), "coerce economically"),
            DiplomaticEvent::Mobilized { country, level } if level.level() >= 2 => mark(code(*country), "escalate"),
            DiplomaticEvent::StreamStarted { stream } if stream.kind == StreamKind::Arms => {
                mark(code(stream.from), "use proxies")
            }
            DiplomaticEvent::TreatySigned { treaty } if treaty.kind == (TreatyKind::Trade { deep: true }) => {
                mark(code(treaty.a), "deepen trade");
                mark(code(treaty.b), "deepen trade");
            }
            DiplomaticEvent::TreatyCancelled { treaty, by }
                if matches!(treaty.kind, TreatyKind::Guarantee | TreatyKind::DefensiveAlliance) =>
            {
                mark(code(*by), "abandon peripheral commitment")
            }
            _ => {}
        }
    }
    for (_, l) in &r.ledger_log {
        if let LedgerLog::EntryWritten {
            actor,
            grade: Some(g),
            code: cause,
            ..
        } = l
        {
            match (g, cause) {
                // Arms-only war involvement is neither (see above).
                (_, CauseCode::WarInvolvementKept | CauseCode::WarWithdrawal) if arms_only_entry(l) => {}
                (
                    Grade::Honoured,
                    CauseCode::AllyAttacked | CauseCode::WarInvolvementKept | CauseCode::CrisisSupportKept,
                ) => mark(code(*actor), "defend important commitment"),
                (Grade::Abandoned, CauseCode::SupportCut | CauseCode::WarWithdrawal) => {
                    mark(code(*actor), "abandon peripheral commitment")
                }
                _ => {}
            }
        }
    }
    for e in &r.reasoning {
        let d = &e.decision;
        if d.chosen && d.kind == DecisionKind::Budget && d.subject.starts_with("cut") {
            mark(e.country.clone(), "reduce military burden");
        }
        if d.chosen && d.kind == DecisionKind::Band(0) && d.subject.starts_with("involvement") {
            mark(e.country.clone(), "abstain");
        }
        if d.chosen && matches!(d.kind, DecisionKind::Band(3..=4)) && d.subject.starts_with("involvement") {
            mark(e.country.clone(), "intervene");
        }
        // Raising an existing involvement band is escalation (hold /
        // withdraw / escalate, DESIGN §14.9).
        if d.chosen && d.subject.starts_with("involvement") {
            if let (Some(now), Some(was)) = (band_in(&d.subject, ": band "), band_in(&d.subject, "(was band ")) {
                if now > was {
                    mark(e.country.clone(), "escalate");
                }
            }
        }
    }
    // Armed a side: band-2 involvement and no intervention in this run.
    for i in r.involvements.iter().filter(|i| i.max_band == 2) {
        let set = out.entry(i.actor.clone()).or_default();
        if !set.contains("intervene") {
            set.insert("armed a side");
        }
    }
    out
}

/// A war-involvement ledger entry for a band-2 (arms-stream) involvement.
/// sim-core writes the band into the cause ("withdrew from the war (band
/// 2)", "stood by its side to the end (band 2)").
fn arms_only_entry(l: &LedgerLog) -> bool {
    matches!(l, LedgerLog::EntryWritten { cause, .. } if band_in(cause, "(band ") == Some(2))
}

fn band_in(subject: &str, marker: &str) -> Option<u8> {
    let i = subject.find(marker)? + marker.len();
    subject[i..].chars().next()?.to_digit(10).map(|d| d as u8)
}

/// Plausibility facts per run (scenario-1980 E.3; the measurable subset).
#[derive(Clone, Debug, Default)]
pub struct Plausibility {
    /// Largest max ÷ min of the energy price within the run.
    pub price_swing: f64,
    pub wars: usize,
    pub nuclear_uses: usize,
    pub war_pairs: BTreeSet<(String, String)>,
    pub regime_changes: usize,
    /// Wars declared on a defender holding an arsenal, by aim (`{aim:?}`),
    /// with the pairs (D68 #5: limited wars on nuclear states are rare but
    /// possible; Major ones must stay nearly unthinkable).
    pub wars_on_arsenal: BTreeMap<String, Vec<(String, String)>>,
    /// Every declaration: attacker, defender, aim (`{aim:?}`).
    pub declared: Vec<(String, String, String)>,
    /// Declarations by an attacker that some country was pledged to defend
    /// at the time (F3: a guarantee underwriting an aggressor), with the
    /// attacker and its protectors.
    pub protected_attackers: Vec<(String, Vec<String>)>,
    /// Sampled turns in which `client` had `protector` pledged to it.
    pub protection_turns: BTreeMap<(String, String), usize>,
    pub sampled_turns: usize,
}

pub fn plausibility(r: &RunResult) -> Plausibility {
    let codes: Vec<String> = r.samples[0].countries.iter().map(|c| c.code.clone()).collect();
    let mut p = Plausibility::default();
    let prices: Vec<f64> = r.samples.iter().map(|s| s.energy_price).collect();
    let (lo, hi) = prices
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    p.price_swing = hi / lo.max(1e-9);
    p.sampled_turns = r.samples.len();
    for smp in &r.samples {
        for c in &smp.countries {
            for pr in &c.protectors {
                *p.protection_turns.entry((c.code.clone(), pr.clone())).or_insert(0) += 1;
            }
        }
    }
    for (t, e) in &r.events {
        match e {
            DiplomaticEvent::WarDeclared {
                attacker,
                defender,
                aim,
                ..
            } => {
                p.wars += 1;
                let (a, b) = (codes[attacker.index()].clone(), codes[defender.index()].clone());
                // The defender's arsenal on the last sample before the declaration.
                let armed = r
                    .samples
                    .iter()
                    .rev()
                    .find(|s| s.turn < *t)
                    .or(r.samples.first())
                    .is_some_and(|s| s.countries[defender.index()].arsenal > 0);
                p.declared.push((a.clone(), b.clone(), format!("{aim:?}")));
                let protectors = r
                    .samples
                    .iter()
                    .rev()
                    .find(|s| s.turn < *t)
                    .or(r.samples.first())
                    .map(|s| s.countries[attacker.index()].protectors.clone())
                    .unwrap_or_default();
                if !protectors.is_empty() {
                    p.protected_attackers.push((a.clone(), protectors));
                }
                if armed {
                    p.wars_on_arsenal
                        .entry(format!("{aim:?}"))
                        .or_default()
                        .push((a.clone(), b.clone()));
                }
                p.war_pairs.insert(if a < b { (a, b) } else { (b, a) });
            }
            DiplomaticEvent::JoinedWar { .. } => {}
            DiplomaticEvent::NuclearStrike { .. } => p.nuclear_uses += 1,
            DiplomaticEvent::PeaceMade {
                result: PeaceResult::AttackerWon,
                aim: sim_core::WarAim::Major,
                reason,
                ..
            } if *reason != sim_core::PeaceReason::Exhaustion => p.regime_changes += 1,
            _ => {}
        }
    }
    p
}

/// Share of runs per (power, behaviour).
pub type PerPower = BTreeMap<(String, &'static str), f64>;
/// Pooled share over all (run, power) pairs per behaviour.
pub type Pooled = BTreeMap<&'static str, f64>;

/// Gate 6 table: share of runs in which each listed power showed each
/// behaviour, plus the pooled share over all (run, power) pairs.
pub fn gate6_table(runs: &[RunResult], powers: &[&str]) -> (PerPower, Pooled) {
    let mut per = PerPower::new();
    let mut pooled = Pooled::new();
    let n = runs.len().max(1) as f64;
    for r in runs {
        let b = behaviours(r);
        for &p in powers {
            let set = b.get(p).cloned().unwrap_or_default();
            for &beh in BEHAVIOURS.iter().chain(&INFO_ROWS) {
                if set.contains(beh) {
                    *per.entry((p.to_string(), beh)).or_insert(0.0) += 1.0 / n;
                    *pooled.entry(beh).or_insert(0.0) += 1.0 / (n * powers.len() as f64);
                }
            }
        }
    }
    (per, pooled)
}

/// The Gate 6 powers: power share (½ GDP + ½ military) ≥ 5% at start.
pub fn gate6_powers(r: &RunResult) -> Vec<String> {
    let first = &r.samples[0].countries;
    let gdp: f64 = first.iter().map(|c| c.gdp).sum();
    let mil: f64 = first.iter().map(|c| c.military).sum();
    first
        .iter()
        .filter(|c| 0.5 * c.gdp / gdp + 0.5 * c.military / mil >= 0.05)
        .map(|c| c.code.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CountrySample, InvolvementSeen, LedgerStats, TurnSample};
    use sim_core::{CountryId, EntryKind, Side, WarId};

    fn country(code: &str) -> CountrySample {
        CountrySample {
            code: code.into(),
            gdp: 1.0,
            stability: 50.0,
            debt_ratio: 0.0,
            military: 1.0,
            security: 50.0,
            initiative: 3,
            military_share: 0.05,
            military_tech: 1,
            strength: 1.0,
            active: true,
            readiness: 1.0,
            arms_in: 0.0,
            legitimacy: 50.0,
            prosperity: 50.0,
            burden: 0.0,
            war_weariness: 0.0,
            growth: 0.0,
            democracy: true,
            behind: 0.0,
            inflation_hit: 0.0,
            debt_service_hit: 0.0,
            protectors: Vec::new(),
            arsenal: 0,
        }
    }

    fn entry(actor: u16, grade: Grade, code: CauseCode, cause: &str) -> (u32, LedgerLog) {
        (
            5,
            LedgerLog::EntryWritten {
                id: sim_core::ledger::EntryId(0),
                kind: EntryKind::Back,
                actor: CountryId(actor),
                counterpart: CountryId(3),
                grade: Some(grade),
                cause: cause.into(),
                code,
            },
        )
    }

    fn seen(actor: &str, war: u32, max_band: u8) -> InvolvementSeen {
        InvolvementSeen {
            actor: actor.into(),
            beneficiary: "CLI".into(),
            war,
            max_band,
        }
    }

    /// D68: arms-stream (band 2) involvement is "armed a side", never
    /// intervene, defend or abandon; fighting (band 3) is all three's business.
    #[test]
    fn arms_only_involvement_is_armed_a_side_not_intervention() {
        let codes = ["ARM", "FGT", "CUT", "CLI"];
        let r = RunResult {
            relations: Vec::new(),
            scenario: "t".into(),
            seed: 1,
            turns: 10,
            samples: vec![TurnSample {
                turn: 0,
                year: 1980.0,
                interest_rate: 0.0,
                monetary_stance: String::new(),
                energy_price: 1.0,
                countries: codes.iter().map(|c| country(c)).collect(),
            }],
            // ARM's AI chose to keep arming (a band-2 involvement decision):
            // commerce, not intervention.
            reasoning: vec![crate::ReasonEntry {
                turn: 2,
                country: "ARM".into(),
                decision: ai::DecisionRecord {
                    subject: "involvement in war 0: band 2".into(),
                    kind: DecisionKind::Band(2),
                    counterpart: Some(CountryId(3)),
                    score: 1.0,
                    chosen: true,
                    lines: Vec::new(),
                    precedents: Vec::new(),
                },
            }],
            ledger: LedgerStats::default(),
            ledger_log: vec![
                // ARM supplied to the end; CUT stopped its stream while the
                // client was losing; FGT fought to the end.
                entry(
                    0,
                    Grade::Honoured,
                    CauseCode::WarInvolvementKept,
                    "stood by its side to the end (band 2)",
                ),
                entry(
                    2,
                    Grade::Abandoned,
                    CauseCode::WarWithdrawal,
                    "withdrew from the war (band 2)",
                ),
                entry(
                    1,
                    Grade::Honoured,
                    CauseCode::WarInvolvementKept,
                    "stood by its side to the end (band 3)",
                ),
            ],
            narration: Vec::new(),
            events: vec![(
                2,
                DiplomaticEvent::JoinedWar {
                    war: WarId(0),
                    country: CountryId(1),
                    side: Side::Defender,
                    band: 3,
                },
            )],
            epitaph: None,
            involvements: vec![seen("ARM", 0, 2), seen("CUT", 0, 2), seen("FGT", 0, 3)],
            final_state: String::new(),
        };
        let b = behaviours(&r);
        let none = BTreeSet::new();
        let get = |c: &str| b.get(c).unwrap_or(&none);
        for c in ["ARM", "CUT"] {
            assert!(get(c).contains("armed a side"), "{c}: {:?}", get(c));
            for row in [
                "intervene",
                "defend important commitment",
                "abandon peripheral commitment",
            ] {
                assert!(!get(c).contains(row), "{c} counted as {row}");
            }
        }
        assert!(get("FGT").contains("intervene"));
        assert!(get("FGT").contains("defend important commitment"));
        assert!(!get("FGT").contains("armed a side"));
    }

    #[test]
    fn intervene_band_is_five_percent_and_up_others_twenty() {
        assert!(in_gate6_band("intervene", 0.05));
        assert!(!in_gate6_band("intervene", 0.049));
        assert!(!in_gate6_band("intervene", 0.96));
        assert!(!in_gate6_band("abstain", 0.10));
        assert!(in_gate6_band("abstain", 0.20));
        assert_eq!(gate6_band("armed a side"), None);
        assert!(in_gate6_band("armed a side", 0.0));
    }
}
