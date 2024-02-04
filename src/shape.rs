use crate::prelude::*;
use crate::shapeparser::*;
use bitvec::{bitarr, BitArr};

const R: u64 = 14;
const M: u64 = 1379;
const SHAPE_TABLE_BUCKETS: usize = M as usize;

#[derive(Default)]
struct ShapeHasher {
    state: u64,
}

impl std::hash::Hasher for ShapeHasher {
    fn write(&mut self, bytes: &[u8]) {
        // We reset so we are guaranteed to get always the same result
        // when hashing multiple shapes with the same hasher.
        self.state = 0;
        for &byte in bytes {
            self.state = (R * self.state + u64::from(byte)) % M
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

type ShapeTable = BitArr!(for SHAPE_TABLE_BUCKETS);

#[derive(Debug, Clone)]
pub enum Shape {
    Custom(Shapes),
    All,
}

impl Default for Shape {
    fn default() -> Self {
        Self::All
    }
}

impl Shape {
    pub fn new_from_pattern(pattern: &str) -> Result<Self, DealerError> {
        todo!()
    }

    pub fn remove_shape(&mut self, pattern: &str) -> Result<(), DealerError> {
        match self {
            Self::Custom(ref mut shape) => shape.remove_shape(pattern),
            Self::All => {
                *self = Self::Custom(Shapes::but(pattern)?);
                Ok(())
            }
        }
    }
    pub fn add_shape(&mut self, pattern: &str) -> Result<(), DealerError> {
        match self {
            Self::Custom(ref mut shape) => shape.add_shape(pattern),
            Self::All => Ok(()),
        }
    }
    #[must_use]
    #[inline]
    pub fn is_member(&self, hand_to_match: &Hand) -> bool {
        match self {
            Self::Custom(shape) => shape.is_member(hand_to_match),
            Self::All => true,
        }
    }

    #[inline]
    pub fn len_ranges(&self) -> [LenRange; 4] {
        match self {
            Self::Custom(shape) => shape.len_ranges(),
            Self::All => [LenRange::default(); 4],
        }
    }
}

// Struct that represents multiple shapes.
#[derive(Clone)]
pub struct Shapes {
    shape_table: Box<ShapeTable>,
    min_ls: [u8; SUITS],
    max_ls: [u8; SUITS],
}

impl std::fmt::Debug for Shapes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shapes")
            .field("min_ls", &self.min_ls)
            .field("max_ls", &self.max_ls)
            .finish()
    }
}

impl std::ops::Index<usize> for Shapes {
    type Output = bool;
    fn index(&self, index: usize) -> &Self::Output {
        &self.shape_table[index]
    }
}

impl Default for Shapes {
    fn default() -> Self {
        Self::new()
    }
}

impl Shapes {
    #[must_use]
    pub fn new() -> Self {
        Self {
            shape_table: Box::new(bitarr!(0; SHAPE_TABLE_BUCKETS)),
            min_ls: [ZERO_LENGTH; SUITS],
            max_ls: [MAX_LENGTH; SUITS],
        }
    }

    pub fn shape_pattern_to_index(pattern: [u8; 4]) -> usize {
        let mut state = 0;
        for byte in pattern {
            state = (R * state + u64::from(byte)) % M
        }
        state as usize
    }

    #[must_use]
    fn is_member(&self, hand_to_match: &Hand) -> bool {
        self.shape_table[Self::shape_pattern_to_index(hand_to_match.shape())]
    }
    pub fn remove_shape(&mut self, shape: &str) -> Result<(), DealerError> {
        // Take shape pattern. Right now we match on equal enums, but I'll probably change
        // implementation in the future so I'll keep it here for future use.

        let patterns = ShapeCreator::build_shape(shape)?;
        for pattern in patterns {
            self.delete_shape(pattern.try_into().unwrap())?;
        }
        Ok(())
    }

    fn all(&mut self) -> &mut Self {
        self.shape_table.fill(true);
        self
    }

    fn new_filled() -> Self {
        Self {
            shape_table: Box::new(bitarr!(1; SHAPE_TABLE_BUCKETS)),
            min_ls: [0; 4],
            max_ls: [13; 4],
        }
    }

    pub fn but(shape: &str) -> Result<Self, DealerError> {
        let mut table = Self::new_filled();
        table.remove_shape(shape)?;
        Ok(table)
    }

    pub fn add_shape(&mut self, shape: &str) -> Result<(), DealerError> {
        // Take shape pattern. Right now we match on equal enums, but I'll probably change
        // implementation in the future so I'll keep it here for future use.
        let patterns = ShapeCreator::build_shape(shape)?;
        for pattern in patterns {
            self.insert_shape(pattern.try_into().unwrap())?;
        }
        Ok(())
    }

    fn insert_shape(&mut self, shape: ShapePattern) -> Result<(), DealerError> {
        // let safe = true; // used by redeal, don't know exactly what its purpose is.
        self.shape_table
            .set(Self::shape_pattern_to_index(shape), true);
        Ok(())
    }

    fn delete_shape(&mut self, shape: ShapePattern) -> Result<(), DealerError> {
        self.shape_table
            .set(Self::shape_pattern_to_index(shape), false);
        Ok(())
    }

    #[must_use]
    pub fn flatten(pattern_descriptor: &[u8]) -> usize {
        let (s, h, d, c) = pattern_descriptor
            .iter()
            .map(|&x| x as usize)
            .next_tuple()
            .unwrap();
        ((((s * (RANKS + 1) as usize + h) * (RANKS + 1) as usize) + d) * (RANKS + 1) as usize) + c
    }
    #[must_use]
    pub fn hash_flatten(pattern_descriptor: &[u8]) -> usize {
        let mut hasher = ShapeHasher::default();
        hasher.write(pattern_descriptor);
        hasher.finish() as usize
    }

    fn update_based_on_length(&mut self) {
        let (rangespades, rangehearts, rangediamonds, rangeclubs) = self
            .min_ls
            .iter()
            .zip(self.max_ls.iter())
            .map(|(&min, &max)| RangeInclusive::new(min, max))
            .next_tuple()
            .unwrap();
        for shape in itertools::iproduct!(rangespades, rangehearts, rangediamonds, rangeclubs)
            .filter(|(s, h, d, c)| s + h + d + c == MAX_LENGTH)
            .map(ShapePattern::from)
        {
            self.shape_table
                .set(Self::shape_pattern_to_index(shape), true);
        }
    }
    #[must_use]
    pub fn len_ranges(&self) -> [LenRange; 4] {
        let mut len_range = [LenRange::default(); 4];
        for (index, (min, max)) in self.min_ls.iter().zip(self.max_ls.iter()).enumerate() {
            len_range[index] = LenRange::new(*min, *max);
        }
        len_range
    }
}

impl From<&[LenRange; SUITS]> for Shapes {
    fn from(len_range: &[LenRange; SUITS]) -> Self {
        let mut min_ls = [ZERO_LENGTH; SUITS];
        let mut max_ls = [MAX_LENGTH; SUITS];
        for &suit in Suit::ALL {
            min_ls[suit as usize] = len_range[suit as usize].min;
            max_ls[suit as usize] = len_range[suit as usize].max;
        }
        let mut shapes = Self {
            min_ls,
            max_ls,
            ..Default::default()
        };
        shapes.update_based_on_length();
        shapes
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LenRange {
    pub min: u8,
    pub max: u8,
}

impl Default for LenRange {
    fn default() -> Self {
        Self::new(ZERO_LENGTH, MAX_LENGTH)
    }
}

impl LenRange {
    #[must_use]
    pub fn new(min: u8, max: u8) -> Self {
        let max = max.clamp(ZERO_LENGTH, MAX_LENGTH);
        let min = min.clamp(ZERO_LENGTH, max);
        Self { min, max }
    }
    #[must_use]
    pub fn as_range(&self) -> RangeInclusive<u8> {
        self.min()..=self.max()
    }
    #[must_use]
    pub fn min(&self) -> u8 {
        self.min
    }
    #[must_use]
    pub fn max(&self) -> u8 {
        self.max
    }
    #[must_use]
    pub fn contains(&self, length: u8) -> bool {
        self.as_range().contains(&length)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
}

impl std::fmt::Debug for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Suit::Clubs => write!(f, "C"),
            Suit::Diamonds => write!(f, "D"),
            Suit::Hearts => write!(f, "H"),
            Suit::Spades => write!(f, "S"),
        }
    }
}

impl std::convert::From<Suit> for usize {
    fn from(suit: Suit) -> usize {
        suit as usize
    }
}

impl Suit {
    /// The name of the suit
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Suit::Clubs => "Clubs",
            Suit::Diamonds => "Diamonds",
            Suit::Hearts => "Hearts",
            Suit::Spades => "Spades",
        }
    }
    /// The unicode character for this suit
    #[must_use]
    pub fn unicode(self) -> char {
        match self {
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
            Suit::Spades => '♠',
        }
    }
    /// The latin character for this suit
    #[must_use]
    pub fn latin(self) -> char {
        match self {
            Suit::Clubs => 'C',
            Suit::Diamonds => 'D',
            Suit::Hearts => 'H',
            Suit::Spades => 'S',
        }
    }
    pub fn iter() -> impl Iterator<Item = Suit> {
        [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
            .iter()
            .copied()
    }
    /// All four suits from lowest to highest
    pub const ALL: &'static [Self] = &[Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

    /// Next suit.
    pub fn next(self) -> Result<Self, String> {
        match self {
            Suit::Spades => Ok(Suit::Hearts),
            Suit::Hearts => Ok(Suit::Diamonds),
            Suit::Diamonds => Ok(Suit::Clubs),
            Suit::Clubs => Err(String::from("Called Suit::Clubs.next() is not permitted.")),
        }
    }
}

#[test]
fn shape_creation_test() {
    let mut shapes = Shapes::new();
    shapes.add_shape("4333").unwrap();
    assert!(shapes.shape_table[Shapes::hash_flatten(&[4, 3, 3, 3])]);
}
#[test]
#[should_panic]
fn shape_error_unbal_parentheses_test() {
    Shapes::new().add_shape("4(333").unwrap();
}

#[test]
fn shape_parens_interpretation_working_test() {
    let mut factory = Shapes::new();
    factory.add_shape("4(34)2").unwrap();
    factory.add_shape("(6331)").unwrap();

    let mut true_arr = Vec::<usize>::new();
    for (i, data) in factory.shape_table.iter().enumerate() {
        if *data {
            true_arr.push(i)
        }
    }
    let mut ok_shapes = vec![
        Shapes::hash_flatten(&[6, 3, 1, 3]),
        Shapes::hash_flatten(&[6, 3, 3, 1]),
        Shapes::hash_flatten(&[6, 1, 3, 3]),
        Shapes::hash_flatten(&[3, 6, 1, 3]),
        Shapes::hash_flatten(&[3, 6, 3, 1]),
        Shapes::hash_flatten(&[1, 6, 3, 3]),
        Shapes::hash_flatten(&[1, 3, 6, 3]),
        Shapes::hash_flatten(&[3, 3, 6, 1]),
        Shapes::hash_flatten(&[3, 1, 6, 3]),
        Shapes::hash_flatten(&[3, 3, 1, 6]),
        Shapes::hash_flatten(&[3, 1, 3, 6]),
        Shapes::hash_flatten(&[1, 3, 3, 6]),
        Shapes::hash_flatten(&[4, 4, 3, 2]),
        Shapes::hash_flatten(&[4, 3, 4, 2]),
    ];
    ok_shapes.sort();
    assert_eq!(ok_shapes, true_arr);
}

#[test]
fn membership_shape_hand_test() {
    let mut factory = Shapes::new();
    factory.add_shape("4(34)2").unwrap();
    let deck = Cards::ALL;
    let clubs = deck.clubs().pick(2).unwrap();
    let diamonds = deck.diamonds().pick(4).unwrap();
    let hearts = deck.hearts().pick(3).unwrap();
    let spades = deck.spades().pick(4).unwrap();
    let cards = Cards::EMPTY
        .union(spades)
        .union(clubs)
        .union(diamonds)
        .union(hearts);

    let hand = Hand { cards };
    assert!(factory.is_member(&hand));
}
#[test]
fn shapes_from_len_range_test() {
    let range_len = [
        LenRange::new(5, 5),
        LenRange::new(2, 3),
        LenRange::new(2, 3),
        LenRange::new(2, 3),
    ];
    let shapes = Shapes::from(&range_len);
    let mut true_arr = Vec::<usize>::new();
    for (i, data) in shapes.shape_table.iter().enumerate() {
        if *data {
            true_arr.push(i)
        }
    }
    let mut ok_shapes = vec![
        Shapes::hash_flatten(&[5, 2, 3, 3]),
        Shapes::hash_flatten(&[5, 3, 2, 3]),
        Shapes::hash_flatten(&[5, 3, 3, 2]),
    ];
    ok_shapes.sort();
    assert_eq!(ok_shapes, true_arr);
}
#[test]
fn jokers_correct_behaviour_test() {
    let mut factory = Shapes::new();
    factory.add_shape("3xx2").unwrap();
    let deck = Cards::ALL;
    let clubs = deck.clubs().pick(2).unwrap();
    let diamonds = deck.diamonds().pick(4).unwrap();
    let hearts = deck.hearts().pick(4).unwrap();
    let spades = deck.spades().pick(3).unwrap();
    let cards = Cards::EMPTY
        .union(spades)
        .union(clubs)
        .union(diamonds)
        .union(hearts);

    let hand = Hand { cards };
    assert!(factory.is_member(&hand));
}
#[test]
fn can_remove_correct_shapes() {
    let deck = Cards::ALL;
    let clubs = deck.clubs().pick(2).unwrap();
    let diamonds = deck.diamonds().pick(5).unwrap();
    let hearts = deck.hearts().pick(3).unwrap();
    let spades = deck.spades().pick(3).unwrap();
    let cards = Cards::EMPTY
        .union(spades)
        .union(clubs)
        .union(diamonds)
        .union(hearts);

    let hand = Hand { cards };
    let mut factory = Shapes::new();
    factory.add_shape("3xx2").unwrap();
    assert!(factory.is_member(&hand));
    factory.remove_shape("3352").unwrap();
    assert!(!factory.is_member(&hand));
}
