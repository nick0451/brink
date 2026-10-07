//! BRINK voice layer (design/behaviour-and-voice.md §2).
//!
//! Presentation only: renders what the simulation actually computed. It
//! never writes simulation state and never touches the simulation RNG.
//! V1 provides the **Plain** register: readable, exact renderings of AI
//! decision records and ledger events. Satirical registers (V1.5+) select
//! lines from the same structured inputs (`ReasonLine.term`,
//! `DecisionRecord.kind`, `LedgerEntry.cause_code`), so every line stays
//! true to the underlying state.

pub mod lines;
pub mod narrator;

use ai::DecisionRecord;
use sim_core::{CountryId, EntryKind, Grade, LedgerLog};

/// Satire intensity setting (design §2.2). Black is the intended default.
/// Gravity caps it further; G4–G5 are always plain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
pub enum Intensity {
    Off,
    Dry,
    #[default]
    Black,
}

/// Plain rendering of a decision: what was decided, the score, and the
/// terms that produced it (largest first), plus remembered precedents.
pub fn plain_decision(actor: &str, record: &DecisionRecord, max_terms: usize) -> String {
    let verdict = if record.chosen { "DECIDED" } else { "DECLINED" };
    let mut out = format!("{actor} {verdict}: {} (score {:+.1})", record.subject, record.score);
    let terms: Vec<String> = record
        .lines
        .iter()
        .take(max_terms)
        .map(|l| format!("{} {:+.1}", l.label, l.value))
        .collect();
    if !terms.is_empty() {
        out.push_str(&format!("\n    because: {}", terms.join("; ")));
    }
    if !record.precedents.is_empty() {
        out.push_str(&format!("\n    remembers: {}", record.precedents.join(" | ")));
    }
    out
}

fn grade_word(grade: Option<Grade>) -> &'static str {
    match grade {
        Some(Grade::Honoured) => "HONOURED",
        Some(Grade::Partial) => "PARTIAL",
        Some(Grade::Abandoned) => "ABANDONED",
        Some(Grade::Lapsed) => "LAPSED",
        None => "RECORDED",
    }
}

fn kind_word(kind: EntryKind) -> String {
    match kind {
        EntryKind::Back => "commitment".into(),
        EntryKind::Threat => "threat".into(),
        EntryKind::Norm(n) => format!("{n:?} norm"),
        EntryKind::Coercion => "coercion".into(),
    }
}

/// Plain rendering of a ledger development-log line, or `None` for lines
/// that need no player-facing text.
pub fn plain_ledger(names: &dyn Fn(CountryId) -> String, line: &LedgerLog) -> Option<String> {
    Some(match line {
        LedgerLog::EntryWritten {
            kind,
            actor,
            counterpart,
            grade,
            cause,
            ..
        } => format!(
            "LEDGER  {} → {}: {} {} ({cause})",
            names(*actor),
            names(*counterpart),
            kind_word(*kind),
            grade_word(*grade)
        ),
        LedgerLog::TestOpened {
            kind,
            actor,
            counterpart,
            deadline,
            cause,
        } => format!(
            "TEST    {} must answer for its {} toward {} by turn {deadline} ({cause})",
            names(*actor),
            kind_word(*kind),
            names(*counterpart)
        ),
        LedgerLog::ShadowTriggered { actor, beneficiary } => format!(
            "SHADOW  {} withdrew from {} shortly before it was attacked",
            names(*actor),
            names(*beneficiary)
        ),
        LedgerLog::NormCreated { norm, actor, cause, .. } => {
            format!("NORM    {} has established a {norm:?} norm ({cause})", names(*actor))
        }
        LedgerLog::NormTest {
            norm,
            actor,
            violator,
            response,
            deltas,
            ..
        } => format!(
            "NORM    {} answered {}'s {norm:?} violation: {} ({} observers revised their view)",
            names(*actor),
            names(*violator),
            grade_word(Some(*response)),
            deltas.len()
        ),
    })
}

/// The campaign epitaph (design/behaviour-and-voice.md §2): *The Official
/// History*, in the regime's own register, beside *The Ledger*, plain.
/// Every number comes from the run: the Back entries the country wrote,
/// its declared wars, and its growth.
pub fn epitaph(
    state: &sim_core::WorldState,
    country: CountryId,
    start_gdp: f64,
    lines: &[lines::Line],
    intensity: Intensity,
    seed: u64,
) -> (Option<String>, Vec<String>) {
    use sim_core::{CauseCode, EntryKind};
    let c = state.country(country);
    let mine = state.ledger.entries.iter().filter(|e| e.actor == country);
    let (mut honoured, mut partial, mut abandoned, mut wars) = (0, 0, 0, 0);
    for e in mine {
        match (e.kind, e.grade) {
            (EntryKind::Back, Some(Grade::Honoured)) => honoured += 1,
            (EntryKind::Back, Some(Grade::Partial)) => partial += 1,
            (EntryKind::Back, Some(Grade::Abandoned)) => abandoned += 1,
            _ => {}
        }
        if matches!(e.cause_code, CauseCode::WarDeclared | CauseCode::PunitiveStrike) {
            wars += 1;
        }
    }
    let years = state.turn / 4;
    let growth = 100.0 * (c.gdp / start_gdp.max(1e-9) - 1.0);
    let ledger = vec![
        format!("THE LEDGER — {} ({} years)", c.name, years),
        format!("  Commitments honoured: {honoured}. Partial: {partial}. Abandoned: {abandoned}."),
        format!("  Wars and strikes begun: {wars}."),
        format!("  Economy: {growth:+.0}%. Stability at the end: {:.0}.", c.stability),
    ];
    let pool: Vec<&lines::Line> = lines
        .iter()
        .filter(|l| l.trigger == lines::Trigger::Epitaph && l.intensity <= intensity)
        .collect();
    let best = pool.iter().map(|l| l.intensity).max();
    let pool: Vec<&&lines::Line> = pool.iter().filter(|l| Some(l.intensity) == best).collect();
    let official = (!pool.is_empty()).then(|| {
        let i = (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) as usize % pool.len();
        let mut text = pool[i].text.clone();
        for (k, v) in [
            ("actor", c.name.clone()),
            ("target", c.name.clone()),
            ("honoured", honoured.to_string()),
            ("partial", partial.to_string()),
            ("abandoned", abandoned.to_string()),
            ("wars", wars.to_string()),
            ("growth", format!("{growth:.0}")),
            ("years", years.to_string()),
        ] {
            text = text.replace(&format!("{{{k}}}"), &v);
        }
        format!("THE OFFICIAL HISTORY — {text}")
    });
    (official, ledger)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai::ReasonLine;
    use sim_core::{DecisionKind, ProposalFamily};

    #[test]
    fn plain_decision_is_exact() {
        let r = DecisionRecord {
            subject: "accept DefensiveAlliance from MAJ".into(),
            kind: DecisionKind::AcceptProposal(ProposalFamily::DefensiveAlliance),
            counterpart: Some(CountryId(0)),
            score: 24.8,
            chosen: true,
            lines: vec![ReasonLine {
                term: "security_need".into(),
                label: "security need".into(),
                value: 32.3,
            }],
            precedents: vec!["Honoured toward us: client in crisis (turn 5)".into()],
        };
        let text = plain_decision("NEU", &r, 5);
        assert!(text.contains("DECIDED") && text.contains("+24.8") && text.contains("security need +32.3"));
        assert!(text.contains("remembers: Honoured toward us"));
    }

    #[test]
    fn black_is_the_default_intensity() {
        assert_eq!(Intensity::default(), Intensity::Black);
    }
}
