// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The bridge deal: four seats' cards and the 52-card partition.
//!
//! A [`Deal`] is four seats' [`Cards`] and the only type that guarantees the
//! 52-card partition. It owns just the cards: board number, dealer,
//! vulnerability and display choice belong to a [`crate::record::Board`].
//!
//! [`Seat`] and [`Vulnerability`] are contract vocabulary and live in
//! [`crate::contract`]; this module borrows [`Seat`] to index the four hands.
//! Deal generation lives in [`crate::dealer`]; nothing here samples or rejects
//! a deal.
//!
//! # Indexing
//!
//! You can index by [`Seat`] or `usize`:
//!
//! ```rust
//! # use squeezer::*;
//! let hands = [
//!     Cards::from_str("SAKQJT98765432")?,
//!     Cards::from_str("HAKQJT98765432")?,
//!     Cards::from_str("DAKQJT98765432")?,
//!     Cards::from_str("CAKQJT98765432")?,
//! ];
//! let deal = Deal::new(hands)?;
//! assert_eq!(deal[Seat::North], deal.north());
//! assert_eq!(deal[0], deal.north()); // Equivalent
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::contract::Seat;
use crate::prelude::*;

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
/// Each seat's `Cards` is 8 bytes (bitboard representation), so a `Deal` is
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
    /// Cards held by each seat, in order: [North, East, South, West]
    hands: [Cards; 4],
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
    /// Creates a deal from the four seats' cards, validating the 52-card
    /// partition.
    ///
    /// Every seat must hold exactly 13 cards and no card may appear twice; as
    /// documented on [`DealError`], those two conditions already imply the four
    /// holdings cover the whole deck.
    ///
    /// # Errors
    ///
    /// Returns [`DealError`] unless every seat holds exactly 13 distinct cards.
    pub fn new(hands: [Cards; 4]) -> Result<Self, DealError> {
        for (index, cards) in hands.iter().enumerate() {
            let len = cards.len();
            if len != 13 {
                return Err(DealError::WrongHandSize {
                    seat: Seat::from(index),
                    len,
                });
            }
        }

        // Fold the four holdings into one bitboard, reporting the first card
        // that was already seen.
        let mut seen = Cards::EMPTY;
        for cards in &hands {
            for card in *cards {
                if seen.has_card(card) {
                    return Err(DealError::DuplicatedCard(card));
                }
                seen = seen.insert(card);
            }
        }

        Ok(Self { hands })
    }

    /// Creates a deal from the four seats' cards without checking the
    /// partition.
    ///
    /// The caller takes full responsibility for the invariant: each seat must
    /// hold exactly 13 distinct cards and the four holdings must cover the
    /// whole deck. Prefer [`Deal::new`] unless you are on a hot path that has
    /// already established the invariant (for example [`StandardDealer`], which
    /// deals from a shuffled deck).
    #[must_use]
    #[inline]
    pub const fn from_cards_unchecked(hands: [Cards; 4]) -> Self {
        Self { hands }
    }

    /// Returns a reference to the four seats' cards.
    ///
    /// Holdings are in order: [North, East, South, West]
    #[must_use]
    #[inline]
    pub fn hands(&self) -> &[Cards; 4] {
        &self.hands
    }

    /// Returns the cards North holds.
    #[must_use]
    #[inline]
    pub fn north(&self) -> Cards {
        self.hands[Seat::North as usize]
    }

    /// Returns the cards South holds.
    #[must_use]
    #[inline]
    pub fn south(&self) -> Cards {
        self.hands[Seat::South as usize]
    }

    /// Returns the cards East holds.
    #[must_use]
    #[inline]
    pub fn east(&self) -> Cards {
        self.hands[Seat::East as usize]
    }

    /// Returns the cards West holds.
    #[must_use]
    #[inline]
    pub fn west(&self) -> Cards {
        self.hands[Seat::West as usize]
    }

    /// Returns an iterator over the four seats' cards.
    ///
    /// Iterates in order: North, East, South, West
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Cards> {
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

impl TryFrom<[Cards; 4]> for Deal {
    type Error = DealError;

    /// Validates the four hands via [`Deal::new`].
    ///
    /// This is the named conversion that [`Deal`]'s `Deserialize` goes
    /// through.
    fn try_from(hands: [Cards; 4]) -> Result<Self, Self::Error> {
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
        let hands = <[Cards; 4]>::deserialize(deserializer)?;
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
    type IntoIter = IntoIter<Cards, 4>;
    type Item = Cards;

    fn into_iter(self) -> Self::IntoIter {
        self.hands.into_iter()
    }
}

impl std::ops::Index<Seat> for Deal {
    type Output = Cards;

    #[inline]
    fn index(&self, index: Seat) -> &Self::Output {
        &self.hands[index as usize]
    }
}

impl std::ops::Index<usize> for Deal {
    type Output = Cards;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.hands[index]
    }
}

impl<'a> IntoIterator for &'a Deal {
    type IntoIter = std::slice::Iter<'a, Cards>;
    type Item = &'a Cards;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PBN emits ranks high -> low within each suit.
    #[test]
    fn test_pbn_rank_order_high_to_low() {
        // One suit per seat: deterministic, with no random fill.
        let deal = Deal::new(sample_deal()).expect("one suit per seat is a valid deal");
        assert_eq!(deal.north().len(), 13);
        let pbn = deal.as_pbn();
        // North hand section starts after "[Deal \"N:".
        let after_n = pbn.split("[Deal \"N:").nth(1).unwrap();
        // High -> low means the section starts with "AKQ", not "QKA".
        assert!(
            after_n.starts_with("AKQJT98765432..."),
            "PBN north spades should start high->low with AKQ, got: {after_n}"
        );
    }

    /// Tests PBN format output. A `Deal` is a hand record only; the `[Board]`
    /// line belongs to the recorded board, not to the four hands.
    #[test]
    fn test_pbn_format() {
        let deal = Deal::new(sample_deal()).expect("one suit per seat is a valid deal");
        let pbn = deal.as_pbn();

        assert!(!pbn.contains("[Board"));
        assert!(pbn.contains("[Deal"));
    }

    /// Tests that hands are correctly indexed by seat.
    #[test]
    fn test_hands_indexing() {
        let deal = Deal::new(sample_deal()).expect("one suit per seat is a valid deal");

        assert_eq!(deal[Seat::North], deal.north());
        assert_eq!(deal[Seat::East], deal.east());
        assert_eq!(deal[Seat::South], deal.south());
        assert_eq!(deal[Seat::West], deal.west());

        assert_eq!(deal[0], deal.north());
        assert_eq!(deal[1], deal.east());
    }

    /// One hand per suit: a valid partition with no duplicates.
    fn sample_deal() -> [Cards; 4] {
        [
            Cards::from_str("SAKQJT98765432").expect("spades hand parses"),
            Cards::from_str("HAKQJT98765432").expect("hearts hand parses"),
            Cards::from_str("DAKQJT98765432").expect("diamonds hand parses"),
            Cards::from_str("CAKQJT98765432").expect("clubs hand parses"),
        ]
    }

    #[test]
    fn deal_new_accepts_valid_partition() {
        let deal = Deal::new(sample_deal()).expect("one suit per seat is a valid deal");
        for seat in Seat::iter() {
            assert_eq!(deal[seat].len(), 13);
        }
    }

    #[test]
    fn deal_new_rejects_wrong_hand_size() {
        let mut hands = sample_deal();
        hands[Seat::South as usize] = Cards::from_str("DAKQJT9876543").expect("12 diamonds parse");
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
