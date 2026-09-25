/// All the code here come from David Roundy's library [bridge-cards](https://github.com/droundy/bridge-cards),
/// I only slightly modified the code to adapt it to my needs.
use crate::prelude::*;
#[cfg(feature = "dds")]
use dds::traits::AsDDSCard;
use rand::Rng;

/// A card, represented as a `u8`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card {
    offset: u8,
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank = self.rank();
        match rank {
            11 => write!(f, "J{:?}", self.suit()),
            12 => write!(f, "Q{:?}", self.suit()),
            13 => write!(f, "K{:?}", self.suit()),
            14 => write!(f, "A{:?}", self.suit()),
            _ => write!(f, "{}{:?}", rank, self.suit()),
        }
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rankchar(), self.suit())
    }
}

impl Card {
    #[must_use]
    #[inline]
    pub const fn new(suit: Suit, rank: u8) -> Self {
        Card {
            offset: rank + 16 * suit as u8,
        }
    }

    pub(crate) const fn from_u8(offset: u8) -> Self {
        Self { offset }
    }

    ///Returns suit of the card.
    ///# Panics
    /// If you request the suit of a `Card::JOKER`
    #[must_use]
    #[inline]
    pub const fn suit(self) -> Suit {
        match self.offset >> 4 {
            0 => Suit::Spades,
            1 => Suit::Hearts,
            2 => Suit::Diamonds,
            3 => Suit::Clubs,
            _ => panic!("requested Suit for a Card::JOKER!"),
        }
    }

    /// Returns the rank of the card.
    #[must_use]
    #[inline]
    pub const fn rank(self) -> u8 {
        self.offset & 15
    }

    /// Returns the name of the rank of the card.
    #[must_use]
    #[inline]
    pub const fn rankname(self) -> &'static str {
        match self.rank() {
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "10",
            11 => "J",
            12 => "Q",
            13 => "K",
            14 => "A",
            _ => "?",
        }
    }

    /// Returns the name of the rank of the card.
    #[must_use]
    #[inline]
    pub const fn rankchar(self) -> char {
        match self.rank() {
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            9 => '9',
            10 => 'T',
            11 => 'J',
            12 => 'Q',
            13 => 'K',
            14 => 'A',
            _ => '?',
        }
    }

    #[must_use]
    #[inline]
    pub(crate) const fn as_u8(self) -> u8 {
        self.offset
    }

    /// 2 of Clubs
    pub const C2: Card = Card::new(Suit::Clubs, 2);
    /// 3 of Clubs
    pub const C3: Card = Card::new(Suit::Clubs, 3);
    /// 4 of Clubs
    pub const C4: Card = Card::new(Suit::Clubs, 4);
    /// 5 of Clubs
    pub const C5: Card = Card::new(Suit::Clubs, 5);
    /// 6 of Clubs
    pub const C6: Card = Card::new(Suit::Clubs, 6);
    /// 7 of Clubs
    pub const C7: Card = Card::new(Suit::Clubs, 7);
    /// 8 of Clubs
    pub const C8: Card = Card::new(Suit::Clubs, 8);
    /// 9 of Clubs
    pub const C9: Card = Card::new(Suit::Clubs, 9);
    /// 10 of Clubs
    pub const C10: Card = Card::new(Suit::Clubs, 10);
    /// Jack of Clubs
    pub const CJ: Card = Card::new(Suit::Clubs, 11);
    /// Queen of Clubs
    pub const CQ: Card = Card::new(Suit::Clubs, 12);
    /// King of Clubs
    pub const CK: Card = Card::new(Suit::Clubs, 13);
    /// Ace of Clubs
    pub const CA: Card = Card::new(Suit::Clubs, 14);

    /// 2 of Diamonds
    pub const D2: Card = Card::new(Suit::Diamonds, 2);
    /// 3 of Diamonds
    pub const D3: Card = Card::new(Suit::Diamonds, 3);
    /// 4 of Diamonds
    pub const D4: Card = Card::new(Suit::Diamonds, 4);
    /// 5 of Diamonds
    pub const D5: Card = Card::new(Suit::Diamonds, 5);
    /// 6 of Diamonds
    pub const D6: Card = Card::new(Suit::Diamonds, 6);
    /// 7 of Diamonds
    pub const D7: Card = Card::new(Suit::Diamonds, 7);
    /// 8 of Diamonds
    pub const D8: Card = Card::new(Suit::Diamonds, 8);
    /// 9 of Diamonds
    pub const D9: Card = Card::new(Suit::Diamonds, 9);
    /// 10 of Diamonds
    pub const D10: Card = Card::new(Suit::Diamonds, 10);
    /// Jack of Diamonds
    pub const DJ: Card = Card::new(Suit::Diamonds, 11);
    /// Queen of Diamonds
    pub const DQ: Card = Card::new(Suit::Diamonds, 12);
    /// King of Diamonds
    pub const DK: Card = Card::new(Suit::Diamonds, 13);
    /// Ace of Diamonds
    pub const DA: Card = Card::new(Suit::Diamonds, 14);

    /// 2 of Hearts
    pub const H2: Card = Card::new(Suit::Hearts, 2);
    /// 3 of Hearts
    pub const H3: Card = Card::new(Suit::Hearts, 3);
    /// 4 of Hearts
    pub const H4: Card = Card::new(Suit::Hearts, 4);
    /// 5 of Hearts
    pub const H5: Card = Card::new(Suit::Hearts, 5);
    /// 6 of Hearts
    pub const H6: Card = Card::new(Suit::Hearts, 6);
    /// 7 of Hearts
    pub const H7: Card = Card::new(Suit::Hearts, 7);
    /// 8 of Hearts
    pub const H8: Card = Card::new(Suit::Hearts, 8);
    /// 9 of Hearts
    pub const H9: Card = Card::new(Suit::Hearts, 9);
    /// 10 of Hearts
    pub const H10: Card = Card::new(Suit::Hearts, 10);
    /// Jack of Hearts
    pub const HJ: Card = Card::new(Suit::Hearts, 11);
    /// Queen of Hearts
    pub const HQ: Card = Card::new(Suit::Hearts, 12);
    /// King of Hearts
    pub const HK: Card = Card::new(Suit::Hearts, 13);
    /// Ace of Hearts
    pub const HA: Card = Card::new(Suit::Hearts, 14);

    /// 2 of Spades
    pub const S2: Card = Card::new(Suit::Spades, 2);
    /// 3 of Spades
    pub const S3: Card = Card::new(Suit::Spades, 3);
    /// 4 of Spades
    pub const S4: Card = Card::new(Suit::Spades, 4);
    /// 5 of Spades
    pub const S5: Card = Card::new(Suit::Spades, 5);
    /// 6 of Spades
    pub const S6: Card = Card::new(Suit::Spades, 6);
    /// 7 of Spades
    pub const S7: Card = Card::new(Suit::Spades, 7);
    /// 8 of Spades
    pub const S8: Card = Card::new(Suit::Spades, 8);
    /// 9 of Spades
    pub const S9: Card = Card::new(Suit::Spades, 9);
    /// 10 of Spades
    pub const S10: Card = Card::new(Suit::Spades, 10);
    /// Jack of Spades
    pub const SJ: Card = Card::new(Suit::Spades, 11);
    /// Queen of Spades
    pub const SQ: Card = Card::new(Suit::Spades, 12);
    /// King of Spades
    pub const SK: Card = Card::new(Suit::Spades, 13);
    /// Ace of Spades
    pub const SA: Card = Card::new(Suit::Spades, 14);
    pub const JOKER: Card = Card { offset: u8::MAX };
    pub const NONCE: Card = Card { offset: 0 };
}

impl TryFrom<&[Card]> for Cards {
    type Error = SqueezerError;

    fn try_from(card_slice: &[Card]) -> Result<Self, Self::Error> {
        let mut cards = Cards::EMPTY;
        for card in card_slice {
            if cards.has_card(*card) {
                return Err(SqueezerError::invalid_input(
                    "building a card set from a slice",
                    format!("duplicate card {card}"),
                ));
            }
            cards = cards.insert(*card);
        }
        Ok(cards)
    }
}

impl From<Card> for Cards {
    fn from(card: Card) -> Self {
        Cards::EMPTY.insert(card)
    }
}

pub struct FastSampler;
pub struct UniformSampler;

pub trait CardSampler {
    fn pick_n<R: Rng>(deck: &mut Cards, num: usize, rng: &mut R) -> Option<Cards>;
}

impl CardSampler for FastSampler {
    fn pick_n<R: Rng>(deck: &mut Cards, mut num: usize, rng: &mut R) -> Option<Cards> {
        let mut bits = deck.bits;
        let mut n_left = deck.len() as usize;
        let mut kept = 0;
        let mut given = 0;
        while n_left > 0 {
            if num == 0 {
                kept |= bits;
                break;
            }
            let chosen = rng.random::<u64>() & bits;
            if chosen != 0 {
                let num_chosen = chosen.count_ones() as usize;
                if num_chosen <= num {
                    bits &= !chosen;
                    given |= chosen;
                    n_left -= num_chosen;
                    num -= num_chosen;
                } else if num_chosen + num < n_left {
                    bits &= !chosen;
                    kept |= chosen;
                    n_left -= num_chosen;
                }
            }
        }
        deck.bits = kept;
        Some(Cards { bits: given })
    }
}

impl CardSampler for UniformSampler {
    fn pick_n<R: Rng>(deck: &mut Cards, num: usize, rng: &mut R) -> Option<Cards> {
        let deck_len = deck.len() as usize;
        if num > deck_len {
            return None;
        }
        let mut positions = [0u8; 52];
        let mut n = 0;
        for i in 0..64u8 {
            if deck.bits & (1u64 << i) != 0 {
                positions[n] = i;
                n += 1;
            }
        }
        // Partial Fisher-Yates: only the first `num` positions need to be shuffled
        // to obtain a uniformly random subset.
        for i in 0..num {
            let j = rng.random_range(i..n);
            positions.swap(i, j);
        }
        let mut to_draw = 0u64;
        for &pos in positions.iter().take(num) {
            to_draw |= 1u64 << pos;
        }
        deck.bits &= !to_draw;
        Some(Cards { bits: to_draw })
    }
}

/// A bunch of [`Card`]s.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Cards {
    bits: u64,
}

#[cfg(feature = "serde")]
impl serde::Serialize for Cards {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Cards {
    /// Parses the suit-symbol string emitted by `Cards`' `Serialize`, keeping
    /// the two directions symmetric (a derived `Deserialize` would expect the
    /// private `bits` field instead).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::UpperHex for Cards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <u64 as std::fmt::UpperHex>::fmt(&self.bits, f)
    }
}

impl std::fmt::LowerHex for Cards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <u64 as std::fmt::LowerHex>::fmt(&self.bits, f)
    }
}

impl std::fmt::Octal for Cards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <u64 as std::fmt::Octal>::fmt(&self.bits, f)
    }
}

impl std::fmt::Binary for Cards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <u64 as std::fmt::Binary>::fmt(&self.bits, f)
    }
}
impl From<&Vec<Card>> for Cards {
    fn from(value: &Vec<Card>) -> Self {
        Cards {
            bits: value.iter().fold(0, |a, c| a | (1u64 << c.offset)),
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
impl Cards {
    #[must_use]
    #[inline]
    /// A new, full French deck without Jokers.
    pub const fn new_deck() -> Self {
        Cards::ALL
    }

    #[must_use]
    #[inline]
    /// Number of cards stored.
    pub const fn len(&self) -> u8 {
        self.bits.count_ones() as u8
    }

    #[must_use]
    #[inline]
    /// True if the instance has no cards.
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[must_use]
    #[inline]
    /// Insert a `card` into the instance.
    pub const fn insert(self, card: Card) -> Self {
        Self {
            bits: self.bits | (1u64 << card.offset),
        }
    }

    #[must_use]
    #[inline]
    /// NOTE: Removes, does not `pop`.
    pub const fn remove(self, card: Card) -> Self {
        Self {
            bits: self.bits & !(1u64 << card.offset),
        }
    }

    #[must_use]
    #[inline]
    /// Returns true if `card` is contained in the instance.
    pub const fn has_card(self, card: Card) -> bool {
        self.bits & (1u64 << card.offset) != 0
    }
    #[must_use]
    #[inline]
    /// Returns true if all the `cards` are contained in the instance.
    pub const fn contains(self, cards: Cards) -> bool {
        cards.bits & self.bits == cards.bits
    }

    /// Sum of two [`Cards`] instances.
    #[must_use]
    #[inline]
    pub const fn union(&self, cards: Cards) -> Self {
        Self {
            bits: self.bits | cards.bits,
        }
    }

    /// Difference of two [`Cards`] instances.
    #[must_use]
    #[inline]
    pub const fn difference(&self, cards: Cards) -> Self {
        Self {
            bits: self.bits & !cards.bits,
        }
    }

    /// Cards in common between two [`Cards`] instances.
    #[must_use]
    const fn intersection(self, cards: Cards) -> Self {
        Cards {
            bits: self.bits & cards.bits,
        }
    }
    /// Returns all the cards of `suit` stored in this instance.
    #[must_use]
    #[inline]
    pub const fn in_suit(self, suit: Suit) -> Self {
        let offset = suit as i32 * 16;
        Cards {
            bits: ((self.bits >> offset) & 0xFFFF) << offset,
        }
    }

    /// Randomly pick `num` cards to remove from the deck.
    /// Returns `None` only if there aren't enough cards.
    #[must_use]
    #[inline]
    pub fn pick(&mut self, num: usize) -> Option<Cards> {
        self.pick_rng::<_, FastSampler>(&mut rand::rng(), num)
    }

    /// Randomly pick `num` cards to remove from the deck using specified RNG.
    /// Returns `None` only if there aren't enough cards.
    /// # Panics
    /// When `num >= 53`
    pub fn pick_rng<R: Rng, S: CardSampler>(&mut self, rng: &mut R, num: usize) -> Option<Cards> {
        assert!(num <= 52);
        if self.len() < num as u8 {
            None
        } else {
            S::pick_n(self, num, rng)
        }
    }

    /// A new, full French deck without Jokers.
    pub const ALL: Cards = Self::SPADES
        .union(Self::HEARTS)
        .union(Self::DIAMONDS)
        .union(Self::CLUBS);

    /// All spades cards.
    pub const SPADES: Cards = Cards { bits: 0x7ffc };

    /// Just the spades from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn spades(self) -> Cards {
        self.intersection(Cards::SPADES)
    }

    /// All hearts cards.
    pub const HEARTS: Cards = Cards { bits: 0x7ffc << 16 };

    /// Just the hearts from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn hearts(self) -> Cards {
        self.intersection(Cards::HEARTS)
    }

    /// All diamonds cards.
    pub const DIAMONDS: Cards = Cards { bits: 0x7ffc << 32 };

    /// Just the diamonds from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn diamonds(self) -> Cards {
        self.intersection(Cards::DIAMONDS)
    }

    /// All clubs cards.
    pub const CLUBS: Cards = Cards { bits: 0x7ffc << 48 };

    /// Just the clubs from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn clubs(self) -> Cards {
        self.intersection(Cards::CLUBS)
    }

    /// A deck with no cards in it.
    pub const EMPTY: Cards = Cards { bits: 0 };

    /// The aces
    pub const ACES: Cards = Cards::EMPTY
        .insert(Card::CA)
        .insert(Card::DA)
        .insert(Card::HA)
        .insert(Card::SA);

    /// Just the aces from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn aces(self) -> Cards {
        self.intersection(Cards::ACES)
    }
    /// The kings
    pub const KINGS: Cards = Cards::EMPTY
        .insert(Card::CK)
        .insert(Card::DK)
        .insert(Card::HK)
        .insert(Card::SK);
    /// Just the kins from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn kings(self) -> Cards {
        self.intersection(Cards::KINGS)
    }
    /// The queens
    pub const QUEENS: Cards = Cards::EMPTY
        .insert(Card::CQ)
        .insert(Card::DQ)
        .insert(Card::HQ)
        .insert(Card::SQ);
    /// Just the queens from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn queens(self) -> Cards {
        self.intersection(Cards::QUEENS)
    }
    /// The jacks
    pub const JACKS: Cards = Cards::EMPTY
        .insert(Card::CJ)
        .insert(Card::DJ)
        .insert(Card::HJ)
        .insert(Card::SJ);
    /// Just the jacks from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn jacks(self) -> Cards {
        self.intersection(Cards::JACKS)
    }
    /// The tens
    pub const TENS: Cards = Cards::EMPTY
        .insert(Card::C10)
        .insert(Card::D10)
        .insert(Card::H10)
        .insert(Card::S10);
    /// Just the tens from this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn tens(self) -> Cards {
        self.intersection(Cards::TENS)
    }

    /// High card points.
    ///
    /// Single pass over the set bits: each card contributes 4/3/2/1 for
    /// A/K/Q/J based on its rank nibble.
    #[must_use]
    #[inline]
    pub const fn high_card_points(self) -> u8 {
        // Each suit occupies a 16-bit field and each rank is one bit in that
        // field. Masking the four honour ranks lets the CPU count all cards of
        // a rank at once instead of walking every set bit in the hand.
        const ACES: u64 = (1u64 << 14) | (1u64 << 30) | (1u64 << 46) | (1u64 << 62);
        const KINGS: u64 = (1u64 << 13) | (1u64 << 29) | (1u64 << 45) | (1u64 << 61);
        const QUEENS: u64 = (1u64 << 12) | (1u64 << 28) | (1u64 << 44) | (1u64 << 60);
        const JACKS: u64 = (1u64 << 11) | (1u64 << 27) | (1u64 << 43) | (1u64 << 59);

        let points = 4 * (self.bits & ACES).count_ones()
            + 3 * (self.bits & KINGS).count_ones()
            + 2 * (self.bits & QUEENS).count_ones()
            + (self.bits & JACKS).count_ones();
        points as u8
    }

    /// Returns the inner u64 type representing this [`Cards`] instance.
    #[must_use]
    #[inline]
    pub const fn as_bits(self) -> u64 {
        self.bits
    }

    #[must_use]
    #[inline]
    /// Returns an instance of [`Cards`] containing only the head of a sequence of cards.
    /// ```
    /// # use std::error::Error;
    /// # use squeezer::prelude::*;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let hand = Cards::from_str("KQ986532")?;
    /// let heads = hand.heads_of_sequences();
    /// // Prints {"♠K963"}
    /// println!("{heads}" );
    ///
    /// # Ok(())
    /// # }
    ///
    pub const fn heads_of_sequences(self) -> Self {
        let mut insert = 2;
        let mut heads = Cards::EMPTY;
        let bits = self.bits;
        let mut index = 63;
        while index > 0 {
            heads.bits |= (bits & (1 << index)) & insert;
            insert = !(bits & (1 << index)) >> 1u8;
            index -= 1;
        }
        heads
    }
}

#[cfg(feature = "dds")]
impl AsDDSCard for Card {
    fn as_card(&self) -> (dds::deal::DdsRank, dds::deal::DdsSuit) {
        use dds::deal::DdsRank;
        (
            DdsRank::new(i32::from(self.rank())).unwrap(),
            self.suit().into(),
        )
    }
}

impl std::ops::Add for Cards {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl std::ops::AddAssign for Cards {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

impl std::ops::Sub for Cards {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.difference(rhs)
    }
}

impl std::ops::SubAssign for Cards {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.difference(rhs);
    }
}

impl std::ops::BitAnd for Cards {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl std::ops::BitAndAssign for Cards {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}

impl IntoIterator for Cards {
    type Item = Card;
    type IntoIter = SuitIterator;

    fn into_iter(self) -> Self::IntoIter {
        SuitIterator { bits: self.bits }
    }
}

#[derive(Copy, Clone)]
pub struct SuitIterator {
    bits: u64,
}

impl SuitIterator {
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub fn len(&self) -> u8 {
        self.bits.count_ones() as u8
    }

    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

#[allow(clippy::cast_possible_truncation, clippy::copy_iterator)]
impl Iterator for SuitIterator {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.leading_zeros();
            self.bits &= !(1u64 << (63 - next));
            Some(Card {
                offset: (63 - next) as u8,
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len() as usize, Some(self.len() as usize))
    }

    fn count(self) -> usize {
        self.len() as usize
    }

    fn max(self) -> Option<Card> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.leading_zeros();
            Some(Card {
                offset: (63 - next) as u8,
            })
        }
    }

    fn min(self) -> Option<Card> {
        self.clone().next_back()
    }
}

impl std::fmt::Display for Cards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
            .iter()
            .copied()
        {
            let cards = self.in_suit(suit);
            if !cards.is_empty() {
                write!(f, "{}", suit.unicode())?;
                // SuitIterator yields high -> low (A,K,Q...), the conventional
                // bridge display order, so no `.rev()` here.
                for c in cards {
                    write!(f, "{}", c.rankchar())?;
                }
            }
        }
        Ok(())
    }
}

/// A string that is not a valid [`Card`] or [`Cards`].
///
/// Carries the offending input so the diagnostic names what was rejected.
/// `Card` and `Cards` share the type because they share the grammar.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct InvalidCardError {
    input: String,
}

impl InvalidCardError {
    fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
        }
    }

    /// The input that failed to parse.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.input
    }
}

impl std::fmt::Debug for InvalidCardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidCardError({:?})", self.input)
    }
}

impl std::fmt::Display for InvalidCardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid card \"{}\"", self.input)
    }
}

impl Error for InvalidCardError {}

/// The suit a designator names: ASCII either case, or the Unicode symbol.
const fn suit_designator(c: char) -> Option<Suit> {
    match c {
        '♠' | 'S' | 's' => Some(Suit::Spades),
        '♥' | 'H' | 'h' => Some(Suit::Hearts),
        '♦' | 'D' | 'd' => Some(Suit::Diamonds),
        '♣' | 'C' | 'c' => Some(Suit::Clubs),
        _ => None,
    }
}

/// The rank a designator names, ignoring `"10"`'s leading `'1'`.
const fn rank_designator(c: char) -> Option<u8> {
    match c {
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'T' | 't' => Some(10),
        'J' | 'j' => Some(11),
        'Q' | 'q' => Some(12),
        'K' | 'k' => Some(13),
        'A' | 'a' => Some(14),
        _ => None,
    }
}

/// One lexical token of the shared card grammar.
#[derive(Copy, Clone)]
enum CardToken {
    /// A suit designator.
    Suit(Suit),
    /// A positional separator (`' '` or `'.'`) advancing to the next suit.
    NextSuit,
    /// A rank.
    Rank(u8),
}

/// Splits input into [`CardToken`]s.
///
/// `Card` and `Cards` both consume this, so they classify every character the
/// same way and cannot drift.
struct CardTokenizer<'a> {
    chars: std::str::Chars<'a>,
    input: &'a str,
}

impl<'a> CardTokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            input,
        }
    }
}

impl Iterator for CardTokenizer<'_> {
    type Item = Result<CardToken, InvalidCardError>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.chars.next()?;
        let token = match c {
            ' ' | '.' => CardToken::NextSuit,
            // `'1'` is a rank only as part of `"10"`; a bare `'1'` is invalid.
            '1' => match self.chars.next() {
                Some('0') => CardToken::Rank(10),
                _ => return Some(Err(InvalidCardError::new(self.input))),
            },
            _ => match suit_designator(c) {
                Some(suit) => CardToken::Suit(suit),
                None => match rank_designator(c) {
                    Some(rank) => CardToken::Rank(rank),
                    None => return Some(Err(InvalidCardError::new(self.input))),
                },
            },
        };
        Some(Ok(token))
    }
}

impl std::str::FromStr for Card {
    type Err = InvalidCardError;

    /// Parses exactly one card as `suit rank` (`"C3"`, `"ST"`, `"S10"`, `"♠A"`).
    ///
    /// The grammar is strict: a rank-first string (`"3C"`), a bare rank
    /// (`"3"`) and a suit with no rank (`"C"`) are all rejected. The old
    /// lenient parser read those as the three of spades, the three of spades
    /// and the two of clubs respectively, which is the silent corruption this
    /// grammar removes.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut suit = None;
        let mut rank = None;
        for token in CardTokenizer::new(s) {
            match token? {
                CardToken::Suit(found) => {
                    if suit.is_some() || rank.is_some() {
                        return Err(InvalidCardError::new(s));
                    }
                    suit = Some(found);
                }
                CardToken::Rank(found) => {
                    if rank.is_some() {
                        return Err(InvalidCardError::new(s));
                    }
                    rank = Some(found);
                }
                CardToken::NextSuit => return Err(InvalidCardError::new(s)),
            }
        }
        match (suit, rank) {
            (Some(suit), Some(rank)) => Ok(Card::new(suit, rank)),
            _ => Err(InvalidCardError::new(s)),
        }
    }
}

impl std::str::FromStr for Cards {
    type Err = InvalidCardError;

    /// Parses a run of cards.
    ///
    /// The grammar accepts two equivalent shapes, both used in-crate:
    ///
    /// - **positional**: the suit starts at spades and a space or `.` advances
    ///   to the next suit, so `"AKQ AJT9 T3 AK95"` and `"AKQ2.KQ32.AK2.32"`
    ///   are four-suit holdings. A leading run of ranks is therefore spades.
    /// - **lettered**: an explicit suit designator (`S`/`H`/`D`/`C`, either
    ///   case, or the Unicode symbol) switches the current suit.
    ///
    /// Ranks (`2`-`9`, `T`/`10`, `J`, `Q`, `K`, `A`, either case) join the
    /// current suit. Any other character is an error rather than being
    /// silently skipped, and a fifth positional suit is an error.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cards = Cards::EMPTY;
        let mut suit = Suit::Spades;
        for token in CardTokenizer::new(s) {
            match token? {
                CardToken::Suit(found) => suit = found,
                CardToken::NextSuit => {
                    suit = suit.next().ok_or_else(|| InvalidCardError::new(s))?;
                }
                CardToken::Rank(rank) => {
                    cards = cards.insert(Card::new(suit, rank));
                }
            }
        }
        Ok(cards)
    }
}
impl From<u64> for Cards {
    fn from(value: u64) -> Self {
        let bits = value & Cards::ALL.bits;
        Cards { bits }
    }
}

#[allow(clippy::cast_possible_truncation)]
impl DoubleEndedIterator for SuitIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.trailing_zeros();
            self.bits &= !(1u64 << next);
            Some(Card { offset: next as u8 })
        }
    }
}

#[test]
fn heads_of_sequences_works() {
    let cards = Cards::DIAMONDS;
    assert_eq!(cards.heads_of_sequences(), Cards::EMPTY.insert(Card::DA));
    let cards = Cards::from_str("KQ986532").unwrap();
    assert_eq!(cards.heads_of_sequences(), Cards::from_str("K963").unwrap());
}

#[test]
fn high_card_points_single_pass_matches_classic() {
    use std::str::FromStr;
    // A/K/Q/J = 4/3/2/1; spot cards contribute 0.
    let hand = Cards::from_str("SAKQJHT987").unwrap();
    // S: A(4)+K(3)+Q(2)+J(1) = 10; H: T = 0.
    assert_eq!(hand.high_card_points(), 10);
    assert_eq!(Cards::EMPTY.high_card_points(), 0);
    assert_eq!(Cards::ALL.high_card_points(), 40);
}

#[test]
fn high_card_points_matches_reference_for_every_rank_layout() {
    fn reference(bits: u64) -> u8 {
        let mut points = 0;
        for bit in 0..64 {
            if bits & (1u64 << bit) != 0 {
                points += match bit & 15 {
                    14 => 4,
                    13 => 3,
                    12 => 2,
                    11 => 1,
                    _ => 0,
                };
            }
        }
        points
    }

    // Every possible rank nibble in every suit catches an off-by-one shift in
    // a mask-based implementation, including the non-card rank 15.
    for spades in 0..16_u64 {
        for hearts in 0..16_u64 {
            for diamonds in 0..16_u64 {
                for clubs in 0..16_u64 {
                    let bits = spades | (hearts << 16) | (diamonds << 32) | (clubs << 48);
                    assert_eq!(
                        Cards { bits }.high_card_points(),
                        reference(bits),
                        "bits {bits:#018x}"
                    );
                }
            }
        }
    }

    for cards in [
        Cards::ALL,
        Cards { bits: 1 << 63 },
        Cards { bits: u64::MAX },
    ] {
        assert_eq!(cards.high_card_points(), reference(cards.bits));
    }
}

#[test]
fn card_from_str_requires_a_suit_before_the_rank() {
    use std::str::FromStr;
    assert_eq!(
        Card::from_str("C3").expect("suit-first three of clubs"),
        Card::C3
    );
    assert_eq!(
        Card::from_str("S10").expect("\"10\" spells a ten"),
        Card::S10
    );
    assert_eq!(Card::from_str("ST").expect("T spells a ten"), Card::S10);
    assert_eq!(
        Card::from_str("♠A").expect("unicode suit designator"),
        Card::SA
    );
    // Suit and rank letters are case-insensitive, but the order is fixed.
    assert_eq!(
        Card::from_str("c3").expect("lowercase suit and rank"),
        Card::C3
    );

    for malformed in ["3C", "3", "C", "", "CS3", "S3C", "S1", "SX", "10S"] {
        assert!(
            Card::from_str(malformed).is_err(),
            "{malformed:?} should not parse"
        );
    }
}

#[test]
fn cards_from_str_keeps_the_positional_and_letter_forms() {
    use std::str::FromStr;
    // The leading positional group defaults to spades; a space advances one suit.
    let positional = Cards::from_str("AKQ AJT9 T3 AK95").expect("positional form");
    let letters = Cards::from_str("SAKQHAJT9DT3CAK95").expect("lettered form");
    assert_eq!(positional, letters);

    // `Hand`'s positional form uses `.` as the separator.
    let dotted = Cards::from_str("AKQ2.KQ32.AK2.32").expect("dotted form");
    assert_eq!(
        dotted,
        Cards::from_str("SAKQ2HKQ32DAK2C32").expect("lettered dotted equivalent")
    );

    // A bare run of ranks is spades, as the `heads_of_sequences` doc example.
    assert_eq!(
        Cards::from_str("KQ986532").expect("bare ranks default to spades"),
        Cards::from_str("SKQ986532").expect("explicit spades")
    );
}

#[test]
fn cards_from_str_rejects_unknown_characters_instead_of_ignoring_them() {
    use std::str::FromStr;
    for malformed in ["AKQ X", "SAKQ!", "1", "S12", "A B C D E"] {
        assert!(
            Cards::from_str(malformed).is_err(),
            "{malformed:?} should not parse"
        );
    }
}

#[test]
fn invalid_card_error_names_the_offending_input() {
    use std::str::FromStr;
    let error = Card::from_str("3C").expect_err("rank-first input is rejected");
    assert!(error.to_string().contains("3C"), "got {error}");
    assert_eq!(error.input(), "3C");

    let error = Cards::from_str("AKQ X").expect_err("unknown character is rejected");
    assert!(error.to_string().contains("AKQ X"), "got {error}");
    assert_eq!(error.input(), "AKQ X");
}
