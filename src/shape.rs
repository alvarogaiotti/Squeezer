// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;
use crate::shapeparser::{ShapeCreator, ShapePattern};
use bitvec::{bitarr, BitArr};

const R: u64 = 14;
const M: u64 = 1379;
#[allow(clippy::cast_possible_truncation)]
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
            self.state = (R * self.state + u64::from(byte)) % M;
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
#[non_exhaustive]
pub enum Shape {
    Custom(Shapes),
    All,
}

#[derive(Default, Hash, PartialEq, Eq, Copy, Clone)]
pub struct SmallShape(u128);

impl SmallShape {
    const _3424: u128 = 1;
    const _4324: u128 = 1 << 1;
    const _2443: u128 = 1 << 2;
    const _2344: u128 = 1 << 3;
    const _4234: u128 = 1 << 4;
    const _3244: u128 = 1 << 5;
    const _4423: u128 = 1 << 6;
    const _3442: u128 = 1 << 7;
    const _4243: u128 = 1 << 8;
    const _4432: u128 = 1 << 9;
    const _4342: u128 = 1 << 10;
    const _2434: u128 = 1 << 11;
    const _3523: u128 = 1 << 12;
    const _2353: u128 = 1 << 13;
    const _5332: u128 = 1 << 14;
    const _3325: u128 = 1 << 15;
    const _3253: u128 = 1 << 16;
    const _2533: u128 = 1 << 17;
    const _3235: u128 = 1 << 18;
    const _2335: u128 = 1 << 19;
    const _5323: u128 = 1 << 20;
    const _3532: u128 = 1 << 21;
    const _3352: u128 = 1 << 22;
    const _5233: u128 = 1 << 23;
    const _3514: u128 = 1 << 24;
    const _3145: u128 = 1 << 25;
    const _5143: u128 = 1 << 26;
    const _3541: u128 = 1 << 27;
    const _5341: u128 = 1 << 28;
    const _1453: u128 = 1 << 29;
    const _4135: u128 = 1 << 30;
    const _3415: u128 = 1 << 31;
    const _3451: u128 = 1 << 32;
    const _5431: u128 = 1 << 33;
    const _1354: u128 = 1 << 34;
    const _4531: u128 = 1 << 35;
    const _1534: u128 = 1 << 36;
    const _5413: u128 = 1 << 37;
    const _4153: u128 = 1 << 38;
    const _1345: u128 = 1 << 39;
    const _4513: u128 = 1 << 40;
    const _5134: u128 = 1 << 41;
    const _5314: u128 = 1 << 42;
    const _1435: u128 = 1 << 43;
    const _1543: u128 = 1 << 44;
    const _3154: u128 = 1 << 45;
    const _4315: u128 = 1 << 46;
    const _4351: u128 = 1 << 47;
    const _4225: u128 = 1 << 48;
    const _2524: u128 = 1 << 49;
    const _2452: u128 = 1 << 50;
    const _4522: u128 = 1 << 51;
    const _2425: u128 = 1 << 52;
    const _2254: u128 = 1 << 53;
    const _4252: u128 = 1 << 54;
    const _5224: u128 = 1 << 55;
    const _5422: u128 = 1 << 56;
    const _2542: u128 = 1 << 57;
    const _5242: u128 = 1 << 58;
    const _2245: u128 = 1 << 59;
    const _4333: u128 = 1 << 60;
    const _3343: u128 = 1 << 61;
    const _3334: u128 = 1 << 62;
    const _3433: u128 = 1 << 63;
    const _2632: u128 = 1 << 64;
    const _2326: u128 = 1 << 65;
    const _3622: u128 = 1 << 66;
    const _2263: u128 = 1 << 67;
    const _2623: u128 = 1 << 68;
    const _2362: u128 = 1 << 69;
    const _2236: u128 = 1 << 70;
    const _6223: u128 = 1 << 71;
    const _3226: u128 = 1 << 72;
    const _6322: u128 = 1 << 73;
    const _6232: u128 = 1 << 74;
    const _3262: u128 = 1 << 75;
    const _2461: u128 = 1 << 76;
    const _4216: u128 = 1 << 77;
    const _6142: u128 = 1 << 78;
    const _6241: u128 = 1 << 79;
    const _6124: u128 = 1 << 80;
    const _4261: u128 = 1 << 81;
    const _4162: u128 = 1 << 82;
    const _1264: u128 = 1 << 83;
    const _2614: u128 = 1 << 84;
    const _4126: u128 = 1 << 85;
    const _1246: u128 = 1 << 86;
    const _2641: u128 = 1 << 87;
    const _2164: u128 = 1 << 88;
    const _2146: u128 = 1 << 89;
    const _6412: u128 = 1 << 90;
    const _4612: u128 = 1 << 91;
    const _6421: u128 = 1 << 92;
    const _1426: u128 = 1 << 93;
    const _1642: u128 = 1 << 94;
    const _1624: u128 = 1 << 95;
    const _2416: u128 = 1 << 96;
    const _1462: u128 = 1 << 97;
    const _6214: u128 = 1 << 98;
    const _4621: u128 = 1 << 99;
    const _6331: u128 = 1 << 100;
    const _3163: u128 = 1 << 101;
    const _6133: u128 = 1 << 102;
    const _3613: u128 = 1 << 103;
    const _6313: u128 = 1 << 104;
    const _3316: u128 = 1 << 105;
    const _3631: u128 = 1 << 106;
    const _1633: u128 = 1 << 107;
    const _3136: u128 = 1 << 108;
    const _1336: u128 = 1 << 109;
    const _3361: u128 = 1 << 110;
    const _1363: u128 = 1 << 111;
    const _1525: u128 = 1 << 112;
    const _2515: u128 = 1 << 113;
    const _5215: u128 = 1 << 114;
    const _5125: u128 = 1 << 115;
    const _5152: u128 = 1 << 116;
    const _5251: u128 = 1 << 117;
    const _5521: u128 = 1 << 118;
    const _2155: u128 = 1 << 119;
    const _1255: u128 = 1 << 120;
    const _1552: u128 = 1 << 121;
    const _5512: u128 = 1 << 122;
    const _2551: u128 = 1 << 123;
    const _1444: u128 = 1 << 124;
    const _4441: u128 = 1 << 125;
    const _4144: u128 = 1 << 126;
    const _4414: u128 = 1 << 127;
}

impl Default for Shape {
    fn default() -> Self {
        Self::All
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
    /// When the pattern is not correct
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

    /// Converts a shape pattern to an index.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn shape_pattern_to_index(pattern: [u8; 4]) -> usize {
        let mut state = 0;
        let mut index = 0;
        while index < 4 {
            state = (R * state + pattern[index] as u64) % M;
            index += 1;
        }
        // We cannot truncate the state because it is modulo M, so smaller than 1379.
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
            self.delete_shape(pattern.into());
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
            self.insert_shape(pattern.into());
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

#[derive(Debug, Copy, Clone)]
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
            num => Err(DealerError::new(&format!("cannot convert {num} to Suit"))),
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
