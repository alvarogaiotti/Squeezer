// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! # Bridge Shape Constraint System
//!
//! This module provides an efficient system for representing and checking bridge hand shapes.
//! A shape in bridge is defined as the distribution of cards across the four suits (spades,
//! hearts, diamonds, clubs), always summing to 13 cards total.
//!
//! ## Core Design
//!
//! The implementation uses a **bit array** to store allowed shapes, providing O(1) lookup
//! with minimal memory overhead (~172 bytes per constraint). Each of the 560 possible valid
//! bridge shapes is mapped to a unique bit position via a custom hash function.
//!
//! ## Performance Characteristics
//!
//! - **Memory**: 172 bytes per `Shapes` instance (1379 bits + min/max arrays)
//! - **Lookup**: O(1) with ~5-10 CPU cycles (hash + bit check)
//! - **Early rejection**: Min/max length ranges checked before bit array access
//! - **Cache-friendly**: Compact bit array fits in L1/L2 cache
//!
//! ## Example Usage
//!
//! ```rust
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! # use squeezer_model::Shapes;
//! # use squeezer_cards::Cards;
//! # use std::str::FromStr;
//! # let hand = Cards::from_str("AKQ2.KQ32.AK2.32")?;
//! // Create constraint for balanced hands (4333, 4432, 5332)
//! let mut shapes = Shapes::new();
//! shapes.add_balanced();
//!
//! // Check if a hand matches
//! if shapes.is_member(hand) {
//!     println!("Hand is balanced");
//! }
//!
//! // Add custom patterns
//! shapes.add_shape("5422")?; // Adds all permutations
//! shapes.remove_shape("4333")?;
//! # Ok(())}
//! ```

use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;

use bitvec::BitArr;
use itertools::Itertools;
use squeezer_cards::{Cards, Suit};

use crate::range::Bounded;
use crate::shapeparser::{CreationShapeError, ShapeCreator, ShapePattern};
use crate::{MAX_LENGTH, RANKS, SUITS, ZERO_LENGTH};

/// Number of buckets in the shape hash table.
///
/// Set to 1379 to minimize collisions for the hash function `(14 * state + byte) % 1379`.
/// Although there are only 560 valid bridge shapes (C(16,3) combinations), using a larger
/// prime number reduces collision probability to zero for this specific hash function.
#[allow(clippy::cast_possible_truncation)]
const SHAPE_TABLE_BUCKETS: usize = 1379;

/// Type alias for the bit array storage of allowed shapes.
///
/// Each bit represents whether a specific shape pattern is allowed. The bit position
/// is determined by the hash function in `shape_pattern_to_index`.
///
/// Size: 1379 bits = ~172 bytes
type ShapeTable = BitArr!(for SHAPE_TABLE_BUCKETS);

/// High-level shape constraint for a bridge hand.
///
/// Represents either a custom set of allowed shapes or the universal "all shapes allowed"
/// constraint. This is the primary public interface for shape constraints.
///
/// # Variants
///
/// - `Custom(Shapes)`: A specific set of allowed shapes stored in a bit array
/// - `All`: Represents no constraints - all possible shapes are allowed
///
/// # Examples
///
/// ```rust
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// # use squeezer_cards::Cards;
/// # use squeezer_model::Shape;
/// # use std::str::FromStr;
/// # let hand = Cards::from_str("AKQ2.KQ32.AK2.32")?;
/// // Create from a pattern string
/// let shape = Shape::try_from("5332")?; // All permutations of 5-3-3-2
///
/// // Create empty and add patterns
/// let mut shape = Shape::new_empty();
/// shape.add_shape("4333")?;
/// shape.add_shape("4432")?;
///
/// // Match against actual hands
/// if shape.is_member(hand) {
///     println!("Hand matches constraint");
/// }
/// # Ok(())}
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Shape {
    /// A custom set of allowed shapes with specific constraints
    Custom(Shapes),
    /// Universal constraint - all shapes are allowed
    #[default]
    All,
}

impl FromStr for Shape {
    type Err = CreationShapeError;

    fn from_str(pattern: &str) -> Result<Self, Self::Err> {
        Shape::new_from_pattern(pattern)
    }
}

impl TryFrom<&str> for Shape {
    type Error = CreationShapeError;
    fn try_from(pattern: &str) -> Result<Self, Self::Error> {
        Shape::new_from_pattern(pattern)
    }
}

impl Shape {
    /// Creates a new shape constraint from a pattern string.
    ///
    /// The pattern string is parsed by `ShapeCreator` which supports various formats:
    /// - Exact patterns: "5332" (generates all 4 permutations)
    /// - Wildcards: "5xxx" (any 5-card suit)
    /// - Ranges: "5+332" (5 or more in first suit)
    ///
    /// # Errors
    ///
    /// Returns [`CreationShapeError`] if the pattern string is malformed or
    /// represents invalid bridge shapes (e.g., suits don't sum to 13).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # use squeezer_model::Shape;
    /// let shape = Shape::new_from_pattern("5422")?;
    /// # Ok(())}
    /// ```
    pub fn new_from_pattern(pattern: &str) -> Result<Self, CreationShapeError> {
        let mut shape = Shapes::new();
        shape.add_shape(pattern)?;
        Ok(Self::Custom(shape))
    }

    /// Creates an empty shape constraint (no shapes allowed).
    ///
    /// Useful as a starting point when building constraints programmatically.
    #[must_use]
    pub fn new_empty() -> Self {
        Self::Custom(Shapes::new())
    }

    /// Creates a shape constraint from multiple pattern strings.
    ///
    /// Each pattern is parsed and added to the constraint set. This is equivalent to
    /// creating an empty shape and calling `add_shape` for each pattern.
    ///
    /// # Errors
    ///
    /// Returns [`CreationShapeError`] if any pattern is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # use squeezer_model::Shape;
    /// let shape = Shape::new_from_patterns(&["4333", "4432", "5332"])?;
    /// # Ok(())}
    /// ```
    pub fn new_from_patterns(patterns: &[&str]) -> Result<Self, CreationShapeError> {
        let mut shape = Shapes::new();
        for pattern in patterns {
            shape.add_shape(pattern)?;
        }
        Ok(Self::Custom(shape))
    }

    /// Removes shapes matching the given pattern from this constraint.
    ///
    /// If this is `Shape::All`, converts it to a `Custom` shape containing all shapes
    /// except those matching the pattern.
    ///
    /// # Errors
    ///
    /// Returns [`CreationShapeError`] if the pattern string is invalid.
    pub fn remove_shape(&mut self, pattern: &str) -> Result<(), CreationShapeError> {
        match *self {
            Self::Custom(ref mut shape) => shape.remove_shape(pattern),
            Self::All => {
                *self = Self::Custom(Shapes::but(pattern)?);
                Ok(())
            }
        }
    }

    /// Adds shapes matching the given pattern to this constraint.
    ///
    /// If this is `Shape::All`, this operation is a no-op since all shapes are already allowed.
    ///
    /// # Errors
    ///
    /// Returns [`CreationShapeError`] if the pattern string is invalid.
    pub fn add_shape(&mut self, pattern: &str) -> Result<(), CreationShapeError> {
        match *self {
            Self::Custom(ref mut shape) => shape.add_shape(pattern),
            Self::All => Ok(()),
        }
    }

    /// Checks if a seat's cards match this shape constraint.
    ///
    /// This is the primary method for validating hands during deal generation.
    ///
    /// # Performance
    ///
    /// - `Shape::All`: O(1) - always returns true
    /// - `Shape::Custom`: O(1) - min/max range checks + bit array lookup
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # use squeezer_cards::Cards;
    /// # use squeezer_model::Shape;
    /// # use std::str::FromStr;
    /// # let hand = Cards::from_str("AKQ2.KQ32.AK2.32")?;
    /// let shape = Shape::new_from_pattern("5332")?;
    /// if shape.is_member(hand) {
    ///     println!("Hand is 5-3-3-2 distribution");
    /// }
    /// # Ok(())}
    /// ```
    #[must_use]
    #[inline]
    pub fn is_member(&self, cards: Cards) -> bool {
        self.is_member_shape(cards.shape())
    }

    /// Membership check on an already computed shape pattern.
    #[must_use]
    #[inline]
    pub(crate) fn is_member_shape(&self, shape_pattern: [u8; 4]) -> bool {
        match *self {
            Self::Custom(ref shape) => shape.is_member_shape(shape_pattern),
            Self::All => true,
        }
    }

    /// Returns the min/max length constraints for each suit.
    ///
    /// Returns an array of 4 `LenRange` objects (one per suit in SHDC order).
    /// For `Shape::All`, returns unrestricted ranges [0-13] for all suits.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use squeezer_model::Shape;
    /// # let shape = Shape::All;
    /// let ranges = shape.len_ranges();
    /// println!("Spades: {}-{}", ranges[0].min(), ranges[0].max());
    /// ```
    #[inline]
    #[must_use]
    pub fn len_ranges(&self) -> [LenRange; 4] {
        match *self {
            Self::Custom(ref shape) => shape.len_ranges(),
            Self::All => [LenRange::default(); 4],
        }
    }

    /// Fast pre-check for shape validity without full hash lookup.
    ///
    /// Checks if the shape pattern's suit lengths fall within the allowed min/max ranges
    /// for each suit. This is faster than `is_member` because it avoids the bit array
    /// access, making it ideal for early rejection during deal generation.
    ///
    /// # Performance
    ///
    /// ~2-3 CPU cycles (4 range comparisons) vs ~5-10 for full `is_member` check.
    ///
    /// # Usage Pattern
    ///
    /// ```rust
    /// # use squeezer_cards::Cards;
    /// # use squeezer_model::Shape;
    /// # use std::str::FromStr;
    /// # let hand = Cards::from_str("AKQ2.KQ32.AK2.32").expect("a bridge hand parses");
    /// # let constraint = Shape::All;
    /// // In deal generation loop:
    /// if !constraint.is_member(hand) {
    ///     // continue;
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub(crate) fn pre_check(&self, shape_pattern: [u8; 4]) -> bool {
        match *self {
            Self::Custom(ref shape) => shape.pre_check(shape_pattern),
            Self::All => true,
        }
    }
}

/// Low-level storage and manipulation of bridge hand shapes.
///
/// This struct is the core data structure for shape constraints. It uses a bit array
/// to efficiently store which of the 560 possible bridge shapes are allowed, along with
/// min/max length constraints per suit for fast pre-filtering.
///
/// # Memory Layout
///
/// ```text
/// Shapes {
///     shape_table: BitArr[1379]        // 172 bytes (inline, no allocation)
///     min_ls: [u8; 4]                  // 4 bytes
///     max_ls: [u8; 4]                  // 4 bytes
/// }
/// Total: 180 bytes (no heap allocation)
/// ```
///
/// # Design Rationale
///
/// - **Bit array**: 1 bit per shape provides O(1) lookup with minimal memory
/// - **Inline storage**: Avoids a heap allocation and an extra pointer indirection
/// - **Min/max arrays**: Enable fast rejection without bit array access
/// - **Custom hash**: Maps shapes to array indices deterministically
///
/// # Example
///
/// ```rust
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// # use squeezer_cards::{Cards, Suit};
/// # use squeezer_model::Shapes;
/// # use std::str::FromStr;
/// # let hand = Cards::from_str("AKQ2.KQ32.AK2.32")?;
/// let mut shapes = Shapes::new();
/// shapes.add_balanced(); // 4333, 4432, 5332
/// shapes.add_shape("5422")?;
///
/// // Fast pre-check before full validation
/// let ranges = shapes.len_ranges();
/// if ranges[0].contains(hand.len_of_suit(Suit::Spades)) {
///     if shapes.is_member(hand) {
///         // Match!
///     }
/// }
/// # Ok(())}
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Shapes {
    /// Bit array storing allowed shapes. Each bit position corresponds to a shape
    /// as determined by `shape_pattern_to_index`. If bit N is set, shape with hash N
    /// is allowed.
    shape_table: ShapeTable,

    /// Minimum suit lengths [spades, hearts, diamonds, clubs].
    /// Used for fast rejection before consulting the bit array.
    min_ls: [u8; SUITS],

    /// Maximum suit lengths [spades, hearts, diamonds, clubs].
    /// Used for fast rejection before consulting the bit array.
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
    /// Creates a new empty `Shapes` instance with no shapes allowed.
    ///
    /// Min/max ranges are initialized to [0, 13] for all suits, which will be updated
    /// as shapes are added. For an instance with all shapes allowed, use `Shape::All`
    /// instead.
    #[must_use]
    pub fn new() -> Self {
        // `ZERO` is the same all-clear table `bitarr!(0; …)` builds. The
        // macro's expansion contains an internal `transmute`, and clippy's
        // `unsafe_derive_deserialize` heuristic flags any `unsafe` block in an
        // inherent impl of a type that derives `Deserialize` — a false
        // positive here, since the derive cannot invalidate the zeroed table.
        Self {
            shape_table: ShapeTable::ZERO,
            min_ls: [ZERO_LENGTH; SUITS],
            max_ls: [MAX_LENGTH; SUITS],
        }
    }

    /// Computes the hash index for a shape pattern.
    ///
    /// Uses a polynomial rolling hash: `state = 14 * (14 * (14 * s + h) + d) + c`,
    /// reduced mod 1379 once at the end. The unreduced value is at most
    /// 14^4 * 13 < 500k, so a single `% M` is exact and avoids 3 divisions.
    ///
    /// # Hash Function Properties
    ///
    /// - **Deterministic**: Same input always produces same output
    /// - **Collision-free**: For all 560 valid bridge shapes (verified by tests)
    /// - **Const-evaluable**: Can be used in const contexts
    /// - **Fast**: 3 multiplications + 3 additions + 1 modulo operation
    ///
    /// # Parameters
    ///
    /// - `pattern`: Array of 4 suit lengths [spades, hearts, diamonds, clubs] summing to 13
    ///
    /// # Returns
    ///
    /// Index in range [0, 1379) for use with the bit array
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    pub const fn shape_pattern_to_index(pattern: [u8; 4]) -> usize {
        const M: u64 = 1379;

        let state = 14u64
            * (14u64 * (14u64 * pattern[0] as u64 + pattern[1] as u64) + pattern[2] as u64)
            + pattern[3] as u64;

        // Safe truncation: state % M is guaranteed < M < usize::MAX
        (state % M) as usize
    }

    /// Checks if a seat's cards hold a shape in this shape set.
    ///
    /// This is a two-stage check:
    /// 1. **Range check**: Verify each suit length is within [min, max] for that suit
    /// 2. **Bit array check**: Look up the shape's hash in the bit array
    ///
    /// # Implementation Note
    ///
    /// The range check acts as a fast-path rejection, avoiding the hash computation
    /// and bit array access for shapes that are obviously excluded.
    #[must_use]
    #[inline]
    pub fn is_member(&self, cards: Cards) -> bool {
        self.is_member_shape(cards.shape())
    }

    /// Fast range-only pre-check on an already computed shape pattern.
    #[must_use]
    #[inline]
    pub(crate) fn pre_check(&self, shape: [u8; 4]) -> bool {
        shape[0] >= self.min_ls[0]
            && shape[0] <= self.max_ls[0]
            && shape[1] >= self.min_ls[1]
            && shape[1] <= self.max_ls[1]
            && shape[2] >= self.min_ls[2]
            && shape[2] <= self.max_ls[2]
            && shape[3] >= self.min_ls[3]
            && shape[3] <= self.max_ls[3]
    }

    /// Membership check on an already computed shape pattern.
    ///
    /// This avoids recomputing the hand's shape when the caller already has it.
    #[must_use]
    #[inline]
    pub(crate) fn is_member_shape(&self, shape: [u8; 4]) -> bool {
        // Fast-path: range check first (single source of truth in pre_check).
        if !self.pre_check(shape) {
            return false;
        }

        // Slow-path: Compute hash and check bit array
        self.shape_table[Self::shape_pattern_to_index(shape)]
    }

    /// Removes all shapes matching the given pattern string.
    ///
    /// The pattern is parsed into one or more concrete shape patterns (e.g., "5332"
    /// generates 4 permutations), and each matching shape is removed from the bit array.
    ///
    /// # Errors
    ///
    /// Returns [`CreationShapeError`] if the pattern string is malformed.
    ///
    /// # Note
    ///
    /// This does NOT update the min/max length arrays. Those represent conservative
    /// bounds and may include excluded shapes in their ranges.
    #[allow(clippy::missing_panics_doc)]
    pub fn remove_shape(&mut self, shape: &str) -> Result<(), CreationShapeError> {
        self.try_remove_pattern(shape)
    }

    /// Removes all shapes matching the given pattern, preserving the typed error.
    ///
    /// Crate-internal core behind [`Shapes::remove_shape`]: parses the pattern
    /// and clears the matching bits, returning the parser's
    /// [`CreationShapeError`] on a malformed pattern.
    pub(crate) fn try_remove_pattern(&mut self, pattern: &str) -> Result<(), CreationShapeError> {
        let patterns = ShapeCreator::build_shape(pattern)?;

        for pattern in patterns {
            self.delete_shape(pattern);
        }
        Ok(())
    }

    /// Adds all balanced hand shapes to this constraint.
    ///
    /// Balanced shapes in bridge are:
    /// - 4-3-3-3 (all permutations)
    /// - 4-4-3-2 (all permutations)
    /// - 5-3-3-2 (all permutations)
    ///
    /// Total: 3 + 10 + 12 = 25 permutations
    pub fn add_balanced(&mut self) {
        let mut permutations = Vec::with_capacity(50);
        Shapes::perm([5, 3, 3, 2], &mut permutations);
        Shapes::perm([4, 3, 3, 3], &mut permutations);
        Shapes::perm([4, 4, 3, 2], &mut permutations);

        for pattern in permutations {
            self.insert_shape(pattern);
        }
    }

    /// Generates all permutations of a shape pattern using Heap's algorithm.
    ///
    /// # Algorithm
    ///
    /// Uses Heap's algorithm which generates permutations in-place with O(n!) time
    /// complexity. For bridge shapes (n=4), this generates up to 24 permutations.
    ///
    /// # Parameters
    ///
    /// - `shape`: The base shape pattern to permute
    /// - `permutations`: Output vector that will contain all unique permutations
    fn perm(shape: ShapePattern, permutations: &mut Vec<ShapePattern>) {
        fn heap_algorithm(n: usize, mut shape: ShapePattern, permutations: &mut Vec<ShapePattern>) {
            if n == 1 {
                permutations.push(shape);
            } else {
                heap_algorithm(n - 1, shape, permutations);
                for index in 0..n - 1 {
                    if n.is_multiple_of(2) {
                        shape.swap(index, n - 1);
                    } else {
                        shape.swap(0, n - 1);
                    }
                    heap_algorithm(n - 1, shape, permutations);
                }
            }
        }
        heap_algorithm(shape.len(), shape, permutations);
    }

    /// Fills the shape table with all possible shapes.
    ///
    /// Sets every bit in the bit array to true. Used internally by `new_filled()`.
    /// For public use, prefer `Shape::All` which avoids allocating the bit array entirely.
    fn fill_all(&mut self) -> &mut Self {
        self.shape_table.fill(true);
        self
    }

    /// Creates a new `Shapes` instance with all shapes allowed.
    ///
    /// **Note**: For public use, prefer `Shape::All` instead, which is more efficient
    /// as it doesn't allocate a bit array.
    fn new_filled() -> Self {
        // `!ZERO` sets every bit, dead array padding included, exactly as
        // `bitarr!(1; …)` does; see `Shapes::new` for why the macro stays out
        // of this impl.
        Self {
            shape_table: !ShapeTable::ZERO,
            min_ls: [0; 4],
            max_ls: [13; 4],
        }
    }

    /// Creates a `Shapes` with every shape allowed (full bit table).
    ///
    /// Crate-internal equivalent of `Shape::All` without the enum dispatch:
    /// the deepened hand-constraint module stores this directly so the hot
    /// `check` path never branches on a variant.
    #[must_use]
    pub(crate) fn all() -> Self {
        Self::new_filled()
    }

    /// Creates a shape set with all shapes except those matching the pattern.
    ///
    /// Crate-internal core behind [`Shapes::but`], preserving the typed error.
    pub(crate) fn all_but(pattern: &str) -> Result<Self, CreationShapeError> {
        let mut table = Self::new_filled();
        table.try_remove_pattern(pattern)?;
        Ok(table)
    }

    /// Returns `true` when no shape is allowed (every bit is clear).
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        !self.shape_table.any()
    }

    /// Creates a shape set containing all shapes except those matching the pattern.
    ///
    /// Equivalent to:
    /// ```rust
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # use squeezer_model::Shape;
    /// let pattern = "4333";
    /// let mut shapes = Shape::All;
    /// shapes.remove_shape(pattern)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CreationShapeError`] if the pattern string is invalid.
    pub fn but(shape: &str) -> Result<Self, CreationShapeError> {
        Self::all_but(shape)
    }

    /// Adds all shapes matching the given pattern string.
    ///
    /// The pattern is parsed into concrete shape patterns, and each is added to the
    /// bit array by setting the corresponding bit.
    ///
    /// # Errors
    ///
    /// Returns [`CreationShapeError`] if the pattern string is malformed.
    #[allow(clippy::missing_panics_doc)]
    pub fn add_shape(&mut self, shape: &str) -> Result<(), CreationShapeError> {
        self.try_add_pattern(shape)
    }

    /// Adds all shapes matching the given pattern, preserving the typed error.
    ///
    /// Crate-internal core behind [`Shapes::add_shape`]: parses the pattern once
    /// via `ShapeCreator`, widens the per-suit min/max fast-reject bounds, and
    /// sets the matching bits.
    pub(crate) fn try_add_pattern(&mut self, pattern: &str) -> Result<(), CreationShapeError> {
        let patterns = ShapeCreator::build_shape(pattern)?;

        for pattern in patterns {
            // Widen the fast-reject bounds in place. (The previous code
            // mutated a temporary `len_ranges()` copy and dropped it, so
            // bounds silently stayed `[0, 13]`.)
            for (len, bounds) in pattern
                .iter()
                .zip(self.min_ls.iter_mut().zip(self.max_ls.iter_mut()))
            {
                let (min, max) = bounds;
                *min = (*min).min(*len);
                *max = (*max).max(*len);
            }
            self.insert_shape(pattern);
        }
        Ok(())
    }

    /// Inserts a single shape pattern into the bit array.
    ///
    /// Sets the bit at the hash index for this shape to true.
    ///
    /// # Parameters
    ///
    /// - `shape`: A 4-element array [spades, hearts, diamonds, clubs] summing to 13
    #[inline]
    fn insert_shape(&mut self, shape: ShapePattern) {
        // let safe = true; // used by redeal, don't know exactly what its purpose is.
        self.shape_table
            .set(Self::shape_pattern_to_index(shape), true);
    }

    /// Removes a single shape pattern from the bit array.
    ///
    /// Sets the bit at the hash index for this shape to false.
    ///
    /// # Parameters
    ///
    /// - `shape`: A 4-element array [spades, hearts, diamonds, clubs] summing to 13
    #[inline]
    fn delete_shape(&mut self, shape: ShapePattern) {
        self.shape_table
            .set(Self::shape_pattern_to_index(shape), false);
    }

    /// Computes a flattened index for a shape pattern (alternative indexing scheme).
    ///
    /// **Note**: This method is not currently used by the bit array indexing.
    /// It's kept for compatibility or alternative use cases. The primary indexing
    /// method is `shape_pattern_to_index`.
    ///
    /// Maps a shape [s, h, d, c] to index: (((s*14 + h)*14 + d)*14 + c)
    ///
    /// # Panics
    ///
    /// Panics if the pattern descriptor doesn't have exactly 4 elements.
    #[must_use]
    pub fn flatten(pattern_descriptor: &[u8]) -> usize {
        let (s, h, d, c) = pattern_descriptor
            .iter()
            .map(|&x| x as usize)
            .next_tuple()
            .unwrap();
        ((((s * (RANKS + 1) as usize + h) * (RANKS + 1) as usize) + d) * (RANKS + 1) as usize) + c
    }

    /// Populates the bit array based on current min/max length constraints.
    ///
    /// Iterates through all possible shape combinations where each suit's length falls
    /// within [`min_ls`[i], `max_ls`[i]] and the total is 13. Sets the corresponding bit
    /// for each valid shape.
    ///
    /// This is used when constructing a `Shapes` from length ranges.
    fn update_based_on_lengths(&mut self) {
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

    /// Returns the min/max length ranges for each suit.
    ///
    /// Returns an array of 4 `LenRange` objects in suit order: [S, H, D, C]
    ///
    /// # Example
    ///
    /// ```rust
    /// # use squeezer_model::Shapes;
    /// let shapes = Shapes::default();
    /// let ranges = shapes.len_ranges();
    /// assert!(ranges[0].contains(5)); // Check if 5 spades is allowed
    /// ```
    #[must_use]
    pub fn len_ranges(&self) -> [LenRange; 4] {
        let mut len_range = [LenRange::default(); 4];
        for (index, (&min, &max)) in self.min_ls.iter().zip(self.max_ls.iter()).enumerate() {
            len_range[index] = LenRange::new(min, max);
        }
        len_range
    }
}

impl From<&[LenRange; SUITS]> for Shapes {
    /// Creates a `Shapes` instance from an array of length ranges.
    ///
    /// This constructor builds the shape constraint by:
    /// 1. Setting min/max arrays from the provided ranges
    /// 2. Calling `update_based_on_length()` to populate the bit array with all
    ///    shapes that satisfy the length constraints
    ///
    /// # Example
    ///
    /// ```rust
    /// # use squeezer_model::{LenRange, Shapes};
    /// let ranges = [
    ///     LenRange::new(5, 7), // Spades: 5-7 cards
    ///     LenRange::new(3, 5), // Hearts: 3-5 cards
    ///     LenRange::new(0, 3), // Diamonds: 0-3 cards
    ///     LenRange::new(0, 3), // Clubs: 0-3 cards
    /// ];
    /// let shapes = Shapes::from(&ranges);
    /// ```
    fn from(len_range: &[LenRange; SUITS]) -> Self {
        let mut min_ls = [ZERO_LENGTH; SUITS];
        let mut max_ls = [MAX_LENGTH; SUITS];

        for &suit in Suit::ALL {
            min_ls[suit as usize] = len_range[suit as usize].min();
            max_ls[suit as usize] = len_range[suit as usize].max();
        }

        let mut shapes = Self {
            min_ls,
            max_ls,
            ..Default::default()
        };
        shapes.update_based_on_lengths();
        shapes
    }
}

/// Represents a range of allowed suit lengths.
///
/// Used to specify constraints like "5-7 spades" or "0-3 clubs". These ranges
/// are used for fast pre-filtering before checking the bit array.
///
/// Thin newtype over the shared [`Bounded`] core (cap [`MAX_LENGTH`]): the
/// same clamping invariant as [`HcpRange`], enforced once.
///
/// # Invariants
///
/// - `min <= max`
/// - Both values are in range [0, 13]
/// - Values are automatically clamped on construction
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Default)]
pub struct LenRange(pub(crate) Bounded<MAX_LENGTH>);

impl LenRange {
    /// Creates a new `LenRange` with automatic bounds clamping.
    ///
    /// # Parameters
    ///
    /// - `min`: Minimum suit length (clamped to [0, max])
    /// - `max`: Maximum suit length (clamped to [0, 13])
    ///
    /// # Example
    ///
    /// ```rust
    /// # use squeezer_model::LenRange;
    /// let range = LenRange::new(5, 7);
    /// assert_eq!(range.min(), 5);
    /// assert_eq!(range.max(), 7);
    ///
    /// // Auto-clamping
    /// let range = LenRange::new(20, 30);
    /// assert_eq!(range.max(), 13); // Clamped to max hand size
    /// ```
    #[must_use]
    pub const fn new(min: u8, max: u8) -> Self {
        Self(Bounded::new(min, max))
    }

    /// Returns this range as an inclusive Rust range.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use squeezer_model::LenRange;
    /// let range = LenRange::new(3, 5);
    /// for len in range.as_range() {
    ///     println!("Length: {}", len); // Prints 3, 4, 5
    /// }
    /// ```
    #[must_use]
    pub const fn as_range(&self) -> RangeInclusive<u8> {
        self.0.as_range()
    }

    /// Returns the minimum length in this range.
    #[must_use]
    #[inline]
    pub const fn min(&self) -> u8 {
        self.0.min()
    }

    /// Returns the maximum length in this range.
    #[must_use]
    #[inline]
    pub const fn max(&self) -> u8 {
        self.0.max()
    }

    /// Checks if a specific suit length falls within this range.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use squeezer_model::LenRange;
    /// let range = LenRange::new(5, 7);
    /// assert!(range.contains(6));
    /// assert!(!range.contains(4));
    /// ```
    #[must_use]
    #[inline]
    pub const fn contains(&self, length: u8) -> bool {
        self.0.contains(length)
    }
}

#[allow(unused_imports)]
mod test {
    use crate::*;
    use squeezer_cards::Cards;

    /// Verifies that the hash function produces no collisions for all valid bridge shapes.
    ///
    /// Tests all 560 possible bridge shapes (combinations where suits sum to 13) and
    /// ensures each maps to a unique hash value.
    #[test]
    fn no_hash_collisions() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();

        for s in 0..=13 {
            for h in 0..=(13 - s) {
                for d in 0..=(13 - s - h) {
                    let c = 13 - s - h - d;
                    let hash = Shapes::shape_pattern_to_index([s, h, d, c]);

                    assert!(
                        hash < super::SHAPE_TABLE_BUCKETS,
                        "Hash {hash} out of bounds for shape [{s}, {h}, {d}, {c}]"
                    );

                    assert!(
                        seen.insert((hash, [s, h, d, c])),
                        "Hash collision at index {hash}: shape [{s}, {h}, {d}, {c}]"
                    );
                }
            }
        }

        assert_eq!(seen.len(), 560, "Expected exactly 560 valid bridge shapes");
    }

    /// Tests that balanced shapes are correctly identified.
    #[test]
    fn test_balanced_shapes() {
        let mut shapes = Shapes::new();
        shapes.add_balanced();

        // Test valid balanced patterns
        assert!(shapes.shape_table[Shapes::shape_pattern_to_index([4, 3, 3, 3])]);
        assert!(shapes.shape_table[Shapes::shape_pattern_to_index([4, 4, 3, 2])]);
        assert!(shapes.shape_table[Shapes::shape_pattern_to_index([5, 3, 3, 2])]);

        // Test unbalanced pattern
        assert!(!shapes.shape_table[Shapes::shape_pattern_to_index([7, 3, 2, 1])]);
    }

    /// Tests `LenRange` boundary conditions.
    #[test]
    fn test_len_range_clamping() {
        let range = LenRange::new(20, 30);
        assert_eq!(range.max(), 13, "Should clamp max to 13");
        assert_eq!(range.min(), 13, "Should clamp min to clamped max");

        let range = LenRange::new(5, 3);
        assert_eq!(range.min(), 3, "Should swap if min > max");
        assert_eq!(
            range.max(),
            3,
            "Max should equal min when initially inverted"
        );
    }

    #[test]
    fn shape_creation_test() {
        let mut shapes = Shapes::new();
        shapes.add_shape("4333").unwrap();
        assert!(shapes.shape_table[Shapes::shape_pattern_to_index([4, 3, 3, 3])]);
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
            Shapes::shape_pattern_to_index([6, 3, 1, 3]),
            Shapes::shape_pattern_to_index([6, 3, 3, 1]),
            Shapes::shape_pattern_to_index([6, 1, 3, 3]),
            Shapes::shape_pattern_to_index([3, 6, 1, 3]),
            Shapes::shape_pattern_to_index([3, 6, 3, 1]),
            Shapes::shape_pattern_to_index([1, 6, 3, 3]),
            Shapes::shape_pattern_to_index([1, 3, 6, 3]),
            Shapes::shape_pattern_to_index([3, 3, 6, 1]),
            Shapes::shape_pattern_to_index([3, 1, 6, 3]),
            Shapes::shape_pattern_to_index([3, 3, 1, 6]),
            Shapes::shape_pattern_to_index([3, 1, 3, 6]),
            Shapes::shape_pattern_to_index([1, 3, 3, 6]),
            Shapes::shape_pattern_to_index([4, 4, 3, 2]),
            Shapes::shape_pattern_to_index([4, 3, 4, 2]),
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

        assert!(factory.is_member(cards));
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
            Shapes::shape_pattern_to_index([5, 2, 3, 3]),
            Shapes::shape_pattern_to_index([5, 3, 2, 3]),
            Shapes::shape_pattern_to_index([5, 3, 3, 2]),
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

        assert!(factory.is_member(cards));
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

        let mut factory = Shapes::new();
        factory.add_shape("3xx2").unwrap();
        assert!(factory.is_member(cards));
        factory.remove_shape("3352").unwrap();
        assert!(!factory.is_member(cards));
    }

    #[test]
    fn test_perm() {
        let mut permutations = Vec::new();
        Shapes::perm([4, 3, 3, 2], &mut permutations);

        assert_eq!(permutations.len(), 24);
    }
}
