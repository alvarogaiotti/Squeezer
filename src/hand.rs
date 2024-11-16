// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;

///Represent a bridge hand: 13 cards, with different and various properties.
///The majority of this properties were implemented in the `bridge_deck` crate,
///github version.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Hand {
    pub cards: Cards,
}

impl Default for Hand {
    fn default() -> Self {
        Hand {
            cards: Cards::EMPTY,
        }
    }
}

impl Hand {
    /// Create a new Hand with 13 random cards from a deck.
    /// Returns the created Hand.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new() -> Self {
        Hand {
            cards: Cards::new_deck().pick(13).unwrap(),
        }
    }

    /// Create a new Hand with no cards.
    /// Returns the created Hand.
    #[must_use]
    pub const fn new_empty() -> Self {
        Hand {
            cards: Cards::EMPTY,
        }
    }

    /// Set the cards of the Hand to the specified cards.
    /// Parameters:
    /// - `cards`: The new set of cards for the Hand.
    pub(crate) fn set_cards(&mut self, cards: Cards) {
        self.cards = cards;
    }

    /// Check if the Hand contains a specific card.
    /// Returns true if the Hand contains the card, false otherwise.
    #[must_use]
    pub fn contains(&self, card: Card) -> bool {
        self.cards.contains(card)
    }

    /// Get the distribution of suits in the Hand.
    /// Returns an array representing the number of cards in each suit.
    #[must_use]
    pub const fn shape(&self) -> ShapePattern {
        let spades = self.slen();
        let hearts = self.hlen();
        let diamonds = self.dlen();
        let clubs = self.clen();
        [spades, hearts, diamonds, clubs]
    }

    /// Returns the number of cards in the specified suit.
    /// Parameters:
    /// - `suit`: The suit for which to count the cards.
    #[must_use]
    pub fn len_of_suit(&self, suit: Suit) -> u8 {
        match suit {
            Suit::Spades => self.slen(),
            Suit::Hearts => self.hlen(),
            Suit::Diamonds => self.dlen(),
            Suit::Clubs => self.clen(),
        }
    }

    /// Get the cards in the Spades suit.
    /// Returns the cards in the Spades suit.
    #[must_use]
    pub const fn spades(&self) -> Cards {
        self.cards.spades()
    }

    /// Get the cards in the Hearts suit.
    /// Returns the cards in the Hearts suit.
    #[must_use]
    pub const fn hearts(&self) -> Cards {
        self.cards.hearts()
    }

    /// Get the cards in the Diamonds suit.
    /// Returns the cards in the Diamonds suit.
    #[must_use]
    pub const fn diamonds(&self) -> Cards {
        self.cards.diamonds()
    }

    /// Get the cards in the Clubs suit.
    /// Returns the cards in the Clubs suit.
    #[must_use]
    pub const fn clubs(&self) -> Cards {
        self.cards.clubs()
    }

    /// Get the number of cards in the Spades suit.
    /// Returns the number of cards in the Spades suit.
    #[must_use]
    pub const fn slen(&self) -> u8 {
        self.spades().len()
    }

    /// Get the number of cards in the Hearts suit.
    /// Returns the number of cards in the Hearts suit.
    #[must_use]
    pub const fn hlen(&self) -> u8 {
        self.hearts().len()
    }

    /// Get the number of cards in the Diamonds suit.
    /// Returns the number of cards in the Diamonds suit.
    #[must_use]
    pub const fn dlen(&self) -> u8 {
        self.diamonds().len()
    }

    /// Get the number of cards in the Clubs suit.
    /// Returns the number of cards in the Clubs suit.
    #[must_use]
    pub const fn clen(&self) -> u8 {
        self.clubs().len()
    }

    /// Get the High Card Points (HCP) of the Hand.
    /// Returns the HCP value of the Hand.
    #[must_use]
    pub fn hcp(&self) -> u8 {
        self.cards.high_card_points()
    }

    /// Get the cards in the Hand as a set.
    /// Returns the cards in the Hand.
    #[must_use]
    pub fn as_cards(&self) -> Cards {
        self.cards
    }

    /// Get the cards in the Hand as bit representation.
    /// Returns the bit representation of the cards in the Hand.
    #[must_use]
    pub fn as_bits(&self) -> u64 {
        self.cards.as_bits()
    }

    /// Get a long string representation of the Hand.
    /// Returns a string representing the cards in the Hand in multiple lines.
    #[must_use]
    pub fn long_str(&self) -> String {
        format!("{}", self.into_iter().format("\n"))
    }
}

impl TryFrom<Cards> for Hand {
    type Error = DealerError;

    fn try_from(value: Cards) -> Result<Self, Self::Error> {
        if value.len() > 13 {
            Err(DealerError::new("too many cards in hand"))
        } else {
            Ok(Hand { cards: value })
        }
    }
}

impl FromStr for Hand {
    type Err = DealerError;

    fn from_str(hand: &str) -> Result<Hand, Self::Err> {
        let cards = Cards::from_str(hand)?;
        if cards.len() != 13 {
            return Err(DealerError::new(&format!(
                "wrong number of cards for a Bridge hand: {cards}"
            )));
        }
        Ok(Hand { cards })
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cards)
    }
}

impl IntoIterator for Hand {
    type Item = Cards;
    type IntoIter = HandIterator;

    fn into_iter(self) -> Self::IntoIter {
        HandIterator {
            hand: self.cards,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct HandIterator {
    hand: Cards,
    index: usize,
}

impl Iterator for HandIterator {
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
/// Represents a range of High Card Points.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HcpRange {
    min_hcp: u8,
    max_hcp: u8,
}

impl HcpRange {
    /// This function guarantees that the `HcpRange` created is possible.
    /// i.e. that the `min_hcp` is less than or equal to the `max_hcp`.
    /// and that the `max_hcp` is clamped to 37.
    #[must_use]
    pub const fn new(min_hcp: u8, max_hcp: u8) -> Self {
        let min_hcp = if min_hcp > 37 { 37 } else { min_hcp };
        let max_hcp = if max_hcp > 37 {
            37
        } else if max_hcp < min_hcp {
            min_hcp
        } else {
            max_hcp
        };
        Self { min_hcp, max_hcp }
    }

    /// Get the minimum value of the HCP range.
    #[must_use]
    pub const fn min(&self) -> u8 {
        self.min_hcp
    }

    /// Get the maximum value of the HCP range.
    #[must_use]
    pub const fn max(&self) -> u8 {
        self.max_hcp
    }

    /// Check if the specified HCP is within this range.
    #[must_use]
    pub const fn contains(&self, hcp: u8) -> bool {
        self.min_hcp <= hcp && self.max_hcp >= hcp
    }

    /// Get the range of HCP values as an inclusive range.
    #[must_use]
    pub const fn as_range(&self) -> RangeInclusive<u8> {
        self.min()..=self.max()
    }
}
impl Default for HcpRange {
    fn default() -> Self {
        HcpRange::new(0, 37)
    }
}

/// Represents a set of possible hands with the accepted shapes and the accepted HCP range.
/// This struct main goal is to express a single hand type that we can accept.
/// If you want to represent multiple hand types, you can use the `HandDescriptor` struct,
/// which embbeds multiple `HandType`s.
#[derive(Debug, Default, Clone)]
pub struct HandType {
    shape: Shape,
    hcp_range: HcpRange,
}

impl HandType {
    /// Create a new `HandType` with the specified shape and HCP range.
    #[must_use]
    pub const fn new(shape: Shape, hcp_range: HcpRange) -> Self {
        Self { shape, hcp_range }
    }

    /// Check if the `HandType` matches the given hand based on shape and HCP range.
    #[must_use]
    pub fn check(&self, hand: Hand) -> bool {
        self.shape.is_member(hand) && self.hcp_range.contains(hand.hcp())
    }

    /// Get the length ranges for each suit based on the accepted shapes.
    #[must_use]
    pub fn len_ranges(&self) -> [LenRange; 4] {
        self.shape.len_ranges()
    }

    /// Get the accepted HCP range for this `HandType`.
    #[must_use]
    pub fn hcp_range(&self) -> HcpRange {
        self.hcp_range
    }
}

/// Represents a descriptor for a set of possible hands with accepted shapes and High Card Point (HCP) ranges.
#[derive(Debug, Default)]
pub struct HandDescriptor {
    possible_hands: Vec<HandType>,
}

impl HandDescriptor {
    /// Check if a given hand matches any of the possible hand types based on shape and HCP range.
    #[must_use]
    pub fn check(&self, hand: Hand) -> bool {
        self.possible_hands
            .iter()
            .any(|hand_type| hand_type.check(hand))
    }

    /// Create a new `HandDescriptor` with the specified list of possible hand types.
    #[must_use]
    pub fn new(possible_hands: Vec<HandType>) -> Self {
        Self { possible_hands }
    }
}

#[derive(Default, Debug)]
pub struct HandTypeBuilder {
    shapes: Option<Shape>,
    hcp_range: Option<HcpRange>,
}

impl HandTypeBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            shapes: None,
            hcp_range: None,
        }
    }
    #[must_use]
    pub fn balanced(min_hcp: u8, max_hcp: u8) -> Self {
        let mut shapes = Shapes::new();
        shapes.add_balanced();
        Self {
            shapes: Some(Shape::Custom(shapes)),
            hcp_range: Some(HcpRange::new(min_hcp, max_hcp)),
        }
    }

    /// # Errors
    /// Errors if the shape is not parsable
    pub fn add_shape(&mut self, pattern: &str) -> Result<&mut Self, DealerError> {
        if let Some(ref mut shapes) = self.shapes {
            shapes.add_shape(pattern)?;
        } else {
            let mut shape = Shapes::new();
            shape.add_shape(pattern)?;
            self.shapes = Some(Shape::Custom(shape));
        }
        Ok(self)
    }
    /// # Errors
    /// Errors if the shape is not parsable
    pub fn remove_shape(&mut self, pattern: &str) -> Result<&mut Self, Box<dyn Error + 'static>> {
        if let Some(ref mut shapes) = self.shapes {
            shapes.remove_shape(pattern)?;
        } else {
            self.shapes = Some(Shape::Custom(Shapes::but(pattern)?));
        }
        Ok(self)
    }

    pub fn with_range(&mut self, min_hcp: u8, max_hcp: u8) -> &mut Self {
        self.hcp_range = Some(HcpRange::new(min_hcp, max_hcp));
        self
    }
    #[allow(clippy::missing_panics_doc)]
    /// # Errors
    /// Errors if the shape is not parsable
    pub fn with_longest(&mut self, suit: Suit) -> &mut Self {
        let shape = Shapes::new();
        self.shapes = Some(Shape::Custom(shape));

        match suit {
            Suit::Spades => {
                self.add_shape("55-4-4-").unwrap();
                self.add_shape("66-5-5").unwrap();
                self.add_shape("7+xxx").unwrap();
            }
            Suit::Hearts => {
                self.add_shape("4-55-5-").unwrap();
                self.add_shape("5-65-5-").unwrap();
                self.add_shape("x7+xx").unwrap();
            }
            Suit::Diamonds => {
                self.add_shape("4-4-55-").unwrap();
                self.add_shape("5-5-66-").unwrap();
                self.add_shape("xx7+x").unwrap();
            }
            Suit::Clubs => {
                self.add_shape("4-4-4-5").unwrap();
                self.add_shape("5-5-5-6").unwrap();
                self.add_shape("xxx7+").unwrap();
            }
        };
        self
    }

    pub fn build(&mut self) -> HandType {
        let shape = if let Some(shapes) = self.shapes.take() {
            shapes
        } else {
            Shape::All
        };
        let hcp_range = self.hcp_range.take().unwrap_or_default();
        HandType { shape, hcp_range }
    }
}

#[cfg(test)]
#[test]
fn test_builder_pattern() {
    let mut builder = HandTypeBuilder::new();
    _ = builder.with_range(8, 15).build();
    _ = builder.add_shape("4333").unwrap().build();
}
