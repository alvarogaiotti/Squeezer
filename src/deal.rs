// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! # Bridge Deal Generation Module
//!
//! This module provides a flexible system for generating bridge deals with various constraints.
//! The core abstraction is the `Dealer` trait, with `StandardDealer` as the main implementation.
//!
//! ## Architecture Overview
//!
//! ```text
//! DealerBuilder → StandardDealer → Deal
//!      ↓              ↓              ↓
//!   Configure    Generate      Format/Display
//! ```
//!
//! ## Key Features
//!
//! - **Predealing**: Assign specific cards to specific seats
//! - **Hand constraints**: Specify shape, HCP, controls, etc. per seat
//! - **Custom acceptance**: Add arbitrary cross-hand validation logic
//! - **Multiple output formats**: PBN, LIN, short, long
//! - **Board numbering**: Sequential or fixed
//!
//! ## Performance Characteristics
//!
//! - **Unconstrained deals**: ~1-5 microseconds per deal
//! - **With shape constraints**: ~10-50 microseconds (depends on rarity)
//! - **Complex multi-hand constraints**: Can vary significantly
//!
//! ## Example Usage
//!
//! ```rust
//! use squeezer::*;
//!
//! // Simple deal with North predealed
//! let dealer = DealerBuilder::new()
//!     .predeal(Seat::North, Cards::from_str("SAKQHAKQDAKQCAKQJ")?.try_into()?)?
//!     .build()?;
//!
//! let deal = dealer.deal()?;
//! println!("{}", deal);
//!
//! // Complex constraint: NS 8+ card major fit
//! let dealer = DealerBuilder::new()
//!     .with_function(|hands: &Hands| {
//!         let ns_spades = hands.north().spades().len() + hands.south().spades().len();
//!         let ns_hearts = hands.north().hearts().len() + hands.south().hearts().len();
//!         ns_spades >= 8 || ns_hearts >= 8
//!     })
//!     .with_hand_descriptor(
//!         Seat::North,
//!         HandDescriptor::new()
//!             .hcp(12..=37)
//!             .shape(Shape::from_str("5332")?)
//!     )
//!     .build()?;
//! ```

use std::{cell::Cell, num::NonZeroU8};

#[cfg(feature = "lin")]
use crate::linparser::LinDeal;
use crate::prelude::*;

/// Type alias for functions that validate whether a deal should be accepted.
///
/// These functions receive the four hands and return `true` if the deal meets
/// the criteria. Typically used for cross-hand constraints like "NS must have
/// an 8+ card spade fit" that can't be expressed through individual hand descriptors.
///
/// The `Send` bound allows the dealer to be used across threads.
///
/// # Example
///
/// ```rust
/// let accept: AcceptFunction = Box::new(|hands: &Hands| {
///     // Accept only if North-South have game-forcing values combined
///     hands.north().hcp() + hands.south().hcp() >= 25
/// });
/// ```
pub type AcceptFunction = Box<dyn Fn(&Hands) -> bool + Send>;

/// A complete bridge deal consisting of four 13-card hands.
///
/// This is the fundamental structure representing a bridge deal. Hands are stored
/// in array order: [North, East, South, West] (indices 0-3).
///
/// # Memory Layout
///
/// Each `Hand` is typically 8 bytes (bitboard representation), so `Hands` is
/// 32 bytes total - very cache-friendly.
///
/// # Indexing
///
/// You can index by `Seat` or `usize`:
/// ```rust
/// let hands: Hands = /* ... */;
/// let north = hands[Seat::North];
/// let north = hands[0]; // Equivalent
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hands {
    /// Array of hands in order: [North, East, South, West]
    hands: [Hand; 4],
}

impl std::fmt::Display for Hands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let west = self.hands[Seat::West as usize].to_string();
        let east = self.hands[Seat::East as usize].to_string();
        let north = self.hands[Seat::North as usize].to_string();
        let south = self.hands[Seat::South as usize].to_string();
        let width = west.chars().count() + east.chars().count() + north.chars().count() / 2;
        write!(
            f,
            "{north:^0$}\n{west:<1$}{east:>1$}\n{south:^0$}",
            width + 1,
            width / 2,
        )
    }
}

impl IntoIterator for Hands {
    type IntoIter = IntoIter<Hand, 4>;
    type Item = Hand;

    fn into_iter(self) -> Self::IntoIter {
        self.hands.into_iter()
    }
}

impl std::ops::Index<Seat> for Hands {
    type Output = Hand;

    #[inline]
    fn index(&self, index: Seat) -> &Self::Output {
        &self.hands[index as usize]
    }
}

impl std::ops::Index<usize> for Hands {
    type Output = Hand;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.hands[index]
    }
}

impl Hands {
    #[inline]
    pub fn iter_mut(&'_ mut self) -> std::slice::IterMut<'_, Hand> {
        self.hands.iter_mut()
    }
}

impl<'a> IntoIterator for &'a mut Hands {
    type IntoIter = std::slice::IterMut<'a, Hand>;
    type Item = &'a mut Hand;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl Hands {
    /// Creates a new `Hands` from an array of four hands.
    ///
    /// # Parameters
    ///
    /// - `hands`: Array of 4 hands in order [North, East, South, West]
    #[inline]
    pub(crate) fn new_from(hands: [Hand; 4]) -> Self {
        Self { hands }
    }

    /// Returns a reference to the internal array of hands.
    ///
    /// Hands are in order: [North, East, South, West]
    #[must_use]
    #[inline]
    pub fn hands(&self) -> &[Hand; 4] {
        &self.hands
    }

    /// Returns a reference to North's hand.
    #[must_use]
    #[inline]
    pub fn north(&self) -> Hand {
        self.hands[Seat::North as usize]
    }

    /// Returns a reference to South's hand.
    #[must_use]
    #[inline]
    pub fn south(&self) -> Hand {
        self.hands[Seat::South as usize]
    }

    /// Returns a reference to East's hand.
    #[must_use]
    #[inline]
    pub fn east(&self) -> Hand {
        self.hands[Seat::East as usize]
    }

    /// Returns a reference to West's hand.
    #[must_use]
    #[inline]
    pub fn west(&self) -> Hand {
        self.hands[Seat::West as usize]
    }

    /// Returns an iterator over the four hands.
    ///
    /// Iterates in order: North, East, South, West
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Hand> {
        self.hands.iter()
    }
}

impl<'a> IntoIterator for &'a Hands {
    type IntoIter = std::slice::Iter<'a, Hand>;
    type Item = &'a Hand;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
    type Error = DealerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'N' | 'n' => Ok(Self::North),
            'S' | 's' => Ok(Self::South),
            'W' | 'w' => Ok(Self::West),
            'E' | 'e' => Ok(Self::East),
            _ => Err(DealerError::new(format!("'{value}' is not a valid seat"))),
        }
    }
}

#[cfg(feature = "dds")]
impl From<Seat> for dds::deal::DdsHandEncoding {
    fn from(value: Seat) -> Self {
        match value {
            Seat::North => Self::North,
            Seat::East => Self::East,
            Seat::South => Self::South,
            Seat::West => Self::West,
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
    /// assert_eq!(Seat::North.next(), Seat::East);
    /// assert_eq!(Seat::West.next(), Seat::North);
    /// ```
    #[must_use]
    #[inline]
    pub fn next(self) -> Seat {
        self + 1
    }

    /// Returns the previous seat in counter-clockwise rotation.
    ///
    /// Order: North ← West ← South ← East ← North (wraps around)
    ///
    /// # Example
    ///
    /// ```rust
    /// assert_eq!(Seat::East.previous(), Seat::North);
    /// assert_eq!(Seat::North.previous(), Seat::West);
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
    /// assert!(Seat::North.is_same_line(&Seat::South));  // Partners
    /// assert!(Seat::East.is_same_line(&Seat::West));    // Partners
    /// assert!(!Seat::North.is_same_line(&Seat::East));  // Opponents
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
    /// assert_eq!(Seat::North.long_str(), "North");
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
    /// let mut iter = Seat::North.iter_from();
    /// assert_eq!(iter.next(), Some(Seat::East));
    /// assert_eq!(iter.next(), Some(Seat::South));
    /// // Continues forever...
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
/// let seats: Vec<_> = Seat::North.iter_from().take(5).collect();
/// assert_eq!(seats, vec![Seat::East, Seat::South, Seat::West, Seat::North, Seat::East]);
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
    /// The first call to `.next()` will return the seat *after* the starting seat.
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
    /// Uses bitwise operation for efficiency:
    /// - NS partnership checks bit 0 (seats 0 and 2 have even parity)
    /// - EW partnership checks bit 1 (seats 1 and 3 have odd parity)
    ///
    /// # Example
    ///
    /// ```rust
    /// let vuln = Vulnerability::NS;
    /// assert_eq!(vuln.is_vulnerable(Seat::North), Vulnerable::Yes);
    /// assert_eq!(vuln.is_vulnerable(Seat::East), Vulnerable::No);
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_vulnerable(&self, seat: Seat) -> Vulnerable {
        // Seat parity determines partnership: even (0,2) = NS, odd (1,3) = EW
        let seat_position = (seat as usize) % 2;

        // Check if the corresponding bit is set in the vulnerability value
        if ((*self as usize) & (1 << seat_position)) != 0 {
            Vulnerable::Yes
        } else {
            Vulnerable::No
        }
    }

    /// Returns the vulnerability state for a given board number.
    ///
    /// Uses the standard duplicate bridge 16-board rotation pattern.
    /// Board numbers wrap: board 17 = board 1, board 18 = board 2, etc.
    ///
    /// # Example
    ///
    /// ```rust
    /// assert_eq!(Vulnerability::from_number(1), Vulnerability::None);
    /// assert_eq!(Vulnerability::from_number(2), Vulnerability::NS);
    /// assert_eq!(Vulnerability::from_number(17), Vulnerability::None); // Wraps
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

/// Builder for creating a `Dealer` with specific constraints.
///
/// This is the primary way to create a dealer. It provides a fluent API for
/// configuring all aspects of deal generation: predealing, hand constraints,
/// acceptance functions, and vulnerability.
///
/// # Builder Pattern
///
/// Methods can be chained and the builder is consumed by `.build()`:
///
/// ```rust
/// let dealer = DealerBuilder::new()
///     .predeal(Seat::North, cards)?
///     .with_hand_descriptor(Seat::South, descriptor)
///     .with_function(|hands| /* validation */)
///     .with_vulnerability(Vulnerability::All)
///     .build()?;
/// ```
///
/// # Performance Considerations
///
/// - **Predealing**: Very fast - just removes cards from the available deck
/// - **Hand constraints**: Speed depends on constraint rarity (tight constraints = slower)
/// - **Accept functions**: Called for every candidate deal, so keep them fast
///
/// # Example: Complex Dealer
///
/// ```rust
/// let dealer = DealerBuilder::new()
///     .predeal(Seat::North, Cards::from_str("SAK")?)? // North has A-K of spades
///     .with_hand_descriptor(
///         Seat::South,
///         HandDescriptor::new()
///             .hcp(12..=14)
///             .shape(Shape::from_str("5332")?)
///     )
///     .with_function(|hands| {
///         // NS must have 8+ spade fit
///         hands.north().spades().len() + hands.south().spades().len() >= 8
///     })
///     .build()?;
/// ```
pub struct DealerBuilder {
    /// Function that validates the complete deal across all four hands
    accept: AcceptFunction,

    /// Available cards for random dealing (cards not predealed)
    deck: Cards,

    /// How board numbers should be assigned to generated deals
    numbering: Option<BoardNumbering>,

    /// Per-seat constraints (shape, HCP, controls, etc.)
    hand_descriptors: [Option<HandDescriptor>; NUMBER_OF_HANDS],

    /// Cards to predeal to specific seats (can be partial hands)
    predealt_hands: [Option<Cards>; NUMBER_OF_HANDS],

    /// Vulnerability state for generated deals
    vulnerability: Vulnerability,
}

impl std::fmt::Debug for DealerBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DealerBuilder")
            .field("hand_descriptors", &self.hand_descriptors)
            .field("predealt_hands", &self.predealt_hands)
            .field("vulnerability", &self.vulnerability)
            .finish_non_exhaustive()
    }
}

impl Default for DealerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DealerBuilder {
    /// Creates a new builder with no constraints.
    ///
    /// The default configuration accepts all deals, has no predealing, uses
    /// sequential board numbering, and has no vulnerability.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            accept: Box::new(|_: &Hands| true),
            hand_descriptors: [None, None, None, None],
            predealt_hands: [None; 4],
            vulnerability: Vulnerability::default(),
            deck: Cards::ALL,
            numbering: None,
        }
    }

    /// Predeals specific cards to a seat.
    ///
    /// Cards can be a partial hand (< 13 cards). The remaining cards will be
    /// dealt randomly from the available deck during generation.
    ///
    /// **Note**: This overwrites any previous predeal for this seat.
    ///
    /// # Errors
    ///
    /// Returns an error if any card in `hand` has already been dealt to another seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = DealerBuilder::new()
    ///     .predeal(Seat::North, Cards::from_str("SAKQJ")?)?  // 5 spades
    ///     .predeal(Seat::South, Cards::from_str("HAK")?)?;   // 2 hearts
    /// ```
    #[inline]
    pub fn predeal(mut self, seat: Seat, hand: Cards) -> Result<Self, DealerError> {
        // Check for duplicate cards
        if !hand.difference(self.deck).is_empty() {
            return Err(DealerError::new(format!(
                "Cards already dealt: {}",
                hand.difference(self.deck)
            )));
        }

        // If this seat was already predealed, return those cards to the deck
        if let Some(cards_already_dealt) = self.predealt_hands[seat as usize] {
            self.deck += cards_already_dealt;
        }

        self.predealt_hands[seat as usize] = Some(hand);
        self.deck -= hand;
        Ok(self)
    }

    /// Predeals cards to all four seats at once.
    ///
    /// Each element can be `None` (no predeal for that seat) or `Some(cards)`.
    /// This is more efficient than calling `predeal()` four times separately.
    ///
    /// **Note**: This overwrites all previous predeals.
    ///
    /// # Errors
    ///
    /// Returns an error if any card appears in multiple hands.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = DealerBuilder::new()
    ///     .predeal_all([
    ///         Some(Cards::from_str("SAKQJ")?),  // North
    ///         None,                              // East - no predeal
    ///         Some(Cards::from_str("HAK")?),     // South
    ///         None,                              // West - no predeal
    ///     ])?;
    /// ```
    #[inline]
    pub fn predeal_all(mut self, hands: [Option<Cards>; 4]) -> Result<Self, DealerError> {
        // Start with a full deck, removing previously predealed cards
        let mut tmp_deck = self.deck;
        for already_dealt_hand in self.predealt_hands.iter().flatten() {
            tmp_deck += *already_dealt_hand;
        }

        // Validate no duplicates and remove cards from deck
        for hand in hands.iter().flatten() {
            if hand.difference(tmp_deck).is_empty() {
                tmp_deck -= *hand;
            } else {
                return Err(DealerError::new(format!(
                    "Card dealt twice: {}",
                    hand.difference(tmp_deck)
                )));
            }
        }

        self.predealt_hands = hands;
        self.deck = tmp_deck;
        Ok(self)
    }

    /// Adds a single card to an existing predeal.
    ///
    /// This is useful for incremental updates, such as when analyzing a hand
    /// and learning about card locations one at a time.
    ///
    /// # Errors
    ///
    /// Returns an error if the card has already been dealt to any seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut builder = DealerBuilder::new();
    /// builder.add_predealt_card(Card::new(Rank::Ace, Suit::Spades), Seat::North)?;
    /// builder.add_predealt_card(Card::new(Rank::King, Suit::Spades), Seat::North)?;
    /// ```
    #[inline]
    pub fn add_predealt_card(&mut self, card: Card, seat: Seat) -> Result<(), DealerError> {
        let prev_deck = self.deck;
        let to_insert = card.into();
        self.deck -= to_insert;

        if prev_deck == self.deck {
            Err(DealerError::new("Card already dealt"))
        } else {
            *self.predealt_hands[seat as usize].get_or_insert(Cards::EMPTY) += to_insert;
            Ok(())
        }
    }

    /// Sets a custom acceptance function for cross-hand validation.
    ///
    /// This function is called after hand constraints are checked. Use it for
    /// constraints that involve multiple hands, such as:
    /// - Partnership fits (e.g., "NS must have 8+ spade fit")
    /// - Combined HCP (e.g., "NS must have 25+ combined HCP")
    /// - Distributional requirements across hands
    ///
    /// **Performance note**: This function is called for every candidate deal,
    /// so keep it as fast as possible. Heavy computations here will slow down
    /// deal generation significantly.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = DealerBuilder::new()
    ///     .with_function(|hands: &Hands| {
    ///         // Require NS to have game-forcing values
    ///         let ns_hcp = hands.north().hcp() + hands.south().hcp();
    ///         ns_hcp >= 25
    ///     });
    /// ```
    #[inline]
    #[must_use]
    pub fn with_function<T: Fn(&Hands) -> bool + Send + 'static>(
        mut self,
        accept_function: T,
    ) -> Self {
        self.accept = Box::new(accept_function);
        self
    }

    /// Sets hand constraints for a specific seat.
    ///
    /// Hand descriptors specify requirements like shape, HCP range, controls,
    /// suit lengths, and other per-hand properties.
    ///
    /// **Note**: This overwrites any previous descriptor for this seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = DealerBuilder::new()
    ///     .with_hand_descriptor(
    ///         Seat::North,
    ///         HandDescriptor::new()
    ///             .hcp(12..=17)
    ///             .shape(Shape::from_str("5332")?)
    ///             .spades(5..=7)
    ///     );
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hand_descriptor(mut self, seat: Seat, hand_description: HandDescriptor) -> Self {
        self.hand_descriptors[seat as usize] = Some(hand_description);
        self
    }

    /// Sets hand constraints for all four seats at once.
    ///
    /// Each element can be `None` (no constraints) or `Some(descriptor)`.
    ///
    /// **Note**: This overwrites all previous hand descriptors.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = DealerBuilder::new()
    ///     .with_hand_descriptors_all([
    ///         Some(descriptor_north),
    ///         None,  // East - no constraints
    ///         Some(descriptor_south),
    ///         None,  // West - no constraints
    ///     ]);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hand_descriptors_all(
        mut self,
        hand_descriptors: [Option<HandDescriptor>; 4],
    ) -> Self {
        self.hand_descriptors = hand_descriptors;
        self
    }

    /// Sets the vulnerability for all generated deals.
    ///
    /// By default, deals have no vulnerability (`Vulnerability::None`).
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = DealerBuilder::new()
    ///     .with_vulnerability(Vulnerability::All);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_vulnerability(mut self, vulnerability: Vulnerability) -> Self {
        self.vulnerability = vulnerability;
        self
    }

    /// Builds the dealer from this configuration.
    ///
    /// Consumes the builder and returns a `StandardDealer` ready to generate deals.
    ///
    /// # Errors
    ///
    /// Currently doesn't return errors, but the return type allows for future
    /// validation during the build phase.
    ///
    /// # Example
    ///
    /// ```rust
    /// let dealer = DealerBuilder::new()
    ///     .predeal(Seat::North, cards)?
    ///     .build()?;
    ///
    /// let deal = dealer.deal()?;
    /// ```
    #[inline]
    pub fn build(self) -> Result<impl Dealer, DealerError> {
        Ok(StandardDealer {
            predeal: self.predealt_hands,
            vulnerability: self.vulnerability,
            deck_starting_state: self.deck,
            accept_function: self.accept,
            hand_constraints: self.hand_descriptors,
            output_as_subsequent: self.numbering.unwrap_or_default(),
        })
    }

    /// Sets both predealing and hand descriptors for all seats at once.
    ///
    /// This is a convenience method combining `predeal_all()` and
    /// `with_hand_descriptors_all()`.
    ///
    /// # Errors
    ///
    /// Returns an error if any cards are duplicated across hands.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = [
    ///     (Some(cards_north), Some(descriptor_north)),
    ///     (None, None),  // East - no constraints
    ///     (Some(cards_south), Some(descriptor_south)),
    ///     (None, None),  // West - no constraints
    /// ];
    /// let builder = DealerBuilder::new().with_complete_config(config)?;
    /// ```
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn with_complete_config(
        self,
        config: [(Option<Cards>, Option<HandDescriptor>); 4],
    ) -> Result<Self, DealerError> {
        let (cards, descriptors): (Vec<_>, Vec<_>) = config.into_iter().unzip();
        self.with_hand_descriptors_all(descriptors.try_into().unwrap())
            .predeal_all(cards.try_into().unwrap())
    }

    /// Disables sequential board numbering.
    ///
    /// All generated deals will be numbered as board 1. Useful when you don't
    /// care about board numbers or want to number deals externally.
    #[inline]
    #[must_use]
    pub fn no_board_numbering(mut self) -> Self {
        self.numbering = Some(BoardNumbering::OutputAlwaysOne);
        self
    }
}

/// Trait for objects that can generate bridge deals.
///
/// This trait abstracts the deal generation process. The main implementation
/// is `StandardDealer`, but you could implement custom dealers with different
/// algorithms or constraints.
pub trait Dealer {
    /// Generates a single bridge deal satisfying all constraints.
    ///
    /// # Errors
    ///
    /// Returns `DealerError` if:
    /// - The deck doesn't have enough cards (invalid predeal configuration)
    /// - Constraints are impossible to satisfy (extremely rare with proper config)
    ///
    /// # Performance
    ///
    /// Generation time varies based on constraint tightness. Tighter constraints
    /// require more candidate deals to be generated and rejected.
    fn deal(&self) -> Result<Deal, DealerError>;

    /// Returns the predealed cards for each seat.
    ///
    /// Used internally and for inspection/debugging.
    fn predealt(&self) -> [Option<Cards>; NUMBER_OF_HANDS];
}

/// Controls how board numbers are assigned to generated deals.
///
/// Board numbers are important in duplicate bridge for determining dealer
/// and vulnerability, though this dealer allows overriding those.
#[derive(Debug, Clone)]
pub enum BoardNumbering {
    /// Increment board number with each deal (1, 2, 3, ..., 127, 1, ...)
    Sequential(Cell<NonZeroU8>),
    /// Always output board number 1
    OutputAlwaysOne,
}

impl Default for BoardNumbering {
    fn default() -> Self {
        Self::Sequential(Cell::new(NonZeroU8::new(1).unwrap()))
    }
}

/// Standard implementation of the `Dealer` trait.
///
/// This dealer uses rejection sampling: it generates random deals and accepts
/// only those that satisfy all constraints. The algorithm:
///
/// 1. Fill predealed cards for each seat
/// 2. Randomly deal remaining cards from the available deck
/// 3. Check hand constraints for each seat
/// 4. Check the acceptance function
/// 5. If all checks pass, return the deal; otherwise, try again
///
/// # Performance Characteristics
///
/// - **No constraints**: ~1-5 microseconds per deal
/// - **Light constraints**: ~10-50 microseconds
/// - **Tight constraints**: Can be much slower (milliseconds)
///
/// Very tight constraints (e.g., "exactly 6-5-2-0 with 15 HCP") may require
/// thousands of attempts and can be slow.
///
/// # Memory
///
/// The dealer is lightweight (~200 bytes) and cheap to create. The expensive
/// part is deal generation, not dealer creation.
pub struct StandardDealer {
    /// Cards to predeal to each seat
    predeal: [Option<Cards>; NUMBER_OF_HANDS],
    /// Vulnerability for generated deals
    vulnerability: Vulnerability,
    /// Available cards for random dealing
    deck_starting_state: Cards,
    /// Per-seat constraints
    hand_constraints: [Option<HandDescriptor>; NUMBER_OF_HANDS],
    /// Cross-hand validation function
    accept_function: AcceptFunction,
    /// Board numbering scheme
    output_as_subsequent: BoardNumbering,
}

impl std::fmt::Debug for StandardDealer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StandardDealer")
            .field("predeal", &self.predeal)
            .field("vulnerability", &self.vulnerability)
            .field("hand_constraints", &self.hand_constraints)
            .finish_non_exhaustive()
    }
}

impl StandardDealer {
    /// Creates a new dealer with no constraints.
    ///
    /// Prefer using `DealerBuilder` for a more ergonomic API.
    #[must_use]
    pub fn new() -> Self {
        Self {
            predeal: [None; 4],
            vulnerability: Vulnerability::default(),
            deck_starting_state: Cards::ALL,
            hand_constraints: [None, None, None, None],
            accept_function: Box::new(|_: &Hands| true),
            output_as_subsequent: BoardNumbering::default(),
        }
    }
}

impl Default for StandardDealer {
    fn default() -> Self {
        Self::new()
    }
}

impl Dealer for StandardDealer {
    #[inline]
    fn predealt(&self) -> [Option<Cards>; NUMBER_OF_HANDS] {
        self.predeal
    }

    /// Generates a deal using rejection sampling.
    ///
    /// This implementation uses a do-while pattern to ensure at least one
    /// candidate is generated before checking constraints.
    ///
    /// # Algorithm
    ///
    /// ```text
    /// loop {
    ///     1. Start with available deck
    ///     2. For each seat:
    ///        - Add predealed cards
    ///        - Fill remainder from deck
    ///     3. Check hand constraints
    ///     4. Check acceptance function
    ///     5. If all pass: return deal
    ///     6. Otherwise: try again
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the deck doesn't have enough cards to complete
    /// a hand (indicates invalid predeal configuration).
    #[inline]
    fn deal(&self) -> Result<Deal, DealerError> {
        let mut hands: [Hand; 4] = [Hand::default(); 4];

        // Do-while loop: generate at least once, then check constraints
        while {
            let mut deck = self.deck_starting_state;

            // Deal to each seat
            for seat in Seat::iter() {
                if let Some(&Some(cards)) = self.predeal.get(seat as usize) {
                    let predeal_len = cards.len();

                    if predeal_len < 13 {
                        // Partial predeal: fill remaining cards from deck
                        let Some(cards_to_add) = deck.pick(13 - predeal_len as usize) else {
                            return Err(DealerError::new(
                                "Deck doesn't contain enough cards. Check predeal configuration.",
                            ));
                        };
                        hands[seat as usize].set_cards(cards + cards_to_add);
                    } else {
                        // Full predeal: use exactly the predealed cards
                        hands[seat as usize].set_cards(cards);
                    }
                } else {
                    // No predeal: deal 13 random cards
                    hands[seat as usize] = if let Some(cards) = deck.pick(13) {
                        Hand { cards }
                    } else {
                        return Err(DealerError::new(
                            "Deck doesn't contain enough cards. Check predeal configuration.",
                        ));
                    };
                }
            }

            let hands = Hands { hands };

            // Continue loop if constraints NOT satisfied (negated condition)
            !((self.accept_function)(&hands) && self.constraints_respected(hands.hands()))
        } {}

        // Constraints satisfied - create the deal
        Ok(Deal {
            hands,
            number: match self.output_as_subsequent {
                BoardNumbering::Sequential(ref num) => {
                    let actual = num.get().get();
                    // Increment for next time, wrapping at 127
                    num.set(match actual {
                        1..=127 => unsafe { NonZeroU8::new_unchecked(actual + 1) },
                        _ => unsafe { NonZeroU8::new_unchecked(1) },
                    });
                    actual
                }
                BoardNumbering::OutputAlwaysOne => 1,
            },
            vulnerability: self.vulnerability,
            ..Default::default()
        })
    }
}

impl StandardDealer {
    /// Checks if all per-seat hand constraints are satisfied.
    ///
    /// Returns `true` only if every seat with a descriptor passes validation.
    /// Seats without descriptors automatically pass.
    ///
    /// # Performance
    ///
    /// This is called for every candidate deal, so it's optimized with
    /// early exit (`all()` short-circuits on first failure).
    #[inline]
    fn constraints_respected(&self, hands: &[Hand; NUMBER_OF_HANDS]) -> bool {
        self.hand_constraints
            .iter()
            .enumerate()
            .all(|(seat, hand_constraint)| {
                if let Some(hand_constraint) = hand_constraint {
                    hand_constraint.check(hands[seat])
                } else {
                    true
                }
            })
    }
}

/// Output format for displaying deals.
///
/// Different formats are used for different purposes:
/// - **Short**: Human-readable table view
/// - **Long**: Verbose human-readable format
/// - **PBN**: Portable Bridge Notation (standard interchange format)
/// - **LIN**: Format used by BBO and other bridge software
#[derive(Default, Debug, Copy, Clone)]
#[non_exhaustive]
pub enum Printer {
    /// Portable Bridge Notation - standard interchange format
    Pbn,
    /// LIN format used by Bridge Base Online
    Lin,
    /// Compact table view (default)
    #[default]
    Short,
    /// Verbose human-readable format
    Long,
}

/// A complete bridge deal with cards, vulnerability, and metadata.
///
/// This is the primary output of a `Dealer`. It contains:
/// - Four 13-card hands (one per seat)
/// - Vulnerability state
/// - Board number
/// - Display format preference
///
/// # Display Formats
///
/// The deal can be displayed in multiple formats via the `Printer` enum:
///
/// ```rust
/// let mut deal = dealer.deal()?;
/// deal.short();  // Compact table
/// println!("{}", deal);
///
/// deal.pbn();    // Portable Bridge Notation
/// println!("{}", deal);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Deal {
    /// Vulnerability state for this deal
    vulnerability: Vulnerability,
    /// The four hands [North, East, South, West]
    hands: [Hand; NUMBER_OF_HANDS],
    /// Display format preference (not serialized)
    #[cfg_attr(feature = "serde", serde(skip))]
    printer: Printer,
    /// Board number (1-255)
    number: u8,
}

#[cfg(feature = "lin")]
impl From<LinDeal> for Deal {
    fn from(lin_deal: LinDeal) -> Deal {
        Deal {
            hands: lin_deal.hands().hands,
            number: lin_deal.number(),
            vulnerability: Vulnerability::from_number(lin_deal.number()),
            ..Default::default()
        }
    }
}

#[cfg(feature = "dds")]
impl dds::deal::AsDDSDeal for Deal {
    fn as_dds_deal(&self) -> dds::deal::DDSDealRepr {
        self.hands().as_dds_deal()
    }
}

#[cfg(feature = "dds")]
impl dds::deal::AsDDSDeal for Hands {
    fn as_dds_deal(&self) -> dds::deal::DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, hand) in self.into_iter().enumerate() {
            for (index, suit) in hand.into_iter().enumerate() {
                remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
            }
        }
        remain_cards.into()
    }
}

impl Default for Deal {
    fn default() -> Self {
        Deal::new()
    }
}

impl Deal {
    /// Creates a new random deal with no constraints.
    ///
    /// This is a convenience method for quick random deal generation.
    /// For constrained deals, use `DealerBuilder`.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            vulnerability: Vulnerability::None,
            hands: Self::deal(),
            printer: Printer::Short,
            number: 1,
        }
    }

    /// Generates four random 13-card hands from a full deck.
    ///
    /// This is a low-level method. Prefer using `Deal::new()` or `Dealer::deal()`.
    ///
    /// # Panics
    ///
    /// Panics if the random card picking fails (should never happen with a full deck).
    #[must_use]
    #[inline]
    pub fn deal() -> [Hand; NUMBER_OF_HANDS] {
        let mut deck = Cards::ALL;
        let north = Hand {
            cards: deck.pick(13).unwrap(),
        };
        let east = Hand {
            cards: deck.pick(13).unwrap(),
        };
        let south = Hand {
            cards: deck.pick(13).unwrap(),
        };
        let west = Hand { cards: deck };
        [north, east, south, west]
    }

    /// Applies a validation function to this deal.
    ///
    /// # Example
    ///
    /// ```rust
    /// if deal.check(|d| d.north().hcp() >= 12) {
    ///     println!("North has opening hand");
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub fn check<T: Fn(&Deal) -> bool>(&self, f: T) -> bool {
        f(self)
    }

    /// Sets the vulnerability for this deal.
    #[inline]
    pub fn set_vuln(&mut self, vuln: Vulnerability) {
        self.vulnerability = vuln;
    }

    /// Returns West's hand.
    #[must_use]
    #[inline]
    pub fn west(&self) -> Hand {
        self.hands[Seat::West as usize]
    }

    /// Returns North's hand.
    #[must_use]
    #[inline]
    pub fn north(&self) -> Hand {
        self.hands[Seat::North as usize]
    }

    /// Returns East's hand.
    #[must_use]
    #[inline]
    pub fn east(&self) -> Hand {
        self.hands[Seat::East as usize]
    }

    /// Returns South's hand.
    #[must_use]
    #[inline]
    pub fn south(&self) -> Hand {
        self.hands[Seat::South as usize]
    }

    /// Sets the display format for this deal.
    #[inline]
    pub fn set_print_style(&mut self, style: Printer) {
        self.printer = style;
    }

    /// Switches to long (verbose) display format.
    #[inline]
    pub fn long(&mut self) {
        self.set_print_style(Printer::Long);
    }

    /// Switches to PBN (Portable Bridge Notation) format.
    #[inline]
    pub fn pbn(&mut self) {
        self.set_print_style(Printer::Pbn);
    }

    /// Switches to short (compact table) display format.
    #[inline]
    pub fn short(&mut self) {
        self.set_print_style(Printer::Short);
    }

    /// Switches to LIN display format.
    #[inline]
    pub fn lin(&mut self) {
        self.set_print_style(Printer::Lin);
    }

    /// Returns the deal as a string in the current display format.
    #[must_use]
    #[inline]
    pub fn as_string(&self) -> String {
        match self.printer {
            Printer::Pbn => self.as_pbn(),
            Printer::Lin => self.as_lin(self.number),
            Printer::Short => self.as_short(),
            Printer::Long => self.as_long(),
        }
    }

    /// Formats the deal in Portable Bridge Notation (PBN).
    ///
    /// PBN is the standard interchange format for bridge deals.
    ///
    /// # Format
    ///
    /// ```text
    /// [Board "1"]
    /// [Deal "N:SAKQ.HAK.DAK.CAK S743.HT9.D8.CJT9 ..."]
    /// ```
    #[must_use]
    #[inline]
    pub fn as_pbn(&self) -> String {
        let mut pbn = format!("[Board \"{}\"]\n[Deal \"N:", self.number);
        pbn = format!(
            "{}",
            format_args!(
                "{}{}\"]",
                pbn,
                self.into_iter()
                    .map(|hand| {
                        hand.into_iter()
                            .map(|holding| holding.into_iter().map(Card::rankchar).rev().format(""))
                            .format(".")
                            .to_string()
                    })
                    .format(" ")
            )
        );
        pbn
    }

    /// Returns the four hands as a `Hands` struct.
    #[must_use]
    #[inline]
    pub fn hands(&self) -> Hands {
        Hands { hands: self.hands }
    }

    /// Formats the deal in LIN format (used by Bridge Base Online).
    ///
    /// # Parameters
    ///
    /// - `board_n`: Board number (automatically wraps beyond 128)
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn as_lin(&self, board_n: u8) -> String {
        let board_n = if board_n.is_multiple_of(MAX_N_OF_BOARDS + 1) {
            1
        } else {
            board_n % (MAX_N_OF_BOARDS + 1)
        };

        let mut stringa = format!(
            "st||md|{}",
            (((board_n % NUMBER_OF_HANDS as u8) + 1) % NUMBER_OF_HANDS as u8) + 1
        );

        for (position, hand) in self.into_iter().enumerate() {
            if position != 0 {
                stringa.push(',');
            }
            for (index, holding) in hand.into_iter().enumerate() {
                stringa.push(match index {
                    0 => 'S',
                    1 => 'H',
                    2 => 'D',
                    3 => 'C',
                    _ => unreachable!(),
                });
                stringa = format!(
                    "{}",
                    format_args!(
                        "{}{}",
                        stringa,
                        holding.into_iter().map(Card::rankchar).rev().format("")
                    )
                );
            }
        }

        let data1 = (board_n - 1) / NUMBER_OF_HANDS as u8;
        let data2 = (board_n - 1) % NUMBER_OF_HANDS as u8;
        let data3 = match (data1 + data2) % NUMBER_OF_HANDS as u8 {
            0 => "o",
            1 => "n",
            2 => "e",
            3 => "b",
            _ => unreachable!(),
        };
        format!("{stringa}|sv|{data3}|rh||ah|Board {board_n}|")
    }

    /// Formats the deal in short (compact table) format.
    ///
    /// This is the default human-readable format showing all four hands
    /// arranged around a table.
    fn as_short(&self) -> String {
        let west = self.hands[Seat::West as usize].to_string();
        let east = self.hands[Seat::East as usize].to_string();
        let north = self.hands[Seat::North as usize].to_string();
        let south = self.hands[Seat::South as usize].to_string();
        let width = west.chars().count() + east.chars().count() + north.chars().count() / 2;
        format!(
            "{north:^0$}\n{west:<1$}{east:>1$}\n{south:^0$}",
            width + 1,
            width / 2,
        )
    }

    /// Formats the deal in long (verbose) format with suit symbols.
    fn as_long(&self) -> String {
        let west_len = self.extract_long_strlen(Seat::West);
        let east_len = self.extract_long_strlen(Seat::East);
        let north_len = self.extract_long_strlen(Seat::North);
        let south_len = self.extract_long_strlen(Seat::South);
        let ns_len = if south_len < north_len {
            north_len
        } else {
            south_len
        };
        let width = west_len + east_len + ns_len - 2;

        let mut stringa = String::with_capacity(210);
        for line in self.hands[Seat::North as usize].long_str().split('\n') {
            stringa = format!("{stringa}{line:^0$}\n", width - 2 - ns_len + line.len());
        }
        for (line_w, line_e) in self.hands[Seat::West as usize]
            .long_str()
            .split('\n')
            .zip(self.hands[Seat::East as usize].long_str().split('\n'))
        {
            stringa = format!(
                "{stringa}{line_w:<0$}{line_e:<1$}\n",
                width - east_len,
                east_len
            );
        }
        stringa = format!(
            "{stringa}{}",
            self.hands[Seat::South as usize]
                .long_str()
                .split('\n')
                .map(|string| format!("{string:^0$}", width - ns_len - 2 + string.len()))
                .format("\n"),
        );
        stringa
    }

    /// Helper function to calculate display width for long format.
    fn extract_long_strlen(&self, seat: Seat) -> usize {
        self.hands[seat as usize]
            .long_str()
            .split('\n')
            .map(str::len)
            .max()
            .unwrap()
    }

    /// Returns an iterator over the four hands.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Hand> {
        self.hands.iter()
    }
}

impl<'a> IntoIterator for &'a Deal {
    type IntoIter = std::slice::Iter<'a, Hand>;
    type Item = &'a Hand;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::ops::Index<Suit> for Deal {
    type Output = Hand;

    #[inline]
    fn index(&self, index: Suit) -> &Self::Output {
        &self.hands[index as usize]
    }
}

impl std::ops::Index<usize> for Deal {
    type Output = Hand;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.hands[index]
    }
}

impl fmt::Display for Deal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl IntoIterator for Deal {
    type Item = Hand;
    type IntoIter = std::array::IntoIter<Hand, NUMBER_OF_HANDS>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.hands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that basic deal generation works without panicking.
    #[test]
    fn test_basic_deal_generation() {
        let dealer = DealerBuilder::new().build().unwrap();
        let deal = dealer.deal().unwrap();

        // All hands should have exactly 13 cards
        for hand in deal {
            assert_eq!(hand.cards.len(), 13);
        }
    }

    /// Tests predealing functionality.
    #[test]
    fn test_predeal() {
        let cards = Cards::from_str("SAKQJ").unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, cards)
            .unwrap()
            .build()
            .unwrap();

        let deal = dealer.deal().unwrap();

        // North should have all predealed cards
        assert!(deal.north().cards().contains(cards));
    }

    /// Tests that predealing the same card twice fails.
    #[test]
    fn test_duplicate_predeal_fails() {
        let cards1 = Cards::from_str("SAK").unwrap();
        let cards2 = Cards::from_str("SAQ").unwrap(); // SA is duplicated

        let result = DealerBuilder::new()
            .predeal(Seat::North, cards1)
            .unwrap()
            .predeal(Seat::South, cards2);

        assert!(result.is_err());
    }

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

    /// Tests board numbering.
    #[test]
    fn test_sequential_numbering() {
        let dealer = DealerBuilder::new().build().unwrap();

        let deal1 = dealer.deal().unwrap();
        let deal2 = dealer.deal().unwrap();
        let deal3 = dealer.deal().unwrap();

        assert_eq!(deal1.number, 1);
        assert_eq!(deal2.number, 2);
        assert_eq!(deal3.number, 3);
    }

    /// Tests that acceptance functions work correctly.
    #[test]
    fn test_accept_function() {
        let dealer = DealerBuilder::new()
            .with_function(|hands: &Hands| {
                // Only accept if North has at least 20 HCP
                hands.north().hcp() >= 20
            })
            .build()
            .unwrap();

        let deal = dealer.deal().unwrap();
        assert!(deal.north().hcp() >= 20);
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

    /// Tests PBN format output.
    #[test]
    fn test_pbn_format() {
        let deal = Deal::new();
        let pbn = deal.as_pbn();

        assert!(pbn.contains("[Board"));
        assert!(pbn.contains("[Deal"));
    }

    /// Tests that hands are correctly indexed by seat.
    #[test]
    fn test_hands_indexing() {
        let dealer = DealerBuilder::new().build().unwrap();
        let deal = dealer.deal().unwrap();
        let hands = deal.hands();

        assert_eq!(hands[Seat::North], hands.north());
        assert_eq!(hands[Seat::East], hands.east());
        assert_eq!(hands[Seat::South], hands.south());
        assert_eq!(hands[Seat::West], hands.west());

        assert_eq!(hands[0], hands.north());
        assert_eq!(hands[1], hands.east());
    }

    /// Tests adding predealt cards incrementally.
    #[test]
    fn test_incremental_predeal() {
        let mut builder = DealerBuilder::new();

        builder
            .add_predealt_card(Card::new(Suit::Spades, 14), Seat::North)
            .unwrap();

        builder
            .add_predealt_card(Card::new(Suit::Spades, 13), Seat::North)
            .unwrap();

        let dealer = builder.build().unwrap();
        let deal = dealer.deal().unwrap();

        assert!(deal.north().has_card(Card::new(Suit::Spades, 14)));
        assert!(deal.north().has_card(Card::new(Suit::Spades, 13)));
    }

    /// Tests that adding duplicate card fails.
    #[test]
    fn test_duplicate_incremental_predeal_fails() {
        let mut builder = DealerBuilder::new();

        builder
            .add_predealt_card(Card::new(Suit::Spades, 14), Seat::North)
            .unwrap();

        let result = builder.add_predealt_card(Card::new(Suit::Spades, 14), Seat::South);

        assert!(result.is_err());
    }

    /// Tests complete configuration.
    #[test]
    fn test_complete_config() {
        let cards_north = Cards::from_str("SAK").unwrap();
        let cards_south = Cards::from_str("HAK").unwrap();

        let config = [
            (Some(cards_north), None),
            (None, None),
            (Some(cards_south), None),
            (None, None),
        ];

        let dealer = DealerBuilder::new()
            .with_complete_config(config)
            .unwrap()
            .build()
            .unwrap();

        let deal = dealer.deal().unwrap();
        assert!(deal.north().cards().contains(cards_north));
        assert!(deal.south().cards().contains(cards_south));
    }
    #[test]
    fn can_deal_test() {
        _ = Deal::new();
    }
    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn predealing_twice_should_panic() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, hand)
            .unwrap()
            .predeal(Seat::West, hand)
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north().cards(), hand);
    }

    #[test]
    fn dealer_builder_test() {
        let dealer_builder = DealerBuilder::new();
        _ = dealer_builder.build();
    }

    #[test]
    fn dealer_deals_test() {
        let db = DealerBuilder::new();
        let dealer = db.build().unwrap();
        _ = dealer.deal().unwrap();
    }

    #[test]
    fn dealer_deals_with_predeal_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, hand)
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north().cards(), hand);
    }
    #[test]
    fn dealer_deals_with_predeal_all_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let hands = [Some(hand), None, None, None];
        let dealer = DealerBuilder::new()
            .predeal_all(hands)
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north().cards(), hand);
    }
    #[test]
    fn dealer_deals_with_predeal_all_twice_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let hande = Cards::from_str("S432H432D432CT987").unwrap();
        let hand_2 = Cards::from_str("SAKQHAKQDAKQC6543").unwrap();
        let predeal_hands = [Some(hand), Some(hande), None, None];
        let dealer = DealerBuilder::new()
            .predeal_all(predeal_hands)
            .unwrap()
            .predeal_all([Some(hand_2), None, None, None])
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north().cards(), hand_2);
        assert_ne!(deal.east().cards(), hande);
    }

    #[test]
    #[should_panic]
    fn dealer_deals_with_predeal_all_test_panic() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let hand_p = Cards::from_str("CQ").unwrap();
        let hands = [Some(hand), Some(hand_p), None, None];
        let _ = DealerBuilder::new().predeal_all(hands).unwrap();
    }

    #[test]
    fn dealer_deals_with_predeal_and_accept_function_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, hand)
            .unwrap()
            .with_function(Box::new(|hands: &Hands| {
                hands.north().slen() + hands.south().slen() > 8
            }))
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert!(deal.north().slen() + deal.south().slen() > 8);
    }
}
