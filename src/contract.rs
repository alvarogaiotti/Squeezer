// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;

/// Per-seat vulnerability answer: is *this* seat vulnerable?
///
/// This is the adjective to [`Vulnerability`]'s noun. `Vulnerability`
/// describes the board state (none / NS / EW / all); `Vulnerable` is the
/// answer for one seat under that state. Convert between them with
/// [`Vulnerable::new`] (or `Vulnerable::from((vulnerability, seat))`) rather
/// than reimplementing the seat-parity bit test at each call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Vulnerable {
    Yes,
    No,
}

impl Vulnerable {
    /// Answers vulnerability for `seat` under `vulnerability`.
    ///
    /// Single home of the seat-parity bit test: even seats (North, South)
    /// read bit 0 (NS), odd seats (East, West) read bit 1 (EW).
    #[must_use]
    pub const fn new(vulnerability: Vulnerability, seat: Seat) -> Self {
        let seat_position = (seat as usize) % 2;
        if ((vulnerability as usize) & (1 << seat_position)) != 0 {
            Self::Yes
        } else {
            Self::No
        }
    }

    #[must_use]
    pub const fn from_number_and_seat(board_number: u8, seat: Seat) -> Self {
        Self::new(Vulnerability::from_number(board_number), seat)
    }

    /// Convenience predicate over the answer.
    #[must_use]
    pub const fn is_yes(self) -> bool {
        matches!(self, Self::Yes)
    }
}

impl From<(Vulnerability, Seat)> for Vulnerable {
    /// Ergonomic bridge from board state plus seat to per-seat answer.
    fn from((vulnerability, seat): (Vulnerability, Seat)) -> Self {
        Self::new(vulnerability, seat)
    }
}
/// A struct representing a contract
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contract {
    vuln: Vulnerable,
    level: u8,
    doubled: Doubled,
    strain: Strain,
    declarer: Seat,
}

impl PartialOrd for Contract {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Contract {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        let level_comparison = self.level().cmp(&other.level);
        match level_comparison {
            Ordering::Equal => self.strain().cmp(&other.strain()),
            _ => level_comparison,
        }
    }
}

#[cfg(feature = "dds")]
impl dds::traits::AsDDSContract for Contract {
    fn as_dds_contract(&self) -> (dds::deal::DdsSuit, dds::deal::DdsHandEncoding) {
        (self.strain.into(), (self.declarer.next()).into())
    }
}

#[cfg(feature = "dds")]
impl dds::traits::ContractScorer for Contract {
    fn score(&self, tricks: u8) -> i32 {
        self.score(tricks)
    }
}

/// Enum modelling whether a [`Contract`] is not doubled,
/// doubled or redoubled
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Doubled {
    NotDoubled = 0,
    Doubled = 1,
    Redoubled = 2,
}

/// The strain of a bridge contract, either some trump or
/// no trump
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
            let comp = (*self as u8).cmp(&(*other as u8));
            match comp {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => comp,
            }
        }
    }
}

#[cfg(feature = "dds")]
impl From<Strain> for dds::deal::DdsSuit {
    fn from(value: Strain) -> Self {
        match value {
            Strain::NoTrumps => Self::NoTrump,
            Strain::Spades => Self::Spades,
            Strain::Hearts => Self::Hearts,
            Strain::Diamonds => Self::Diamonds,
            Strain::Clubs => Self::Clubs,
        }
    }
}

impl From<u8> for Strain {
    fn from(value: u8) -> Self {
        match value % 5 {
            0 => Self::Spades,
            1 => Self::Hearts,
            2 => Self::Diamonds,
            3 => Self::Clubs,
            4 => Self::NoTrumps,
            _ => unreachable!(),
        }
    }
}
pub trait Scorable {
    fn score(&self, tricks: u8) -> i32;
}

impl Scorable for Contract {
    fn score(&self, tricks: u8) -> i32 {
        self.score(tricks)
    }
}
impl Contract {
    /// Returns the strain of the contract.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn strain(&self) -> Strain {
        self.strain
    }

    #[must_use]
    #[inline]
    pub fn score(&self, tricks: u8) -> i32 {
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
            }

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
            }
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

    /// Constructs a `Contract` from a string representation: "4CN" or "3NW".
    ///
    /// # Errors
    /// Errors if the contract string is improperly formatted
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_str_and_vuln(s: &str, vuln: Vulnerable) -> Result<Contract, DealerError> {
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
        }
        let strain = chars
            .next()
            .ok_or(DealerError::new("no contract strain"))?
            .try_into()?;
        let declarer = chars
            .next()
            .ok_or(DealerError::new("no contract dealer"))?
            .try_into()?;
        Ok(Self {
            vuln,
            doubled,
            level: level as u8,
            strain,
            declarer,
        })
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

impl FromStr for Contract {
    type Err = DealerError;
    /// Constructs a `Contract` from a string representation: "4CN" or "3NW".
    ///
    /// # Errors
    /// Errors if the contract string is improperly formatted
    #[allow(clippy::cast_possible_truncation)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let level = chars
            .next()
            .ok_or(DealerError::new("no contract level"))?
            .to_digit(10)
            .ok_or(DealerError::new("contract level too high"))?;
        if !(1..=7).contains(&level) {
            return Err(DealerError::new("Wrong contract level"));
        }
        let strain = chars
            .next()
            .ok_or(DealerError::new("no contract strain"))?
            .try_into()?;
        let mut chars = chars.peekable();
        let doubled = if let Some(doubled) = chars.peek() {
            match *doubled {
                'x' | 'X' => {
                    _ = chars.next();
                    if let Some(redoubled) = chars.peek() {
                        match redoubled {
                            'x' | 'X' => {
                                _ = chars.next();
                                Doubled::Redoubled
                            }
                            _ => Doubled::Doubled,
                        }
                    } else {
                        return Err(DealerError::new(
                            "contract is incomplete: hasn't dealer specified",
                        ));
                    }
                }
                _ => Doubled::NotDoubled,
            }
        } else {
            return Err(DealerError::new(
                "contract is incomplete: hasn't dealer specified",
            ));
        };
        let declarer = chars
            .next()
            .ok_or(DealerError::new("no contract dealer"))?
            .try_into()?;
        let vuln = if let Some(vuln) = chars.next() {
            match vuln {
                'n' | 'N' => Vulnerable::No,
                'v' | 'V' => Vulnerable::Yes,
                _ => return Err(DealerError::new("unrecognized vulnerability")),
            }
        } else {
            Vulnerable::No
        };
        Ok(Self {
            vuln,
            doubled,
            level: level as u8,
            strain,
            declarer,
        })
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
    pub const ALL: [Self; 5] = [
        Self::Spades,
        Self::Hearts,
        Self::Diamonds,
        Self::Clubs,
        Self::NoTrumps,
    ];
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
#[cfg(test)]
mod test {
    use super::{Contract, Vulnerable};
    #[test]
    fn can_create_contract_from_str_test() {
        let contract_c = Contract::from_str_and_vuln("3CN", Vulnerable::No).unwrap();
        let contract_d = Contract::from_str_and_vuln("3DN", Vulnerable::No).unwrap();
        let contract_s = Contract::from_str_and_vuln("3SN", Vulnerable::No).unwrap();
        let contract_h = Contract::from_str_and_vuln("3HN", Vulnerable::No).unwrap();
        let contract_n = Contract::from_str_and_vuln("3NNXX", Vulnerable::No).unwrap();
        assert_eq!(contract_c.to_string(), "3♣N");
        assert_eq!(contract_d.to_string(), "3♦N");
        assert_eq!(contract_h.to_string(), "3♥N");
        assert_eq!(contract_s.to_string(), "3♠N");
        assert_eq!(contract_n.to_string(), "3NTNXX");
    }
}
