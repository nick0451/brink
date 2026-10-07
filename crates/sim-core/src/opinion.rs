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
    /// The last harmful act that deepened it (D97), and the quarters a
    /// harmful act has stood against us so far.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deepened: Option<HostileAct>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub deepened_turns: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
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
    /// A neighbouring producer floods the market while we, an indebted
    /// exporter, live on oil revenue (economic warfare).
    OilFlood,
    /// A creditor held us, a debtor in distress, to our war loan (issue 24).
    DebtHeld,
}

impl HostileAct {
    /// Does the act harm its victim every turn it continues (D97)? Such a
    /// standing campaign deepens the victim's grudge a step per turn. The
    /// one-off acts carry their own opinion hit when they happen (sanction,
    /// denunciation, war, exposures, arming an enemy), so they only renew:
    /// deepening them too would count the same act twice. An arms build-up
    /// is a response to a threat, mutual and renewed both ways, and harms
    /// nobody directly: it only renews.
    pub fn harms_each_turn(self) -> bool {
        matches!(self, HostileAct::OilFlood | HostileAct::DebtHeld)
    }
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
                deepened,
                deepened_turns,
            }) => {
                write!(f, " (fading memory; renewed turn {t} by {act:?}, {renewals} renewals")?;
                if let Some(d) = deepened {
                    write!(f, "; deepened by {d:?}, {deepened_turns} quarters")?;
                }
                write!(f, ")")
            }
            Some(_) => write!(f, " (fading memory; never renewed)"),
            None if self.decay > 0.0 => write!(f, " (decays {:.2}/turn)", self.decay),
            None => Ok(()),
        }
    }
}

/// Source of a grudge a harmful campaign created where none existed (D97).
pub const GRIEVANCE: &str = "grievance";

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

    /// A harmful act by `actor` stood against `victim` this turn (D97):
    /// deepen the victim's grudge by `step`, no lower than `cap` (summed
    /// over its grudges), creating one ("grievance") where none exists.
    /// The fade and floor rules of D81 then apply to the deepened value.
    pub fn deepen(&mut self, victim: CountryId, actor: CountryId, act: HostileAct, step: f64, cap: f64) {
        let room = (self.memory(victim, actor) - cap).max(0.0);
        let delta = step.min(room);
        let cell = self.modifiers_mut(victim, actor);
        if let Some(m) = cell.iter_mut().find(|m| m.memory.is_some()) {
            m.value -= delta;
            let mem = m.memory.as_mut().expect("memory");
            mem.deepened = Some(act);
            mem.deepened_turns += 1;
        } else if delta > 0.0 {
            cell.push(OpinionModifier {
                source: GRIEVANCE.into(),
                value: -delta,
                decay: 0.0,
                memory: Some(Memory {
                    deepened: Some(act),
                    deepened_turns: 1,
                    ..Memory::default()
                }),
            });
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
