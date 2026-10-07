use serde::{Deserialize, Serialize};

/// Maximum Initiative that can be banked for later turns (DESIGN §2.3).
pub const BANK_CAP: u8 = 2;

/// Per-turn action slots. Spending draws from this turn's allowance first,
/// then from the bank. Unspent allowance banks up to [`BANK_CAP`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Initiative {
    pub allowance: u8,
    pub banked: u8,
}

impl Initiative {
    pub fn available(&self) -> u8 {
        self.allowance + self.banked
    }

    /// Spend `n` Initiative. Returns false (and spends nothing) if unaffordable.
    pub fn spend(&mut self, n: u8) -> bool {
        if n > self.available() {
            return false;
        }
        let from_allowance = n.min(self.allowance);
        self.allowance -= from_allowance;
        self.banked -= n - from_allowance;
        true
    }

    pub fn end_turn(&mut self) {
        self.banked = (self.banked + self.allowance).min(BANK_CAP);
        self.allowance = 0;
    }

    pub fn start_turn(&mut self, allowance: u8) {
        self.allowance = allowance;
    }
}

/// This turn's allowance from base Initiative and Stability (DESIGN §2.3, §9.2).
pub fn allowance(base: u8, stability: f64) -> u8 {
    let mut a = base as i32;
    if stability >= 70.0 {
        a += 1;
    }
    if stability < 40.0 {
        a -= 1;
    }
    if stability < 25.0 {
        a -= 1;
    }
    a.max(0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spends_allowance_before_bank() {
        let mut i = Initiative {
            allowance: 3,
            banked: 2,
        };
        assert!(i.spend(4));
        assert_eq!(
            i,
            Initiative {
                allowance: 0,
                banked: 1
            }
        );
        assert!(!i.spend(2));
        assert_eq!(i.available(), 1);
    }

    #[test]
    fn banking_is_capped() {
        let mut i = Initiative {
            allowance: 3,
            banked: 1,
        };
        i.end_turn();
        assert_eq!(
            i,
            Initiative {
                allowance: 0,
                banked: BANK_CAP
            }
        );
    }

    #[test]
    fn allowance_follows_stability_bands() {
        assert_eq!(allowance(3, 75.0), 4);
        assert_eq!(allowance(3, 50.0), 3);
        assert_eq!(allowance(3, 30.0), 2);
        assert_eq!(allowance(3, 10.0), 1);
        assert_eq!(allowance(0, 10.0), 0);
    }
}
