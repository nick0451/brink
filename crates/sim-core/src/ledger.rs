//! The Event Ledger: a shared, append-only, fog-filtered record of what
//! countries did (DESIGN §21). It is the only memory store; Trust and
//! Credibility are read from it per observer (see [`crate::reputation`]).
//!
//! The simulation writes entries; nobody edits them afterwards except to
//! flip a covert entry to `Exposed`.

use serde::{Deserialize, Serialize};

use crate::diplomacy::{DiplomaticEvent, TreatyKind};
use crate::ids::CountryId;
use crate::intel;
use crate::opinion::OpinionModifier;
use crate::reputation::{self, RepKind};
use crate::world::WorldState;

/// Turns a withdrawn or lapsed commitment stays in shadow (DESIGN §21.2).
pub const SHADOW_TURNS: u32 = 8;
/// Weight of an Abandoned grade produced by a shadow commitment.
pub const SHADOW_WEIGHT: f64 = 0.75;
/// Turns a claimant has to respond to a violation of its norm.
pub const NORM_RESPONSE_TURNS: u32 = 2;
/// Turns a patron must keep a support stream through a client's crisis.
pub const CRISIS_SUPPORT_TURNS: u32 = 4;
/// Stability below which a client is "in need".
pub const NEED_STABILITY: f64 = 40.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntryId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormTag {
    Aggression,
    Proliferation,
    ChokepointClosure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    /// Defence, guarantee, support stream, involvement.
    Back,
    /// Ultimatum, red line.
    Threat,
    /// An implied principle (DESIGN §21.4).
    Norm(NormTag),
    /// Sanction, punitive strike, intervention against someone. Ungraded;
    /// feeds intent estimates.
    Coercion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    Abandoned,
    Partial,
    Honoured,
    /// Ended by mutual agreement or without anyone in need: neutral.
    Lapsed,
}

impl Grade {
    /// Contribution to an expectation; `None` for neutral grades.
    pub fn score(self) -> Option<f64> {
        match self {
            Grade::Honoured => Some(1.0),
            Grade::Partial => Some(0.5),
            Grade::Abandoned => Some(0.0),
            Grade::Lapsed => None,
        }
    }
}

/// Structured cause of a ledger entry (stable key for presentation, logs
/// and the voice layer; `cause` stays the human-readable evidence text).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CauseCode {
    SanctionsImposed,
    Denunciation,
    SupportCut,
    CrisisSupportKept,
    AllianceCancelled,
    GuaranteeCancelled,
    ShadowAbandonment,
    NormClaimed,
    NormTested,
    Historical,
    WarDeclared,
    PunitiveStrike,
    BrokePact,
    AllyAttacked,
    WarInvolvementKept,
    WarWithdrawal,
    ArmedBothSides,
    CovertExposed,
    NuclearStrike,
    ProgrammeExposed,
    ArsenalDismantled,
    #[default]
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Covert,
    Exposed,
}

/// Set of observers, one bit per country.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observers(Vec<u64>);

impl Observers {
    pub fn contains(&self, c: CountryId) -> bool {
        self.0
            .get(c.index() / 64)
            .is_some_and(|w| w & (1 << (c.index() % 64)) != 0)
    }

    pub fn insert(&mut self, c: CountryId) {
        let word = c.index() / 64;
        if self.0.len() <= word {
            self.0.resize(word + 1, 0);
        }
        self.0[word] |= 1 << (c.index() % 64);
    }

    pub fn iter(&self, n: usize) -> impl Iterator<Item = CountryId> + '_ {
        (0..n).map(|i| CountryId(i as u16)).filter(|&c| self.contains(c))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: EntryId,
    /// Turn of the event; negative for seeded historical entries.
    pub turn: i32,
    pub actor: CountryId,
    /// Beneficiary (Back), target (Threat, Coercion) or violator (Norm test).
    pub counterpart: CountryId,
    pub kind: EntryKind,
    /// `None` for ungraded kinds (Coercion).
    pub grade: Option<Grade>,
    /// 0–1. Costly honouring counts for more.
    pub cost_paid: f64,
    /// Extra weight multiplier (e.g. [`SHADOW_WEIGHT`]).
    pub weight: f64,
    /// Local entries matter only to the counterpart (a calm withdrawal).
    pub local: bool,
    pub visibility: Visibility,
    pub seen_by: Observers,
    /// What produced the entry, for logs and "because" chains.
    pub cause: String,
    #[serde(default)]
    pub cause_code: CauseCode,
    /// Links the entry to the public statement made when the commitment
    /// was created (rendered by the voice layer; the simulation stores only
    /// the id).
    #[serde(default)]
    pub statement_id: Option<u32>,
}

/// A commitment that has been called on and awaits a response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingTest {
    pub actor: CountryId,
    pub counterpart: CountryId,
    pub kind: EntryKind,
    pub opened: u32,
    pub deadline: u32,
    /// Best response so far. Starts Abandoned.
    pub best: Grade,
    pub cost_paid: f64,
    pub cause: String,
    pub cause_code: CauseCode,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shadow {
    pub actor: CountryId,
    pub beneficiary: CountryId,
    pub kind: TreatyKind,
    pub until: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NormClaim {
    pub actor: CountryId,
    pub norm: NormTag,
    pub cause: String,
    pub turn: u32,
    pub entry: EntryId,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    pub entries: Vec<LedgerEntry>,
    pub tests: Vec<PendingTest>,
    pub shadows: Vec<Shadow>,
    pub norms: Vec<NormClaim>,
    next_id: u32,
}

/// Structured development log (DESIGN §21.4 instrumentation requirement).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LedgerLog {
    EntryWritten {
        id: EntryId,
        kind: EntryKind,
        actor: CountryId,
        counterpart: CountryId,
        grade: Option<Grade>,
        cause: String,
        code: CauseCode,
    },
    TestOpened {
        kind: EntryKind,
        actor: CountryId,
        counterpart: CountryId,
        deadline: u32,
        cause: String,
    },
    ShadowTriggered {
        actor: CountryId,
        beneficiary: CountryId,
    },
    NormCreated {
        norm: NormTag,
        actor: CountryId,
        cause: String,
        target: CountryId,
        turn: u32,
        seen_by: Vec<CountryId>,
    },
    NormTest {
        norm: NormTag,
        actor: CountryId,
        violator: CountryId,
        response: Grade,
        turn: u32,
        /// Change in each observer's Norm expectation of the actor.
        deltas: Vec<(CountryId, f64)>,
    },
}

/// Who sees an entry when it is written.
fn initial_observers(
    state: &WorldState,
    actor: CountryId,
    counterpart: CountryId,
    visibility: Visibility,
) -> Observers {
    let mut seen = Observers::default();
    for c in &state.countries {
        let sees = match visibility {
            Visibility::Public | Visibility::Exposed => true,
            Visibility::Covert => {
                c.id == actor
                    || c.id == counterpart
                    || intel::coverage(c, state.country(actor)) >= state.params.covert_visibility
            }
        };
        if sees {
            seen.insert(c.id);
        }
    }
    seen
}

#[allow(clippy::too_many_arguments)]
pub fn record(
    state: &mut WorldState,
    actor: CountryId,
    counterpart: CountryId,
    kind: EntryKind,
    grade: Option<Grade>,
    cost_paid: f64,
    weight: f64,
    local: bool,
    visibility: Visibility,
    cause_code: CauseCode,
    cause: impl Into<String>,
    log: &mut Vec<LedgerLog>,
) -> EntryId {
    let id = EntryId(state.ledger.next_id);
    state.ledger.next_id += 1;
    let cause = cause.into();
    let seen_by = initial_observers(state, actor, counterpart, visibility);
    log.push(LedgerLog::EntryWritten {
        id,
        kind,
        actor,
        counterpart,
        grade,
        cause: cause.clone(),
        code: cause_code,
    });
    state.ledger.entries.push(LedgerEntry {
        id,
        turn: state.turn as i32,
        actor,
        counterpart,
        kind,
        grade,
        cost_paid: cost_paid.clamp(0.0, 1.0),
        weight,
        local,
        visibility,
        seen_by,
        cause,
        cause_code,
        statement_id: None,
    });
    id
}

/// Seed a historical entry from scenario data (`turns_ago` before start).
pub fn seed_history(state: &mut WorldState, mut entry: LedgerEntry, turns_ago: u32) -> EntryId {
    let id = EntryId(state.ledger.next_id);
    state.ledger.next_id += 1;
    entry.id = id;
    entry.turn = -(turns_ago as i32);
    entry.seen_by = initial_observers(state, entry.actor, entry.counterpart, entry.visibility);
    state.ledger.entries.push(entry);
    id
}

/// A covert entry becomes public and highly salient (DESIGN §8.4).
pub fn expose(state: &mut WorldState, id: EntryId) {
    let n = state.countries.len();
    if let Some(e) = state.ledger.entries.iter_mut().find(|e| e.id == id) {
        if e.visibility == Visibility::Covert {
            e.visibility = Visibility::Exposed;
            for i in 0..n {
                e.seen_by.insert(CountryId(i as u16));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn open_test(
    state: &mut WorldState,
    actor: CountryId,
    counterpart: CountryId,
    kind: EntryKind,
    turns: u32,
    cause_code: CauseCode,
    cause: impl Into<String>,
    log: &mut Vec<LedgerLog>,
) {
    let cause = cause.into();
    let deadline = state.turn + turns;
    log.push(LedgerLog::TestOpened {
        kind,
        actor,
        counterpart,
        deadline,
        cause: cause.clone(),
    });
    state.ledger.tests.push(PendingTest {
        actor,
        counterpart,
        kind,
        opened: state.turn,
        deadline,
        best: Grade::Abandoned,
        cost_paid: 0.0,
        cause,
        cause_code,
    });
}

/// An actor publicly enforced a principle: write a Norm claim (DESIGN §21.4).
/// Called by band-3+ actions and ultimatums citing a crisis type (step 7).
pub fn create_norm(
    state: &mut WorldState,
    actor: CountryId,
    norm: NormTag,
    target: CountryId,
    cause: impl Into<String>,
    log: &mut Vec<LedgerLog>,
) {
    let cause = cause.into();
    if state.ledger.norms.iter().any(|c| c.actor == actor && c.norm == norm) {
        return;
    }
    let entry = record(
        state,
        actor,
        target,
        EntryKind::Norm(norm),
        Some(Grade::Honoured),
        0.5,
        1.0,
        false,
        Visibility::Public,
        CauseCode::NormClaimed,
        cause.clone(),
        log,
    );
    let n = state.countries.len();
    let seen_by = state.ledger.entries.last().unwrap().seen_by.iter(n).collect();
    log.push(LedgerLog::NormCreated {
        norm,
        actor,
        cause: cause.clone(),
        target,
        turn: state.turn,
        seen_by,
    });
    state.ledger.norms.push(NormClaim {
        actor,
        norm,
        cause,
        turn: state.turn,
        entry,
    });
}

/// Someone violated a norm: every other claimant faces a test.
pub fn report_violation(state: &mut WorldState, violator: CountryId, norm: NormTag, log: &mut Vec<LedgerLog>) {
    let claimants: Vec<CountryId> = state
        .ledger
        .norms
        .iter()
        .filter(|c| c.norm == norm && c.actor != violator)
        .map(|c| c.actor)
        .collect();
    for actor in claimants {
        let cause = format!("{norm:?} violation by country {}", violator.0);
        open_test(
            state,
            actor,
            violator,
            EntryKind::Norm(norm),
            NORM_RESPONSE_TURNS,
            CauseCode::NormTested,
            cause,
            log,
        );
    }
}

/// A country was attacked. Shadow commitments to it become Abandoned
/// (DESIGN §21.2). Back tests for live alliances/guarantees are opened by
/// the war system (build step 7), which also grades them by forces sent.
pub fn report_attack(state: &mut WorldState, victim: CountryId, log: &mut Vec<LedgerLog>) {
    let turn = state.turn;
    let triggered: Vec<Shadow> = state
        .ledger
        .shadows
        .iter()
        .filter(|s| s.beneficiary == victim && s.until >= turn)
        .cloned()
        .collect();
    state
        .ledger
        .shadows
        .retain(|s| !(s.beneficiary == victim && s.until >= turn));
    for s in triggered {
        log.push(LedgerLog::ShadowTriggered {
            actor: s.actor,
            beneficiary: s.beneficiary,
        });
        record(
            state,
            s.actor,
            s.beneficiary,
            EntryKind::Back,
            Some(Grade::Abandoned),
            0.0,
            SHADOW_WEIGHT,
            false,
            Visibility::Public,
            CauseCode::ShadowAbandonment,
            format!("withdrew {:?} shortly before an attack", s.kind),
            log,
        );
    }
}

/// Third-party reaction magnitude for an event (DESIGN §7.1). Positive for
/// supportive acts, negative for hostile ones. Returns (actor, target, magnitude).
fn reaction(event: &DiplomaticEvent) -> Option<(CountryId, CountryId, f64)> {
    use DiplomaticEvent::*;
    match *event {
        SanctionImposed { by, target } => Some((by, target, -10.0)),
        Denounced { by, target } => Some((by, target, -5.0)),
        DebtForgiven { creditor, debtor, .. } => Some((creditor, debtor, 5.0)),
        DebtHeld { creditor, debtor, .. } => Some((creditor, debtor, -5.0)),
        AidPledged { from, to, .. } | ArmsTransferred { from, to, .. } => Some((from, to, 5.0)),
        StreamStarted { stream } if !stream.covert => Some((stream.from, stream.to, 8.0)),
        StreamStopped { stream } if !stream.covert => Some((stream.from, stream.to, -5.0)),
        WarDeclared {
            attacker,
            defender,
            casus_belli,
            ..
        } => Some((attacker, defender, if casus_belli { -15.0 } else { -25.0 })),
        JoinedWar { .. } => None,
        TreatySigned { treaty } => match treaty.kind {
            TreatyKind::DefensiveAlliance | TreatyKind::Guarantee => Some((treaty.a, treaty.b, 6.0)),
            TreatyKind::Trade { .. } | TreatyKind::NonAggression | TreatyKind::Basing => {
                Some((treaty.a, treaty.b, 2.0))
            }
        },
        TreatyCancelled { treaty, by } => treaty.other(by).map(|other| (by, other, -5.0)),
        Mediated { by, a, .. } => Some((by, a, 3.0)),
        _ => None,
    }
}

fn apply_third_party_reaction(state: &mut WorldState, actor: CountryId, target: CountryId, magnitude: f64) {
    let ids: Vec<CountryId> = state.ids().collect();
    for o in ids {
        if o == actor || o == target {
            continue;
        }
        let observer = state.country(o);
        let delta = magnitude
            * (0.6 * state.opinions.opinion(o, target) / 100.0
                + 0.4 * observer.personality.ideology * observer.affinity(state.country(target)));
        if delta.abs() > 1e-6 {
            state.opinions.add(
                o,
                actor,
                OpinionModifier {
                    source: "reaction to their policy".into(),
                    value: delta,
                    decay: 0.5,
                    memory: None,
                },
            );
        }
    }
}

fn in_need(state: &WorldState, c: CountryId) -> bool {
    state.country(c).stability < NEED_STABILITY
}

/// Ledger phase (DESIGN §4.3 step 9): turn this turn's diplomatic events
/// into entries, tests, shadows and third-party reactions; then grade due
/// tests and open crisis tests.
pub fn process(state: &mut WorldState, events: &[DiplomaticEvent], log: &mut Vec<LedgerLog>) {
    for event in events {
        if let Some((actor, target, magnitude)) = reaction(event) {
            apply_third_party_reaction(state, actor, target, magnitude);
        }
        match *event {
            DiplomaticEvent::SanctionImposed { by, target } => {
                record(
                    state,
                    by,
                    target,
                    EntryKind::Coercion,
                    None,
                    0.0,
                    1.0,
                    false,
                    Visibility::Public,
                    CauseCode::SanctionsImposed,
                    "sanctions",
                    log,
                );
                respond_to_tests(state, by, target, Grade::Partial);
            }
            DiplomaticEvent::Denounced { by, target } => {
                record(
                    state,
                    by,
                    target,
                    EntryKind::Coercion,
                    None,
                    0.0,
                    0.5,
                    false,
                    Visibility::Public,
                    CauseCode::Denunciation,
                    "denunciation",
                    log,
                );
            }
            DiplomaticEvent::StreamStopped { stream }
                if !state
                    .wars
                    .involvements
                    .iter()
                    .any(|i| i.actor == stream.from && i.beneficiary == stream.to && i.band == 2) =>
            {
                // Cutting a client in need fails any open crisis test and is
                // itself an abandonment; otherwise it is neutral. A cut of
                // wartime proxy supply is graded by the war system instead.
                let had_test = state
                    .ledger
                    .tests
                    .iter()
                    .any(|t| t.actor == stream.from && t.counterpart == stream.to && t.kind == EntryKind::Back);
                state
                    .ledger
                    .tests
                    .retain(|t| !(t.actor == stream.from && t.counterpart == stream.to && t.kind == EntryKind::Back));
                let grade = if had_test || in_need(state, stream.to) {
                    Grade::Abandoned
                } else {
                    Grade::Lapsed
                };
                record(
                    state,
                    stream.from,
                    stream.to,
                    EntryKind::Back,
                    Some(grade),
                    0.0,
                    1.0,
                    false,
                    if stream.covert {
                        Visibility::Covert
                    } else {
                        Visibility::Public
                    },
                    CauseCode::SupportCut,
                    "cut support stream",
                    log,
                );
            }
            DiplomaticEvent::TreatyCancelled { treaty, by } => {
                if matches!(treaty.kind, TreatyKind::DefensiveAlliance | TreatyKind::Guarantee) {
                    let beneficiary = treaty.other(by).expect("party");
                    // A calm withdrawal costs reputation only with the former
                    // beneficiary, and leaves a shadow (DESIGN §21.2).
                    record(
                        state,
                        by,
                        beneficiary,
                        EntryKind::Back,
                        Some(Grade::Abandoned),
                        0.0,
                        1.0,
                        true,
                        Visibility::Public,
                        if treaty.kind == TreatyKind::Guarantee {
                            CauseCode::GuaranteeCancelled
                        } else {
                            CauseCode::AllianceCancelled
                        },
                        format!("cancelled {:?}", treaty.kind),
                        log,
                    );
                    state.ledger.shadows.push(Shadow {
                        actor: by,
                        beneficiary,
                        kind: treaty.kind,
                        until: state.turn + SHADOW_TURNS,
                    });
                }
            }
            _ => {}
        }
    }

    open_crisis_tests(state, log);
    grade_due_tests(state, log);
    let turn = state.turn;
    state.ledger.shadows.retain(|s| s.until >= turn);
}

/// Record a response by `actor` against `counterpart` for any matching test.
pub fn respond_to_tests(state: &mut WorldState, actor: CountryId, counterpart: CountryId, grade: Grade) {
    for t in state
        .ledger
        .tests
        .iter_mut()
        .filter(|t| t.actor == actor && t.counterpart == counterpart)
    {
        if grade > t.best && grade != Grade::Lapsed {
            t.best = grade;
        }
    }
}

/// A client in need with a live support stream opens a Back test for its
/// patron: keep the stream for [`CRISIS_SUPPORT_TURNS`] to honour it.
fn open_crisis_tests(state: &mut WorldState, log: &mut Vec<LedgerLog>) {
    let streams = state.diplomacy.streams.clone();
    for s in streams {
        let open = state
            .ledger
            .tests
            .iter()
            .any(|t| t.actor == s.from && t.counterpart == s.to && t.kind == EntryKind::Back);
        if !open && in_need(state, s.to) {
            open_test(
                state,
                s.from,
                s.to,
                EntryKind::Back,
                CRISIS_SUPPORT_TURNS,
                CauseCode::CrisisSupportKept,
                "client in crisis",
                log,
            );
            if let Some(t) = state.ledger.tests.last_mut() {
                t.best = Grade::Honoured; // kept unless the stream is cut
            }
        }
    }
}

fn grade_due_tests(state: &mut WorldState, log: &mut Vec<LedgerLog>) {
    let turn = state.turn;
    let (due, open): (Vec<_>, Vec<_>) = std::mem::take(&mut state.ledger.tests)
        .into_iter()
        .partition(|t| t.deadline <= turn);
    state.ledger.tests = open;
    let n = state.countries.len();
    for t in due {
        let before: Vec<f64> = match t.kind {
            EntryKind::Norm(_) => (0..n)
                .map(|o| reputation::expectation(state, CountryId(o as u16), t.actor, RepKind::Norm))
                .collect(),
            _ => Vec::new(),
        };
        let cost = if t.kind == EntryKind::Back && t.best == Grade::Honoured {
            // Cost of keeping a stream: its size relative to the patron's GDP.
            let amount: f64 = state
                .diplomacy
                .streams
                .iter()
                .filter(|s| s.from == t.actor && s.to == t.counterpart)
                .map(|s| s.value())
                .sum();
            (amount * CRISIS_SUPPORT_TURNS as f64 / state.country(t.actor).gdp * 10.0).min(1.0)
        } else {
            t.cost_paid
        };
        record(
            state,
            t.actor,
            t.counterpart,
            t.kind,
            Some(t.best),
            cost,
            1.0,
            false,
            Visibility::Public,
            t.cause_code,
            t.cause.clone(),
            log,
        );
        if let EntryKind::Norm(norm) = t.kind {
            let deltas = (0..n)
                .map(|o| {
                    let id = CountryId(o as u16);
                    (
                        id,
                        reputation::expectation(state, id, t.actor, RepKind::Norm) - before[o],
                    )
                })
                .filter(|(id, d)| *id != t.actor && d.abs() > 1e-9)
                .collect();
            log.push(LedgerLog::NormTest {
                norm,
                actor: t.actor,
                violator: t.counterpart,
                response: t.best,
                turn,
                deltas,
            });
        }
    }
}
