use crate::prelude::*;

const R: u64 = 61;
const M: u64 = 2039;
const SHAPE_TABLE_BUCKETS: usize = 2048;

#[derive(Default)]
struct ShapeHasher {
    state: u64,
}

impl std::hash::Hasher for ShapeHasher {
    fn write(&mut self, bytes: &[u8]) {
        // We reset so we are guaranteed to get always the same result.
        self.state = 0;
        for &byte in bytes {
            self.state = (R * self.state + u64::from(byte)) % M
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

struct BuildShapeHasher;

impl std::hash::BuildHasher for BuildShapeHasher {
    type Hasher = ShapeHasher;
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher { state: 0 }
    }
}

// Struct that represents multiple shapes.
#[derive(Copy, Clone)]
pub struct Shapes {
    shape_table: [bool; SHAPE_TABLE_BUCKETS],
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

impl<'a> Shapes {
    #[must_use] pub fn new() -> Self {
        Self {
            shape_table: [false; SHAPE_TABLE_BUCKETS],
            min_ls: [ZERO_LENGTH; SUITS],
            max_ls: [MAX_LENGTH; SUITS],
        }
    }
    pub fn remove_shape(&mut self, shape: ShapeDescriptor) -> Result<(), DealerError> {
        // Take shape pattern. Right now we match on equal enums, but I'll probably change
        // implementation in the future so I'll keep it here for future use.
        let shape_pattern = match shape {
            ShapeDescriptor::SingleShape { shape_pattern } => shape_pattern,
            ShapeDescriptor::ClassOfShapes { shape_pattern } => shape_pattern,
        };

        let mut shape_string: Vec<char> = shape_pattern.pattern.chars().collect();
        // Pattern parsed
        let mut parsed: Vec<u8> = Vec::new();

        // Pattern collected
        let mut collected: Vec<Vec<u8>> = Vec::new();
        Shapes::get_patterns(&mut shape_string, &mut parsed, &mut collected)?;
        for pattern in &collected {
            self.delete_shape(pattern)?;
        }
        Ok(())
    }

    pub fn add_shape(&mut self, shape: ShapeDescriptor) -> Result<(), DealerError> {
        // Take shape pattern. Right now we match on equal enums, but I'll probably change
        // implementation in the future so I'll keep it here for future use.
        let shape_pattern = match shape {
            ShapeDescriptor::SingleShape { shape_pattern } => shape_pattern,
            ShapeDescriptor::ClassOfShapes { shape_pattern } => shape_pattern,
        };

        let mut shape_string: Vec<char> = shape_pattern.pattern.chars().collect();
        // Pattern parsed
        let mut parsed: Vec<u8> = Vec::new();

        // Pattern collected
        let mut collected: Vec<Vec<u8>> = Vec::new();
        Shapes::get_patterns(&mut shape_string, &mut parsed, &mut collected)?;
        for pattern in &collected {
            self.insert_shape(pattern)?;
        }
        Ok(())
    }

    fn get_patterns(
        shape_pattern: &mut Vec<char>,
        parsed: &mut Vec<u8>,
        collected: &'a mut Vec<Vec<u8>>,
    ) -> Result<&'a Vec<Vec<u8>>, DealerError> {
        // If empty, we return
        if shape_pattern.is_empty() {
            collected.push(parsed.clone());
            return Ok(collected);
        }
        let head: Vec<u8>;
        // If we start with a bracket, we call a function that parses the
        // parts inside of the brackets as a whole to, later, save all the permutations
        // E.g. (43)42 will output 4342 and 3442.
        if let Some('(') = shape_pattern.first() {
            shape_pattern.remove(0);
            let Some(closing_bracket_index) = shape_pattern.iter().position(|&x| x == ')')
                else {
                    return Err(DealerError::new("Unbalanced parentheses."));
                };
            // Parse until the closing bracket.
            head = Shapes::parse_chars_to_nums(shape_pattern, closing_bracket_index)?;
            _ = shape_pattern.drain(..=closing_bracket_index);
            let head_len = head.len();
            for perm in head.into_iter().permutations(head_len) {
                parsed.extend(perm);
                Shapes::get_patterns(shape_pattern, parsed, collected)?;
                _ = parsed.drain((parsed.len() - head_len)..);
            }
        } else {
            // else we parse a single char
            head = Shapes::parse_chars_to_nums(shape_pattern, 1)?;
            let popped = shape_pattern.remove(0);
            parsed.extend(head);
            Shapes::get_patterns(shape_pattern, parsed, collected)?;
            parsed.pop();
            shape_pattern.push(popped);
        }
        Ok(collected)
    }

    fn insert_shape(&mut self, shape: &[u8]) -> Result<(), DealerError> {
        // let safe = true; // used by redeal, don't know exactly what its purpose is.
        let mut table = [false; SHAPE_TABLE_BUCKETS];
        let (min_ls, max_ls) = Shapes::table_from_pattern(Vec::from(shape), &mut table)?;
        for suit in Suit::ALL {
            let suit = *suit as usize;
            self.min_ls[suit] = u8::min(self.min_ls[suit], min_ls[suit]);
            self.max_ls[suit] = u8::max(self.max_ls[suit], max_ls[suit]);
        }
        for (i, bit) in table.iter().enumerate() {
            self.shape_table[i] |= bit;
        }
        Ok(())
    }

    fn delete_shape(&mut self, shape: &[u8]) -> Result<(), DealerError> {
        let mut table = [false; SHAPE_TABLE_BUCKETS];
        let (_min_ls, _max_ls) = Shapes::table_from_pattern(Vec::from(shape), &mut table)?;
        for (i, bit) in table.iter().enumerate() {
            self.shape_table[i] &= !bit;
        }
        Ok(())
    }

    #[must_use] pub fn is_member(&self, hand_to_match: Hand) -> bool {
        self.shape_table[Shapes::hash_flatten(&hand_to_match.shape())]
    }

    #[must_use] pub fn flatten(pattern_descriptor: &[u8]) -> usize {
        let (s, h, d, c) = pattern_descriptor
            .iter()
            .map(|&x| x as usize)
            .next_tuple()
            .unwrap();
        ((((s * (RANKS + 1) as usize + h) * (RANKS + 1) as usize) + d) * (RANKS + 1) as usize) + c
    }
    #[must_use] pub fn hash_flatten(pattern_descriptor: &[u8]) -> usize {
        let mut hasher = ShapeHasher::default();
        hasher.write(pattern_descriptor);
        hasher.finish() as usize
    }

    fn table_from_pattern(
        shape: Vec<u8>,
        table: &mut [bool; SHAPE_TABLE_BUCKETS],
        // In the Python implementation there is a `safe: bool`, but is always passed as true, so we
        // can avoid it.
    ) -> Result<([u8; SUITS], [u8; SUITS]), DealerError> {
        // Get the sum of the total we are at whitout the xs.
        let pre_set: u8 = shape.iter().filter(|&&x| x != RANKS + 1).sum();
        // Min and max lengths, implemented in the Python library for smartstacking.
        // Here we do not have smartstacking but maybe it'll be implemented in the future.
        let mut min_ls = [ZERO_LENGTH; SUITS];
        let mut max_ls = [ZERO_LENGTH; SUITS];
        // Jokers is a `x` in a shape pattern
        // e.g. 4xx2.
        if let Some(joker_index) = shape.iter().position(|&x| x == RANKS + 1) {
            if pre_set > MAX_LENGTH {
                return Err(DealerError::new("Invalid ambiguous shape."));
            }
            // Every possible length of the x
            for possible_lengths in ZERO_LENGTH..=(MAX_LENGTH - pre_set) {
                let mut new_shape = Vec::with_capacity(4);
                new_shape.extend_from_slice(&shape[..joker_index]);
                new_shape.push(possible_lengths);
                new_shape.extend_from_slice(&shape[joker_index + 1..]);
                let is_len_correct =
                    new_shape.iter().filter(|&&x| x != RANKS + 1).sum::<u8>() == 13u8;
                let still_jokers = any(new_shape.iter(), |&x| x == RANKS + 1);

                if !still_jokers && !is_len_correct {
                    continue;
                };
                Shapes::table_from_pattern(new_shape, table)?;
            }
            Ok((min_ls, max_ls))
        } else {
            match Shapes::add_shape_pattern_to_table(
                pre_set,
                &mut min_ls,
                &shape,
                &mut max_ls,
                table,
            ) {
                Ok(()) => Ok((min_ls, max_ls)),
                Err(error) => Err(error),
            }
        }
    }

    // Get pattern of a shape from a vector of chars.
    // via a recursive function call.

    fn add_shape_pattern_to_table(
        pre_set: u8,
        min_ls: &mut [u8; SUITS],
        shape: &[u8],
        max_ls: &mut [u8; SUITS],
        table: &'a mut [bool; SHAPE_TABLE_BUCKETS],
    ) -> Result<(), DealerError> {
        if pre_set != MAX_LENGTH {
            return Err(DealerError::new("Wrong number of cards in shape."));
        }
        for suit in Suit::ALL {
            let suit = *suit as usize;
            min_ls[suit] = u8::min(min_ls[suit], shape[suit]);
            max_ls[suit] = u8::max(max_ls[suit], shape[suit]);
        }
        table[Shapes::hash_flatten(shape)] = true;
        Ok(())
    }
    /* Old function used to parse bracket patterns.
     * fn parse_with_bracket(shape_pattern: &mut Vec<char>) -> Result<Vec<u8>, Box<dyn Error>> {
        let closing_bracket_index =
            if let Some(index) = shape_pattern.iter().position(|&x| x == ')') {
                index
            } else {
                return Err(Box::new(DealerError::new("Unbalanced parentheses.")));
            };
        let head = match Shapes::parse_chars_to_nums(shape_pattern, closing_bracket_index) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };
        *shape_pattern = (shape_pattern[closing_bracket_index + 1..]).to_vec();
        Ok(head)
    }*/

    fn parse_chars_to_nums(shape_pattern: &mut [char], end: usize) -> Result<Vec<u8>, DealerError> {
        let mut errors = vec![];
        let head: Vec<u8> = (shape_pattern[0..end])
            .iter()
            .map(|&x| {
                if x == ShapeFactory::JOKER {
                    Ok(RANKS + 1)
                } else {
                    match x.to_digit(10) {
                        Some(value) => Ok(value as u8),
                        None => Err(DealerError::new("Shape pattern contains unknown chars.")),
                    }
                }
            })
            .filter_map(|x| x.map_err(|e| errors.push(e)).ok())
            .collect();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Ok(head)
    }
    fn update_based_on_length(&mut self) {
        let (rangespades, rangehearts, rangediamonds, rangeclubs) = self
            .min_ls
            .iter()
            .zip(self.max_ls.iter())
            .map(|(&min, &max)| RangeInclusive::new(min, max))
            .next_tuple()
            .unwrap();
        for (s, h, d, c) in
            itertools::iproduct!(rangespades, rangehearts, rangediamonds, rangeclubs)
                .filter(|(s, h, d, c)| s + h + d + c == MAX_LENGTH)
        {
            self.shape_table[Shapes::hash_flatten(&[s, h, d, c])] = true;
        }
    }
    pub const ALL: Shapes = Shapes {
        shape_table: [true; SHAPE_TABLE_BUCKETS],
        min_ls: [ZERO_LENGTH; 4],
        max_ls: [MAX_LENGTH; 4],
    };
    #[must_use] pub fn len_ranges(&self) -> [LenRange; 4] {
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
    #[must_use] pub fn new(min: u8, max: u8) -> Self {
        let max = max.clamp(ZERO_LENGTH, MAX_LENGTH);
        let min = min.clamp(ZERO_LENGTH, max);
        Self { min, max }
    }
    #[must_use] pub fn as_range(&self) -> RangeInclusive<u8> {
        self.min()..=self.max()
    }
    #[must_use] pub fn min(&self) -> u8 {
        self.min
    }
    #[must_use] pub fn max(&self) -> u8 {
        self.max
    }
    #[must_use] pub fn contains(&self, length: u8) -> bool {
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
    #[must_use] pub fn name(self) -> &'static str {
        match self {
            Suit::Clubs => "Clubs",
            Suit::Diamonds => "Diamonds",
            Suit::Hearts => "Hearts",
            Suit::Spades => "Spades",
        }
    }
    /// The unicode character for this suit
    #[must_use] pub fn unicode(self) -> char {
        match self {
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
            Suit::Spades => '♠',
        }
    }
    /// The latin character for this suit
    #[must_use] pub fn latin(self) -> char {
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

pub struct StringShapePattern {
    pattern: String,
}

impl StringShapePattern {
    #[must_use] pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
        }
    }
}

pub enum ShapeDescriptor {
    SingleShape { shape_pattern: StringShapePattern }, // TODO: Make this a Vec<u8> already
    ClassOfShapes { shape_pattern: StringShapePattern },
}

impl ShapeDescriptor {
    #[must_use] pub fn from_string(pattern: &str) -> Self {
        if pattern.contains('(') {
            Self::ClassOfShapes {
                shape_pattern: StringShapePattern {
                    pattern: pattern.to_string(),
                },
            }
        } else {
            Self::SingleShape {
                shape_pattern: StringShapePattern {
                    pattern: pattern.to_string(),
                },
            }
        }
    }
}

impl ShapeDescriptor {
    #[must_use] pub fn new(pattern: &str) -> Self {
        if pattern.contains('(') {
            Self::ClassOfShapes {
                shape_pattern: StringShapePattern {
                    pattern: pattern.to_owned(),
                },
            }
        } else {
            Self::SingleShape {
                shape_pattern: StringShapePattern {
                    pattern: pattern.to_owned(),
                },
            }
        }
    }
}

#[cfg(test)]
#[test]
fn factory_get_pattern_test() {
    let mut collected: Vec<Vec<u8>> = Vec::new();

    Shapes::get_patterns(
        &mut "4333".chars().collect(),
        &mut Vec::new(),
        &mut collected,
    )
    .unwrap();
    assert_eq!(collected.pop().unwrap(), vec![4u8, 3u8, 3u8, 3u8]);
}
#[test]
fn shape_creation_test() {
    let mut shapes = Shapes::new();
    shapes
        .add_shape(ShapeDescriptor::SingleShape {
            shape_pattern: StringShapePattern {
                pattern: String::from("4333"),
            },
        })
        .unwrap();
    assert!(shapes.shape_table[Shapes::hash_flatten(&[4, 3, 3, 3])]);
}
#[test]
#[should_panic]
fn shape_error_unbal_parentheses_test() {
    let mut factory = ShapeFactory::new();
    let pattern: Vec<char> = "4(333".chars().collect();
    factory.insert(pattern).unwrap();
}

#[test]
fn shape_parens_interpretation_working_test() {
    let mut factory = Shapes::new();
    factory
        .add_shape(ShapeDescriptor::SingleShape {
            shape_pattern: StringShapePattern {
                pattern: String::from("4(34)2"),
            },
        })
        .unwrap();
    factory
        .add_shape(ShapeDescriptor::SingleShape {
            shape_pattern: StringShapePattern {
                pattern: String::from("(6331)"),
            },
        })
        .unwrap();

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
    factory
        .add_shape(ShapeDescriptor::SingleShape {
            shape_pattern: StringShapePattern {
                pattern: String::from("4(34)2"),
            },
        })
        .unwrap();
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
    //println!("{}", ShapeFactory::flatten(hand.shape()));
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
    factory
        .add_shape(ShapeDescriptor::SingleShape {
            shape_pattern: StringShapePattern {
                pattern: String::from("3xx2"),
            },
        })
        .unwrap();
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
    //println!("{}", ShapeFactory::flatten(hand.shape()));
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
    factory
        .add_shape(ShapeDescriptor::SingleShape {
            shape_pattern: StringShapePattern {
                pattern: String::from("3xx2"),
            },
        })
        .unwrap();
    assert!(factory.is_member(hand));
    factory
        .remove_shape(ShapeDescriptor::SingleShape {
            shape_pattern: StringShapePattern {
                pattern: String::from("3352"),
            },
        })
        .unwrap();
    //println!("{}", ShapeFactory::flatten(hand.shape()));
    assert!(!factory.is_member(hand));
}
