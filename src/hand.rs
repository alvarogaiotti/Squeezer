// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::hand_constraint::HandSummary;
use crate::prelude::*;
use crate::range::Bounded;

/// Represents an *inclusive* range of High Card Points.
///
/// Thin newtype over the shared [`Bounded`] core (cap [`MAX_HCP_IN_HAND`]):
/// construction, clamping, and membership live in exactly one place, shared
/// with [`LenRange`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct HcpRange(pub(crate) Bounded<MAX_HCP_IN_HAND>);

impl From<(u8, u8)> for HcpRange {
    fn from((min, max): (u8, u8)) -> Self {
        let mut min = min.clamp(0, MAX_HCP_IN_HAND);
        let mut max = max.clamp(0, MAX_HCP_IN_HAND);
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        Self(Bounded::new(min, max))
    }
}

impl From<HcpRange> for (u8, u8) {
    fn from(range: HcpRange) -> Self {
        range.0.as_pair()
    }
}

impl HcpRange {
    /// This function guarantees that the `HcpRange` created is possible.
    /// i.e. that the `min_hcp` is less than or equal to the `max_hcp`.
    /// and that the `max_hcp` is clamped to 37.
    #[must_use]
    pub const fn new(min: u8, max: u8) -> Self {
        Self(Bounded::new(min, max))
    }

    /// Get the minimum value of the HCP range.
    #[must_use]
    pub const fn range_min(&self) -> u8 {
        self.0.min()
    }

    /// Get the maximum value of the HCP range.
    #[must_use]
    pub const fn range_max(&self) -> u8 {
        self.0.max()
    }

    /// Check if the specified HCP is within this range.
    #[must_use]
    pub const fn contains(&self, hcp: u8) -> bool {
        self.0.contains(hcp)
    }

    /// Get the range of HCP values as an inclusive range.
    #[must_use]
    pub const fn as_range(&self) -> RangeInclusive<u8> {
        self.0.as_range()
    }
    #[must_use]
    #[inline]
    pub fn from_range<R, U>(hcp_range: R) -> Self
    where
        R: std::ops::RangeBounds<U>,
        U: Into<u8> + Copy,
    {
        let mut min = match hcp_range.start_bound() {
            std::ops::Bound::Included(&min) => min.into(),
            std::ops::Bound::Excluded(&min) => min.into().saturating_sub(1),
            std::ops::Bound::Unbounded => 0,
        }
        .clamp(0, MAX_HCP_IN_HAND);
        let mut max = match hcp_range.end_bound() {
            std::ops::Bound::Included(&max) => max.into(),
            std::ops::Bound::Excluded(&max) => max.into().saturating_sub(1),
            std::ops::Bound::Unbounded => MAX_HCP_IN_HAND,
        }
        .clamp(0, MAX_HCP_IN_HAND);
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        Self(Bounded::new(min, max))
    }
}

/// Represents a set of possible hands with the accepted shapes and the accepted HCP range.
/// This struct main goal is to express a single hand type that we can accept.
/// If you want to represent multiple hand types, you can use the [`HandDescriptor`] struct,
/// which embbeds multiple [`HandType`]s.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone)]
// TODO: Add a custom evaluator field, so we can pass a Evaluator and have a custom ValuationRange or something
// like that for more fine grained control over the hand.
pub struct HandType {
    shape: Shape,
    hcp_range: HcpRange,
}

impl HandType {
    /// Create a new `HandType` with the specified shape and HCP range.
    #[must_use]
    #[inline]
    pub const fn new(shape: Shape, hcp_range: HcpRange) -> Self {
        Self { shape, hcp_range }
    }

    /// Check if the `HandType` matches the given cards based on shape and HCP
    /// range.
    #[must_use]
    #[inline]
    pub fn check(&self, cards: Cards) -> bool {
        self.check_cached(HandSummary::of(cards))
    }

    /// Check the `HandType` using an already computed summary.
    #[must_use]
    #[inline]
    pub(crate) fn check_cached(&self, summary: HandSummary) -> bool {
        self.hcp_range.contains(summary.hcp)
            && self.shape.pre_check(summary.shape)
            && self.shape.is_member_shape(summary.shape)
    }

    /// Get the length ranges for each suit based on the accepted shapes.
    #[must_use]
    #[inline]
    pub fn len_ranges(&self) -> [LenRange; 4] {
        self.shape.len_ranges()
    }

    /// Get the accepted HCP range for this `HandType`.
    #[must_use]
    #[inline]
    pub fn hcp_range(&self) -> HcpRange {
        self.hcp_range
    }

    #[must_use]
    #[inline]
    pub fn builder() -> HandTypeBuilder {
        HandTypeBuilder::new()
    }

    /// Splits the type into its compiled shape set and HCP range.
    ///
    /// Crate-internal bridge to the deepened `HandConstraint`: `Shape::All`
    /// materializes to a full table here.
    pub(crate) fn into_parts(self) -> (Shapes, HcpRange) {
        let shapes = match self.shape {
            Shape::Custom(shapes) => shapes,
            Shape::All => Shapes::all(),
        };
        (shapes, self.hcp_range)
    }
}

/// Represents a descriptor for a set of possible hands with accepted shapes and High Card Point (HCP) ranges.
/// Basically, instead of writing a function that the [`Dealer`] will have to compute and check every time, we can pass
/// one `HandDescriptor` per seat.
/// A `HandDescriptor` is formed by a series of `HandType`s, which are a collection of shapes and HCP ranges allowed for the shapes.
/// This means that we can provide the dealer with a set of hand types we want to be selected, building them only onece, and the
/// dealer will simply check if the hand dealed match with one of the allowed [`HandType`]s.
///
/// # Example
///
/// ```
/// # use squeezer::prelude::*;
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>>{
/// // Create the HandType s using the HandTypeBuilder
/// let weak1n = HandType::builder()
///     .add_shape("(4432)")?
///     .add_shape("(4333)")?
///     .add_shape("(4414)")?
///     .add_shape("(5332)")?
///     .remove_shape("(5x)(2+2+)")?
///     .with_range(11, 14)
///     .build();
/// let unbal_with_clubs = HandType::builder()
///     .with_longest(Suit::Clubs)
///     .remove_shape("(4x)x5")?
///     .remove_shape("(5-5-5-)6+")?
///     .with_range(11, 14)
///     .build();
/// let strong_any = HandType::builder()
///     .add_shape("xxxx")?
///     .with_range(18, 37)
///     .build();
/// // Create the hand descriptor...
/// let polish_club = HandDescriptor::new(vec![weak1n, strong_any, unbal_with_clubs]);
/// // Give it to the dealer and he'll use it to give you what you asked for: a Polish Club 1♣ opening
/// let mut d_builder = DealerBuilder::new().with_hand_descriptor(Seat::South, polish_club);
/// let dealer = d_builder.build()?;
/// println!("Example of a Polish Club 1♣ opening:\n{}",dealer.deal()?);
/// # Ok(())
/// # }
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone)]
pub struct HandDescriptor {
    possible_hands: Vec<HandType>,
    pub(crate) min: u8,
    pub(crate) max: u8,
}

impl HandDescriptor {
    /// Check if given cards match any of the possible hand types based on shape
    /// and HCP range.
    #[must_use]
    #[inline]
    pub fn check(&self, cards: Cards) -> bool {
        let summary = HandSummary::of(cards);
        if (self.min..=self.max).contains(&summary.hcp) {
            self.possible_hands
                .iter()
                .any(|hand_type| hand_type.check_cached(summary))
        } else {
            false
        }
    }

    /// Create a new `HandDescriptor` with the specified list of possible hand types.
    #[must_use]
    #[inline]
    pub fn new(possible_hands: Vec<HandType>) -> Self {
        let mut min = MAX_HCP_IN_DECK + 1;
        let mut max = 0;
        for hand in &possible_hands {
            let hcp_range = hand.hcp_range();
            if hcp_range.range_min() < min {
                min = hcp_range.range_min();
            }
            if hcp_range.range_max() > max {
                max = hcp_range.range_max();
            }
        }
        Self {
            possible_hands,
            min,
            max,
        }
    }

    /// This method will create a new `HandDescriptor` that accepts each hand with a
    /// specific shape
    #[must_use]
    #[inline]
    pub fn shape(shape: Shape) -> Self {
        Self {
            possible_hands: vec![HandType {
                shape,
                hcp_range: HcpRange::default(),
            }],
            min: 0,
            max: MAX_HCP_IN_HAND,
        }
    }

    /// This method will create a new `HandDescriptor` that accepts each hand with a
    /// specific hcp range
    #[must_use]
    #[inline]
    pub fn hcp<U: std::ops::RangeBounds<T>, T: Into<u8> + Copy>(hcp_range: U) -> Self {
        let hcp_range = HcpRange::from_range(hcp_range);
        Self {
            min: hcp_range.range_min(),
            max: hcp_range.range_max(),
            possible_hands: vec![HandType {
                shape: Shape::All,
                hcp_range,
            }],
        }
    }

    /// Splits the descriptor into per-type `(shapes, HCP)` pairs.
    ///
    /// Crate-internal bridge to the deepened `HandConstraint`, which stores
    /// one branch per pair.
    pub(crate) fn into_branches(self) -> Vec<(Shapes, HcpRange)> {
        self.possible_hands
            .into_iter()
            .map(HandType::into_parts)
            .collect()
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
    pub fn add_shape(&mut self, pattern: &str) -> Result<&mut Self, CreationShapeError> {
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
    pub fn remove_shape(&mut self, pattern: &str) -> Result<&mut Self, CreationShapeError> {
        if let Some(ref mut shapes) = self.shapes {
            shapes.remove_shape(pattern)?;
        } else {
            self.shapes = Some(Shape::Custom(Shapes::but(pattern)?));
        }
        Ok(self)
    }

    /// You need to validate input for this function to work
    pub fn with_range(&mut self, min_hcp: u8, max_hcp: u8) -> &mut Self {
        self.hcp_range = Some(HcpRange::new(min_hcp, max_hcp));
        self
    }

    pub fn all_shapes(&mut self) -> &mut Self {
        self.shapes = Some(Shape::All);
        self
    }

    #[allow(clippy::missing_panics_doc)]
    /// Choose the longest suit. Be aware that is your responsiblity not to call
    /// this function more than once.
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
        }
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
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_builder_pattern() {
        let mut builder = HandTypeBuilder::new();
        _ = builder.with_range(8, 15).build();
        _ = builder.add_shape("4333").unwrap().build();
    }

    #[test]
    fn test_hcp_range() {
        let range = HcpRange::new(12, 15);
        assert!(range.contains(13));
        assert!(!range.contains(11));
        assert!(!range.contains(16));

        // Test clamping
        let range = HcpRange::new(40, 45);
        assert_eq!(range.range_max(), 37);
    }

    #[test]
    fn test_hand_type() {
        let hand_type = HandType::new(Shape::from_str("(4432)").unwrap(), HcpRange::new(12, 14));

        let valid_hand = Cards::from_str("AKQ2.KQ32.32.432").unwrap();
        assert!(hand_type.check(valid_hand));

        let invalid_hand = Cards::from_str("AKQJ2.KQ32.A2.32").unwrap(); // 5332 shape
        assert!(!hand_type.check(invalid_hand));
    }

    #[test]
    fn test_hand_descriptor() {
        let weak_nt = HandType::builder()
            .add_shape("(4432)")
            .unwrap()
            .add_shape("(4333)")
            .unwrap()
            .with_range(12, 14)
            .build();

        let strong_hand = HandType::builder().with_range(18, 37).build();

        let descriptor = HandDescriptor::new(vec![weak_nt, strong_hand]);

        let weak_nt_hand = Cards::from_str("AK52.K532.A2.432").unwrap();
        assert!(descriptor.check(weak_nt_hand));

        let strong_hand = Cards::from_str("AKQJ.AKQ2.AK2.32").unwrap();
        assert!(descriptor.check(strong_hand));

        let weak_unbalanced = Cards::from_str("AT632.K6432.T2.2").unwrap();
        assert!(!descriptor.check(weak_unbalanced));
    }

    #[test]
    fn test_complex_hand_descriptor() {
        let weak_two = HandType::builder()
            .with_longest(Suit::Hearts)
            .with_range(6, 10)
            .build();

        let three_level_preempt = HandType::builder()
            .add_shape("7xxx")
            .unwrap()
            .with_range(5, 9)
            .build();

        let descriptor = HandDescriptor::new(vec![weak_two, three_level_preempt]);

        // Valid weak two hand
        let weak_two_hand = Cards::from_str("Q2.KQJ432.432.32").unwrap();
        assert!(descriptor.check(weak_two_hand));

        // Valid three-level preempt
        let preempt_hand = Cards::from_str("KQJT432.32.32.32").unwrap();
        assert!(descriptor.check(preempt_hand));

        // Invalid hand (too strong)
        let strong_hand = Cards::from_str("AKQJ32.KQ432.A.2").unwrap();
        assert!(!descriptor.check(strong_hand));
    }

    #[test]
    fn test_error_handling() {
        // Test invalid shape pattern
        let mut builder = HandType::builder();
        let result = builder.add_shape("invalid");
        assert!(result.is_err());
    }
}
