/// All the code here come from David Roundy's library (https://github.com/droundy/bridge-cards),
/// whom I thank for his precious work. I basically copied the code to adapt it to my needs.
use crate::prelude::*;
use rand::Rng;

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

impl Card {
    pub const fn new(suit: Suit, rank: u8) -> Self {
        Card {
            offset: rank + 16 * suit as u8,
        }
    }
    pub const fn suit(self) -> Suit {
        match self.offset >> 4 {
            0 => Suit::Spades,
            1 => Suit::Hearts,
            2 => Suit::Diamonds,
            _ => Suit::Clubs,
        }
    }
    /// What is my rank?
    pub const fn rank(self) -> u8 {
        self.offset % 16
    }
    /// What is my rank called?
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
    /// What is my rank called?
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
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Cards {
    bits: u64,
}

impl Cards {
    pub const fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }
    pub const fn not_empty(&self) -> bool {
        self.bits != 0
    }
    pub const fn insert(self, card: Card) -> Self {
        Self {
            bits: self.bits | (1 << card.offset),
        }
    }
    pub const fn remove(self, card: Card) -> Self {
        Self {
            bits: self.bits & !(1 << card.offset),
        }
    }
    pub const fn contains(self, card: Card) -> bool {
        self.bits & (1 << card.offset) != 0
    }

    pub const fn union(&self, cards: Cards) -> Self {
        Self {
            bits: self.bits | cards.bits,
        }
    }

    pub const fn difference(&self, cards: Cards) -> Self {
        Self {
            bits: self.bits & !cards.bits,
        }
    }
    pub const fn in_suit(self, suit: Suit) -> Self {
        let offset = suit as i32 * 16;
        Cards {
            bits: ((self.bits >> offset) & 0xFFFF) << offset,
        }
    }

    pub fn pick(&mut self, num: usize) -> Option<Cards> {
        self.pick_rng(&mut rand::thread_rng(), num)
    }

    /// Randomly pick `num` cards to remove from the deck using specified RNG.
    /// Returns `None` only if there aren't enough cards.
    pub fn pick_rng<R: Rng>(&mut self, rng: &mut R, mut num: usize) -> Option<Cards> {
        let mut bits = self.bits;
        let mut n_left = self.len();
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
    const fn intersection(self, cards: Cards) -> Self {
        Cards {
            bits: self.bits & cards.bits,
        }
    }
    pub const ALL: Cards = Self::SPADES
        .union(Self::HEARTS)
        .union(Self::DIAMONDS)
        .union(Self::CLUBS);
    /// All club cards.
    pub const SPADES: Cards = Cards { bits: 0x7ffc };
    /// Just the spades from this hand
    pub const fn spades(self) -> Cards {
        self.intersection(Cards::SPADES)
    }
    /// All diamond cards.
    pub const HEARTS: Cards = Cards { bits: 0x7ffc << 16 };
    /// Just the hearts from this hand
    pub const fn hearts(self) -> Cards {
        self.intersection(Cards::HEARTS)
    }
    /// All diamonds cards.
    pub const DIAMONDS: Cards = Cards { bits: 0x7ffc << 32 };
    /// Just the diamonds from this hand
    pub const fn diamonds(self) -> Cards {
        self.intersection(Cards::DIAMONDS)
    }
    /// All clubs cards.
    pub const CLUBS: Cards = Cards { bits: 0x7ffc << 48 };
    /// Just the clubs from this hand
    pub const fn clubs(self) -> Cards {
        self.intersection(Cards::CLUBS)
    }

    /// A deck or hand with no cards in it.
    pub const EMPTY: Cards = Cards { bits: 0 };

    /// The aces
    pub const ACES: Cards = Cards::EMPTY
        .insert(Card::CA)
        .insert(Card::DA)
        .insert(Card::HA)
        .insert(Card::SA);
    /// Just the aces from this hand
    pub const fn aces(self) -> Cards {
        self.intersection(Cards::ACES)
    }
    /// The kings
    pub const KINGS: Cards = Cards::EMPTY
        .insert(Card::CK)
        .insert(Card::DK)
        .insert(Card::HK)
        .insert(Card::SK);
    /// Just the kins from this hand
    pub const fn kings(self) -> Cards {
        self.intersection(Cards::KINGS)
    }
    /// The queens
    pub const QUEENS: Cards = Cards::EMPTY
        .insert(Card::CQ)
        .insert(Card::DQ)
        .insert(Card::HQ)
        .insert(Card::SQ);
    /// Just the queens from this hand
    pub const fn queens(self) -> Cards {
        self.intersection(Cards::QUEENS)
    }
    /// The jacks
    pub const JACKS: Cards = Cards::EMPTY
        .insert(Card::CJ)
        .insert(Card::DJ)
        .insert(Card::HJ)
        .insert(Card::SJ);
    /// Just the jacks from this hand
    pub const fn jacks(self) -> Cards {
        self.intersection(Cards::JACKS)
    }
    /// The tens
    pub const TENS: Cards = Cards::EMPTY
        .insert(Card::C10)
        .insert(Card::D10)
        .insert(Card::H10)
        .insert(Card::S10);
    /// Just the tens from this hand
    pub const fn tens(self) -> Cards {
        self.intersection(Cards::TENS)
    }

    /// High card points
    pub const fn high_card_points(self) -> usize {
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
}

impl std::ops::Add for Cards {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl std::ops::AddAssign for Cards {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.union(rhs)
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
        *self = self.difference(rhs)
    }
}

impl std::ops::BitAnd for Cards {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl Iterator for Cards {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = self.bits.trailing_zeros();
            self.bits = self.bits & !(1 << next);
            Some(Card { offset: next as u8 })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

    fn count(self) -> usize {
        self.len()
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
            .cloned()
        {
            let cards = self.in_suit(suit);
            if cards.len() > 0 {
                write!(f, "{}", suit.unicode())?;
                for c in cards.rev() {
                    write!(f, "{}", c.rankchar())?;
                }
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for Cards {
    type Err = String;

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
impl DoubleEndedIterator for Cards {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let next = 63 - self.bits.leading_zeros();
            self.bits = self.bits & !(1 << next);
            Some(Card { offset: next as u8 })
        }
    }
}