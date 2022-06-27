use crate::prelude::*;

///Represent a bridge hand: 13 cards, with different and various properties.
///The majority of this properties were implemented in the bridge_deck crate,
///github version.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Hand {
    pub cards: Cards,
}

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

impl Hand {
    pub fn new() -> Self {
        let mut deck = Cards::ALL;
        Hand {
            cards: deck.pick(13).unwrap(),
        }
    }

    pub fn contains(&self, card: Card) -> bool {
        self.cards.contains(card)
    }
    pub fn shape(&self) -> Vec<u8> {
        let spades = self.cards.spades().len() as u8;
        let hearts = self.cards.hearts().len() as u8;
        let diamonds = self.cards.diamonds().len() as u8;
        let clubs = self.cards.clubs().len() as u8;
        vec![spades, hearts, diamonds, clubs]
    }

    pub fn spades(&self) -> Cards {
        self.cards.spades()
    }
    pub fn hearts(&self) -> Cards {
        self.cards.hearts()
    }
    pub fn diamonds(&self) -> Cards {
        self.cards.diamonds()
    }
    pub fn clubs(&self) -> Cards {
        self.cards.clubs()
    }
    pub fn slen(&self) -> usize {
        self.cards.spades().len()
    }
    pub fn hlen(&self) -> usize {
        self.cards.hearts().len()
    }
    pub fn dlen(&self) -> usize {
        self.cards.diamonds().len()
    }
    pub fn clen(&self) -> usize {
        self.cards.clubs().len()
    }
    pub fn hcp(&self) -> usize {
        self.cards.high_card_points()
    }
    pub fn from_str(hand: &str) -> Result<Hand, String> {
        Ok(Hand {
            cards: Cards::from_str(hand)?,
        })
    }
    pub fn as_bits(&self) -> u64 {
        self.cards
            .into_iter()
            .map(|x| 1 << (x.rank() + 16 * x.suit() as u8))
            .sum()
    }
    pub fn long_str(&self) -> String {
        format!("{}", self.into_iter().format("\n"))
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cards)
    }
}

impl<'a> IntoIterator for &'a Hand {
    type Item = Cards;
    type IntoIter = HandIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HandIterator {
            hand: &self.cards,
            index: 0,
        }
    }
}

///Iterator for the Hand struct. I couldn't do better ;(
#[derive(Debug)]
pub struct HandIterator<'a> {
    hand: &'a Cards,
    index: usize,
}

impl<'a> Iterator for HandIterator<'a> {
    type Item = Cards;
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => self.hand.spades(),
            1 => self.hand.hearts(),
            2 => self.hand.diamonds(),
            3 => self.hand.clubs(),
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}
