/// All the code here come from David Roundy's library [bridge-cards](https://github.com/droundy/bridge-cards),
/// I only slightly modified the code to adapt it to my needs.
use crate::prelude::*;
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

    ///Returns suit of the card.
    #[must_use]
    #[inline]
    pub const fn suit(self) -> Suit {
        match self.offset >> 4 {
            0 => Suit::Spades,
            1 => Suit::Hearts,
            2 => Suit::Diamonds,
            _ => Suit::Clubs,
        }
    }

    /// Returns the rank of the card.
    #[must_use]
    #[inline]
    pub const fn rank(self) -> u8 {
        self.offset % 16
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
}

/// A bunch of [`Card`]s.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Cards {
    bits: u64,
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
            bits: self.bits | (1 << card.offset),
        }
    }

    #[must_use]
    #[inline]
    /// NOTE: Removes, does not `pop`.
    pub const fn remove(self, card: Card) -> Self {
        Self {
            bits: self.bits & !(1 << card.offset),
        }
    }

    #[must_use]
    #[inline]
    /// Returns true if `card` is contained in the instance.
    pub const fn contains(self, card: Card) -> bool {
        self.bits & (1 << card.offset) != 0
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
        self.pick_rng(&mut rand::thread_rng(), num)
    }

    /// Randomly pick `num` cards to remove from the deck using specified RNG.
    /// Returns `None` only if there aren't enough cards.
    fn pick_rng<R: Rng>(&mut self, rng: &mut R, mut num: usize) -> Option<Cards> {
        let mut bits = self.bits;
        let mut n_left = self.len() as usize;
        if num > n_left {
            return None;
        }
        let mut kept = 0;
        let mut given = 0;
        while n_left > 0 {
            if num == 0 {
                kept |= bits;
                break;
            }
            let chosen = rng.gen::<u64>() & bits;
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
        self.bits = kept;
        Some(Cards { bits: given })
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
    #[must_use]
    #[inline]
    pub const fn high_card_points(self) -> u8 {
        self.aces().len()
            + self.intersection(Cards::ACES.union(Cards::KINGS)).len()
            + self
                .intersection(Cards::ACES.union(Cards::KINGS).union(Cards::QUEENS))
                .len()
            + self
                .intersection(
                    Cards::ACES
                        .union(Cards::KINGS)
                        .union(Cards::QUEENS)
                        .union(Cards::JACKS),
                )
                .len()
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
    pub fn heads_of_sequences(self) -> Self {
        let mut insert = 2;
        let mut heads = Cards::EMPTY;
        let bits = self.bits;
        for index in (1..64u64).rev() {
            heads.bits |= (bits & 1 << index) & insert;
            insert = !(bits & 1 << index) >> 1u8;
        }
        heads
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

#[derive(Clone)]
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

#[allow(clippy::cast_possible_truncation)]
impl Iterator for SuitIterator {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.leading_zeros();
            self.bits &= !(1 << (63 - next));
            Some(Card {
                offset: 63 - next as u8,
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
        self.clone().next()
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
                for c in cards.into_iter().rev() {
                    write!(f, "{}", c.rankchar())?;
                }
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for Card {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut suit = Suit::Spades;
        let mut card = Card::new(suit, 2);
        for c in s.chars() {
            match c {
                '♣' | 'C' | 'c' => {
                    suit = Suit::Clubs;
                }
                '♦' | 'D' | 'd' => {
                    suit = Suit::Diamonds;
                }
                '♥' | 'H' | 'h' => {
                    suit = Suit::Hearts;
                }
                '♠' | 'S' | 's' => {
                    suit = Suit::Spades;
                }
                '2' => {
                    card = Card::new(suit, 2);
                }
                '3' => {
                    card = Card::new(suit, 3);
                }
                '4' => {
                    card = Card::new(suit, 4);
                }
                '5' => {
                    card = Card::new(suit, 5);
                }
                '6' => {
                    card = Card::new(suit, 6);
                }
                '7' => {
                    card = Card::new(suit, 7);
                }
                '8' => {
                    card = Card::new(suit, 8);
                }
                '9' => {
                    card = Card::new(suit, 9);
                }
                'T' | '1' => {
                    card = Card::new(suit, 10);
                }
                'J' => {
                    card = Card::new(suit, 11);
                }
                'Q' => {
                    card = Card::new(suit, 12);
                }
                'K' => {
                    card = Card::new(suit, 13);
                }
                'A' => {
                    card = Card::new(suit, 14);
                }
                _ => (),
            }
        }
        Ok(card)
    }
}

impl std::str::FromStr for Cards {
    type Err = DealerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cards = Cards::EMPTY;
        let mut suit = Suit::Spades;
        for c in s.chars() {
            match c {
                '♣' | 'C' | 'c' => {
                    suit = Suit::Clubs;
                }
                '♦' | 'D' | 'd' => {
                    suit = Suit::Diamonds;
                }
                '♥' | 'H' | 'h' => {
                    suit = Suit::Hearts;
                }
                '♠' | 'S' | 's' => {
                    suit = Suit::Spades;
                }
                ' ' => {
                    suit = suit.next().ok_or(DealerError::new(
                        "more than 4 suits, check input string for spaces",
                    ))?;
                }
                '2' => {
                    cards = cards.insert(Card::new(suit, 2));
                }
                '3' => {
                    cards = cards.insert(Card::new(suit, 3));
                }
                '4' => {
                    cards = cards.insert(Card::new(suit, 4));
                }
                '5' => {
                    cards = cards.insert(Card::new(suit, 5));
                }
                '6' => {
                    cards = cards.insert(Card::new(suit, 6));
                }
                '7' => {
                    cards = cards.insert(Card::new(suit, 7));
                }
                '8' => {
                    cards = cards.insert(Card::new(suit, 8));
                }
                '9' => {
                    cards = cards.insert(Card::new(suit, 9));
                }
                'T' | '1' => {
                    cards = cards.insert(Card::new(suit, 10));
                }
                'J' => {
                    cards = cards.insert(Card::new(suit, 11));
                }
                'Q' => {
                    cards = cards.insert(Card::new(suit, 12));
                }
                'K' => {
                    cards = cards.insert(Card::new(suit, 13));
                }
                'A' => {
                    cards = cards.insert(Card::new(suit, 14));
                }
                _ => (),
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
            let next = 63 - self.bits.leading_zeros();
            self.bits &= !(1 << next);
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
