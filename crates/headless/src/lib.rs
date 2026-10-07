//! Headless campaign runner: single runs, seeded batches and summaries.
//! Each campaign is fully independent, so batch results are identical
//! regardless of thread count.

pub mod diagnose;
pub mod stats;
use ai::{Controller, DecisionRecord, Strategist};
use rayon::prelude::*;
use scenario::ScenarioDef;
use serde::Serialize;
use sim_core::{observe, resolve_turn, LedgerLog, OrderSet, WorldState};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CountrySample {
    pub code: String,
    pub gdp: f64,
    pub stability: f64,
    pub debt_ratio: f64,
    /// Military power (strength × quality × readiness).
    pub military: f64,
    pub security: f64,
    pub initiative: u8,
    /// Calibration fields (military trajectories).
    pub military_share: f64,
    pub military_tech: u8,
    pub strength: f64,
    pub active: bool,
    pub readiness: f64,
    pub arms_in: f64,
    /// Crisis diagnostics (`brink diagnose`).
    pub legitimacy: f64,
    pub prosperity: f64,
    pub burden: f64,
    pub war_weariness: f64,
    pub growth: f64,
    pub democracy: bool,
    /// Legitimacy pressure from falling behind the rival bloc.
    pub behind: f64,
    /// Prosperity lost to inflation and to debt service (P7).
    pub inflation_hit: f64,
    pub debt_service_hit: f64,
    /// Codes of the countries pledged to defend this one (alliances and
    /// guarantors; `brink diagnose` protection report).
    pub protectors: Vec<String>,
    /// Strategic arsenal level (canonical; wars on nuclear states by aim).
    pub arsenal: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TurnSample {
    pub turn: u32,
    pub year: f64,
    /// World energy price relative to its base (1 when not modelled).
    /// World interest rate (annualised %) and monetary stance (P7).
    pub interest_rate: f64,
    pub monetary_stance: String,
    pub energy_price: f64,
    pub countries: Vec<CountrySample>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReasonEntry {
    pub turn: u32,
    pub country: String,
    pub decision: DecisionRecord,
}

/// Ledger and implied-norm statistics per campaign (DESIGN §21.4
/// instrumentation requirement).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct LedgerStats {
    pub entries: usize,
    pub tests_opened: usize,
    pub shadows_triggered: usize,
    pub norms_created: usize,
    pub norm_tests: usize,
    /// Norm tests that changed at least one observer's expectation.
    pub norm_tests_with_effect: usize,
    /// Largest single expectation change produced by a norm test.
    pub max_norm_delta: f64,
}

impl LedgerStats {
    pub fn absorb(&mut self, log: &[LedgerLog]) {
        for line in log {
            match line {
                LedgerLog::EntryWritten { .. } => self.entries += 1,
                LedgerLog::TestOpened { .. } => self.tests_opened += 1,
                LedgerLog::ShadowTriggered { .. } => self.shadows_triggered += 1,
                LedgerLog::NormCreated { .. } => self.norms_created += 1,
                LedgerLog::NormTest { deltas, .. } => {
                    self.norm_tests += 1;
                    if !deltas.is_empty() {
                        self.norm_tests_with_effect += 1;
                    }
                    for (_, d) in deltas {
                        self.max_norm_delta = self.max_norm_delta.max(d.abs());
                    }
                }
            }
        }
    }
}

/// One narrated situation (voice layer): the plain fact always, and the
/// selected satirical line when one passed selection.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NarrationEntry {
    pub turn: u32,
    pub trigger: String,
    pub fact: String,
    /// `(heading, speaker, text, line id)`.
    pub line: Option<(String, String, String, String)>,
}

/// Voice settings for a run. `None` lines = no narration.
#[derive(Clone, Copy, Debug)]
pub struct VoiceSettings<'a> {
    pub lines: &'a [voice::lines::Line],
    pub intensity: voice::Intensity,
}

/// One country's involvement in one war as seen at turn ends (DESIGN §14.9
/// bands). Measurement only: read from canonical state after each turn, so
/// Gate 6 can tell arms-stream involvement (band 2, created by arms streams
/// to a belligerent) from fighting (band 3–4, joined by order).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InvolvementSeen {
    pub actor: String,
    pub beneficiary: String,
    pub war: u32,
    /// Highest band held in this war during the run.
    pub max_band: u8,
}

/// Fold the current involvements into the per-run record (pure read).
fn note_involvements(state: &WorldState, seen: &mut std::collections::BTreeMap<(u16, u16, u32), u8>) {
    for i in &state.wars.involvements {
        let band = seen.entry((i.actor.0, i.beneficiary.0, i.war.0)).or_insert(0);
        *band = (*band).max(i.band);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RunResult {
    pub scenario: String,
    pub seed: u64,
    pub turns: u32,
    pub samples: Vec<TurnSample>,
    pub reasoning: Vec<ReasonEntry>,
    pub ledger: LedgerStats,
    /// Full ledger development log, turn by turn.
    pub ledger_log: Vec<(u32, LedgerLog)>,
    /// Voice narration (empty unless voice settings were given).
    pub narration: Vec<NarrationEntry>,
    /// Every resolved event, turn by turn (diplomacy, war, scenario events).
    pub events: Vec<(u32, sim_core::DiplomaticEvent)>,
    /// Campaign epitaph for the first country (only with voice settings):
    /// the Official History line and the plain Ledger column.
    pub epitaph: Option<(Option<String>, Vec<String>)>,
    /// Every war involvement seen during the run (Gate 6 statistics).
    pub involvements: Vec<InvolvementSeen>,
    /// Full canonical end state as JSON (the determinism fingerprint).
    pub final_state: String,
    /// Tension and opinion matrices every [`RELATION_SAMPLE`] turns
    /// (sanction-exit instrumentation, issue 15). Not serialised.
    #[serde(skip)]
    pub relations: Vec<RelationSample>,
}

/// Turns between relation samples.
pub const RELATION_SAMPLE: u32 = 4;

/// Row-major n x n tension and directed opinion (`opinion[from * n + to]`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelationSample {
    pub turn: u32,
    pub tension: Vec<f64>,
    pub opinion: Vec<f64>,
    /// Historical grudge (D81) still held, and its renewals so far.
    pub memory: Vec<f64>,
    pub renewals: Vec<u32>,
    /// Quarters a harmful campaign deepened the grudge so far, and the last
    /// act that did (D97).
    pub deepened: Vec<u32>,
    pub deepened_by: Vec<Option<sim_core::opinion::HostileAct>>,
}

fn relation_sample(state: &WorldState) -> RelationSample {
    let ids: Vec<sim_core::CountryId> = state.ids().collect();
    let mut tension = Vec::with_capacity(ids.len() * ids.len());
    let mut opinion = Vec::with_capacity(ids.len() * ids.len());
    let mut memory = Vec::with_capacity(ids.len() * ids.len());
    let mut renewals = Vec::with_capacity(ids.len() * ids.len());
    let mut deepened = Vec::with_capacity(ids.len() * ids.len());
    let mut deepened_by = Vec::with_capacity(ids.len() * ids.len());
    for &a in &ids {
        for &b in &ids {
            tension.push(state.tension.get(a, b));
            opinion.push(state.opinions.opinion(a, b));
            memory.push(state.opinions.memory(a, b));
            renewals.push(
                state
                    .opinions
                    .modifiers(a, b)
                    .iter()
                    .filter_map(|m| m.memory.map(|x| x.renewals))
                    .sum(),
            );
            let mem = state.opinions.modifiers(a, b).iter().filter_map(|m| m.memory);
            deepened.push(mem.clone().map(|x| x.deepened_turns).sum());
            deepened_by.push(mem.filter_map(|x| x.deepened).next_back());
        }
    }
    RelationSample {
        turn: state.turn,
        tension,
        opinion,
        memory,
        renewals,
        deepened,
        deepened_by,
    }
}

pub fn sample(state: &WorldState) -> TurnSample {
    let burden: Vec<f64> = state
        .ids()
        .map(|id| sim_core::domestic::defence_burden(state, id).penalty)
        .collect();
    TurnSample {
        turn: state.turn,
        year: state.year(),
        energy_price: state.energy.price,
        interest_rate: 400.0 * state.money.rate,
        monetary_stance: format!("{:?}", state.money.stance),
        countries: state
            .countries
            .iter()
            .zip(burden)
            .map(|(c, burden)| CountrySample {
                code: c.code.clone(),
                gdp: c.gdp,
                stability: c.stability,
                debt_ratio: c.debt_ratio(),
                military: c.power(),
                security: c.security,
                initiative: c.initiative.available(),
                military_share: c.budget.military,
                military_tech: c.military_tech,
                strength: c.forces.strength(),
                active: c.active,
                readiness: c.forces.readiness,
                arms_in: c.arms_in,
                legitimacy: c.legitimacy,
                prosperity: c.prosperity,
                burden,
                war_weariness: c.war_weariness,
                growth: c.last_growth,
                democracy: c.government == sim_core::Government::Democracy,
                behind: sim_core::domestic::falling_behind(state, c),
                inflation_hit: c.inflation_hit,
                debt_service_hit: c.debt_service_hit,
                protectors: sim_core::domestic::defenders(state, c.id)
                    .into_iter()
                    .map(|d| state.country(d).code.clone())
                    .collect(),
                arsenal: c.arsenal,
            })
            .collect(),
    }
}

pub fn run_campaign(def: &ScenarioDef, seed: u64, turns: u32) -> Result<RunResult, String> {
    run_campaign_with(def, seed, turns, None)
}

/// Run a campaign, optionally narrated. Narration is a pure read: it never
/// changes the simulation (asserted by tests).
pub fn run_campaign_with(
    def: &ScenarioDef,
    seed: u64,
    turns: u32,
    voice_settings: Option<VoiceSettings>,
) -> Result<RunResult, String> {
    let mut state = scenario::build(def, Some(seed))?;
    let mut narrator = voice_settings.map(|v| voice::narrator::Narrator::new(v.lines.to_vec(), v.intensity, seed));
    let mut narration = Vec::new();
    let mut controllers: Vec<Box<dyn Controller>> = state
        .countries
        .iter()
        .map(|_| Box::new(Strategist::with_seed(seed)) as Box<dyn Controller>)
        .collect();

    let mut samples = vec![sample(&state)];
    let mut relations = vec![relation_sample(&state)];
    let mut reasoning = Vec::new();
    let mut ledger = LedgerStats::default();
    let mut ledger_log = Vec::new();
    let mut events = Vec::new();
    let mut involvements = std::collections::BTreeMap::new();
    note_involvements(&state, &mut involvements);
    for _ in 0..turns {
        // Simultaneous planning: every view is taken from the same start-of-turn state.
        let views: Vec<_> = state.ids().map(|id| observe(&state, id)).collect();
        let mut orders = Vec::with_capacity(views.len());
        let mut turn_decisions = Vec::new();
        for (view, controller) in views.iter().zip(controllers.iter_mut()) {
            let decision = controller.decide(view);
            turn_decisions.extend(decision.records.iter().cloned().map(|r| (view.observer, r)));
            reasoning.extend(decision.records.into_iter().map(|r| ReasonEntry {
                turn: view.turn,
                country: view.own.code.clone(),
                decision: r,
            }));
            orders.push(OrderSet {
                country: view.observer,
                orders: decision.orders,
            });
        }
        if let Some(n) = narrator.as_mut() {
            n.observe_before(&state);
        }
        let report = resolve_turn(&mut state, orders);
        if let Some(n) = narrator.as_mut() {
            let input = voice::narrator::TurnInput {
                state: &state,
                report: &report,
                decisions: &turn_decisions,
            };
            narration.extend(n.narrate(input).into_iter().map(|x| NarrationEntry {
                turn: x.turn,
                trigger: format!("{:?}", x.trigger),
                fact: x.fact,
                line: x.line,
            }));
        }
        events.extend(report.events.iter().cloned().map(|e| (report.turn, e)));
        ledger.absorb(&report.ledger_log);
        ledger_log.extend(report.ledger_log.into_iter().map(|l| (report.turn, l)));
        samples.push(sample(&state));
        if state.turn % RELATION_SAMPLE == 0 {
            relations.push(relation_sample(&state));
        }
        note_involvements(&state, &mut involvements);
        // A join into a war that ends within the same turn (a punitive
        // strike accepted at once) leaves no standing involvement to sample.
        for e in &report.events {
            if let sim_core::DiplomaticEvent::JoinedWar { war, country, side, band } = *e {
                let leader = events.iter().find_map(|(_, d)| match *d {
                    sim_core::DiplomaticEvent::WarDeclared { war: w, attacker, defender, .. } if w == war => {
                        Some(if side == sim_core::Side::Attacker { attacker } else { defender })
                    }
                    _ => None,
                });
                if let Some(l) = leader {
                    let b = involvements.entry((country.0, l.0, war.0)).or_insert(0);
                    *b = (*b).max(band);
                }
            }
        }
    }

    let epitaph = voice_settings.map(|v| {
        voice::epitaph(
            &state,
            sim_core::CountryId(0),
            samples[0].countries[0].gdp,
            v.lines,
            v.intensity,
            seed,
        )
    });
    let final_state = serde_json::to_string(&state).map_err(|e| e.to_string())?;
    let code = |c: u16| state.country(sim_core::CountryId(c)).code.clone();
    let involvements = involvements
        .into_iter()
        .map(|((a, b, war), max_band)| InvolvementSeen {
            actor: code(a),
            beneficiary: code(b),
            war,
            max_band,
        })
        .collect();
    Ok(RunResult {
        scenario: def.name.clone(),
        seed,
        turns,
        samples,
        reasoning,
        ledger,
        ledger_log,
        narration,
        events,
        epitaph,
        involvements,
        final_state,
        relations,
    })
}

pub fn run_batch(def: &ScenarioDef, seeds: &[u64], turns: u32, threads: usize) -> Result<Vec<RunResult>, String> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .map_err(|e| e.to_string())?;
    pool.install(|| seeds.par_iter().map(|&s| run_campaign(def, s, turns)).collect())
}
