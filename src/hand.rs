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

    pub fn with_range(&mut self, min_hcp: u8, max_hcp: u8) -> &mut Self {
        self.hcp_range = Some(HcpRange::new(min_hcp, max_hcp));
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
    let hand_type_1 = builder.with_range(8, 15).build();
    let hand_type_2 = builder.add_shape("4333").build();
}
