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
        Hand {
            cards: Cards::EMPTY,
        }
    }
}

impl Hand {
    pub fn new() -> Self {
        let mut deck = Cards::ALL;
        Hand {
            cards: deck.pick(13).unwrap(),
        }
    }

    pub fn set_cards(&mut self, cards: &Cards) {
        self.cards = *cards;
    }

    pub fn contains(&self, card: Card) -> bool {
        self.cards.contains(card)
    }
    pub fn shape(&self) -> [u8; 4] {
        let spades = self.cards.spades().len() as u8;
        let hearts = self.cards.hearts().len() as u8;
        let diamonds = self.cards.diamonds().len() as u8;
        let clubs = self.cards.clubs().len() as u8;
        [spades, hearts, diamonds, clubs]
    }

    pub fn len_of_suit(&self, suit: Suit) -> usize {
        match suit {
            Suit::Spades => self.slen(),
            Suit::Hearts => self.hlen(),
            Suit::Diamonds => self.dlen(),
            Suit::Clubs => self.clen(),
        }
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
        self.spades().len()
    }
    pub fn hlen(&self) -> usize {
        self.hearts().len()
    }
    pub fn dlen(&self) -> usize {
        self.diamonds().len()
    }
    pub fn clen(&self) -> usize {
        self.clubs().len()
    }
    pub fn hcp(&self) -> usize {
        self.cards.high_card_points()
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HcpRange {
    min_hcp: u8,
    max_hcp: u8,
}

impl HcpRange {
    pub fn new(min_hcp: u8, max_hcp: u8) -> Self {
        Self { min_hcp, max_hcp }
    }

    pub fn min(&self) -> u8 {
        self.min_hcp
    }
    pub fn max(&self) -> u8 {
        self.max_hcp
    }

    pub fn check(&self, hand: &Hand) -> bool {
        let hcp = hand.hcp();
        self.min_hcp <= hcp as u8 && self.max_hcp >= hcp as u8
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
    pub fn new(shape: Shapes, hcp_range: HcpRange) -> Self {
        Self { shape, hcp_range }
    }

    pub fn check(&self, hand: &Hand) -> bool {
        self.shape.is_member(hand) && self.hcp_range.check(hand)
    }
    pub fn len_ranges(&self) -> [LenRange; 4] {
        self.shape.len_ranges()
    }
    pub fn hcp_range(&self) -> HcpRange {
        self.hcp_range
    }
}

#[derive(Debug, Default)]
pub struct HandDescriptor {
    possible_hands: Vec<HandType>,
}

impl HandDescriptor {
    pub fn check(&self, hand: &Hand) -> bool {
        self.possible_hands
            .iter()
            .any(|hand_archetype| hand_archetype.check(hand))
    }
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
    pub fn new() -> Self {
        Self {
            shapes: None,
            hcp_range: None,
        }
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
        let shape = if let Some(shapes) = self.shapes.take() {
            shapes
        } else {
            Shapes::ALL
        };
        let hcp_range = if let Some(hcp_range) = self.hcp_range {
            hcp_range
        } else {
            HcpRange::default()
        };
        self.shapes = None;
        self.hcp_range = None;
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
