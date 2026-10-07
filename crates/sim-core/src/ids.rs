use serde::{Deserialize, Serialize};

/// Index of a country in `WorldState::countries`. Ordering is the canonical
/// iteration order for every system, which keeps resolution deterministic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CountryId(pub u16);

impl CountryId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
