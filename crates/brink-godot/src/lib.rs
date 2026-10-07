//! BRINK ↔ Godot bridge (D54). A thin GDExtension class over the
//! engine-free [`session::Session`]: Godot calls `load`, then `step` once per
//! turn, and draws what comes back. Presentation only reads; it never
//! changes simulation rules.

// gdext's #[class(init)] expansion trips this lint on `base: Base<Node>`.
#![allow(clippy::redundant_field_names)]

pub mod session;

use std::path::PathBuf;

use godot::prelude::*;
use sim_core::CountryId;

use session::{Portrait, Session};

struct BrinkExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BrinkExtension {}

#[derive(GodotClass)]
#[class(base = Node, init)]
pub struct BrinkSim {
    session: Option<Session>,
    base: Base<Node>,
}

fn code(s: &Session, c: Option<CountryId>) -> GString {
    match c {
        Some(c) => GString::from(s.world.country(c).code.as_str()),
        None => GString::new(),
    }
}

#[godot_api]
impl BrinkSim {
    /// Load a scenario file and voice lines (absolute or relative paths).
    #[func]
    fn load(&mut self, scenario: GString, lines: GString, seed: i64) -> bool {
        let lines_path = PathBuf::from(lines.to_string());
        let lines = (!lines.is_empty()).then_some(lines_path.as_path());
        match Session::load(&PathBuf::from(scenario.to_string()), lines, seed as u64) {
            Ok(s) => {
                self.session = Some(s);
                true
            }
            Err(e) => {
                godot_error!("BRINK: {e}");
                false
            }
        }
    }

    /// Resolve one turn. Returns {turn, events: [..], narration: [..]}.
    #[func]
    fn step(&mut self) -> VarDictionary {
        let mut out = VarDictionary::new();
        let Some(s) = self.session.as_mut() else { return out };
        let step = s.step();
        let s = self.session.as_ref().expect("session");
        let mut events = VarArray::new();
        for e in &step.events {
            let mut d = VarDictionary::new();
            d.set("kind", e.kind);
            d.set("a", &code(s, e.a));
            d.set("b", &code(s, e.b));
            d.set("value", e.value);
            d.set("gravity", e.gravity as i64);
            events.push(&d.to_variant());
        }
        let mut narration = VarArray::new();
        for n in &step.narration {
            let mut d = VarDictionary::new();
            d.set("fact", n.fact.as_str());
            d.set("gravity", n.gravity as i64);
            let (heading, speaker, text) = n.line.clone().unwrap_or_default();
            d.set("heading", heading.as_str());
            d.set("speaker", speaker.as_str());
            d.set("text", text.as_str());
            d.set("speaker_code", &code(s, n.speaker));
            let portrait = match (n.portrait, n.speaker) {
                (Portrait::Face, Some(c)) => s.regime_key(c),
                (Portrait::Seal, _) => "seal".to_string(),
                _ => String::new(),
            };
            d.set("portrait", portrait.as_str());
            narration.push(&d.to_variant());
        }
        out.set("turn", step.turn as i64);
        out.set("events", &events);
        out.set("narration", &narration);
        out
    }

    /// Every active country: code, name, area, government, gdp, stability,
    /// power, at_war, alignment, arsenal, regime (portrait key).
    #[func]
    fn countries(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.session.as_ref() else { return out };
        for c in s.world.countries.iter().filter(|c| c.active) {
            let mut d = VarDictionary::new();
            d.set("code", c.code.as_str());
            d.set("name", c.name.as_str());
            d.set("area", c.area.clone().unwrap_or_default().as_str());
            d.set("government", format!("{:?}", c.government).as_str());
            d.set("alignment", c.alignment.clone().unwrap_or_default().as_str());
            d.set("gdp", c.gdp);
            d.set("stability", c.stability);
            d.set("power", c.power());
            d.set("arsenal", c.arsenal as i64);
            d.set("at_war", s.world.wars.is_belligerent(c.id));
            d.set("regime", s.regime_key(c.id).as_str());
            out.push(&d.to_variant());
        }
        out
    }

    /// Active wars: attacker, defender, aim, progress (attacker-positive).
    #[func]
    fn wars(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.session.as_ref() else { return out };
        for w in &s.world.wars.active {
            let mut d = VarDictionary::new();
            d.set("attacker", &code(s, Some(w.attacker)));
            d.set("defender", &code(s, Some(w.defender)));
            d.set("aim", format!("{:?}", w.aim).as_str());
            d.set("progress", w.progress());
            out.push(&d.to_variant());
        }
        out
    }

    /// Bilateral tension between two country codes (0–100).
    #[func]
    fn tension(&self, a: GString, b: GString) -> f64 {
        let Some(s) = self.session.as_ref() else { return 0.0 };
        match (s.world.find(&a.to_string()), s.world.find(&b.to_string())) {
            (Some(x), Some(y)) => s.world.tension.get(x, y),
            _ => 0.0,
        }
    }

    #[func]
    fn global_tension(&self) -> f64 {
        self.session.as_ref().map_or(0.0, |s| s.world.global_tension)
    }

    #[func]
    fn year(&self) -> f64 {
        self.session.as_ref().map_or(0.0, |s| s.world.year())
    }

    /// World interest rate, annualised % (P7).
    #[func]
    fn interest_rate(&self) -> f64 {
        self.session.as_ref().map_or(0.0, |s| 400.0 * s.world.money.rate)
    }

    #[func]
    fn energy_price(&self) -> f64 {
        self.session.as_ref().map_or(1.0, |s| s.world.energy.price)
    }
}
