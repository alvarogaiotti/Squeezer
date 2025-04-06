// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;
use crate::shapeparser::{ShapeCreator, ShapePattern};
use bitvec::{bitarr, BitArr};

#[allow(clippy::cast_possible_truncation)]
const SHAPE_TABLE_BUCKETS: usize = ShapeHasher::M as usize;

#[derive(Default)]
struct ShapeHasher {
    state: u64,
}

impl ShapeHasher {
    const fn new() -> Self {
        Self { state: 0 }
    }

    const R: u64 = 14;
    const M: u64 = 1379;
}

impl std::hash::Hasher for ShapeHasher {
    fn write(&mut self, bytes: &[u8]) {
        // We reset so we are guaranteed to get always the same result
        // when hashing multiple shapes with the same hasher.
        self.state = 0;
        for &byte in bytes {
            self.state = (Self::R * self.state + u64::from(byte)) % Self::M;
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

/// Type alias for the storage table of shapes.
/// We use a bit array to store the shapes so they are efficient memory wise.
/// The shape table is a hash table with a fixed size.
type ShapeTable = BitArr!(for SHAPE_TABLE_BUCKETS);

/// Public enum representing a Shape. I'll probably add something more
/// fine tuned later, for now we discriminate just based on whether we need
/// all the shapes or just some subset of them.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Shape {
    Custom(Shapes),
    All,
}

impl Default for Shape {
    fn default() -> Self {
        Self::All
    }
}

impl FromStr for Shape {
    type Err = DealerError;

    fn from_str(pattern: &str) -> Result<Self, Self::Err> {
        Shape::new_from_pattern(pattern)
    }
}

impl Shape {
    /// # Errors
    /// When the pattern is not correct
    pub fn new_from_pattern(pattern: &str) -> Result<Self, DealerError> {
        let mut shape = Shapes::new();
        shape.add_shape(pattern)?;
        Ok(Self::Custom(shape))
    }

    #[must_use]
    pub fn new_empty() -> Self {
        Self::Custom(Shapes::new())
    }

    /// # Errors
    /// When one of the pattern is not correct
    pub fn new_from_patterns(patterns: &[&str]) -> Result<Self, DealerError> {
        let mut shape = Shapes::new();
        for pattern in patterns {
            shape.add_shape(pattern)?;
        }
        Ok(Self::Custom(shape))
    }

    /// # Errors
    /// When the pattern is not correct
    pub fn remove_shape(&mut self, pattern: &str) -> Result<(), DealerError> {
        match *self {
            Self::Custom(ref mut shape) => shape.remove_shape(pattern),
            Self::All => {
                *self = Self::Custom(Shapes::but(pattern)?);
                Ok(())
            }
        }
    }
    /// # Errors
    /// When the pattern is not correct
    pub fn add_shape(&mut self, pattern: &str) -> Result<(), DealerError> {
        match *self {
            Self::Custom(ref mut shape) => shape.add_shape(pattern),
            Self::All => Ok(()),
        }
    }
    #[must_use]
    #[inline]
    pub fn is_member(&self, hand_to_match: Hand) -> bool {
        match *self {
            Self::Custom(ref shape) => shape.is_member(hand_to_match),
            Self::All => true,
        }
    }

    #[inline]
    #[must_use]
    pub fn len_ranges(&self) -> [LenRange; 4] {
        match *self {
            Self::Custom(ref shape) => shape.len_ranges(),
            Self::All => [LenRange::default(); 4],
        }
    }
}

/// Struct that represents multiple shapes.
/// The shapes are stored in a bit array and this is the main data structure
/// for keeping track of the shape we are interested into.
/// Offers all the necessary methods to manipulate the shapes.
#[derive(Clone)]
#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
            .finish_non_exhaustive()
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
    /// Creates a new Shapes instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shape_table: Box::new(bitarr!(0; SHAPE_TABLE_BUCKETS)),
            min_ls: [ZERO_LENGTH; SUITS],
            max_ls: [MAX_LENGTH; SUITS],
        }
    }

    /// Converts a shape pattern to an index. Necessary to avoid the hasher so we can have a `const fn`.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn shape_pattern_to_index(pattern: [u8; 4]) -> usize {
        let mut state = 0;
        let mut index = 0;
        while index < 4 {
            state = (ShapeHasher::R * state + pattern[index] as u64) % ShapeHasher::M;
            index += 1;
        }
        // We can truncate the state because it is modulo M, so smaller than SHAPE_TABLE_BUCKETS.
        state as usize
    }

    /// Checks if a hand is a member of the shape.
    #[must_use]
    fn is_member(&self, hand_to_match: Hand) -> bool {
        self.shape_table[Self::shape_pattern_to_index(hand_to_match.shape())]
    }

    /// Removes shapes based on a given string.
    /// # Errors
    /// When the pattern is not correct
    #[allow(clippy::missing_panics_doc)]
    pub fn remove_shape(&mut self, shape: &str) -> Result<(), DealerError> {
        // Implementation can change in the future.
        let patterns = ShapeCreator::build_shape(shape)?;

        // Returns error before panicking
        for pattern in patterns {
            self.delete_shape(pattern);
        }
        Ok(())
    }

    /// Adds balanced shapes.
    pub fn add_balanced(&mut self) {
        let mut permutations = Vec::with_capacity(50);
        Shapes::perm([5, 3, 3, 2], &mut permutations);
        Shapes::perm([4, 3, 3, 3], &mut permutations);
        Shapes::perm([4, 4, 3, 2], &mut permutations);
        let len = permutations.len();
        let mut index = 0;
        while index < len {
            let pattern = permutations[index];
            self.insert_shape(pattern);
            index += 1;
        }
    }

    /// Generates permutations.
    fn perm(shape: ShapePattern, permutations: &mut Vec<ShapePattern>) {
        fn heap_algorithm(n: usize, mut shape: ShapePattern, permutations: &mut Vec<ShapePattern>) {
            if n == 1 {
                permutations.push(shape);
            } else {
                heap_algorithm(n - 1, shape, permutations);
                for index in 0..n - 1 {
                    if n % 2 == 0 {
                        shape.swap(index, n - 1);
                    } else {
                        shape.swap(0, n - 1);
                    }
                    heap_algorithm(n - 1, shape, permutations);
                }
            }
        }
        let shape_pattern = shape;
        heap_algorithm(shape_pattern.len(), shape_pattern, permutations);
    }

    /// Fills the shape table with all shapes.
    fn all(&mut self) -> &mut Self {
        self.shape_table.fill(true);
        self
    }

    /// Creates a new Shapes instance filled with shapes.
    /// Should use `Shapes::All` for normal purposes.
    fn new_filled() -> Self {
        Self {
            shape_table: Box::new(bitarr!(1; SHAPE_TABLE_BUCKETS)),
            min_ls: [0; 4],
            max_ls: [13; 4],
        }
    }

    /// Removes a shape and returns a new table instance.
    /// # Errors
    /// When the pattern is not correct
    pub fn but(shape: &str) -> Result<Self, DealerError> {
        let mut table = Self::new_filled();
        table.remove_shape(shape)?;
        Ok(table)
    }

    /// Adds a shape to the table.
    /// # Errors
    /// When the pattern is not correct
    #[allow(clippy::missing_panics_doc)]
    pub fn add_shape(&mut self, shape: &str) -> Result<(), DealerError> {
        // Implementation can change in the future.
        let patterns = ShapeCreator::build_shape(shape)?;
        for pattern in patterns {
            self.insert_shape(pattern);
        }
        Ok(())
    }

    /// Inserts a shape in the table.
    fn insert_shape(&mut self, shape: ShapePattern) {
        // let safe = true; // used by redeal, don't know exactly what its purpose is.
        self.shape_table
            .set(Self::shape_pattern_to_index(shape), true);
    }

    /// Deletes a shape from the table.
    fn delete_shape(&mut self, shape: ShapePattern) {
        self.shape_table
            .set(Self::shape_pattern_to_index(shape), false);
    }

    /// Get the flattened version of the pattern descriptor.
    /// # Panics
    /// When the pattern is not correct
    #[must_use]
    pub fn flatten(pattern_descriptor: &[u8]) -> usize {
        let (s, h, d, c) = pattern_descriptor
            .iter()
            .map(|&x| x as usize)
            .next_tuple()
            .unwrap();
        ((((s * (RANKS + 1) as usize + h) * (RANKS + 1) as usize) + d) * (RANKS + 1) as usize) + c
    }

    /// Get the hash value of the flattened pattern descriptor.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn hash_flatten(pattern_descriptor: &[u8]) -> usize {
        let mut hasher = ShapeHasher::default();
        hasher.write(pattern_descriptor);
        hasher.finish() as usize
    }

    /// Update the table based on length constraints.
    fn update_based_on_length(&mut self) {
        let (rangespades, rangehearts, rangediamonds, rangeclubs) = self
            .min_ls
            .iter()
            .zip(self.max_ls.iter())
            .map(|(&min, &max)| RangeInclusive::new(min, max))
            .next_tuple()
            .unwrap();
        for shape in itertools::iproduct!(rangespades, rangehearts, rangediamonds, rangeclubs)
            .filter(|&(s, h, d, c)| s + h + d + c == MAX_LENGTH)
            .map(ShapePattern::from)
        {
            self.shape_table
                .set(Self::shape_pattern_to_index(shape), true);
        }
    }

    /// Get the length ranges for the shapes.
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Hash)]
/// Represents a range of lengths for shapes.
pub struct LenRange {
    /// The minimum length in the range.
    pub min: u8,
    /// The maximum length in the range.
    pub max: u8,
}

impl Default for LenRange {
    /// Creates a new `LenRange` with default values.
    fn default() -> Self {
        Self::new(ZERO_LENGTH, MAX_LENGTH)
    }
}

impl LenRange {
    /// Creates a new `LenRange`.
    #[must_use]
    pub fn new(min: u8, max: u8) -> Self {
        let max = max.clamp(ZERO_LENGTH, MAX_LENGTH);
        let min = min.clamp(ZERO_LENGTH, max);
        Self { min, max }
    }
    /// Returns the range as a range inclusive.
    #[must_use]
    pub fn as_range(&self) -> RangeInclusive<u8> {
        self.min()..=self.max()
    }
    /// Gets the minimum length in the range.
    #[must_use]
    pub fn min(&self) -> u8 {
        self.min
    }
    /// Gets the maximum length in the range.
    #[must_use]
    pub fn max(&self) -> u8 {
        self.max
    }
    /// Checks if the range contains a specific length.
    #[must_use]
    pub fn contains(&self, length: u8) -> bool {
        self.as_range().contains(&length)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Enum representing the suits in a standard deck of cards.
pub enum Suit {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
}

impl TryFrom<i32> for Suit {
    type Error = DealerError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Suit::Spades),
            1 => Ok(Suit::Hearts),
            2 => Ok(Suit::Diamonds),
            3 => Ok(Suit::Clubs),
            num => Err(DealerError::new(format!("cannot convert {num} to Suit"))),
        }
    }
}

impl TryFrom<Strain> for Suit {
    type Error = DealerError;
    fn try_from(strain: Strain) -> Result<Self, Self::Error> {
        match strain {
            Strain::Spades => Ok(Suit::Spades),
            Strain::Hearts => Ok(Suit::Hearts),
            Strain::Diamonds => Ok(Suit::Diamonds),
            Strain::Clubs => Ok(Suit::Clubs),
            Strain::NoTrumps => Err(DealerError::new("cannot convert NoTrumps to a Suit")),
        }
    }
}

impl std::fmt::Debug for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Suit::Clubs => write!(f, "C"),
            Suit::Diamonds => write!(f, "D"),
            Suit::Hearts => write!(f, "H"),
            Suit::Spades => write!(f, "S"),
        }
    }
}

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unicode())
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
            Suit::Clubs => '\u{2663}',
            Suit::Diamonds => '\u{2666}',
            Suit::Hearts => '\u{2665}',
            Suit::Spades => '\u{2660}',
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
    /// # Errors
    #[must_use]
    pub fn next(self) -> Option<Self> {
        match self {
            Suit::Spades => Some(Suit::Hearts),
            Suit::Hearts => Some(Suit::Diamonds),
            Suit::Diamonds => Some(Suit::Clubs),
            Suit::Clubs => None,
        }
    }

    #[must_use]
    pub fn rotating_next(self) -> Self {
        match self {
            Suit::Spades => Suit::Hearts,
            Suit::Hearts => Suit::Diamonds,
            Suit::Diamonds => Suit::Clubs,
            Suit::Clubs => Suit::Spades,
        }
    }
}

#[allow(unused_imports)]
mod test {
    use crate::*;

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
                true_arr.push(i);
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
        ok_shapes.sort_unstable();
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
        assert!(factory.is_member(hand));
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
                true_arr.push(i);
            }
        }
        let mut ok_shapes = vec![
            Shapes::hash_flatten(&[5, 2, 3, 3]),
            Shapes::hash_flatten(&[5, 3, 2, 3]),
            Shapes::hash_flatten(&[5, 3, 3, 2]),
        ];
        ok_shapes.sort_unstable();
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
        assert!(factory.is_member(hand));
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
        assert!(factory.is_member(hand));
        factory.remove_shape("3352").unwrap();
        assert!(!factory.is_member(hand));
    }

    #[test]
    fn test_perm() {
        let mut permutations = Vec::new();
        Shapes::perm([4, 3, 3, 2], &mut permutations);

        assert_eq!(permutations.len(), 24);
    }
}
