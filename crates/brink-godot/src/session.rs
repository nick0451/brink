//! Engine-free session core for the Godot front-end: owns a world, runs the
//! AI and the narrator turn by turn exactly as the headless runner does, and
//! flattens each turn into plain views the presentation layer can draw.
//! Kept free of Godot types so it is unit-testable (determinism vs headless).

use std::path::Path;

use ai::{Controller, Strategist};
use sim_core::{observe, resolve_turn, CountryId, DiplomaticEvent, Government, OrderSet, WorldState};
use voice::narrator::{Narrator, TurnInput};

/// One map-worthy event (DESIGN §1 resolution playback).
#[derive(Clone, Debug, PartialEq)]
pub struct EventView {
    pub kind: &'static str,
    pub a: Option<CountryId>,
    pub b: Option<CountryId>,
    pub value: f64,
    /// Gravity for presentation (behaviour-and-voice §2.2): 5 = nuclear.
    pub gravity: u8,
}

/// One narration: always a plain fact, sometimes a flavoured line.
#[derive(Clone, Debug, PartialEq)]
pub struct NarrationView {
    pub fact: String,
    pub gravity: u8,
    /// (heading, speaker, text) when a line was chosen.
    pub line: Option<(String, String, String)>,
    /// The country speaking, when the speaker is a country.
    pub speaker: Option<CountryId>,
    /// Portrait policy (D56): a face only on G1–G2; an office seal on G3;
    /// nothing on G4–G5.
    pub portrait: Portrait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Portrait {
    Face,
    Seal,
    None,
}

impl Portrait {
    pub fn for_gravity(g: u8) -> Self {
        match g {
            0..=2 => Portrait::Face,
            3 => Portrait::Seal,
            _ => Portrait::None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StepOutput {
    pub turn: u32,
    pub events: Vec<EventView>,
    pub narration: Vec<NarrationView>,
}

pub struct Session {
    pub world: WorldState,
    ais: Vec<Strategist>,
    narrator: Option<Narrator>,
}

impl Session {
    /// Load a scenario (and optionally voice lines) with a seed.
    pub fn load(scenario: &Path, lines: Option<&Path>, seed: u64) -> Result<Self, String> {
        let def = scenario::load(scenario)?;
        let world = scenario::build(&def, Some(seed))?;
        let ais = world.ids().map(|_| Strategist::with_seed(seed)).collect();
        let narrator = match lines {
            Some(p) => Some(Narrator::new(
                voice::lines::load_lines(p)?,
                voice::Intensity::default(),
                seed,
            )),
            None => None,
        };
        Ok(Session { world, ais, narrator })
    }

    /// Resolve one turn: simultaneous AI planning from the same state (the
    /// same loop as `headless::run_campaign_with`), then narration.
    pub fn step(&mut self) -> StepOutput {
        let views: Vec<_> = self.world.ids().map(|c| observe(&self.world, c)).collect();
        let mut orders = Vec::with_capacity(views.len());
        let mut decisions = Vec::new();
        for (view, ai) in views.iter().zip(self.ais.iter_mut()) {
            let d = ai.decide(view);
            decisions.extend(d.records.iter().cloned().map(|r| (view.observer, r)));
            orders.push(OrderSet {
                country: view.observer,
                orders: d.orders,
            });
        }
        if let Some(n) = self.narrator.as_mut() {
            n.observe_before(&self.world);
        }
        let report = resolve_turn(&mut self.world, orders);
        let narration = match self.narrator.as_mut() {
            Some(n) => n
                .narrate(TurnInput {
                    state: &self.world,
                    report: &report,
                    decisions: &decisions,
                })
                .into_iter()
                .map(|x| {
                    let speaker = x.line.as_ref().and_then(|l| self.world.countries.iter().find(|c| c.name == l.1).map(|c| c.id));
                    NarrationView {
                        fact: x.fact,
                        gravity: x.gravity,
                        line: x.line.map(|(h, s, t, _)| (h, s, t)),
                        speaker,
                        portrait: Portrait::for_gravity(x.gravity),
                    }
                })
                .collect(),
            None => Vec::new(),
        };
        StepOutput {
            turn: report.turn,
            events: report.events.iter().filter_map(event_view).collect(),
            narration,
        }
    }

    /// Portrait set key for a country (D56: by regime state, never date).
    pub fn regime_key(&self, c: CountryId) -> String {
        let x = self.world.country(c);
        let state = match (x.government, x.transition.reformed_turn) {
            (Government::Democracy, Some(_)) => "reformed",
            (Government::Revolutionary, _) => "revolutionary",
            _ => "base",
        };
        format!("{}_{}", x.code.to_lowercase(), state)
    }
}

/// Map-worthy events only; routine diplomacy stays in the ledger panel.
pub fn event_view(e: &DiplomaticEvent) -> Option<EventView> {
    use DiplomaticEvent as E;
    let v = |kind, a, b, value, gravity| {
        Some(EventView {
            kind,
            a: Some(a),
            b,
            value,
            gravity,
        })
    };
    match *e {
        E::WarDeclared { attacker, defender, .. } => v("war", attacker, Some(defender), 0.0, 3),
        E::FrontReport { war: _, progress, .. } => {
            // Fronts are drawn from the war list; the report only animates.
            let _ = progress;
            None
        }
        E::PeaceMade { attacker, defender, .. } => v("peace", attacker, Some(defender), 0.0, 3),
        E::JoinedWar { country, .. } => v("joined_war", country, None, 0.0, 3),
        E::NuclearStrike { by, target, .. } => v("nuclear", by, Some(target), 0.0, 5),
        E::SanctionImposed { by, target } => v("sanction", by, Some(target), 0.0, 2),
        E::TreatySigned { treaty } => v("treaty", treaty.a, Some(treaty.b), 0.0, 1),
        E::TransitionCrisis { country } => v("crisis", country, None, 0.0, 2),
        E::Transition { country, .. } => v("transition", country, None, 0.0, 2),
        E::Secession { parent, successor } => v("secession", parent, Some(successor), 0.0, 2),
        E::ProgrammeExposed { country, by } => v("programme", country, Some(by), 0.0, 2),
        E::Mobilized { country, level } => v("mobilized", country, None, level.level() as f64, 2),
        E::EnergyPolicy { country, .. } => v("energy", country, None, 0.0, 1),
        _ => None,
    }
}
