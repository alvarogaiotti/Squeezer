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
    #[must_use]
    pub fn new() -> Self {
        Hand {
            cards: Cards::new_deck().pick(13).unwrap(),
        }
    }
    #[must_use]
    pub const fn new_empty() -> Self {
        Hand {
            cards: Cards::EMPTY,
        }
    }

    pub fn set_cards(&mut self, cards: Cards) {
        self.cards = cards;
    }

    #[must_use]
    pub fn contains(&self, card: Card) -> bool {
        self.cards.contains(card)
    }
    #[must_use]
    pub fn shape(&self) -> [u8; 4] {
        let spades = self.slen();
        let hearts = self.hlen();
        let diamonds = self.dlen();
        let clubs = self.clen();
        [spades, hearts, diamonds, clubs]
    }

    #[must_use]
    pub fn len_of_suit(&self, suit: Suit) -> u8 {
        match suit {
            Suit::Spades => self.slen(),
            Suit::Hearts => self.hlen(),
            Suit::Diamonds => self.dlen(),
            Suit::Clubs => self.clen(),
        }
    }
    #[must_use]
    pub fn spades(&self) -> Cards {
        self.cards.spades()
    }
    #[must_use]
    pub fn hearts(&self) -> Cards {
        self.cards.hearts()
    }
    #[must_use]
    pub fn diamonds(&self) -> Cards {
        self.cards.diamonds()
    }
    #[must_use]
    pub fn clubs(&self) -> Cards {
        self.cards.clubs()
    }
    #[must_use]
    pub fn slen(&self) -> u8 {
        self.spades().len()
    }
    #[must_use]
    pub fn hlen(&self) -> u8 {
        self.hearts().len()
    }
    #[must_use]
    pub fn dlen(&self) -> u8 {
        self.diamonds().len()
    }
    #[must_use]
    pub fn clen(&self) -> u8 {
        self.clubs().len()
    }

    #[must_use]
    pub fn hcp(&self) -> u8 {
        self.cards.high_card_points()
    }
    #[must_use]
    pub fn as_cards(&self) -> Cards {
        self.cards
    }
    #[must_use]
    pub fn as_bits(&self) -> u64 {
        self.cards.as_bits()
    }
    #[must_use]
    pub fn long_str(&self) -> String {
        format!("{}", self.into_iter().format("\n"))
    }
}

impl FromStr for Hand {
    type Err = String;

    fn from_str(hand: &str) -> Result<Hand, Self::Err> {
        let cards = Cards::from_str(hand)?;
        if cards.len() != 13 {
            return Err("Wrong number of cards for a Bridge hand!".to_string());
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HcpRange {
    min_hcp: u8,
    max_hcp: u8,
}

impl HcpRange {
    #[must_use]
    pub fn new(min_hcp: u8, max_hcp: u8) -> Self {
        let max_hcp = max_hcp.clamp(ZERO_HCP, MAX_HCP_IN_HAND);
        let min_hcp = min_hcp.clamp(ZERO_HCP, max_hcp);
        Self { min_hcp, max_hcp }
    }

    #[must_use]
    pub fn min(&self) -> u8 {
        self.min_hcp
    }
    #[must_use]
    pub fn max(&self) -> u8 {
        self.max_hcp
    }

    #[must_use]
    pub fn contains(&self, hcp: u8) -> bool {
        self.as_range().contains(&hcp)
    }
    #[must_use]
    pub fn as_range(&self) -> RangeInclusive<u8> {
        self.min()..=self.max()
    }
}

impl Default for HcpRange {
    fn default() -> Self {
        HcpRange::new(0, 37)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct HandType {
    shape: Shapes,
    hcp_range: HcpRange,
}

impl HandType {
    #[must_use]
    pub fn new(shape: Shapes, hcp_range: HcpRange) -> Self {
        Self { shape, hcp_range }
    }

    #[must_use]
    pub fn check(&self, hand: Hand) -> bool {
        self.shape.is_member(hand) && self.hcp_range.contains(hand.hcp())
    }
    #[must_use]
    pub fn len_ranges(&self) -> [LenRange; 4] {
        self.shape.len_ranges()
    }
    #[must_use]
    pub fn hcp_range(&self) -> HcpRange {
        self.hcp_range
    }
}

#[derive(Debug, Default)]
pub struct HandDescriptor {
    possible_hands: Vec<HandType>,
}

impl HandDescriptor {
    #[must_use]
    pub fn check(&self, hand: Hand) -> bool {
        self.possible_hands
            .iter()
            .any(|hand_archetype| hand_archetype.check(hand))
    }
    #[must_use]
    pub fn new(possible_hands: Vec<HandType>) -> Self {
        Self { possible_hands }
    }
}

#[derive(Default, Debug)]
pub struct HandTypeBuilder {
    shapes: Option<Shapes>,
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
        let mut new = Self {
            shapes: Some(Shapes::new()),
            hcp_range: Some(HcpRange::new(min_hcp, max_hcp)),
        };
        new.add_shape("(5332)");
        new.add_shape("(3334)");
        new.add_shape("(4234)");
        new
    }

    pub fn add_shape(&mut self, pattern: &str) -> &mut Self {
        if let Some(shapes) = &mut self.shapes {
            shapes.add_shape(ShapeDescriptor::from_string(pattern));
        } else {
            let mut shape = Shapes::new();
            shape.add_shape(ShapeDescriptor::from_string(pattern));
            self.shapes = Some(shape);
        }
        self
    }
    pub fn remove_shape(&mut self, pattern: &str) -> &mut Self {
        if let Some(shapes) = &mut self.shapes {
            shapes.remove_shape(ShapeDescriptor::from_string(pattern));
        } else {
            let mut shape = Shapes::new();
            shape.remove_shape(ShapeDescriptor::from_string(pattern));
            self.shapes = Some(shape);
        }
        self
    }

    pub fn with_range(&mut self, min_hcp: u8, max_hcp: u8) -> &mut Self {
        self.hcp_range = Some(HcpRange::new(min_hcp, max_hcp));
        self
    }
    pub fn with_longest(&mut self, suit: Suit) -> &mut Self {
        let shape = Shapes::new();
        self.shapes = Some(shape);
        match suit {
            Suit::Spades => {
                self.add_shape("5xxx");
                self.add_shape("6xxx");
                self.remove_shape("x(6xx)");
                self.remove_shape("x(7xx)");
                self.remove_shape("x(8xx)");
                self.add_shape("7xxx");
                self.add_shape("8xxx");
                self.add_shape("9xxx");
                self.add_shape("x(300)");
                self.add_shape("x(111)");
                self.add_shape("x(210)");
                self.add_shape("x(110)");
                self.add_shape("x(002)");
                self.add_shape("x(100)");
                self.add_shape("x000");
            }
            Suit::Hearts => {
                self.add_shape("x5xx");
                self.add_shape("x6xx");
                self.remove_shape("6x(xx)");
                self.remove_shape("xx(6x)");
                self.remove_shape("7x(xx)");
                self.remove_shape("xx(7x)");
                self.remove_shape("8x(xx)");
                self.remove_shape("xx(8x)");
                self.add_shape("x7xx");
                self.add_shape("x8xx");
                self.add_shape("x9xx");
                self.add_shape("0x(30)");
                self.add_shape("3x(00)");
                self.add_shape("2x(01)");
                self.add_shape("0x(21)");
                self.add_shape("1x(20)");
                self.add_shape("1x(11)");
                self.add_shape("0x(02)");
                self.add_shape("2x(00)");
                self.add_shape("0x(11)");
                self.add_shape("1x(10)");
                self.add_shape("1x(00)");
                self.add_shape("0x(01)");
                self.add_shape("0x00");
            }
            Suit::Diamonds => {
                self.add_shape("xx5x");
                self.add_shape("xx6x");
                self.remove_shape("(xx)x6");
                self.remove_shape("(6x)xx");
                self.remove_shape("(7x)xx");
                self.remove_shape("xxx7");
                self.remove_shape("(8x)xx");
                self.remove_shape("xxx8");
                self.add_shape("xx7x");
                self.add_shape("xx8x");
                self.add_shape("xx9x");
                self.add_shape("(30)x0");
                self.add_shape("(0x)x3");
                self.add_shape("(01)x2");
                self.add_shape("(21)x0");
                self.add_shape("(20)x1");
                self.add_shape("11x1");
                self.add_shape("(10)x1");
                self.add_shape("11x0");
                self.add_shape("(02)x0");
                self.add_shape("(00)x2");
                self.add_shape("(00)x1");
                self.add_shape("(01)x0");
                self.add_shape("00x0");
            }
            Suit::Clubs => {
                self.add_shape("xxx5");
                self.add_shape("xxx6");
                self.remove_shape("(xx6)x");
                self.remove_shape("(7xx)x");
                self.remove_shape("(8xx)x");
                self.add_shape("xxx7");
                self.add_shape("xxx8");
                self.add_shape("xxx9");
                self.add_shape("(300)x");
                self.add_shape("(012)x");
                self.add_shape("111x");
                self.add_shape("(101)x");
                self.add_shape("(020)x");
                self.add_shape("(001)x");
                self.add_shape("000x");
            }
        };
        self
    }

    pub fn build(&mut self) -> HandType {
        let shape = if let Some(shapes) = self.shapes {
            shapes
        } else {
            Shapes::ALL
        };
        let hcp_range = if let Some(hcp_range) = self.hcp_range {
            hcp_range
        } else {
            HcpRange::default()
        };
        HandType { shape, hcp_range }
    }
}

#[cfg(test)]
#[test]
fn test_builder_pattern() {
    let mut builder = HandTypeBuilder::new();
    _ = builder.with_range(8, 15).build();
    _ = builder.add_shape("4333").build();
}
