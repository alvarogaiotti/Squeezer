// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vulnerable {
    Yes,
    No,
}

impl Vulnerable {
    #[must_use]
    pub const fn from_number_and_seat(board_number: u8, seat: Seat) -> Self {
        let state = Vulnerability::from_number(board_number);
        state.is_vulnerable(seat)
    }
}
/// A struct representing a contract
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Contract {
    vuln: Vulnerable,
    level: u8,
    doubled: Doubled,
    strain: Strain,
    declarer: Seat,
}

#[cfg(feature = "dds")]
impl dds::traits::AsDDSContract for Contract {
    fn as_dds_contract(&self) -> (i32, i32) {
        (self.strain as i32, self.declarer.next() as i32)
    }
}

#[cfg(feature = "dds")]
impl dds::traits::ContractScorer for Contract {
    fn score(&self, tricks: u8) -> i32 {
        let target: i32 = i32::from(self.level) + 6i32;
        let overtricks: i32 = i32::from(tricks) - target;
        if overtricks >= 0 {
            let per_trick: i32 = match self.strain {
                Strain::Clubs | Strain::Diamonds => 20,
                _ => 30,
            };
            let mut per_overtrick: i32 = per_trick;
            let mut base_score: i32 = per_trick * i32::from(self.level);
            let mut bonus: i32 = 0;
            if self.strain == Strain::NoTrumps {
                base_score += 10;
            };

            match self.doubled {
                Doubled::Doubled => {
                    base_score *= 2;
                    bonus += 50;
                    per_overtrick = 100;
                }
                Doubled::Redoubled => {
                    base_score *= 4;
                    bonus += 100;
                    per_overtrick = 200;
                }
                Doubled::NotDoubled => {}
            };
            bonus += if base_score >= 100 {
                if matches!(self.vuln, Vulnerable::Yes) {
                    500
                } else {
                    300
                }
            } else {
                50
            };
            bonus += if self.level == 6 {
                if matches!(self.vuln, Vulnerable::Yes) {
                    750
                } else {
                    500
                }
            } else {
                0
            };
            bonus += if self.level == 7 {
                if matches!(self.vuln, Vulnerable::Yes) {
                    1500
                } else {
                    1000
                }
            } else {
                0
            };
            bonus += overtricks * per_overtrick;
            base_score + bonus
        } else {
            let mut score: i32;
            if matches!(self.doubled, Doubled::NotDoubled) {
                let per_undertrick = if matches!(self.vuln, Vulnerable::Yes) {
                    100
                } else {
                    50
                };
                score = overtricks * per_undertrick;
            } else {
                match overtricks {
                    -1 => {
                        score = if matches!(self.vuln, Vulnerable::Yes) {
                            -200
                        } else {
                            -100
                        }
                    }
                    -2 => {
                        score = if matches!(self.vuln, Vulnerable::Yes) {
                            -500
                        } else {
                            -300
                        }
                    }
                    _ => {
                        score = if matches!(self.vuln, Vulnerable::Yes) {
                            300 * overtricks + 100
                        } else {
                            300 * overtricks + 400
                        }
                    }
                }
                if matches!(self.doubled, Doubled::Redoubled) {
                    score *= 2;
                }
            }
            score
        }
    }
}

/// Enum modelling whether a [`Contract`] is not doubled,
/// doubled or redoubled
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum Doubled {
    NotDoubled = 0,
    Doubled = 1,
    Redoubled = 2,
}

/// The strain of a bridge contract, either some trump or
/// no trump
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Strain {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
    NoTrumps = 4,
}

impl PartialOrd for Strain {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Strain {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if let Strain::NoTrumps = self {
            match other {
                Strain::NoTrumps => Ordering::Equal,
                _ => Ordering::Greater,
            }
        } else {
            let comp = (*self as usize).cmp(&(*other as usize));
            match comp {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => comp,
            }
        }
    }
}

impl Contract {
    /// Returns the strain of the contract.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn strain(&self) -> Strain {
        self.strain
    }

    /// Returns the next player after the declarer, i.e., the leader of the contract.
    #[must_use]
    pub fn leader(&self) -> Seat {
        self.declarer().next()
    }

    #[must_use]
    pub fn level(&self) -> u8 {
        self.level
    }

    /// Returns the declarer of the contract.
    #[must_use]
    pub fn declarer(&self) -> Seat {
        self.declarer
    }

    /// Constructs a `Contract` from a string representation: "4CN" or "3NW".
    ///
    /// # Errors
    /// Errors if the contract string is improperly formatted
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_str(s: &str, vuln: Vulnerable) -> Result<Self, DealerError> {
        let doubled = match s.len() - s.trim_end_matches('X').len() {
            0 => Doubled::NotDoubled,
            1 => Doubled::Doubled,
            2 => Doubled::Redoubled,
            _ => unreachable!("too many `X`"),
        };
        let mut chars = s.chars();
        let level = chars
            .next()
            .ok_or(DealerError::new("no contract level"))?
            .to_digit(10)
            .ok_or(DealerError::new("contract level too high"))?;
        if !(1..=7).contains(&level) {
            return Err(DealerError::new("Wrong contract level"));
        };
        Ok(Self {
            vuln,
            doubled,
            level: level as u8,
            strain: chars
                .next()
                .ok_or(DealerError::new("no contract strain"))?
                .try_into()?,
            declarer: chars
                .next()
                .ok_or(DealerError::new("no contract dealer"))?
                .try_into()?,
        })
    }

    /// Returns a non-unicode string representation of the contract.
    #[must_use]
    pub fn not_unicode_str(&self) -> String {
        format!(
            "{}{}{}{}",
            self.level,
            self.strain.not_unicode_str(),
            self.declarer,
            if matches!(self.doubled, Doubled::NotDoubled) {
                String::new()
            } else {
                let mut stringa = String::new();
                for _ in 0..(self.doubled as usize) {
                    stringa.push('X');
                }
                stringa
            }
        )
    }

    /// Constructs a new `Contract` instance.
    #[must_use]
    pub fn new(
        level: u8,
        strain: Strain,
        declarer: Seat,
        vuln: Vulnerable,
        doubled: Doubled,
    ) -> Self {
        Self {
            vuln,
            level,
            doubled,
            strain,
            declarer,
        }
    }
}
impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.level,
            self.strain,
            self.declarer,
            if matches!(self.doubled, Doubled::NotDoubled) {
                String::new()
            } else {
                let mut stringa = String::new();
                for _ in 0..(self.doubled as usize) {
                    stringa.push('X');
                }
                stringa
            }
        )
    }
}

impl TryFrom<char> for Strain {
    type Error = DealerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'S' => Ok(Self::Spades),
            'H' => Ok(Self::Hearts),
            'D' => Ok(Self::Diamonds),
            'C' => Ok(Self::Clubs),
            'N' => Ok(Self::NoTrumps),
            _ => Err(DealerError::new("Not a strain.")),
        }
    }
}

impl Strain {
    fn not_unicode_str(self) -> String {
        match self {
            Self::Spades => String::from("S"),
            Self::Hearts => String::from("H"),
            Self::Diamonds => String::from("D"),
            Self::NoTrumps => String::from("NT"),
            Self::Clubs => String::from("C"),
        }
    }
}

impl fmt::Display for Strain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spades => write!(f, "♠"),
            Self::Hearts => write!(f, "♥"),
            Self::Diamonds => write!(f, "♦"),
            Self::NoTrumps => write!(f, "NT"),
            Self::Clubs => write!(f, "♣"),
        }
    }
}
