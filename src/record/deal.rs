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
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use squeezer::*;
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
//!     .with_function(|deal: &Deal| {
//!         let ns_spades = deal.north().spades().len() + deal.south().spades().len();
//!         let ns_hearts = deal.north().hearts().len() + deal.south().hearts().len();
//!         ns_spades >= 8 || ns_hearts >= 8
//!     })
//!     .with_hand_descriptor(
//!         Seat::North,
//!         HandDescriptor::hcp(12..38)
//!     )
//!     .build()?;
//! # Ok(())
//! # }
//! ```

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
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use squeezer::*;
/// let accept: AcceptFunction = Box::new(|deal: &Deal| {
///     // Accept only if North-South have game-forcing values combined
///     deal.north().hcp() + deal.south().hcp() >= 25
/// });
/// # Ok(())
/// # }
/// ```
/// Cross-hand acceptance predicate shared by every dealer.
///
/// `Send + Sync` bound: dealers are shared across simulation threads, so the
/// predicate must be too. Plain (non-capturing or plain-data-capturing)
/// closures satisfy this automatically.
pub type AcceptFunction = Box<dyn Fn(&Deal) -> bool + Send + Sync>;

/// Default cap on rejection-sampling attempts.
///
/// The dealer never samples forever: once this many candidate deals have been
/// rejected it returns [`DealerError::Unsatisfiable`]. The value is chosen so
/// the tightest constraints shipped with the crate (a joint 18-20 HCP, six-card
/// minor and spades-longest filter, averaging ~10⁵ attempts) succeed with a
/// wide margin, while a genuinely impossible spec returns in bounded time
/// instead of hanging. Override it with [`DealerBuilder::with_attempt_budget`].
pub const DEFAULT_ATTEMPT_BUDGET: u32 = 2_000_000;

/// A complete bridge deal consisting of four 13-card hands.
///
/// This is the fundamental structure representing a bridge deal. Hands are stored
/// in array order: [North, East, South, West] (indices 0-3).
///
/// A `Deal` owns *only* the four hands: board number, dealer, vulnerability and
/// display choice belong to a [`crate::record::Board`], not to the cards.
///
/// # Memory Layout
///
/// Each `Hand` is typically 8 bytes (bitboard representation), so a `Deal` is
/// 32 bytes total - very cache-friendly.
///
/// # Indexing
///
/// You can index by `Seat` or `usize`:
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use squeezer::*;
/// let deal = DealerBuilder::new().build()?.deal()?;
/// let north = deal[Seat::North];
/// let north = deal[0]; // Equivalent
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deal {
    /// Array of hands in order: [North, East, South, West]
    hands: [Hand; 4],
}

/// Why a set of four hands is not a valid bridge deal.
///
/// The only invariant a [`Deal`] carries is the 52-card partition. It is
/// checked as two conditions — each seat holds exactly 13 cards, and no card
/// appears in two hands — which together already imply the third condition,
/// that the four hands cover the whole deck: 4 × 13 = 52 pairwise-distinct
/// cards can only be the 52-card deck. There is therefore no separate
/// "missing card" failure to report.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DealError {
    /// A seat does not hold exactly 13 cards.
    WrongHandSize {
        /// The offending seat.
        seat: Seat,
        /// How many cards that seat actually holds.
        len: u8,
    },
    /// The same card appears in more than one hand.
    DuplicatedCard(Card),
}

impl std::fmt::Display for DealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongHandSize { seat, len } => {
                write!(f, "{seat} holds {len} cards, expected 13")
            }
            Self::DuplicatedCard(card) => write!(f, "card {card} appears in more than one hand"),
        }
    }
}

impl std::error::Error for DealError {}

impl Deal {
    /// Creates a deal from four hands, validating the 52-card partition.
    ///
    /// Every seat must hold exactly 13 cards and no card may appear twice; as
    /// documented on [`DealError`], those two conditions already imply the four
    /// hands cover the whole deck.
    ///
    /// # Errors
    ///
    /// Returns [`DealError`] unless every seat holds exactly 13 distinct cards.
    pub fn new(hands: [Hand; 4]) -> Result<Self, DealError> {
        for (index, hand) in hands.iter().enumerate() {
            let len = hand.cards().len();
            if len != 13 {
                return Err(DealError::WrongHandSize {
                    seat: Seat::from(index),
                    len,
                });
            }
        }

        // Fold the four hands into one bitboard, reporting the first card that
        // was already seen.
        let mut seen = Cards::EMPTY;
        for hand in &hands {
            for card in hand.cards() {
                if seen.has_card(card) {
                    return Err(DealError::DuplicatedCard(card));
                }
                seen = seen.insert(card);
            }
        }

        Ok(Self { hands })
    }

    /// Creates a deal from four hands without checking the partition.
    ///
    /// The caller takes full responsibility for the invariant: each seat must
    /// hold exactly 13 distinct cards and the four hands must cover the whole
    /// deck. Prefer [`Deal::new`] unless you are on a hot path that has already
    /// established the invariant (for example [`StandardDealer`], which deals
    /// from a shuffled deck).
    #[must_use]
    #[inline]
    pub const fn from_hands_unchecked(hands: [Hand; 4]) -> Self {
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

    /// Returns North's hand.
    #[must_use]
    #[inline]
    pub fn north(&self) -> Hand {
        self.hands[Seat::North as usize]
    }

    /// Returns South's hand.
    #[must_use]
    #[inline]
    pub fn south(&self) -> Hand {
        self.hands[Seat::South as usize]
    }

    /// Returns East's hand.
    #[must_use]
    #[inline]
    pub fn east(&self) -> Hand {
        self.hands[Seat::East as usize]
    }

    /// Returns West's hand.
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

    /// Applies a validation function to this deal.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let deal = DealerBuilder::new().build()?.deal()?;
    /// if deal.check(|d: &Deal| d.north().hcp() >= 12) {
    ///     println!("North has opening hand");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn check<T: Fn(&Deal) -> bool>(&self, f: T) -> bool {
        f(self)
    }
}

impl TryFrom<[Hand; 4]> for Deal {
    type Error = DealError;

    /// Validates the four hands via [`Deal::new`].
    ///
    /// This is the named conversion that [`Deal`]'s `Deserialize` goes
    /// through.
    fn try_from(hands: [Hand; 4]) -> Result<Self, Self::Error> {
        Self::new(hands)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Deal {
    /// Serializes the deal as its four hands, mirroring the array shape that
    /// [`Deal`]'s `Deserialize` re-validates.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&self.hands, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Deal {
    /// Deserializes four hands and re-checks the partition through
    /// [`TryFrom`], so a standalone deserialized `Deal` cannot carry
    /// corruption.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hands = <[Hand; 4]>::deserialize(deserializer)?;
        Deal::try_from(hands).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for Deal {
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

impl IntoIterator for Deal {
    type IntoIter = IntoIter<Hand, 4>;
    type Item = Hand;

    fn into_iter(self) -> Self::IntoIter {
        self.hands.into_iter()
    }
}

impl std::ops::Index<Seat> for Deal {
    type Output = Hand;

    #[inline]
    fn index(&self, index: Seat) -> &Self::Output {
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

impl<'a> IntoIterator for &'a Deal {
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
/// # Ok(())
/// # }
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use squeezer::*;
/// let dealer = DealerBuilder::new()
///     .predeal(Seat::North, Card::HQ.into())?
///     .with_hand_descriptor(Seat::South, HandDescriptor::hcp(2..15))
///     .with_function(|deal| /* validation */ true)
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Performance Considerations
///
/// - **Predealing**: Very fast - just removes cards from the available deck
/// - **Hand constraints**: Speed depends on constraint rarity (tight constraints = slower)
/// - **Accept functions**: Called only for candidates that pass every per-seat
///   constraint, so keep them fast
///
/// # Example: Complex Dealer
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use squeezer::*;
/// let dealer = DealerBuilder::new()
///     .predeal(Seat::North, Cards::from_str("SAK")?)? // North has A-K of spades
///     .with_hand_descriptor(
///         Seat::South,
///         HandDescriptor::hcp(12..14)
///     )
///     .with_function(|deal| {
///         // NS must have 8+ spade fit
///         deal.north().spades().len() + deal.south().spades().len() >= 8
///     })
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct DealerBuilder {
    /// Function that validates the complete deal across all four hands
    accept: AcceptFunction,

    /// Available cards for random dealing (cards not predealed)
    deck: Cards,

    /// Per-seat constraints (shape, HCP, controls, etc.)
    hand_constraints: [Option<HandConstraint>; NUMBER_OF_HANDS],

    /// Cards to predeal to specific seats (can be partial hands)
    predealt_hands: [Option<Cards>; NUMBER_OF_HANDS],

    /// Cards fixed to a seat by conditioning beyond `predeal`: played cards
    /// (via [`DealerBuilder::with_played`]) and cards seeded directly (via
    /// [`DealerBuilder::with_known`]). The dealer's known set is this unioned
    /// with the predealt cards, per seat.
    conditioned_by: [Cards; NUMBER_OF_HANDS],

    /// The subset of `conditioned_by` that the seat has already played. Kept
    /// separately from `conditioned_by` so a candidate check can tell a card
    /// still in hand from one that is already gone.
    played_by: [Cards; NUMBER_OF_HANDS],

    /// Bounded rejection-sampling budget.
    attempt_budget: u32,
}

impl std::fmt::Debug for DealerBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DealerBuilder")
            .field("hand_constraints", &self.hand_constraints)
            .field("predealt_hands", &self.predealt_hands)
            .field("conditioned_by", &self.conditioned_by)
            .field("played_by", &self.played_by)
            .field("attempt_budget", &self.attempt_budget)
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
    /// The default configuration accepts all deals and has no predealing.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            accept: Box::new(|_: &Deal| true),
            hand_constraints: [None, None, None, None],
            predealt_hands: [None; 4],
            conditioned_by: [Cards::EMPTY; 4],
            played_by: [Cards::EMPTY; 4],
            deck: Cards::ALL,
            attempt_budget: DEFAULT_ATTEMPT_BUDGET,
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let builder = DealerBuilder::new()
    ///     .predeal(Seat::North, Cards::from_str("SAKQJ")?)?  // 5 spades
    ///     .predeal(Seat::South, Cards::from_str("HAK")?)?;   // 2 hearts
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let builder = DealerBuilder::new()
    ///     .predeal_all([
    ///         Some(Cards::from_str("SAKQJ")?),  // North
    ///         None,                              // East - no predeal
    ///         Some(Cards::from_str("HAK")?),     // South
    ///         None,                              // West - no predeal
    ///     ])?;
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let mut builder = DealerBuilder::new();
    /// builder.add_predealt_card(Card::new(Suit::Spades,14), Seat::North)?;
    /// builder.add_predealt_card(Card::new(Suit::Spades, 13), Seat::North)?;
    /// # Ok(())
    /// # }
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

    /// Seeds every played card as a known card for the seat that played it.
    ///
    /// This is how a partially-known deal is modelled: the cards already
    /// played are fixed to their seats, and the rejection sampler fills each
    /// hand to 13 from the remaining deck. Calling it more than once unions
    /// the records; a card that ends up fixed to two different seats (played
    /// by two seats, or predealt to one and played by another) is rejected by
    /// [`DealerBuilder::build`], while a card both predealt and played by the
    /// same seat is simply one known card.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let record = PlayRecord::new(vec![
    ///     Play::new(Seat::West, Card::C2),
    ///     Play::new(Seat::North, Card::C3),
    /// ]);
    /// let dealer = DealerBuilder::new().with_played(&record).build()?;
    /// let deal = dealer.deal()?;
    /// assert!(deal.west().cards().has_card(Card::C2));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn with_played(mut self, record: &PlayRecord) -> Self {
        for play in record.plays() {
            let seat = play.seat as usize;
            self.conditioned_by[seat] = self.conditioned_by[seat].insert(play.card);
            // Record the played subset too: a played card is fixed to the
            // seat's reconstructed original hand, but is no longer playable.
            self.played_by[seat] = self.played_by[seat].insert(play.card);
        }
        self
    }

    /// Seeds cards as known for a seat directly.
    ///
    /// The escape hatch for information that is not a `PlayRecord` (for
    /// example, cards revealed by a claim or an opponent's explanation). The
    /// cards join the predealt and played cards in the seat's known set and,
    /// unlike played cards, are treated as still in hand by
    /// [`Dealer::current_hand`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let known = Cards::from_str("HAK")?;
    /// let dealer = DealerBuilder::new().with_known(Seat::South, known).build()?;
    /// let deal = dealer.deal()?;
    /// assert!(deal.south().cards().contains(known));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn with_known(mut self, seat: Seat, cards: Cards) -> Self {
        self.conditioned_by[seat as usize] = self.conditioned_by[seat as usize].union(cards);
        self
    }

    /// Sets the bounded rejection-sampling budget.
    ///
    /// The dealer rejects at most this many candidate deals before returning
    /// [`DealerError::Unsatisfiable`]. Defaults to
    /// [`DEFAULT_ATTEMPT_BUDGET`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let dealer = DealerBuilder::new().with_attempt_budget(500).build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn with_attempt_budget(mut self, attempts: u32) -> Self {
        self.attempt_budget = attempts;
        self
    }

    /// Sets a custom acceptance function for cross-hand validation.
    ///
    /// This function runs after the per-seat hand constraints: the dealer calls
    /// it only for candidates whose every seat passed its constraint. A
    /// candidate that fails a seat's constraint is discarded before the
    /// function is reached. Use it for constraints that involve multiple hands,
    /// such as:
    /// - Partnership fits (e.g., "NS must have 8+ spade fit")
    /// - Combined HCP (e.g., "NS must have 25+ combined HCP")
    /// - Distributional requirements across hands
    ///
    /// **Performance note**: This function runs once per candidate that passes
    /// the per-seat constraints; rejected candidates never reach it. Keep it as
    /// fast as possible, since it is on the path of every accepted deal.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let builder = DealerBuilder::new()
    ///     .with_function(|deal: &Deal| {
    ///         // Require NS to have game-forcing values
    ///         let ns_hcp = deal.north().hcp() + deal.south().hcp();
    ///         ns_hcp >= 25
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn with_function<T: Fn(&Deal) -> bool + Send + Sync + 'static>(
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let builder = DealerBuilder::new()
    ///     .with_hand_descriptor(
    ///         Seat::North,
    ///         HandDescriptor::hcp(12..=17)
    ///     );
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hand_descriptor(mut self, seat: Seat, hand_description: HandDescriptor) -> Self {
        self.hand_constraints[seat as usize] = Some(hand_description.into());
        self
    }

    /// Sets the hand constraint for a specific seat.
    ///
    /// This is the primary constraint entry point: the [`HandConstraint`]
    /// parses its patterns once here and the dealer runs a single fused
    /// lookup per candidate. `with_hand_descriptor` remains as a
    /// compatibility wrapper converting the legacy descriptor branch by
    /// branch.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let builder = DealerBuilder::new()
    ///     .with_hand_constraint(
    ///         Seat::North,
    ///         HandConstraint::hcp(12..=17)
    ///     );
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hand_constraint(mut self, seat: Seat, constraint: HandConstraint) -> Self {
        self.hand_constraints[seat as usize] = Some(constraint);
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let builder = DealerBuilder::new()
    ///     .with_hand_descriptors_all([
    ///         Some(HandDescriptor::hcp(12..14)),
    ///         None,  // East - no constraints
    ///         Some(HandDescriptor::hcp(12..14)),
    ///         None,  // West - no constraints
    ///     ]);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hand_descriptors_all(
        mut self,
        hand_descriptors: [Option<HandDescriptor>; 4],
    ) -> Self {
        self.hand_constraints =
            hand_descriptors.map(|descriptor| descriptor.map(HandConstraint::from));
        self
    }

    /// Sets hand constraints for all four seats at once.
    ///
    /// Each element can be `None` (no constraints) or `Some(constraint)`.
    /// Preferred over `with_hand_descriptors_all` for new code.
    #[inline]
    #[must_use]
    pub fn with_hand_constraints_all(mut self, constraints: [Option<HandConstraint>; 4]) -> Self {
        self.hand_constraints = constraints;
        self
    }

    /// Builds the dealer from this configuration.
    ///
    /// Consumes the builder and returns a `StandardDealer` ready to generate deals.
    ///
    /// # Errors
    ///
    /// Returns [`DealerError`] when the conditioning is structurally
    /// impossible before any sampling happens: a card fixed to two different
    /// seats, or more than 13 known cards for a seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let dealer = DealerBuilder::new()
    ///     .predeal(Seat::North, Card::SK.into())?
    ///     .build()?;
    ///
    /// let deal = dealer.deal()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn build(self) -> Result<impl Dealer, DealerError> {
        let known = self.resolve_known()?;

        // The random deck is whatever the known cards leave. Computing it
        // here (rather than trusting the incremental `deck` bookkeeping) means
        // conditioning seeded by `with_played`/`with_known` is excluded too.
        let mut deck = Cards::ALL;
        for cards in known {
            deck -= cards;
        }

        // Resolve the fill order before moving the builder's fields out.
        let fill_order = self.fill_order();

        Ok(StandardDealer {
            predeal: self.predealt_hands,
            known,
            played: self.played_by,
            deck_starting_state: deck,
            accept_function: self.accept,
            fill_order,
            hand_constraints: self.hand_constraints,
            attempt_budget: self.attempt_budget,
        })
    }

    /// Computes the order in which [`StandardDealer`] fills and checks seats.
    ///
    /// Seats carrying a [`HandConstraint`] come first, so a candidate that
    /// cannot satisfy one of them is rejected after the first hand rather than
    /// the fourth. Within each tier the canonical North, East, South, West
    /// order is kept. Known cards never move a seat between tiers: a seat with
    /// thirteen known cards still belongs to the group its constraint puts it
    /// in, because its check outcome is constant for the run.
    ///
    /// The order is fixed for the dealer's lifetime; it is an implementation
    /// detail, not part of the builder's configuration.
    #[inline]
    fn fill_order(&self) -> [Seat; NUMBER_OF_HANDS] {
        let mut order = [Seat::North; NUMBER_OF_HANDS];
        let mut next = 0;
        for constrained in [true, false] {
            for seat in Seat::iter() {
                if self.hand_constraints[seat as usize].is_some() == constrained {
                    order[next] = seat;
                    next += 1;
                }
            }
        }
        order
    }

    /// Resolves the per-seat known set and validates it up front.
    ///
    /// `known[seat] = predeal[seat] ∪ conditioned_by[seat]`, deduplicated per
    /// seat. A card fixed to two different seats cannot be placed in a hand,
    /// so it is an error here rather than a silent misdeal.
    ///
    /// Two bounds from the design need no separate check once those hold: a
    /// per-suit overflow (`> 13` of one suit) is impossible when the whole
    /// hand is at most 13 cards, and a deck overflow (`> 52` across seats) is
    /// impossible when no card is shared and each seat is at most 13.
    #[inline]
    fn resolve_known(&self) -> Result<[Cards; NUMBER_OF_HANDS], DealerError> {
        let mut known = [Cards::EMPTY; NUMBER_OF_HANDS];
        for seat in Seat::iter() {
            let predealt = self.predealt_hands[seat as usize].unwrap_or(Cards::EMPTY);
            known[seat as usize] = predealt.union(self.conditioned_by[seat as usize]);
        }

        // Walk the seats once, accumulating what has already been assigned.
        // Any overlap is a card attributed to two seats.
        let mut assigned = Cards::EMPTY;
        for seat in Seat::iter() {
            let overlap = known[seat as usize] & assigned;
            if !overlap.is_empty() {
                let card = overlap.into_iter().next().expect("overlap is non-empty");
                let first = Seat::iter()
                    .find(|earlier| known[*earlier as usize].has_card(card))
                    .expect("overlap came from an earlier seat");
                return Err(DealerError::CardAttributedToTwoSeats {
                    card,
                    first,
                    second: seat,
                });
            }
            assigned += known[seat as usize];
        }

        for seat in Seat::iter() {
            let count = known[seat as usize].len();
            if count > 13 {
                return Err(DealerError::TooManyKnownForSeat { seat, count });
            }
        }

        Ok(known)
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let config = [
    ///     (Some(Card::HQ.into()), Some(HandDescriptor::hcp(12..14))),
    ///     (None, None),  // East - no constraints
    ///     (Some(Card::HK.into()), Some(HandDescriptor::hcp(12..14))),
    ///     (None, None),  // West - no constraints
    /// ];
    /// let builder = DealerBuilder::new().with_complete_config(config)?;
    /// # Ok(())
    /// # }
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
}

/// Trait for objects that can generate bridge deals.
///
/// This trait abstracts the deal generation process. The main implementation
/// is `StandardDealer`, but you could implement custom dealers with different
/// algorithms or constraints.
pub trait Dealer {
    /// Generates a single bridge deal satisfying all constraints.
    ///
    /// [`StandardDealer`] checks each seat's hand constraint as soon as that
    /// seat's cards are dealt: a failure discards the candidate immediately, so
    /// the remaining seats are not filled and the cross-hand acceptance function
    /// is not called. That function therefore only ever sees candidates that
    /// passed every per-seat constraint.
    ///
    /// For [`StandardDealer`], the seats are visited in its fixed fill order:
    /// seats with a [`HandConstraint`] first, then those without, keeping
    /// North, East, South, West inside each tier. Reordering how the hands of
    /// a fixed partition are drawn does not change the partition's
    /// distribution, so this only moves where the check cost lands.
    ///
    /// # Errors
    ///
    /// Returns `DealerError` if:
    /// - The deck doesn't have enough cards (invalid predeal configuration)
    /// - The bounded rejection budget is exhausted before a deal satisfies the
    ///   constraints ([`DealerError::Unsatisfiable`])
    ///
    /// # Performance
    ///
    /// Generation time varies based on constraint tightness. Tighter constraints
    /// require more candidate deals to be generated and rejected.
    fn deal(&self) -> Result<Deal, DealerError>;

    /// Returns the predealed cards for each seat.
    ///
    /// Used internally and for inspection/debugging. This is the raw
    /// predeal only; see [`Dealer::known`] for the full conditioning seed.
    fn predealt(&self) -> [Option<Cards>; NUMBER_OF_HANDS];

    /// Returns the known cards for each seat.
    ///
    /// The conditioning seed: predealt ∪ played ∪ directly-known, deduplicated
    /// per seat. These are the cards the sampler fixes before filling each
    /// hand to 13.
    fn known(&self) -> [Cards; NUMBER_OF_HANDS];

    /// Returns the cards `seat` can still play.
    ///
    /// This is [`Dealer::known`] minus the cards `seat` has already played:
    /// conditioning fixes a played card to the seat's reconstructed original
    /// hand, but that card is no longer available. Card-EV candidate validation
    /// uses this so a played card cannot pass as a candidate.
    fn current_hand(&self, seat: Seat) -> Cards;
}

/// Standard implementation of the `Dealer` trait.
///
/// This dealer uses rejection sampling: it generates random deals and accepts
/// only those that satisfy all constraints. The algorithm:
///
/// 1. Start with the deck left by the known cards
/// 2. For each seat in the dealer's fill order:
///    - Fill the seat from its known cards plus a random draw
///    - Check that seat's hand constraint; a failure discards the candidate
///      at once, without filling the remaining seats
/// 3. Check the acceptance function, which therefore only sees candidates that
///    passed every per-seat constraint
/// 4. If accepted, return the deal; otherwise, try again
///
/// # Fill order
///
/// The order is computed once when the dealer is built: seats carrying a
/// [`HandConstraint`] come first, then the unconstrained seats, with North,
/// East, South, West kept inside each tier. This makes the same constraint cost
/// roughly the same wherever it sits, because a doomed candidate dies after the
/// first hand instead of the third or fourth. Known cards never change a seat's
/// tier.
///
/// Any fixed fill order produces the same distribution of deals: it only
/// changes the order in which the hands of the partition are drawn from the
/// shared deck, not which partitions are accepted.
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
    /// Cards fixed to each seat: `predeal ∪ played ∪ directly-known`.
    known: [Cards; NUMBER_OF_HANDS],
    /// Cards each seat has already played; a subset of `known`.
    played: [Cards; NUMBER_OF_HANDS],
    /// Available cards for random dealing
    deck_starting_state: Cards,
    /// The order in which seats are filled and checked for each candidate.
    ///
    /// Constrained seats first, then unconstrained ones, N, E, S, W inside
    /// each tier. Fixed when the dealer is built; see
    /// [`DealerBuilder::fill_order`].
    fill_order: [Seat; NUMBER_OF_HANDS],
    /// Per-seat constraints
    hand_constraints: [Option<HandConstraint>; NUMBER_OF_HANDS],
    /// Cross-hand validation function
    accept_function: AcceptFunction,
    /// Bounded rejection-sampling budget.
    attempt_budget: u32,
}

impl std::fmt::Debug for StandardDealer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StandardDealer")
            .field("predeal", &self.predeal)
            .field("known", &self.known)
            .field("played", &self.played)
            .field("fill_order", &self.fill_order)
            .field("hand_constraints", &self.hand_constraints)
            .field("attempt_budget", &self.attempt_budget)
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
            known: [Cards::EMPTY; 4],
            played: [Cards::EMPTY; 4],
            deck_starting_state: Cards::ALL,
            // With no constraints every seat is in the same tier, so the
            // canonical positional order is also the constrained-first order.
            fill_order: [Seat::North, Seat::East, Seat::South, Seat::West],
            hand_constraints: [None, None, None, None],
            accept_function: Box::new(|_: &Deal| true),
            attempt_budget: DEFAULT_ATTEMPT_BUDGET,
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

    #[inline]
    fn known(&self) -> [Cards; NUMBER_OF_HANDS] {
        self.known
    }

    #[inline]
    fn current_hand(&self, seat: Seat) -> Cards {
        self.known[seat as usize].difference(self.played[seat as usize])
    }

    /// Generates a deal using rejection sampling.
    ///
    /// This implementation fixes each seat's known cards, fills the remainder
    /// from the available deck, and checks each seat's constraint as soon as
    /// that hand is filled. It tries at most `attempt_budget` candidate deals.
    ///
    /// # Algorithm
    ///
    /// ```text
    /// for _ in 0..attempt_budget {
    ///     1. Start with the deck left by the known cards
    ///     2. For each seat:
    ///        - Start from its known cards
    ///        - Fill the remainder from the deck
    ///        - If its hand constraint fails, discard the candidate:
    ///          do not fill the remaining seats, do not run the closure
    ///     3. Check the acceptance function on the complete deal
    ///     4. If it accepts: return deal
    ///     5. Otherwise: try again
    /// }
    /// return Unsatisfiable
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`DealerError::Unsatisfiable`] when the budget is exhausted, or
    /// a deck error if the conditioning left too few cards to complete a hand.
    #[inline]
    fn deal(&self) -> Result<Deal, DealerError> {
        let mut hands: [Hand; 4] = [Hand::default(); 4];
        // Hoist the RNG out of the rejection loop: creating a thread-local RNG
        // for every one of the four picks per attempt is needlessly expensive.
        let mut rng = rand::rng();

        'candidates: for _ in 0..self.attempt_budget {
            let mut deck = self.deck_starting_state;

            // Deal to each seat in the fixed fill order: known cards first,
            // then the remainder from the shared deck. Dealing from one deck
            // keeps the 52-card partition, so the hot path skips re-checking
            // it.
            for &seat in &self.fill_order {
                let known = self.known[seat as usize];
                let fill = 13 - usize::from(known.len());
                // Once the remaining cards exactly fill this seat, there is no
                // sampling decision left. Take them directly so the final hand
                // does not spend RNG draws re-discovering a deterministic set.
                let cards_to_add = if fill == usize::from(deck.len()) {
                    deck.take_all()
                } else {
                    let Some(cards_to_add) = deck.pick_rng::<_, FastSampler>(&mut rng, fill) else {
                        return Err(DealerError::new(
                            "Deck doesn't contain enough cards. Check predeal configuration.",
                        ));
                    };
                    cards_to_add
                };
                hands[seat as usize].set_cards(known + cards_to_add);

                // A seat that fails its constraint dooms the candidate, so
                // stop dealing and never reach the cross-hand closure.
                if self.hand_constraints[seat as usize]
                    .as_ref()
                    .is_some_and(|constraint| !constraint.check(hands[seat as usize]))
                {
                    continue 'candidates;
                }
            }

            let deal = Deal::from_hands_unchecked(hands);

            if (self.accept_function)(&deal) {
                return Ok(deal);
            }
        }

        Err(DealerError::Unsatisfiable {
            attempts: self.attempt_budget,
        })
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

    /// Dealers are shareable across simulation threads: no `Cell`, and the
    /// acceptance predicate is `Send + Sync`.
    #[test]
    fn test_dealer_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StandardDealer>();
    }

    /// PBN emits ranks high -> low within each suit.
    #[test]
    fn test_pbn_rank_order_high_to_low() {
        use std::str::FromStr;
        // Full 13-card predeal: deterministic, no random fill.
        let cards = Cards::from_str("SAKQJT98765432").unwrap();
        assert_eq!(cards.len(), 13);
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, cards)
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        let pbn = deal.as_pbn();
        // North hand section starts after "[Deal \"N:".
        let after_n = pbn.split("[Deal \"N:").nth(1).unwrap();
        // High -> low means the section starts with "AKQ", not "QKA".
        assert!(
            after_n.starts_with("AKQJT98765432..."),
            "PBN north spades should start high->low with AKQ, got: {after_n}"
        );
    }

    /// Tests that acceptance functions work correctly.
    #[test]
    fn test_accept_function() {
        let dealer = DealerBuilder::new()
            .with_function(|deal: &Deal| {
                // Only accept if North has at least 20 HCP
                deal.north().hcp() >= 20
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

    /// A card predealt and played by the same seat is one known card, not a
    /// duplicate error.
    #[test]
    fn conditioning_dedups_same_seat_overlap() {
        let predeal = Cards::from_str("SAK").expect("predeal parses");
        let record = PlayRecord::new(vec![
            Play::new(Seat::North, Card::SA),
            Play::new(Seat::East, Card::H2),
        ]);
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, predeal)
            .expect("predeal is unique")
            .with_played(&record)
            .build()
            .expect("same-seat overlap is not a conflict");

        assert_eq!(dealer.known()[Seat::North as usize], predeal);
        assert_eq!(dealer.known()[Seat::East as usize], Cards::from(Card::H2));

        // A played card stays in the known (original-hand) seed but drops out
        // of the current hand.
        assert_eq!(dealer.current_hand(Seat::North), Cards::from(Card::SK));
        assert_eq!(dealer.current_hand(Seat::East), Cards::EMPTY);

        let deal = dealer.deal().expect("dealer deals");
        assert!(deal.north().cards().contains(predeal));
        assert!(deal.east().cards().has_card(Card::H2));
        // A played card is fixed to its seat, so no other hand can hold it.
        for seat in [Seat::North, Seat::South, Seat::West] {
            assert!(!deal[seat].cards().has_card(Card::H2));
        }
    }

    /// `with_known` seeds the known set directly, without a `PlayRecord`.
    #[test]
    fn with_known_seeds_cards_directly() {
        let known = Cards::from_str("HAK").expect("known cards parse");
        let dealer = DealerBuilder::new()
            .with_known(Seat::South, known)
            .build()
            .expect("directly-known cards are valid");
        assert_eq!(dealer.known()[Seat::South as usize], known);
        let deal = dealer.deal().expect("dealer deals");
        assert!(deal.south().cards().contains(known));
    }

    /// A card fixed to two different seats is a structural error.
    #[test]
    fn card_attributed_to_two_seats_is_an_error() {
        // Predealt to North, played by East.
        let record = PlayRecord::new(vec![Play::new(Seat::East, Card::SA)]);
        let Err(err) = DealerBuilder::new()
            .predeal(Seat::North, Cards::from(Card::SA))
            .expect("predeal is unique")
            .with_played(&record)
            .build()
        else {
            panic!("a card cannot be in two seats");
        };
        match err {
            DealerError::CardAttributedToTwoSeats {
                card,
                first,
                second,
            } => {
                assert_eq!(card, Card::SA);
                assert_eq!(first, Seat::North);
                assert_eq!(second, Seat::East);
            }
            other => panic!("expected a cross-seat conflict, got {other:?}"),
        }

        // Played by two different seats.
        let record = PlayRecord::new(vec![
            Play::new(Seat::North, Card::SA),
            Play::new(Seat::East, Card::SA),
        ]);
        let Err(err) = DealerBuilder::new().with_played(&record).build() else {
            panic!("a card cannot be played by two seats");
        };
        assert!(matches!(
            err,
            DealerError::CardAttributedToTwoSeats { card, .. } if card == Card::SA
        ));
    }

    /// More than 13 known cards for one seat is rejected before sampling.
    #[test]
    fn too_many_known_for_one_seat_is_an_error() {
        let fourteen = Cards::from_str("SAKQJT98765432H2").expect("14 distinct cards parse");
        assert_eq!(fourteen.len(), 14);
        let Err(err) = DealerBuilder::new()
            .with_known(Seat::North, fourteen)
            .build()
        else {
            panic!("a hand holds at most 13 cards");
        };
        assert!(matches!(
            err,
            DealerError::TooManyKnownForSeat {
                seat: Seat::North,
                count: 14
            }
        ));
    }

    /// Constraints run on the reconstructed 13-card hand, not the remainder.
    ///
    /// North has nine known black cards, so its remainder is four cards. The
    /// constraint can only be satisfied by the reconstructed hand: the four
    /// random cards must all be hearts.
    #[test]
    fn conditioning_constraints_check_reconstructed_full_hands() {
        let predeal = Cards::from_str("SAK").expect("predeal parses");
        let played: Vec<Play> = ["S2", "S3", "S4", "S5", "S6", "S7", "S8"]
            .iter()
            .map(|rank| Play::new(Seat::North, Card::from_str(rank).expect("card parses")))
            .collect();
        let record = PlayRecord::new(played);

        let dealer = DealerBuilder::new()
            .predeal(Seat::North, predeal)
            .expect("predeal is unique")
            .with_played(&record)
            .with_function(|deal: &Deal| deal.north().hearts().len() == 4)
            .build()
            .expect("conditioning is valid");

        let deal = dealer.deal().expect("dealer finds a 4-heart hand");
        assert_eq!(deal.north().cards().len(), 13);
        assert_eq!(deal.north().hearts().len(), 4);
        assert!(deal.north().cards().contains(predeal));
    }

    /// Nine spades fixed to North (two predealt, seven played); the caller adds
    /// the constraint under test.
    fn conditioned_north_spades_dealer(builder: DealerBuilder) -> impl Dealer {
        let predeal = Cards::from_str("SAK").expect("predeal parses");
        let played: Vec<Play> = ["S2", "S3", "S4", "S5", "S6", "S7", "S8"]
            .iter()
            .map(|rank| Play::new(Seat::North, Card::from_str(rank).expect("card parses")))
            .collect();
        builder
            .predeal(Seat::North, predeal)
            .expect("predeal is unique")
            .with_played(&PlayRecord::new(played))
            .build()
            .expect("conditioning is valid")
    }

    /// A `HandConstraint` also runs on the reconstructed 13-card hand: the
    /// nine known spades force the shape `9400`, so the four random cards must
    /// all be hearts.
    #[test]
    fn conditioning_hand_constraint_checks_reconstructed_full_hand() {
        let dealer = conditioned_north_spades_dealer(DealerBuilder::new().with_hand_constraint(
            Seat::North,
            HandConstraint::new(&["9400"], HcpRange::new(0, 37)).expect("shape parses"),
        ));
        let deal = dealer.deal().expect("dealer finds a 9-4-0-0 hand");
        assert_eq!(deal.north().spades().len(), 9);
        assert_eq!(deal.north().hearts().len(), 4);
    }

    /// The legacy `HandDescriptor` wrapper shares the same full-hand path.
    #[test]
    fn conditioning_hand_descriptor_checks_reconstructed_full_hand() {
        let shape = Shape::try_from("9400").expect("shape parses");
        let descriptor = HandDescriptor::new(vec![HandType::new(shape, HcpRange::new(0, 37))]);
        let dealer = conditioned_north_spades_dealer(
            DealerBuilder::new().with_hand_descriptor(Seat::North, descriptor),
        );
        let deal = dealer.deal().expect("dealer finds a 9-4-0-0 hand");
        assert_eq!(deal.north().hearts().len(), 4);
    }

    /// A bounded budget returns a distinct error instead of sampling forever.
    #[test]
    fn exhausted_attempt_budget_is_a_bounded_error() {
        let dealer = DealerBuilder::new()
            .with_attempt_budget(10)
            .with_function(|_: &Deal| false)
            .build()
            .expect("a rejecting dealer still builds");
        let err = dealer
            .deal()
            .expect_err("no deal can satisfy the rejection");
        assert!(matches!(err, DealerError::Unsatisfiable { attempts: 10 }));
    }

    /// Tests PBN format output. A `Deal` is a hand record only; the `[Board]`
    /// line belongs to the recorded board, not to the four hands.
    #[test]
    fn test_pbn_format() {
        let deal = DealerBuilder::new()
            .build()
            .expect("unconstrained dealer builds")
            .deal()
            .expect("unconstrained dealer deals");
        let pbn = deal.as_pbn();

        assert!(!pbn.contains("[Board"));
        assert!(pbn.contains("[Deal"));
    }

    /// Tests that hands are correctly indexed by seat.
    #[test]
    fn test_hands_indexing() {
        let dealer = DealerBuilder::new().build().unwrap();
        let deal = dealer.deal().unwrap();

        assert_eq!(deal[Seat::North], deal.north());
        assert_eq!(deal[Seat::East], deal.east());
        assert_eq!(deal[Seat::South], deal.south());
        assert_eq!(deal[Seat::West], deal.west());

        assert_eq!(deal[0], deal.north());
        assert_eq!(deal[1], deal.east());
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
        _ = DealerBuilder::new()
            .build()
            .expect("unconstrained dealer builds")
            .deal()
            .expect("unconstrained dealer deals");
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
            .with_function(Box::new(|deal: &Deal| {
                deal.north().slen() + deal.south().slen() > 8
            }))
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert!(deal.north().slen() + deal.south().slen() > 8);
    }

    /// One hand per suit: a valid partition with no duplicates.
    fn sample_deal() -> [Hand; 4] {
        [
            Hand::from_str("SAKQJT98765432").expect("spades hand parses"),
            Hand::from_str("HAKQJT98765432").expect("hearts hand parses"),
            Hand::from_str("DAKQJT98765432").expect("diamonds hand parses"),
            Hand::from_str("CAKQJT98765432").expect("clubs hand parses"),
        ]
    }

    #[test]
    fn deal_new_accepts_valid_partition() {
        let deal = Deal::new(sample_deal()).expect("one suit per seat is a valid deal");
        for seat in Seat::iter() {
            assert_eq!(deal[seat].cards().len(), 13);
        }
    }

    #[test]
    fn deal_new_rejects_wrong_hand_size() {
        let mut hands = sample_deal();
        hands[Seat::South as usize] =
            Hand::try_from(Cards::from_str("DAKQJT9876543").expect("12 diamonds parse"))
                .expect("12 cards fit in a hand");
        assert_eq!(
            Deal::new(hands),
            Err(DealError::WrongHandSize {
                seat: Seat::South,
                len: 12,
            })
        );
    }

    #[test]
    fn deal_new_rejects_duplicated_card() {
        let repeated = sample_deal()[Seat::North as usize];
        let hands = [repeated; 4];
        assert!(matches!(
            Deal::new(hands),
            Err(DealError::DuplicatedCard(_))
        ));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deal_deserialize_round_trips_through_validation() {
        let deal = Deal::new(sample_deal()).expect("valid deal");
        let json = serde_json::to_string(&deal).expect("deal serializes");
        let parsed: Deal = serde_json::from_str(&json).expect("deal deserializes");
        assert_eq!(parsed, deal);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deal_deserialize_rejects_corrupt_partition() {
        // Four copies of one hand: each holds 13 cards, but every card repeats.
        let corrupt = [sample_deal()[Seat::North as usize]; 4];
        let json = serde_json::to_string(&corrupt).expect("hands serialize");
        assert!(serde_json::from_str::<Deal>(&json).is_err());
    }
}
