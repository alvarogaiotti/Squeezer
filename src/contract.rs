// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The contract vocabulary: seats, vulnerability, and what was bid.
//!
//! This module is the one home of [`Seat`], [`Vulnerability`], [`Vulnerable`],
//! [`Doubled`], [`Strain`] and [`Contract`]. It imports the specific items it
//! needs (the card vocabulary, the table-size constant and standard-library
//! traits) rather than the crate prelude glob, so the dependency runs one way:
//! the deal, board, auction, position, serializer and parser modules import
//! these types from here, and nothing here names a record type.
//!
//! Parsing a seat or a contract reports [`SeatError`] and [`ContractError`],
//! the contract-local errors. They are not dealer errors: a malformed contract
//! string is an input failure of this module, whether it arrives from a file,
//! a literal or a test.

use std::array::IntoIter;
use std::fmt;
use std::str::FromStr;

use crate::NUMBER_OF_HANDS;
use crate::card::Suit;

/// Why a character is not a seat.
///
/// [`Seat`] parses from the four letters `N`, `E`, `S` and `W`, in either case.
/// The error names the offending character rather than the whole input, so a
/// caller can point at the token that failed.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SeatError {
    /// The character is not one of `N`, `E`, `S` or `W`.
    NotASeat {
        /// The character that failed to name a seat.
        character: char,
    },
}

impl fmt::Display for SeatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotASeat { character } => write!(f, "'{character}' is not a valid seat"),
        }
    }
}

impl std::error::Error for SeatError {}

/// Why a string is not a contract.
///
/// This is the contract layer's parse error. It wraps [`SeatError`] for a
/// declarer that is not a seat, so the cause stays typed rather than being
/// flattened into a message.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContractError {
    /// The string ended before a contract level.
    MissingLevel,
    /// The level character was not a decimal digit.
    LevelNotADigit(char),
    /// The level was a digit outside 1-7.
    LevelOutOfRange(u8),
    /// The string ended before a strain.
    MissingStrain,
    /// The strain character is not one of `S`, `H`, `D`, `C` or `N`.
    UnknownStrain(char),
    /// The string ended before a declarer.
    MissingDeclarer,
    /// The string ended after an `X` or `XX` suffix, before the declarer.
    MissingDeclarerAfterDoubled,
    /// The declarer character is not a seat.
    Seat(SeatError),
    /// The vulnerability suffix is not `N` or `V`.
    UnknownVulnerability(char),
}

impl fmt::Display for ContractError {
    /// The messages are the wording the contract parser produced before this
    /// module owned the error (it came from the dealer error), preserved
    /// verbatim so the move changes no user-visible output. The variants, not
    /// the strings, are the structured contract.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLevel => write!(f, "no contract level"),
            Self::LevelNotADigit(_) => write!(f, "contract level too high"),
            Self::LevelOutOfRange(_) => write!(f, "Wrong contract level"),
            Self::MissingStrain => write!(f, "no contract strain"),
            Self::UnknownStrain(_) => write!(f, "Not a strain."),
            Self::MissingDeclarer => write!(f, "no contract dealer"),
            Self::MissingDeclarerAfterDoubled => {
                write!(f, "contract is incomplete: hasn't dealer specified")
            }
            Self::Seat(cause) => write!(f, "{cause}"),
            Self::UnknownVulnerability(_) => write!(f, "unrecognized vulnerability"),
        }
    }
}

impl std::error::Error for ContractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Seat(cause) => Some(cause),
            Self::MissingLevel
            | Self::LevelNotADigit(_)
            | Self::LevelOutOfRange(_)
            | Self::MissingStrain
            | Self::UnknownStrain(_)
            | Self::MissingDeclarer
            | Self::MissingDeclarerAfterDoubled
            | Self::UnknownVulnerability(_) => None,
        }
    }
}

impl From<SeatError> for ContractError {
    fn from(value: SeatError) -> Self {
        Self::Seat(value)
    }
}

/// Represents a seat at the bridge table.
///
/// The four seats are arranged around the table with North-South as partners
/// and East-West as partners. Seats are numbered 0-3 for efficient array indexing.
///
/// # Partnerships
///
/// - **North-South**: Same parity (0, 2) - both even
/// - **East-West**: Same parity (1, 3) - both odd
///
/// Use `is_same_line()` to check if two seats are partners.
///
/// # Rotation Order
///
/// Seats rotate clockwise: North → East → South → West → North
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Seat {
    /// North (index 0), partners with South
    #[default]
    North = 0,
    /// East (index 1), partners with West
    East = 1,
    /// South (index 2), partners with North
    South = 2,
    /// West (index 3), partners with East
    West = 3,
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::North => write!(f, "N"),
            Self::East => write!(f, "E"),
            Self::West => write!(f, "W"),
            Self::South => write!(f, "S"),
        }
    }
}

impl TryFrom<char> for Seat {
    type Error = SeatError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'N' | 'n' => Ok(Self::North),
            'S' | 's' => Ok(Self::South),
            'W' | 'w' => Ok(Self::West),
            'E' | 'e' => Ok(Self::East),
            _ => Err(SeatError::NotASeat { character: value }),
        }
    }
}

impl Seat {
    /// Returns the next seat in clockwise rotation.
    ///
    /// Order: North → East → South → West → North (wraps around)
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// assert_eq!(Seat::North.next(), Seat::East);
    /// assert_eq!(Seat::West.next(), Seat::North);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn next(self) -> Seat {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    /// Returns the previous seat in counter-clockwise rotation.
    ///
    /// Order: North ← West ← South ← East ← North (wraps around)
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// assert_eq!(Seat::East.previous(), Seat::North);
    /// assert_eq!(Seat::North.previous(), Seat::West);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn previous(self) -> Seat {
        match self {
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
            Self::West => Self::South,
        }
    }

    /// Checks if two seats are partners (on the same line).
    ///
    /// In bridge, North-South are partners (both have even indices: 0, 2)
    /// and East-West are partners (both have odd indices: 1, 3).
    ///
    /// This is implemented via a parity check: same parity = same partnership.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// assert!(Seat::North.is_same_line(&Seat::South));  // Partners
    /// assert!(Seat::East.is_same_line(&Seat::West));    // Partners
    /// assert!(!Seat::North.is_same_line(&Seat::East));  // Opponents
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn is_same_line(&self, other: &Seat) -> bool {
        (*self as u8) % 2 == (*other as u8) % 2
    }

    /// Returns the partner of this player
    #[must_use]
    #[inline]
    pub fn partner(&self) -> Seat {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    /// Returns an iterator over all four seats starting from North.
    ///
    /// Order: North → East → South → West
    #[must_use]
    #[inline]
    pub fn iter() -> IntoIter<Seat, 4> {
        [Seat::North, Seat::East, Seat::South, Seat::West].into_iter()
    }

    /// Returns the full English name of the seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// assert_eq!(Seat::North.long_str(), "North");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn long_str(&self) -> &str {
        match self {
            Self::North => "North",
            Self::East => "East",
            Self::West => "West",
            Self::South => "South",
        }
    }

    /// Returns an infinite iterator starting from this seat, rotating clockwise.
    ///
    /// **Warning**: This iterator never terminates! Use `.take(n)` to limit it.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let mut iter = Seat::North.iter_from();
    /// assert_eq!(iter.next(), Some(Seat::North));
    /// assert_eq!(iter.next(), Some(Seat::East));
    /// // Continues forever...
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn iter_from(self) -> RotatingSeatIterator {
        RotatingSeatIterator::new(self)
    }
}

/// An infinite iterator that cycles through seats in clockwise order.
///
/// **Warning**: This iterator never stops! Always use `.take(n)` or break manually.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use squeezer::*;
/// let seats: Vec<_> = Seat::North.iter_from().take(5).collect();
/// assert_eq!(seats, vec![Seat::North, Seat::East, Seat::South, Seat::West, Seat::North]);
/// # Ok(())
/// # }
/// ```
pub struct RotatingSeatIterator {
    state: Seat,
}

impl Iterator for RotatingSeatIterator {
    type Item = Seat;

    fn next(&mut self) -> Option<Self::Item> {
        self.state = self.state.next();
        Some(self.state)
    }
}

impl RotatingSeatIterator {
    /// Creates a new rotating iterator starting from the given seat.
    ///
    /// The first call to `.next()` will return the starting seat.
    #[inline]
    #[must_use]
    pub fn new(state: Seat) -> Self {
        Self {
            state: state.previous(),
        }
    }

    /// Updates the iterator's current position.
    pub fn set_state(&mut self, state: Seat) {
        self.state = state;
    }

    /// Returns the current seat position (before the next `.next()` call).
    #[must_use]
    pub fn state(&self) -> Seat {
        self.state
    }
}

/// Macro to implement arithmetic operations and conversions for `Seat`.
///
/// This allows natural seat rotation via addition: `Seat::North + 2 == Seat::South`
///
/// The modulo operation ensures wrapping: `Seat::West + 1 == Seat::North`
macro_rules! impl_add_and_from_ints_for_seat {
    ($($t:ty),*) => {
        $(
        impl std::ops::Add<$t> for Seat {
            type Output = Seat;

            #[inline]
            fn add(self, rhs: $t) -> Self::Output {
                (self as $t + rhs).into()
            }
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        impl From<$t> for Seat {
            #[inline]
            fn from(n: $t) -> Self {
                match n % NUMBER_OF_HANDS as $t {
                    x if x == Seat::North as $t => Seat::North,
                    x if x == Seat::East as $t => Seat::East,
                    x if x == Seat::South as $t => Seat::South,
                    x if x == Seat::West as $t => Seat::West,
                    _ => unreachable!(),
                }
            }
        }

        impl std::ops::Add<&$t> for Seat {
            type Output = Seat;

            #[inline]
            fn add(self, rhs: &$t) -> Self::Output {
                (self as $t + *rhs).into()
            }
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        impl From<&$t> for Seat {
            #[inline]
            fn from(n: &$t) -> Self {
                match *n % NUMBER_OF_HANDS as $t {
                    x if x == Seat::North as $t => Seat::North,
                    x if x == Seat::East as $t => Seat::East,
                    x if x == Seat::South as $t => Seat::South,
                    x if x == Seat::West as $t => Seat::West,
                    _ => unreachable!(),
                }
            }
        }
        )*
    };
}
impl_add_and_from_ints_for_seat!(usize, u64, u32, u16, u8, isize, i64, i32, i16, i8);

/// Represents vulnerability state in a bridge game.
///
/// Vulnerability affects scoring: vulnerable pairs score more when making contracts
/// but lose more when failing. The two partnerships (NS and EW) can be independently
/// vulnerable or not.
///
/// # Binary Encoding
///
/// The enum values use bit flags:
/// - Bit 0 (value 1): NS vulnerable
/// - Bit 1 (value 2): EW vulnerable
///
/// This allows efficient `is_vulnerable()` checks via bitwise operations.
///
/// # Standard Rotation
///
/// In duplicate bridge, vulnerability rotates through 16 boards following a standard
/// pattern (see `VULNERABILITY_TABLE`). Use `from_number()` to get the correct
/// vulnerability for any board number.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
#[repr(u8)]
pub enum Vulnerability {
    /// Neither side vulnerable (bit pattern: 00)
    #[default]
    None = 0,
    /// North-South vulnerable only (bit pattern: 01)
    NS = 1,
    /// East-West vulnerable only (bit pattern: 10)
    EW = 2,
    /// Both sides vulnerable (bit pattern: 11)
    All = 3,
}

impl Vulnerability {
    /// Standard vulnerability rotation for duplicate bridge boards 1-16.
    ///
    /// This pattern repeats every 16 boards. Pattern:
    /// 1. None, 2. NS, 3. EW, 4. All, 5. NS, 6. EW, 7. All, 8. None,
    /// 9. EW, 10. All, 11. None, 12. NS, 13. All, 14. None, 15. NS, 16. EW
    pub const VULNERABILITY_TABLE: [Vulnerability; 16] = [
        Vulnerability::None,
        Vulnerability::NS,
        Vulnerability::EW,
        Vulnerability::All,
        Vulnerability::NS,
        Vulnerability::EW,
        Vulnerability::All,
        Vulnerability::None,
        Vulnerability::EW,
        Vulnerability::All,
        Vulnerability::None,
        Vulnerability::NS,
        Vulnerability::All,
        Vulnerability::None,
        Vulnerability::NS,
        Vulnerability::EW,
    ];

    /// Checks if a specific seat is vulnerable given this vulnerability state.
    ///
    /// Delegates to [`Vulnerable::new`], the single home of the seat-parity
    /// bit test shared with contract scoring and LIN parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let vuln = Vulnerability::NS;
    /// assert_eq!(vuln.is_vulnerable(Seat::North), Vulnerable::Yes);
    /// assert_eq!(vuln.is_vulnerable(Seat::East), Vulnerable::No);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_vulnerable(&self, seat: Seat) -> Vulnerable {
        Vulnerable::new(*self, seat)
    }

    /// Returns the vulnerability state for a given board number.
    ///
    /// Uses the standard duplicate bridge 16-board rotation pattern.
    /// Board numbers wrap: board 17 = board 1, board 18 = board 2, etc.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// assert_eq!(Vulnerability::from_number(1), Vulnerability::None);
    /// assert_eq!(Vulnerability::from_number(2), Vulnerability::NS);
    /// assert_eq!(Vulnerability::from_number(17), Vulnerability::None); // Wraps
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_number(board_number: u8) -> Self {
        Self::VULNERABILITY_TABLE[((board_number - 1) % 16) as usize]
    }
}

/// Iterator over vulnerability states following the standard 16-board pattern.
///
/// This iterator cycles infinitely through the vulnerability rotation.
pub struct VulnerabilityIterator {
    board_number: u8,
}

impl VulnerabilityIterator {
    /// Creates a new iterator starting from board 1.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { board_number: 1 }
    }

    /// Creates an iterator starting from a specific vulnerability state.
    ///
    /// Starts at the first board (1-4) that has the given vulnerability.
    #[inline]
    #[must_use]
    pub fn from_state(state: Vulnerability) -> Self {
        let board_number = match state {
            Vulnerability::None => 1,
            Vulnerability::NS => 2,
            Vulnerability::EW => 3,
            Vulnerability::All => 4,
        };
        Self { board_number }
    }

    /// Creates an iterator starting from a specific board number.
    #[inline]
    #[must_use]
    pub const fn from_board_number(board_number: u8) -> Self {
        Self { board_number }
    }
}

impl Default for VulnerabilityIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for VulnerabilityIterator {
    type Item = Vulnerability;

    fn next(&mut self) -> Option<Self::Item> {
        let state = Vulnerability::from_number(self.board_number);
        self.board_number = self.board_number.wrapping_add(1);
        if self.board_number == 0 {
            self.board_number = 1; // Handle overflow
        }
        Some(state)
    }
}

impl IntoIterator for Vulnerability {
    type IntoIter = VulnerabilityIterator;
    type Item = Vulnerability;

    fn into_iter(self) -> Self::IntoIter {
        VulnerabilityIterator::from_state(self)
    }
}

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

/// Parses the `<level><strain>` prefix shared by both contract constructors.
///
/// Only the prefix is shared: [`Contract::from_str_and_vuln`] takes the
/// vulnerability as an argument, while [`Contract::from_str`] reads a doubled
/// suffix and a vulnerability letter from the rest of the string. Level fits
/// in a `u8` because the `1..=7` check rejects every larger digit first.
#[allow(clippy::cast_possible_truncation)]
fn parse_level_and_strain(chars: &mut std::str::Chars<'_>) -> Result<(u8, Strain), ContractError> {
    let level_char = chars.next().ok_or(ContractError::MissingLevel)?;
    let level = level_char
        .to_digit(10)
        .ok_or(ContractError::LevelNotADigit(level_char))?;
    if !(1..=7).contains(&level) {
        return Err(ContractError::LevelOutOfRange(level as u8));
    }
    let strain = chars
        .next()
        .ok_or(ContractError::MissingStrain)?
        .try_into()?;
    Ok((level as u8, strain))
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

    /// Returns whether the declarer's side is vulnerable under this contract.
    #[must_use]
    pub const fn vulnerable(&self) -> Vulnerable {
        self.vuln
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
    pub fn from_str_and_vuln(s: &str, vuln: Vulnerable) -> Result<Contract, ContractError> {
        let doubled = match s.len() - s.trim_end_matches('X').len() {
            0 => Doubled::NotDoubled,
            1 => Doubled::Doubled,
            2 => Doubled::Redoubled,
            _ => unreachable!("too many `X`"),
        };
        let mut chars = s.chars();
        let (level, strain) = parse_level_and_strain(&mut chars)?;
        let declarer = chars
            .next()
            .ok_or(ContractError::MissingDeclarer)?
            .try_into()?;
        Ok(Self {
            vuln,
            doubled,
            level,
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
    type Err = ContractError;
    /// Constructs a `Contract` from a string representation: "4CN" or "3NW".
    ///
    /// # Errors
    /// Errors if the contract string is improperly formatted
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let (level, strain) = parse_level_and_strain(&mut chars)?;
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
                        return Err(ContractError::MissingDeclarerAfterDoubled);
                    }
                }
                _ => Doubled::NotDoubled,
            }
        } else {
            return Err(ContractError::MissingDeclarerAfterDoubled);
        };
        let declarer = chars
            .next()
            .ok_or(ContractError::MissingDeclarer)?
            .try_into()?;
        let vuln = if let Some(vuln) = chars.next() {
            match vuln {
                'n' | 'N' => Vulnerable::No,
                'v' | 'V' => Vulnerable::Yes,
                _ => return Err(ContractError::UnknownVulnerability(vuln)),
            }
        } else {
            Vulnerable::No
        };
        Ok(Self {
            vuln,
            doubled,
            level,
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
    type Error = ContractError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'S' => Ok(Self::Spades),
            'H' => Ok(Self::Hearts),
            'D' => Ok(Self::Diamonds),
            'C' => Ok(Self::Clubs),
            'N' => Ok(Self::NoTrumps),
            _ => Err(ContractError::UnknownStrain(value)),
        }
    }
}

impl Strain {
    /// The trump suit this strain names, or `None` for no-trumps.
    ///
    /// This is the partial inverse of `From<Suit> for Strain`, and it is a
    /// method here rather than a `TryFrom<Strain> for Suit` because `Suit` and
    /// `Strain` are destined for different crates: only the contract layer can
    /// name both types.
    #[must_use]
    pub const fn as_suit(self) -> Option<Suit> {
        match self {
            Self::Spades => Some(Suit::Spades),
            Self::Hearts => Some(Suit::Hearts),
            Self::Diamonds => Some(Suit::Diamonds),
            Self::Clubs => Some(Suit::Clubs),
            Self::NoTrumps => None,
        }
    }

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

/// Every suit is a strain; the reverse conversion is [`Strain::as_suit`].
impl From<Suit> for Strain {
    fn from(suit: Suit) -> Self {
        match suit {
            Suit::Spades => Self::Spades,
            Suit::Hearts => Self::Hearts,
            Suit::Diamonds => Self::Diamonds,
            Suit::Clubs => Self::Clubs,
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
#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::{Contract, ContractError, Seat, SeatError, Strain, Vulnerability, Vulnerable};
    use crate::card::Suit;

    /// Tests seat rotation.
    #[test]
    fn test_seat_rotation() {
        assert_eq!(Seat::North.next(), Seat::East);
        assert_eq!(Seat::East.next(), Seat::South);
        assert_eq!(Seat::South.next(), Seat::West);
        assert_eq!(Seat::West.next(), Seat::North);
    }

    /// Tests partnership detection.
    #[test]
    fn test_partnerships() {
        assert!(Seat::North.is_same_line(&Seat::South));
        assert!(Seat::East.is_same_line(&Seat::West));
        assert!(!Seat::North.is_same_line(&Seat::East));
        assert!(!Seat::South.is_same_line(&Seat::West));
    }

    /// Tests vulnerability calculation.
    #[test]
    fn test_vulnerability() {
        assert_eq!(
            Vulnerability::NS.is_vulnerable(Seat::North),
            Vulnerable::Yes
        );
        assert_eq!(
            Vulnerability::NS.is_vulnerable(Seat::South),
            Vulnerable::Yes
        );
        assert_eq!(Vulnerability::NS.is_vulnerable(Seat::East), Vulnerable::No);
        assert_eq!(Vulnerability::NS.is_vulnerable(Seat::West), Vulnerable::No);

        assert_eq!(
            Vulnerability::All.is_vulnerable(Seat::North),
            Vulnerable::Yes
        );
        assert_eq!(
            Vulnerability::None.is_vulnerable(Seat::North),
            Vulnerable::No
        );
    }

    /// Tests vulnerability from board number.
    #[test]
    fn test_vulnerability_from_board() {
        assert_eq!(Vulnerability::from_number(1), Vulnerability::None);
        assert_eq!(Vulnerability::from_number(2), Vulnerability::NS);
        assert_eq!(Vulnerability::from_number(3), Vulnerability::EW);
        assert_eq!(Vulnerability::from_number(4), Vulnerability::All);

        // Test wrapping
        assert_eq!(Vulnerability::from_number(17), Vulnerability::None);
    }

    /// Tests rotating seat iterator.
    #[test]
    fn test_rotating_iterator() {
        let seats: Vec<_> = Seat::North.iter_from().take(5).collect();
        assert_eq!(
            seats,
            vec![
                Seat::North,
                Seat::East,
                Seat::South,
                Seat::West,
                Seat::North
            ]
        );
    }

    #[test]
    fn seat_parse_error_names_the_offending_character() {
        assert_eq!(
            Seat::try_from('Q'),
            Err(SeatError::NotASeat { character: 'Q' })
        );
        assert_eq!(
            SeatError::NotASeat { character: 'Q' }.to_string(),
            "'Q' is not a valid seat"
        );
    }

    /// The contract error is typed per failure, and a bad declarer keeps the
    /// seat error as its cause rather than flattening it.
    #[test]
    fn contract_parse_errors_are_typed() {
        assert_eq!(
            Contract::from_str("").expect_err("empty string has no level"),
            ContractError::MissingLevel
        );
        assert_eq!(
            Contract::from_str("8SN").expect_err("level 8 is out of range"),
            ContractError::LevelOutOfRange(8)
        );
        assert_eq!(
            Contract::from_str("4QN").expect_err("Q is not a strain"),
            ContractError::UnknownStrain('Q')
        );
        assert_eq!(
            Contract::from_str("4SNQ").expect_err("Q is not a vulnerability"),
            ContractError::UnknownVulnerability('Q')
        );

        let error = Contract::from_str("4NQ").expect_err("Q is not a declarer");
        assert_eq!(
            error,
            ContractError::Seat(SeatError::NotASeat { character: 'Q' })
        );
        assert_eq!(error.to_string(), "'Q' is not a valid seat");
        assert!(
            std::error::Error::source(&error)
                .and_then(|source| source.downcast_ref::<SeatError>())
                .is_some(),
            "the seat error stays the source"
        );
    }

    /// The pre-move messages survive the typed error, so callers that printed
    /// a parse failure see the same text.
    #[test]
    fn contract_parse_error_display_preserves_messages() {
        assert_eq!(
            Contract::from_str("4QN")
                .expect_err("Q is not a strain")
                .to_string(),
            "Not a strain."
        );
        assert_eq!(
            Contract::from_str("4S")
                .expect_err("a contract needs a declarer")
                .to_string(),
            "contract is incomplete: hasn't dealer specified"
        );
    }

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

    #[test]
    fn strain_as_suit_names_the_four_trumps_and_no_suit_for_no_trumps() {
        for (strain, suit) in [
            (Strain::Spades, Some(Suit::Spades)),
            (Strain::Hearts, Some(Suit::Hearts)),
            (Strain::Diamonds, Some(Suit::Diamonds)),
            (Strain::Clubs, Some(Suit::Clubs)),
            (Strain::NoTrumps, None),
        ] {
            assert_eq!(strain.as_suit(), suit, "{strain}");
            if let Some(suit) = suit {
                assert_eq!(Strain::from(suit), strain);
            }
        }
    }
}
