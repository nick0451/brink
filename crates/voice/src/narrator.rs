//! The narrator: detects satirical triggers in real simulation output and
//! selects at most a few lines per turn (design/behaviour-and-voice.md §2;
//! approval items 3, 4, 11, 12).
//!
//! - Every narration carries a plain **FACT** built from real numbers.
//! - A flavour line is attached only when a line for that trigger passes
//!   intensity, gravity, cooldown, per-campaign cap, per-turn budget and a
//!   deterministic fall-to-plain roll. Silence makes the next line land.
//! - Contradictions outrank everything else (priority 1 first).
//! - Pure read: the narrator never touches simulation state or its RNG.
//!   Like the headless log, V1.5 narrates as an omniscient observer;
//!   player-facing use will filter inputs through the viewer's fog.

use std::collections::{BTreeMap, BTreeSet};

use ai::DecisionRecord;
use sim_core::diplomacy::DiplomaticEvent;
use sim_core::{
    tension, CauseCode, CountryId, DecisionKind, EntryKind, Grade, LedgerLog, Mobilization, Order, PeaceResult,
    ProposalFamily, StreamKind, TreatyKind, TurnReport, WarAim, WarId, WorldState,
};

use crate::lines::{Line, Surface, Trigger};
use crate::Intensity;

/// Flavoured lines per turn: ordinary turns stay at 0–2; a cascade of
/// several high-priority triggers may reach 3 (approval item 12).
pub const BUDGET_BLACK: usize = 2;
pub const BUDGET_BLACK_CASCADE: usize = 3;
pub const BUDGET_DRY: usize = 1;
/// Share of eligible slots that stay plain anyway.
pub const FALL_TO_PLAIN: f64 = 0.3;
/// How long a denunciation counts as "recent" for hypocrisy-by-request.
pub const DENOUNCE_MEMORY: u32 = 20;
/// Client dependence (stream ÷ revenue) that makes a cut notable.
pub const NOTABLE_DEPENDENCE: f64 = 0.15;

/// One narrated situation: always a plain fact, sometimes a line.
#[derive(Clone, Debug, PartialEq)]
pub struct Narration {
    pub turn: u32,
    pub trigger: Trigger,
    pub gravity: u8,
    pub fact: String,
    /// `(heading, speaker, text, line id)` when a flavour line was selected.
    pub line: Option<(String, String, String, String)>,
    /// The hit's actor and target (who the fact is about), so a viewer's
    /// fog can decide whether the fact is one it could know.
    pub actor: CountryId,
    pub target: CountryId,
}

/// A detected trigger with the values its placeholders may use.
#[derive(Clone, Debug)]
struct Hit {
    trigger: Trigger,
    actor: CountryId,
    target: CountryId,
    values: BTreeMap<&'static str, String>,
    fact: String,
    /// Episode key: a trigger fires once per episode (e.g. per sanction).
    episode: Option<String>,
}

fn hit(
    trigger: Trigger,
    actor: CountryId,
    target: CountryId,
    values: Vec<(&'static str, String)>,
    fact: String,
    episode: Option<String>,
) -> Hit {
    Hit {
        trigger,
        actor,
        target,
        values: values.into_iter().collect(),
        fact,
        episode,
    }
}

pub struct TurnInput<'a> {
    /// State after the turn resolved.
    pub state: &'a WorldState,
    pub report: &'a TurnReport,
    /// Decisions taken during this turn's planning, by decider.
    pub decisions: &'a [(CountryId, DecisionRecord)],
}

pub struct Narrator {
    lines: Vec<Line>,
    intensity: Intensity,
    seed: u64,
    uses: BTreeMap<String, u32>,
    last_used: BTreeMap<String, u32>,
    /// (line, speaker) pairs already voiced: a speaker never repeats a line
    /// in a campaign, whatever the target (anti-fatigue).
    spoken: BTreeSet<(String, CountryId)>,
    /// Turn each line was last voiced by anyone: no echo within one turn.
    last_turn: BTreeMap<String, u32>,
    episodes: BTreeSet<String>,
    refusals: BTreeMap<(CountryId, CountryId), u32>,
    prev_deficit: Vec<f64>,
    prev_condition: Option<u8>,
    war_started: BTreeMap<WarId, u32>,
    /// Pair tensions before the turn resolved (for "calm before the attack").
    prev_tension: BTreeMap<(CountryId, CountryId), f64>,
    /// Each country's last announced goals (for strategy reversals).
    last_goals: BTreeMap<CountryId, String>,
    /// Nuclear silence: no flavour through this turn (approval item 7:
    /// the rest of the resolution plus the next decision turn).
    silent_until: Option<u32>,
    current_turn: u32,
}

fn pct(x: f64) -> String {
    format!("{:.1}", x * 100.0)
}

fn name(state: &WorldState, c: CountryId) -> String {
    state.country(c).name.clone()
}

fn kind_name(kind: TreatyKind) -> &'static str {
    match kind {
        TreatyKind::Trade { deep: true } => "deep trade agreement",
        TreatyKind::Trade { deep: false } => "trade agreement",
        TreatyKind::NonAggression => "non-aggression pact",
        TreatyKind::DefensiveAlliance => "alliance",
        TreatyKind::Guarantee => "guarantee",
        TreatyKind::Basing => "basing agreement",
    }
}

fn describe_order(order: &Order) -> &'static str {
    match order {
        Order::ProposeTreaty { .. } => "propose a treaty",
        Order::Respond { .. } => "answer a proposal",
        Order::CancelTreaty { .. } => "cancel a treaty",
        Order::IssueGuarantee { .. } => "issue a guarantee",
        Order::Sanction { .. } => "impose sanctions",
        Order::Aid { .. } => "send aid",
        Order::StartStream { .. } => "start a support programme",
        Order::ArmsTransfer { .. } => "ship arms",
        Order::StartArmsStream { .. } => "start an arms programme",
        Order::SellArms { .. } => "sell arms",
        Order::Denounce { .. } => "denounce anyone",
        Order::Mediate { .. } => "mediate",
        _ => "act",
    }
}

/// Deterministic value in [0, 1) for fall-to-plain and variant choice.
fn roll(seed: u64, turn: u32, key: &str) -> f64 {
    let mut x = seed ^ (turn as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for b in key.bytes() {
        x = (x ^ b as u64).wrapping_mul(0x100_0000_01B3);
    }
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 11) as f64 / (1u64 << 53) as f64
}

impl Narrator {
    pub fn new(lines: Vec<Line>, intensity: Intensity, seed: u64) -> Self {
        Narrator {
            lines,
            intensity,
            seed,
            uses: BTreeMap::new(),
            last_used: BTreeMap::new(),
            spoken: BTreeSet::new(),
            last_turn: BTreeMap::new(),
            episodes: BTreeSet::new(),
            refusals: BTreeMap::new(),
            prev_deficit: Vec::new(),
            prev_condition: None,
            war_started: BTreeMap::new(),
            prev_tension: BTreeMap::new(),
            last_goals: BTreeMap::new(),
            silent_until: None,
            current_turn: 0,
        }
    }

    /// Record the pre-turn baseline (deficits, readiness) before resolving.
    pub fn observe_before(&mut self, state: &WorldState) {
        self.prev_deficit = state.countries.iter().map(|c| c.deficit_ratio).collect();
        self.prev_condition = Some(tension::readiness_condition(state.global_tension));
        let ids: Vec<CountryId> = state.ids().collect();
        self.prev_tension = ids
            .iter()
            .flat_map(|&a| ids.iter().map(move |&b| (a, b)))
            .map(|(a, b)| ((a, b), state.tension.get(a, b)))
            .collect();
    }

    pub fn narrate(&mut self, input: TurnInput) -> Vec<Narration> {
        let mut hits = self.detect(&input);
        let turn = input.report.turn;
        for e in &input.report.events {
            if let DiplomaticEvent::NuclearStrike { by, target, .. } = *e {
                self.silent_until = Some(turn + 1);
                let st = input.state;
                hits.push(hit(
                    Trigger::NuclearUse,
                    by,
                    target,
                    vec![],
                    format!("{} used nuclear weapons against {}.", name(st, by), name(st, target)),
                    None,
                ));
            }
        }
        hits.retain(|h| match &h.episode {
            Some(key) => self.episodes.insert(key.clone()),
            None => true,
        });
        self.select(input.report.turn, input.state, hits)
    }

    fn decision<'a>(
        input: &'a TurnInput,
        decider: CountryId,
        kind: DecisionKind,
        counterpart: CountryId,
    ) -> Option<&'a DecisionRecord> {
        input
            .decisions
            .iter()
            .find(|(c, r)| *c == decider && r.kind == kind && r.counterpart == Some(counterpart))
            .map(|(_, r)| r)
    }

    fn detect(&mut self, input: &TurnInput) -> Vec<Hit> {
        let st = input.state;
        let turn = input.report.turn;
        let mut hits = Vec::new();

        for event in &input.report.events {
            match event {
                DiplomaticEvent::ProposalRefused { proposal: p } => {
                    let (refuser, asker) = (p.to, p.from);
                    let family = ProposalFamily::of(p.kind);
                    let record = Self::decision(input, refuser, DecisionKind::AcceptProposal(family), asker);
                    let worst = record.and_then(|r| {
                        r.lines
                            .iter()
                            .filter(|l| l.value < 0.0)
                            .min_by(|a, b| a.value.total_cmp(&b.value))
                            .cloned()
                    });
                    if p.kind == TreatyKind::Guarantee {
                        let n = self.refusals.entry((asker, refuser)).or_insert(0);
                        *n += 1;
                        let n = *n;
                        let term = worst.as_ref().map_or("unstated".to_string(), |l| l.label.clone());
                        hits.push(hit(
                            Trigger::GuaranteeRefused,
                            refuser,
                            asker,
                            vec![("term", term.clone())],
                            format!(
                                "{} declined {}'s request for a guarantee. Decisive factor: {term}.",
                                name(st, refuser),
                                name(st, asker)
                            ),
                            None,
                        ));
                        if n >= 2 {
                            hits.push(hit(
                                Trigger::RepeatedRequestRefused,
                                refuser,
                                asker,
                                vec![("n", n.to_string())],
                                format!(
                                    "{} has asked {} for a guarantee {n} times. All declined.",
                                    name(st, asker),
                                    name(st, refuser)
                                ),
                                None,
                            ));
                        }
                    }
                    if st.diplomacy.streams.iter().any(|s| s.from == asker && s.to == refuser) {
                        hits.push(hit(
                            Trigger::RefusedPatron,
                            refuser,
                            asker,
                            vec![("treaty", kind_name(p.kind).to_string())],
                            format!(
                                "{} refused a {} from {}, which funds it.",
                                name(st, refuser),
                                kind_name(p.kind),
                                name(st, asker)
                            ),
                            None,
                        ));
                    }
                    if worst.as_ref().is_some_and(|l| l.term == "status_quo") {
                        hits.push(hit(
                            Trigger::StatusQuoRefusal,
                            refuser,
                            asker,
                            vec![("treaty", kind_name(p.kind).to_string())],
                            format!(
                                "{} declined {}'s {}. Decisive factor: inertia.",
                                name(st, refuser),
                                name(st, asker),
                                kind_name(p.kind)
                            ),
                            None,
                        ));
                    }
                }
                DiplomaticEvent::Proposed { proposal: p } if p.kind == TreatyKind::Guarantee => {
                    let denounced = st.ledger.entries.iter().filter(|e| {
                        e.actor == p.from
                            && e.counterpart == p.to
                            && e.cause_code == CauseCode::Denunciation
                            && e.turn >= turn as i32 - DENOUNCE_MEMORY as i32
                    });
                    if let Some(latest) = denounced.max_by_key(|e| e.turn) {
                        let ago = (turn as i32 - latest.turn).max(0);
                        hits.push(hit(
                            Trigger::ProtectionFromDenounced,
                            p.from,
                            p.to,
                            vec![("turns_ago", ago.to_string())],
                            format!(
                                "{} requested a guarantee from {}, which it denounced {ago} turns ago.",
                                name(st, p.from),
                                name(st, p.to)
                            ),
                            None,
                        ));
                    }
                }
                DiplomaticEvent::SanctionImposed { by, target } => {
                    let (by, target) = (*by, *target);
                    if st
                        .diplomacy
                        .find_treaty(TreatyKind::Trade { deep: true }, by, target)
                        .is_some_and(|t| t.kind == TreatyKind::Trade { deep: true })
                    {
                        hits.push(hit(
                            Trigger::DeepTradeThenSanction,
                            by,
                            target,
                            vec![],
                            format!(
                                "Deep trade agreement between {} and {} in force. {} imposes sanctions.",
                                name(st, by),
                                name(st, target),
                                name(st, by)
                            ),
                            None,
                        ));
                    }
                    let adapted = st.country(target).sanction_adaptation;
                    // Adaptation gained this turn is 10%; more means it was learned earlier.
                    if adapted > sim_core::trade::ADAPTATION_PER_TURN + 1e-9 {
                        hits.push(hit(
                            Trigger::SanctionsReimposed,
                            by,
                            target,
                            vec![("rerouted", pct(adapted))],
                            format!(
                                "Sanctions on {} reimposed; {}% of its lost trade is already rerouted.",
                                name(st, target),
                                pct(adapted)
                            ),
                            None,
                        ));
                    }
                    if let Some(r) = input
                        .decisions
                        .iter()
                        .find(|(c, r)| *c == by && r.kind == DecisionKind::JoinSanction && r.chosen)
                    {
                        let top =
                            r.1.lines
                                .iter()
                                .filter(|l| l.value > 0.0)
                                .max_by(|a, b| a.value.total_cmp(&b.value));
                        if let Some(top) = top.filter(|l| l.term.contains("solidarity") || l.term.contains("standing"))
                        {
                            let leader = r.1.counterpart.unwrap_or(target);
                            hits.push(hit(
                                Trigger::CoalitionByDependence,
                                by,
                                leader,
                                vec![("term", top.label.clone())],
                                format!(
                                    "{} joined {}'s sanctions on {}. Decisive factor: {}.",
                                    name(st, by),
                                    name(st, leader),
                                    name(st, target),
                                    top.label
                                ),
                                None,
                            ));
                        }
                    }
                }
                DiplomaticEvent::SanctionLifted { by, target } => {
                    let adapted = st.country(*target).sanction_adaptation;
                    if adapted >= 0.3 {
                        hits.push(hit(
                            Trigger::SanctionsLiftedAfterAdaptation,
                            *by,
                            *target,
                            vec![("rerouted", pct(adapted))],
                            format!(
                                "Sanctions on {} lifted after it rerouted {}% of the lost trade.",
                                name(st, *target),
                                pct(adapted)
                            ),
                            None,
                        ));
                    }
                }
                DiplomaticEvent::StreamStopped { stream } => {
                    let recipient = st.country(stream.to);
                    let dependence = stream.value() / (recipient.gdp * recipient.tax_rate).max(1e-9);
                    let what = match stream.kind {
                        StreamKind::Aid => "support",
                        StreamKind::Arms => "arms supply",
                    };
                    if dependence >= NOTABLE_DEPENDENCE {
                        hits.push(hit(
                            Trigger::ClientCutWhileDependent,
                            stream.from,
                            stream.to,
                            vec![
                                ("dependence", pct(dependence)),
                                ("amount", format!("{:.2}", stream.amount)),
                            ],
                            format!(
                                "{} terminated {what} to {} at a dependence of {}% of its revenue.",
                                name(st, stream.from),
                                name(st, stream.to),
                                pct(dependence)
                            ),
                            None,
                        ));
                    }
                }
                DiplomaticEvent::TreatyCancelled { treaty, by } => {
                    if matches!(treaty.kind, TreatyKind::DefensiveAlliance | TreatyKind::Guarantee) {
                        let other = treaty.other(*by).unwrap_or(treaty.b);
                        hits.push(hit(
                            Trigger::CommitmentCancelled,
                            *by,
                            other,
                            vec![("treaty", kind_name(treaty.kind).to_string())],
                            format!(
                                "{} cancelled its {} with {}.",
                                name(st, *by),
                                kind_name(treaty.kind),
                                name(st, other)
                            ),
                            None,
                        ));
                    }
                }
                _ => {}
            }
        }

        self.detect_war(input, &mut hits);
        self.detect_v3(input, &mut hits);

        for line in &input.report.ledger_log {
            match line {
                LedgerLog::ShadowTriggered { actor, beneficiary } => hits.push(hit(
                    Trigger::ShadowThenAttack,
                    *actor,
                    *beneficiary,
                    vec![],
                    format!(
                        "{} withdrew its commitment to {} shortly before {} was attacked.",
                        name(st, *actor),
                        name(st, *beneficiary),
                        name(st, *beneficiary)
                    ),
                    None,
                )),
                LedgerLog::NormTest {
                    norm,
                    actor,
                    violator,
                    response,
                    ..
                } => {
                    let friendly = st.opinions.opinion(*actor, *violator) > 10.0
                        || st
                            .diplomacy
                            .has_treaty(TreatyKind::DefensiveAlliance, *actor, *violator);
                    let norm_s = format!("{norm:?}").to_lowercase();
                    if *response == Grade::Abandoned && friendly {
                        hits.push(hit(
                            Trigger::NormIgnoredByFriend,
                            *actor,
                            *violator,
                            vec![("norm", norm_s.clone())],
                            format!(
                                "{} holds an {norm_s} norm. Violation by friendly {}. Response: none.",
                                name(st, *actor),
                                name(st, *violator)
                            ),
                            None,
                        ));
                    } else if *response == Grade::Partial {
                        hits.push(hit(
                            Trigger::NormPartialResponse,
                            *actor,
                            *violator,
                            vec![("norm", norm_s.clone())],
                            format!(
                                "{} answered {}'s {norm_s} violation with sanctions only (graded Partial).",
                                name(st, *actor),
                                name(st, *violator)
                            ),
                            None,
                        ));
                    }
                }
                LedgerLog::EntryWritten {
                    kind: EntryKind::Back,
                    actor,
                    counterpart,
                    grade: Some(Grade::Honoured),
                    code: CauseCode::CrisisSupportKept,
                    ..
                } => hits.push(hit(
                    Trigger::CrisisSupportKept,
                    *actor,
                    *counterpart,
                    vec![],
                    format!(
                        "{} kept supporting {} through its crisis (graded Honoured).",
                        name(st, *actor),
                        name(st, *counterpart)
                    ),
                    None,
                )),
                _ => {}
            }
        }

        // Standing sanctions: self-inflicted costs and substitution (once per episode).
        for s in &st.diplomacy.sanctions {
            let (by, target) = (st.country(s.by), st.country(s.target));
            let own = by.trade_lost / by.gdp;
            let theirs = target.trade_lost / target.gdp;
            if own > theirs && own > 0.005 {
                hits.push(hit(
                    Trigger::SanctionsHurtSender,
                    s.by,
                    s.target,
                    vec![("own_loss", pct(own)), ("target_loss", pct(theirs))],
                    format!(
                        "{} sanctions {}. {}'s trade lost to sanctions: {}% of GDP. {}'s own losses from the sanctions it imposes: {}%.",
                        by.name,
                        target.name,
                        target.name,
                        pct(theirs),
                        by.name,
                        pct(own)
                    ),
                    Some(format!("hurt:{}:{}:{}", s.by.0, s.target.0, s.since)),
                ));
            }
            if target.sanction_adaptation >= 0.5 {
                hits.push(hit(
                    Trigger::SanctionsSubstituted,
                    s.by,
                    s.target,
                    vec![("rerouted", pct(target.sanction_adaptation))],
                    format!(
                        "{} has rerouted {}% of the trade lost to {}'s sanctions.",
                        target.name,
                        pct(target.sanction_adaptation),
                        by.name
                    ),
                    Some(format!("subst:{}:{}:{}", s.by.0, s.target.0, s.since)),
                ));
            }
        }

        // The debt brake: deficit spending stopped under heavy debt.
        for c in &st.countries {
            let before = self.prev_deficit.get(c.id.index()).copied().unwrap_or(0.0);
            if before > 0.0 && c.deficit_ratio == 0.0 && c.debt_ratio() > ai::DEBT_LIMIT {
                hits.push(hit(
                    Trigger::DebtBrake,
                    c.id,
                    c.id,
                    vec![("debt", pct(c.debt_ratio()))],
                    format!(
                        "{} ended deficit spending at debt of {}% of annual GDP.",
                        c.name,
                        pct(c.debt_ratio())
                    ),
                    None,
                ));
            }
        }

        for (country, order, reason) in &input.report.rejected {
            if reason == "not enough Initiative" && country.index() < st.countries.len() {
                let action = describe_order(order);
                hits.push(hit(
                    Trigger::OutOfInitiative,
                    *country,
                    *country,
                    vec![("action", action.to_string())],
                    format!("{} could not {action}: no Initiative left.", name(st, *country)),
                    None,
                ));
            }
        }

        let condition = tension::readiness_condition(st.global_tension);
        if self.prev_condition.is_some_and(|prev| condition < prev) {
            hits.push(hit(
                Trigger::ReadinessRising,
                CountryId(0),
                CountryId(0),
                vec![
                    ("condition", condition.to_string()),
                    ("tension", format!("{:.0}", st.global_tension)),
                ],
                format!(
                    "Global tension {:.0}: Readiness Condition {condition}.",
                    st.global_tension
                ),
                None,
            ));
        }
        hits
    }

    /// V2 war registers. Casualty figures appear only in plain FACTs.
    fn detect_war(&mut self, input: &TurnInput, hits: &mut Vec<Hit>) {
        let st = input.state;
        let turn = input.report.turn;
        for e in &input.report.events {
            match *e {
                DiplomaticEvent::WarDeclared {
                    war,
                    attacker,
                    defender,
                    aim,
                    casus_belli,
                    ..
                } => {
                    self.war_started.insert(war, turn);
                    let readiness = st.country(defender).forces.readiness;
                    if readiness < 0.3 {
                        hits.push(hit(
                            Trigger::CutMilitarySpendingThenThreatened,
                            defender,
                            attacker,
                            vec![("readiness", format!("{:.0}", 100.0 * readiness))],
                            format!(
                                "{} was attacked with its forces at {:.0}% readiness after spending cuts.",
                                name(st, defender),
                                100.0 * readiness
                            ),
                            None,
                        ));
                    }
                    let tension = self
                        .prev_tension
                        .get(&(attacker, defender))
                        .copied()
                        .unwrap_or_else(|| st.tension.get(attacker, defender));
                    if !casus_belli {
                        hits.push(hit(
                            Trigger::WarWithoutCasusBelli,
                            attacker,
                            defender,
                            vec![("tension", format!("{tension:.0}"))],
                            format!(
                                "{} attacked {} without a casus belli (tension before the attack {tension:.0}, below 40).",
                                name(st, attacker),
                                name(st, defender)
                            ),
                            None,
                        ));
                    } else if aim == WarAim::Punitive {
                        hits.push(hit(
                            Trigger::PunitiveStrike,
                            attacker,
                            defender,
                            vec![],
                            format!(
                                "{} launched a punitive strike on {}.",
                                name(st, attacker),
                                name(st, defender)
                            ),
                            None,
                        ));
                    } else {
                        let aim_s = if aim == WarAim::Major { "major" } else { "limited" };
                        hits.push(hit(
                            Trigger::WarBegan,
                            attacker,
                            defender,
                            vec![("aim", aim_s.to_string())],
                            format!(
                                "{} declared war on {} with {aim_s} aims.",
                                name(st, attacker),
                                name(st, defender)
                            ),
                            None,
                        ));
                    }
                }
                DiplomaticEvent::WarEscalated { war, .. } => {
                    if let Some(w) = st.wars.find(war) {
                        hits.push(hit(
                            Trigger::StrikeRefusedBecameWar,
                            w.attacker,
                            w.defender,
                            vec![],
                            format!(
                                "{} refused to accept {}'s punitive strike; it is now a limited war.",
                                name(st, w.defender),
                                name(st, w.attacker)
                            ),
                            None,
                        ));
                    }
                }
                DiplomaticEvent::PeaceMade {
                    war,
                    attacker,
                    defender,
                    aim,
                    result,
                    ..
                } => {
                    let turns = self.war_started.remove(&war).map_or(0, |t| turn.saturating_sub(t));
                    if result == PeaceResult::WhitePeace && aim != WarAim::Punitive && turns >= 4 {
                        hits.push(hit(
                            Trigger::WhitePeaceAfterWar,
                            attacker,
                            defender,
                            vec![("turns", turns.to_string())],
                            format!(
                                "{} and {} made peace after {turns} quarters of war; neither side gained anything.",
                                name(st, attacker),
                                name(st, defender)
                            ),
                            None,
                        ));
                    }
                }
                DiplomaticEvent::StreamStarted { stream } if stream.sale && !stream.covert => {
                    for x in sim_core::diplomacy::sale_objectors(st, stream.from, stream.to) {
                        // Lines about paying the seller need a real payment:
                        // `funds` exists only when the objector funds it.
                        let paid: f64 = st
                            .diplomacy
                            .streams
                            .iter()
                            .filter(|s| s.from == x && s.to == stream.from && !s.sale)
                            .map(|s| s.value())
                            .sum();
                        let seller = st.country(stream.from);
                        let mut values = vec![("buyer", name(st, stream.to))];
                        if paid > 0.0 {
                            let share = 100.0 * paid / (seller.gdp * seller.tax_rate).max(1e-9);
                            values.push(("funds", format!("{share:.1}")));
                        }
                        hits.push(hit(
                            Trigger::ArmsSoldToPatronsRival,
                            x,
                            stream.from,
                            values,
                            format!(
                                "{} objected to {} selling arms to {}.",
                                name(st, x),
                                name(st, stream.from),
                                name(st, stream.to)
                            ),
                            Some(format!("sale:{}:{}:{}", x.0, stream.from.0, stream.to.0)),
                        ));
                    }
                }
                DiplomaticEvent::CovertExposed { from, to } => hits.push(hit(
                    Trigger::CovertArmsExposed,
                    from,
                    to,
                    vec![],
                    format!(
                        "{}'s covert arms supply to {} was exposed.",
                        name(st, from),
                        name(st, to)
                    ),
                    None,
                )),
                DiplomaticEvent::Mobilized { country, level }
                    if level.level() >= Mobilization::Full.level() && !st.wars.is_belligerent(country) =>
                {
                    hits.push(hit(
                        Trigger::MobilizedAtPeace,
                        country,
                        country,
                        vec![("level", format!("{level:?}").to_lowercase())],
                        format!("{} moved to {level:?} mobilisation while at peace.", name(st, country)),
                        None,
                    ));
                }
                _ => {}
            }
        }

        let mut both_sides: BTreeMap<CountryId, Vec<CountryId>> = BTreeMap::new();
        for l in &input.report.ledger_log {
            let LedgerLog::EntryWritten {
                actor,
                counterpart,
                grade: Some(grade),
                code,
                ..
            } = l
            else {
                continue;
            };
            match (code, grade) {
                (CauseCode::AllyAttacked, Grade::Abandoned) => hits.push(hit(
                    Trigger::GuarantorStayedOut,
                    *actor,
                    *counterpart,
                    vec![],
                    format!(
                        "{} was attacked. {}, pledged to defend it, did nothing (graded Abandoned).",
                        name(st, *counterpart),
                        name(st, *actor)
                    ),
                    None,
                )),
                (CauseCode::AllyAttacked, Grade::Partial) => {
                    let fighting = st
                        .wars
                        .wars_of(*counterpart)
                        .find_map(|w| w.belligerent(*actor).map(|b| (w, b.allocation)));
                    if let Some((w, alloc)) = fighting {
                        let side = w.side_of(*counterpart).expect("belligerent");
                        let enemy: f64 = w
                            .side(side.other())
                            .map(|b| st.country(b.country).power() * b.allocation)
                            .sum();
                        let committed = st.country(*actor).power() * alloc;
                        hits.push(hit(
                            Trigger::TokenForces,
                            *actor,
                            *counterpart,
                            vec![
                                ("committed", format!("{committed:.0}")),
                                ("needed", format!("{:.0}", 0.5 * enemy)),
                            ],
                            format!(
                                "{} joined the war for {} with {committed:.0} power against the {:.0} needed to honour it (graded Partial).",
                                name(st, *actor),
                                name(st, *counterpart),
                                0.5 * enemy
                            ),
                            None,
                        ));
                    } else {
                        let arming = st
                            .diplomacy
                            .streams
                            .iter()
                            .any(|s| s.from == *actor && s.to == *counterpart && s.kind == StreamKind::Arms);
                        let response = if arming { "arms shipments" } else { "sanctions" };
                        hits.push(hit(
                            Trigger::GuarantorSentSanctions,
                            *actor,
                            *counterpart,
                            vec![("response", response.to_string())],
                            format!(
                                "{} was attacked. {}, pledged to defend it, sent {response} (graded Partial).",
                                name(st, *counterpart),
                                name(st, *actor)
                            ),
                            None,
                        ));
                    }
                }
                (CauseCode::ArmedBothSides, _) => both_sides.entry(*actor).or_default().push(*counterpart),
                _ => {}
            }
        }
        for (supplier, sides) in both_sides {
            if let [a, b, ..] = sides[..] {
                hits.push(hit(
                    Trigger::ArmsToBothSides,
                    supplier,
                    a,
                    vec![("other", name(st, b))],
                    format!(
                        "{} is arming both {} and {}, who are at war with each other.",
                        name(st, supplier),
                        name(st, a),
                        name(st, b)
                    ),
                    None,
                ));
            }
        }

        // A client at war with the supplier whose weapons it still holds.
        for w in &st.wars.active {
            for b in &w.belligerents {
                for e in w.side(b.side.other()) {
                    let arms = st.country(b.country).arms_from(e.country);
                    if arms >= 1.0 {
                        hits.push(hit(
                            Trigger::TransferredArmsUsedAgainstSupplier,
                            e.country,
                            b.country,
                            vec![("arms", format!("{arms:.0}"))],
                            format!(
                                "{} is fighting {} with {arms:.0} strength of weapons {} supplied.",
                                name(st, b.country),
                                name(st, e.country),
                                name(st, e.country)
                            ),
                            Some(format!("arms-back:{}:{}:{}", w.id.0, e.country.0, b.country.0)),
                        ));
                    }
                }
            }
        }
    }

    /// V3: scenario events, peacetime arms to clients, strategy reversals.
    fn detect_v3(&mut self, input: &TurnInput, hits: &mut Vec<Hit>) {
        let st = input.state;
        for e in &input.report.events {
            let nuclear = match *e {
                DiplomaticEvent::ProgrammeExposed { country, by } => Some((
                    Trigger::ProgrammeExposed,
                    country,
                    by,
                    format!(
                        "{}'s secret weapons programme was exposed by {}.",
                        name(st, country),
                        name(st, by)
                    ),
                )),
                DiplomaticEvent::ArsenalDeclared { country } => Some((
                    Trigger::ArsenalDeclared,
                    country,
                    country,
                    format!("{} declared its nuclear arsenal.", name(st, country)),
                )),
                DiplomaticEvent::Secession { parent, successor } => Some((
                    Trigger::Secession,
                    parent,
                    successor,
                    format!("{} seceded from {}.", name(st, successor), name(st, parent)),
                )),
                DiplomaticEvent::ArsenalDismantled { country } => Some((
                    Trigger::ArsenalDismantled,
                    country,
                    country,
                    format!(
                        "{} disclosed and dismantled its nuclear arsenal and programme.",
                        name(st, country)
                    ),
                )),
                _ => None,
            };
            if let Some((trigger, actor, target, fact)) = nuclear {
                hits.push(hit(trigger, actor, target, vec![], fact, None));
            }
            if let DiplomaticEvent::MonetaryStance { by, stance } = *e {
                let target = sim_core::money::target_rate(st.params.base_interest_rate, stance, st.global_tension);
                let rate = format!("{:.1}", 400.0 * target);
                let (trigger, verb) = match stance {
                    sim_core::MonetaryStance::Tight => (Trigger::RatesRaised, "tightened"),
                    _ => (Trigger::RatesCut, "eased"),
                };
                hits.push(hit(
                    trigger,
                    by,
                    by,
                    vec![("rate", rate.clone())],
                    format!("{} {verb} money: the world interest rate is heading for {rate}% a year.", name(st, by)),
                    None,
                ));
            }
            if let DiplomaticEvent::ScenarioEvent {
                template,
                subject,
                object,
            } = *e
            {
                let Some(t) = st.event_templates.get(template as usize) else {
                    continue;
                };
                let target = object.unwrap_or(subject);
                let fact = match object {
                    Some(o) => format!("{}: {} and {}.", t.title, name(st, subject), name(st, o)),
                    None => format!("{}: {}.", t.title, name(st, subject)),
                };
                hits.push(hit(
                    Trigger::ScenarioEvent,
                    subject,
                    target,
                    vec![("event_id", t.id.clone())],
                    fact,
                    None,
                ));
            }
        }
        for (decider, r) in input.decisions {
            if r.chosen && r.kind == DecisionKind::StreamStart {
                if let Some(c) = r.counterpart {
                    let amount = st
                        .diplomacy
                        .streams
                        .iter()
                        .find(|s| s.from == *decider && s.to == c && s.kind == StreamKind::Arms)
                        .map_or(0.0, |s| s.amount);
                    if amount > 0.0 {
                        hits.push(hit(
                            Trigger::ArmedClient,
                            *decider,
                            c,
                            vec![("amount", format!("{amount:.1}"))],
                            format!(
                                "{} began supplying {} with {amount:.1} strength of arms per quarter.",
                                name(st, *decider),
                                name(st, c)
                            ),
                            None,
                        ));
                    }
                }
            }
            if r.kind == DecisionKind::Goals {
                let now = r.subject.trim_start_matches("strategic goals: ").to_string();
                if let Some(old) = self.last_goals.insert(*decider, now.clone()) {
                    if old.contains("contain ") && !now.contains("contain ") {
                        hits.push(hit(
                            Trigger::StrategyReversed,
                            *decider,
                            *decider,
                            vec![("old", old.clone()), ("new", now.clone())],
                            format!("{} dropped its goals ({old}) for new ones ({now}).", name(st, *decider)),
                            None,
                        ));
                    }
                }
            }
        }
    }

    fn budget(&self, hits: &[Hit]) -> usize {
        if self.silent_until.is_some_and(|t| self.current_turn <= t) {
            return 0;
        }
        match self.intensity {
            Intensity::Off => 0,
            Intensity::Dry => BUDGET_DRY,
            Intensity::Black => {
                let urgent = hits.iter().filter(|h| self.best_priority(h.trigger) <= 3).count();
                if urgent >= 3 {
                    BUDGET_BLACK_CASCADE
                } else {
                    BUDGET_BLACK
                }
            }
        }
    }

    fn best_priority(&self, trigger: Trigger) -> u8 {
        self.lines
            .iter()
            .filter(|l| l.trigger == trigger)
            .map(|l| l.priority)
            .min()
            .unwrap_or(6)
    }

    fn select(&mut self, turn: u32, st: &WorldState, mut hits: Vec<Hit>) -> Vec<Narration> {
        self.current_turn = turn;
        hits.sort_by_key(|h| (self.best_priority(h.trigger), h.trigger, h.actor, h.target));
        let mut budget = self.budget(&hits);
        let mut out = Vec::new();
        // One flavour line per pair per turn: the highest-priority trigger
        // speaks; the rest stay plain (anti-fatigue, approval item 12).
        let mut voiced_pairs: BTreeSet<(CountryId, CountryId)> = BTreeSet::new();
        for h in hits {
            let gravity = h.trigger.gravity();
            let mut chosen = None;
            let pair = if h.actor <= h.target {
                (h.actor, h.target)
            } else {
                (h.target, h.actor)
            };
            if budget > 0 && !voiced_pairs.contains(&pair) {
                let key_base = format!("{:?}:{}:{}", h.trigger, h.actor.0, h.target.0);
                let mut candidates: Vec<&Line> = self
                    .lines
                    .iter()
                    .filter(|l| l.trigger == h.trigger && l.intensity <= self.intensity && gravity <= l.max_gravity)
                    .filter(|l| l.event.is_none() || l.event.as_ref() == h.values.get("event_id"))
                    .filter(|l| self.uses.get(&l.id).copied().unwrap_or(0) < l.max_per_campaign)
                    .filter(|l| !self.spoken.contains(&(l.id.clone(), h.actor)))
                    .filter(|l| self.last_turn.get(&l.id) != Some(&turn))
                    // Only lines whose placeholders this hit can fill: a
                    // line about money needs the money to exist.
                    .filter(|l| render(&l.text, st, &h).is_some())
                    .filter(|l| {
                        self.last_used
                            .get(&format!("{}:{}", l.id, key_base))
                            .is_none_or(|t| turn >= t + l.cooldown)
                    })
                    .collect();
                // Highest priority first; within it, the most intense voice the
                // setting allows (Black prefers Black lines), then least used.
                candidates.sort_by_key(|l| {
                    (
                        l.priority,
                        std::cmp::Reverse(l.intensity),
                        self.uses.get(&l.id).copied().unwrap_or(0),
                        l.id.clone(),
                    )
                });
                if let Some(first) = candidates.first() {
                    let (best, voice) = (first.priority, first.intensity);
                    let pool: Vec<&&Line> = candidates
                        .iter()
                        .filter(|l| l.priority == best && l.intensity == voice)
                        .collect();
                    let pick = pool[(roll(self.seed, turn, &key_base) * pool.len() as f64) as usize % pool.len()];
                    if roll(self.seed, turn, &format!("plain:{}", pick.id)) >= FALL_TO_PLAIN {
                        if let Some(text) = render(&pick.text, st, &h) {
                            *self.uses.entry(pick.id.clone()).or_insert(0) += 1;
                            self.last_used.insert(format!("{}:{}", pick.id, key_base), turn);
                            self.spoken.insert((pick.id.clone(), h.actor));
                            self.last_turn.insert(pick.id.clone(), turn);
                            let speaker = if h.trigger == Trigger::ReadinessRising {
                                "wire services".to_string()
                            } else {
                                name(st, h.actor)
                            };
                            chosen = Some((pick.surface.heading().to_string(), speaker, text, pick.id.clone()));
                            budget -= 1;
                            voiced_pairs.insert(pair);
                        }
                    }
                }
            }
            out.push(Narration {
                turn,
                trigger: h.trigger,
                gravity,
                fact: h.fact,
                line: chosen,
                actor: h.actor,
                target: h.target,
            });
        }
        out
    }
}

/// Fill placeholders from the hit; `None` if any value is missing (the line
/// is then ineligible rather than wrong).
fn render(template: &str, st: &WorldState, h: &Hit) -> Option<String> {
    let mut out = template.to_string();
    for p in crate::lines::placeholders_in(template) {
        let value = match p.as_str() {
            "actor" => name(st, h.actor),
            "target" => name(st, h.target),
            other => h.values.get(other)?.clone(),
        };
        out = out.replace(&format!("{{{p}}}"), &value);
    }
    Some(out)
}

/// Surface helper for display code.
pub fn is_private(line: &Option<(String, String, String, String)>) -> bool {
    line.as_ref().is_some_and(|(heading, ..)| {
        heading != Surface::OfficialStatement.heading() && heading != Surface::Ticker.heading()
    })
}
