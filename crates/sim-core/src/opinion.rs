use serde::{Deserialize, Serialize};

use crate::ids::CountryId;

/// One visible, decaying contribution to an opinion (DESIGN §7.1:
/// "+20 trade partners, −25 border tension ...").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpinionModifier {
    pub source: String,
    pub value: f64,
    /// Absolute amount the value moves toward zero each turn. 0 = permanent.
    pub decay: f64,
    /// A historical grudge (the scenario's starting hostility): it fades
    /// unless hostile acts renew it (D81, [`crate::grudge`]). `None` for
    /// every other modifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<Memory>,
}

/// Fade state of a historical grudge (D81).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    /// Last turn a hostile act between the pair renewed it, and the act.
    pub renewed: Option<(u32, HostileAct)>,
    /// Renewals so far.
    pub renewals: u32,
}

/// A hostile act between two countries that renews their historical grudge
/// (D81). Standing measures renew nothing after they start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostileAct {
    Sanction,
    Denunciation,
    War,
    ArmedEnemyAtWar,
    CovertArmsExposed,
    ProgrammeExposed,
    NuclearStrike,
    /// One of us is arming beyond its habit against the other (arms race).
    ArmsBuildUp,
}

impl OpinionModifier {
    pub fn new(source: impl Into<String>, value: f64, decay: f64) -> Self {
        OpinionModifier {
            source: source.into(),
            value,
            decay,
            memory: None,
        }
    }
}

/// The opinion breakdown line: "the second Cold War -28.5 (fading memory;
/// renewed turn 34 by Sanction)".
impl std::fmt::Display for OpinionModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:+.1}", self.source, self.value)?;
        match self.memory {
            Some(Memory {
                renewed: Some((t, act)),
                renewals,
            }) => write!(f, " (fading memory; renewed turn {t} by {act:?}, {renewals} renewals)"),
            Some(_) => write!(f, " (fading memory; never renewed)"),
            None if self.decay > 0.0 => write!(f, " (decays {:.2}/turn)", self.decay),
            None => Ok(()),
        }
    }
}

/// Directed opinions: `opinion(from, to)` is how much `from` likes `to`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpinionBook {
    n: usize,
    cells: Vec<Vec<OpinionModifier>>,
}

impl OpinionBook {
    pub fn new(n: usize) -> Self {
        OpinionBook {
            n,
            cells: vec![Vec::new(); n * n],
        }
    }

    fn idx(&self, from: CountryId, to: CountryId) -> usize {
        from.index() * self.n + to.index()
    }

    pub fn add(&mut self, from: CountryId, to: CountryId, modifier: OpinionModifier) {
        let i = self.idx(from, to);
        self.cells[i].push(modifier);
    }

    pub fn opinion(&self, from: CountryId, to: CountryId) -> f64 {
        self.cells[self.idx(from, to)]
            .iter()
            .map(|m| m.value)
            .sum::<f64>()
            .clamp(-100.0, 100.0)
    }

    /// Remove every modifier from `source` (e.g. when a treaty ends).
    pub fn remove_source(&mut self, from: CountryId, to: CountryId, source: &str) {
        let i = self.idx(from, to);
        self.cells[i].retain(|m| m.source != source);
    }

    pub fn modifiers(&self, from: CountryId, to: CountryId) -> &[OpinionModifier] {
        &self.cells[self.idx(from, to)]
    }

    pub fn modifiers_mut(&mut self, from: CountryId, to: CountryId) -> &mut Vec<OpinionModifier> {
        let i = self.idx(from, to);
        &mut self.cells[i]
    }

    /// Sum of the historical grudges `from` holds against `to` (D81).
    pub fn memory(&self, from: CountryId, to: CountryId) -> f64 {
        self.modifiers(from, to)
            .iter()
            .filter(|m| m.memory.is_some())
            .map(|m| m.value)
            .sum()
    }

    /// Renew `from`'s historical grudges against `to` (D81).
    pub fn renew(&mut self, from: CountryId, to: CountryId, turn: u32, act: HostileAct) {
        for m in self.modifiers_mut(from, to) {
            if let Some(mem) = &mut m.memory {
                if mem.renewed.is_none_or(|(t, _)| t != turn) {
                    mem.renewals += 1;
                }
                mem.renewed = Some((turn, act));
            }
        }
    }

    /// Move every decaying modifier toward zero and drop the spent ones.
    pub fn decay(&mut self) {
        for cell in &mut self.cells {
            for m in cell.iter_mut() {
                if m.decay > 0.0 {
                    let step = m.decay.min(m.value.abs());
                    m.value -= step * m.value.signum();
                }
            }
            cell.retain(|m| m.decay == 0.0 || m.value != 0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(value: f64, decay: f64) -> OpinionModifier {
        OpinionModifier::new("test", value, decay)
    }

    #[test]
    fn sums_and_clamps() {
        let mut b = OpinionBook::new(2);
        b.add(CountryId(0), CountryId(1), m(80.0, 0.0));
        b.add(CountryId(0), CountryId(1), m(50.0, 0.0));
        assert_eq!(b.opinion(CountryId(0), CountryId(1)), 100.0);
        assert_eq!(b.opinion(CountryId(1), CountryId(0)), 0.0);
    }

    #[test]
    fn decays_toward_zero_and_drops_spent() {
        let mut b = OpinionBook::new(2);
        b.add(CountryId(0), CountryId(1), m(-5.0, 2.0));
        b.decay();
        assert_eq!(b.opinion(CountryId(0), CountryId(1)), -3.0);
        b.decay();
        b.decay();
        assert_eq!(b.opinion(CountryId(0), CountryId(1)), 0.0);
        assert!(b.modifiers(CountryId(0), CountryId(1)).is_empty());
    }
}
