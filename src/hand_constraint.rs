// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! # Hand-constraint module
//!
//! This module is the single home for "which hands may the dealer accept",
//! and the only per-seat constraint surface on the crate:
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use squeezer::*;
//! // Parse once, check millions of times.
//! let constraint = HandConstraint::new(&["(4432)", "(4333)", "(5332)"], HcpRange::new(12, 14))?;
//! let hand = Cards::from_str("AK52.K532.A2.432")?;
//! assert!(constraint.check(hand));
//! # Ok(())
//! # }
//! ```
//!
//! ## Why this shape
//!
//! - **Strings stay behind the seam.** Callers pass pattern strings once at
//!   construction; the scanner/parser/interpreter pipeline (`shapeparser`)
//!   is an internal implementation detail. There is a single
//!   parse-once/check-many path, so the old double pipeline (a dead
//!   `validate_shape` beside `ShapeCreator::build_shape`) cannot recur.
//! - **No `All` variant on the hot path.** "All shapes allowed" is stored as
//!   a full bit table (`Shapes::all()`), so `check` never dispatches on an
//!   enum variant with surprising semantics (`add` as silent no-op).
//! - **One fused check.** `check` computes the hand's `(shape, hcp)` summary
//!   once and does a single range-plus-table lookup per branch. The range
//!   pre-check lives in exactly one place: `Shapes::is_member_shape`.
//! - **One typed error.** Every fallible constructor returns
//!   [`ConstraintError`], preserving the parser's cause instead of
//!   flattening it into strings.

use crate::prelude::*;
use crate::range::Bounded;
use std::ops::RangeBounds;

/// Represents an *inclusive* range of High Card Points.
///
/// Thin newtype over the shared `Bounded` core (cap [`MAX_HCP_IN_HAND`]), so
/// clamping and membership share one implementation with [`LenRange`].
///
/// Constructors normalize inverted bounds differently: [`HcpRange::new`]
/// clamps `min` down to `max` (through `Bounded::new`), while `from` and
/// [`HcpRange::from_range`] swap the pair. Both preserve `min <= max`; only
/// the resulting window differs.
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

/// Pattern sets for [`HandConstraintBuilder::with_longest`], one per suit.
///
/// Each table unions to "this suit is (one of) the longest": two exact-shape
/// groups plus the 7+ anchor. Kept as `const` tables next to the builder so
/// the DSL strings appear exactly once in the crate.
const LONGEST_SPADES: [&str; 3] = ["55-4-4-", "66-5-5", "7+xxx"];
const LONGEST_HEARTS: [&str; 3] = ["4-55-5-", "5-65-5-", "x7+xx"];
const LONGEST_DIAMONDS: [&str; 3] = ["4-4-55-", "5-5-66-", "xx7+x"];
const LONGEST_CLUBS: [&str; 3] = ["4-4-4-5", "5-5-5-6", "xxx7+"];

/// Returns the longest-suit pattern table for a suit.
const fn longest_patterns(suit: Suit) -> [&'static str; 3] {
    match suit {
        Suit::Spades => LONGEST_SPADES,
        Suit::Hearts => LONGEST_HEARTS,
        Suit::Diamonds => LONGEST_DIAMONDS,
        Suit::Clubs => LONGEST_CLUBS,
    }
}

/// Typed error for everything the hand-constraint surface can reject.
///
/// This replaces the old split where `add_shape` returned `DealerError`
/// (a display string, detail discarded) while `remove_shape` returned
/// `Box<dyn Error>`. Every variant preserves its cause for `source()`-based
/// inspection and testing.
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ConstraintError {
    /// A shape pattern string failed to scan, parse, or interpret.
    InvalidPattern {
        /// The offending pattern, echoed for diagnostics.
        pattern: String,
        /// The underlying scanner/parser/interpreter cause.
        cause: CreationShapeError,
    },
    /// Construction would accept no hand at all (empty shape set).
    EmptyConstraint,
    /// `with_longest` was called for two different suits.
    ConflictingLongest {
        /// The suit from the first `with_longest` call.
        first: Suit,
        /// The suit from the conflicting second call.
        second: Suit,
    },
}

impl std::fmt::Display for ConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPattern { pattern, cause } => {
                write!(f, "invalid shape pattern {pattern:?}: {cause}")
            }
            Self::EmptyConstraint => write!(f, "hand constraint accepts no shape"),
            Self::ConflictingLongest { first, second } => write!(
                f,
                "conflicting longest suits: {} then {} (call with_longest at most once per suit family)",
                first.name(),
                second.name()
            ),
        }
    }
}

impl std::error::Error for ConstraintError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPattern { cause, .. } => Some(cause),
            Self::EmptyConstraint | Self::ConflictingLongest { .. } => None,
        }
    }
}

/// One acceptable hand family: a shape set plus an HCP window.
///
/// Private to this module: callers think in [`HandConstraint`]s (a union of
/// branches), never in branches directly.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct ConstraintBranch {
    shapes: Shapes,
    hcp: HcpRange,
}

/// Cached per-candidate summary: shape pattern + HCP.
///
/// Bundling the pair avoids recomputing shape/HCP once per branch (each costs
/// several popcounts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HandSummary {
    /// Suit lengths in S,H,D,C order.
    pub shape: [u8; 4],
    /// High card points.
    pub hcp: u8,
}

impl HandSummary {
    /// Computes the `(shape, hcp)` summary of a seat's cards.
    #[must_use]
    #[inline]
    fn of(cards: Cards) -> Self {
        Self {
            shape: cards.shape(),
            hcp: cards.hcp(),
        }
    }
}

/// Deep hand constraint: a union of acceptable `(shapes, HCP)` families.
///
/// This is the interface the dealer checks per candidate. Construction parses
/// pattern strings once; `check` is the only hot-path method and performs a
/// single fused lookup per branch.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HandConstraint {
    branches: Vec<ConstraintBranch>,
    /// Union HCP window over all branches: private fast-reject bound.
    ///
    /// Deliberately not exposed: it is derived at construction and used only
    /// inside `check`, so the branches stay the single source of truth.
    min_hcp: u8,
    max_hcp: u8,
}

impl HandConstraint {
    /// Builds a constraint from pattern strings and an HCP range.
    ///
    /// An empty pattern list means "any shape". Patterns are parsed once
    /// here; the resulting bit table serves every later `check`.
    ///
    /// # Errors
    ///
    /// Returns [`ConstraintError::InvalidPattern`] if any pattern is malformed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let weak_nt = HandConstraint::new(&["(4432)", "(4333)"], HcpRange::new(12, 14))?;
    /// assert!(weak_nt.check(Cards::from_str("AK52.K532.A2.432")?));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(patterns: &[&str], hcp: HcpRange) -> Result<Self, ConstraintError> {
        // No patterns means no shape restriction: start from the full table
        // rather than the empty one (which would accept nothing).
        let mut shapes = if patterns.is_empty() {
            Shapes::all()
        } else {
            Shapes::new()
        };
        for pattern in patterns {
            shapes
                .try_add_pattern(pattern)
                .map_err(|cause| ConstraintError::InvalidPattern {
                    pattern: (*pattern).to_owned(),
                    cause,
                })?;
        }
        Ok(Self::from_compiled(shapes, hcp))
    }

    /// Builds a "this suit is longest" constraint (see [`HandConstraintBuilder::with_longest`]).
    ///
    /// # Errors
    ///
    /// Returns [`ConstraintError::InvalidPattern`] if the internal pattern
    /// table ever stops parsing (a bug, not user input: the tables are
    /// `const` and covered by per-suit tests).
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer::*;
    /// let spades = HandConstraint::longest(Suit::Spades, HcpRange::new(5, 10))?;
    /// assert!(spades.check(Cards::from_str("AKQJ432.32.32.32")?));
    /// # Ok(())
    /// # }
    /// ```
    pub fn longest(suit: Suit, hcp: HcpRange) -> Result<Self, ConstraintError> {
        let mut shapes = Shapes::new();
        for pattern in longest_patterns(suit) {
            shapes
                .try_add_pattern(pattern)
                .map_err(|cause| ConstraintError::InvalidPattern {
                    pattern: pattern.to_owned(),
                    cause,
                })?;
        }
        Ok(Self::from_compiled(shapes, hcp))
    }

    /// Builds a constraint over HCP only (any shape).
    ///
    /// Infallible: no patterns to parse.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use squeezer::*;
    /// let opener = HandConstraint::hcp(12..=17);
    /// assert!(opener.check(Cards::from_str("AK52.K532.A2.432").unwrap()));
    /// ```
    #[must_use]
    pub fn hcp<R, U>(hcp_range: R) -> Self
    where
        R: RangeBounds<U>,
        U: Into<u8> + Copy,
    {
        let hcp = HcpRange::from_range(hcp_range);
        Self::from_compiled(Shapes::all(), hcp)
    }

    /// Builds a constraint accepting every hand.
    #[must_use]
    pub fn all() -> Self {
        Self::from_compiled(Shapes::all(), HcpRange::default())
    }

    /// Unions several constraints: a hand passes if any alternative passes.
    ///
    /// This is how multi-family hands (e.g. a Polish Club opening with a
    /// weak, an unbalanced, and a strong alternative) are expressed without
    /// exposing branch plumbing on the interface.
    ///
    /// # Errors
    ///
    /// Returns [`ConstraintError::EmptyConstraint`] for an empty list.
    pub fn any(constraints: Vec<Self>) -> Result<Self, ConstraintError> {
        if constraints.is_empty() {
            return Err(ConstraintError::EmptyConstraint);
        }
        let mut branches = Vec::new();
        let mut min_hcp = u8::MAX;
        let mut max_hcp = u8::MIN;
        for constraint in constraints {
            min_hcp = min_hcp.min(constraint.min_hcp);
            max_hcp = max_hcp.max(constraint.max_hcp);
            branches.extend(constraint.branches);
        }
        Ok(Self {
            branches,
            min_hcp,
            max_hcp,
        })
    }

    /// Creates a builder for combinable construction.
    #[must_use]
    pub fn builder() -> HandConstraintBuilder {
        HandConstraintBuilder::new()
    }

    /// Wraps one precompiled shape set and HCP range.
    ///
    /// Private seam for constructors that already hold a `Shapes`: no
    /// parsing, no failure modes.
    fn from_compiled(shapes: Shapes, hcp: HcpRange) -> Self {
        let (min_hcp, max_hcp) = hcp.into();
        Self {
            branches: vec![ConstraintBranch { shapes, hcp }],
            min_hcp,
            max_hcp,
        }
    }

    /// Checks whether a seat's cards satisfy any branch of this constraint.
    ///
    /// The cards' `(shape, hcp)` summary is computed once, then each branch
    /// runs a single fused range-plus-table lookup. Holdings outside the union
    /// HCP window are rejected without touching the shape tables.
    #[must_use]
    #[inline]
    pub fn check(&self, cards: Cards) -> bool {
        let summary = HandSummary::of(cards);
        if !(self.min_hcp..=self.max_hcp).contains(&summary.hcp) {
            return false;
        }
        self.branches.iter().any(|branch| {
            branch.hcp.contains(summary.hcp) && branch.shapes.is_member_shape(summary.shape)
        })
    }
}

/// Consuming builder for [`HandConstraint`].
///
/// Methods take `self` and return `Self` (matching `DealerBuilder` per the
/// Rust API guidelines), so configuration chains without `mut` bindings.
/// Every pattern-taking method is fallible with [`ConstraintError`]; nothing
/// here panics on user input.
#[derive(Debug, Default)]
pub struct HandConstraintBuilder {
    shapes: Option<Shapes>,
    hcp: Option<HcpRange>,
    /// Tracks `with_longest` so a conflicting second call is a typed error,
    /// not silent nonsense (Q12-b).
    longest: Option<Suit>,
}

impl HandConstraintBuilder {
    /// Starts an unconstrained builder (all shapes, full HCP range at `build`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a builder accepting balanced shapes (`4333`, `4432`, `5332`).
    #[must_use]
    pub fn balanced(min_hcp: u8, max_hcp: u8) -> Self {
        let mut shapes = Shapes::new();
        shapes.add_balanced();
        Self {
            shapes: Some(shapes),
            hcp: Some(HcpRange::new(min_hcp, max_hcp)),
            longest: None,
        }
    }

    /// Unions the shapes matching `pattern` into the constraint.
    ///
    /// # Errors
    ///
    /// Returns [`ConstraintError::InvalidPattern`] if the pattern is malformed.
    pub fn add_shape(self, pattern: &str) -> Result<Self, ConstraintError> {
        let mut this = self;
        if let Some(ref mut shapes) = this.shapes {
            shapes
                .try_add_pattern(pattern)
                .map_err(|cause| ConstraintError::InvalidPattern {
                    pattern: pattern.to_owned(),
                    cause,
                })?;
        } else {
            let mut shapes = Shapes::new();
            shapes
                .try_add_pattern(pattern)
                .map_err(|cause| ConstraintError::InvalidPattern {
                    pattern: pattern.to_owned(),
                    cause,
                })?;
            this.shapes = Some(shapes);
        }
        Ok(this)
    }

    /// Removes the shapes matching `pattern` from the constraint.
    ///
    /// Called before any `add_shape`, it starts from "all shapes" (the old
    /// `Shapes::but` behavior), now with a typed error.
    ///
    /// # Errors
    ///
    /// Returns [`ConstraintError::InvalidPattern`] if the pattern is malformed.
    pub fn remove_shape(self, pattern: &str) -> Result<Self, ConstraintError> {
        let mut this = self;
        if let Some(ref mut shapes) = this.shapes {
            shapes.try_remove_pattern(pattern).map_err(|cause| {
                ConstraintError::InvalidPattern {
                    pattern: pattern.to_owned(),
                    cause,
                }
            })?;
        } else {
            this.shapes = Some(Shapes::all_but(pattern).map_err(|cause| {
                ConstraintError::InvalidPattern {
                    pattern: pattern.to_owned(),
                    cause,
                }
            })?);
        }
        Ok(this)
    }

    /// Unions the "this suit is longest" family into the constraint.
    ///
    /// The ergonomic core of opening-bid descriptors (e.g. "unbalanced with
    /// clubs"): one call buys the exact-shape groups plus the 7+ anchor that
    /// callers would otherwise hand-roll from the DSL. Backed by the `const`
    /// per-suit tables, so the pattern strings exist once in the crate.
    ///
    /// Calling twice with the same suit is idempotent; calling with a
    /// different suit is a [`ConstraintError::ConflictingLongest`] — a
    /// longest-suit family for two suits at once is almost certainly a
    /// caller bug, and a runtime guard keeps the builder free of typestate
    /// generics.
    ///
    /// # Errors
    ///
    /// Returns [`ConstraintError::ConflictingLongest`] on a second call with
    /// a different suit, or [`ConstraintError::InvalidPattern`] if an
    /// internal table entry ever stops parsing.
    pub fn with_longest(self, suit: Suit) -> Result<Self, ConstraintError> {
        let mut this = self;
        match this.longest {
            Some(first) if first == suit => return Ok(this),
            Some(first) => {
                return Err(ConstraintError::ConflictingLongest {
                    first,
                    second: suit,
                });
            }
            None => {}
        }
        for pattern in longest_patterns(suit) {
            this = this.add_shape(pattern)?;
        }
        this.longest = Some(suit);
        Ok(this)
    }

    /// Sets the HCP window (infallible; values clamp to `[0, 37]`).
    #[must_use]
    pub fn with_range(self, min_hcp: u8, max_hcp: u8) -> Self {
        Self {
            hcp: Some(HcpRange::new(min_hcp, max_hcp)),
            ..self
        }
    }

    /// Accepts all shapes (full table, no enum dispatch on the hot path).
    #[must_use]
    pub fn all_shapes(self) -> Self {
        Self {
            shapes: Some(Shapes::all()),
            ..self
        }
    }

    /// Finishes the constraint.
    ///
    /// Unset shapes default to all shapes; unset HCP defaults to the full
    /// `[0, 37]` range (mirroring the legacy builder's defaults).
    ///
    /// # Errors
    ///
    /// Returns [`ConstraintError::EmptyConstraint`] when the accumulated
    /// shape set is empty (e.g. everything added was then removed).
    pub fn build(self) -> Result<HandConstraint, ConstraintError> {
        let shapes = self.shapes.unwrap_or_else(Shapes::all);
        if shapes.is_empty() {
            return Err(ConstraintError::EmptyConstraint);
        }
        Ok(HandConstraint::from_compiled(
            shapes,
            self.hcp.unwrap_or_default(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    fn spades_longest_hand() -> Cards {
        Cards::from_str("AKQJ432.32.32.32").unwrap()
    }

    fn hearts_longest_hand() -> Cards {
        Cards::from_str("32.AKQJ432.32.32").unwrap()
    }

    #[test]
    fn new_checks_shape_and_hcp() {
        let constraint = HandConstraint::new(&["(4432)"], HcpRange::new(12, 14)).unwrap();
        assert!(constraint.check(Cards::from_str("AKQ2.KQ32.32.432").unwrap()));
        // Right shape, wrong strength.
        assert!(!constraint.check(Cards::from_str("AKQJ.AKQ2.AK2.32").unwrap()));
        // Right strength, wrong shape.
        assert!(!constraint.check(Cards::from_str("AKQJ2.KQ32.A2.32").unwrap()));
    }

    #[test]
    fn bare_pattern_is_positional_group_is_permuted() {
        // Documents the DSL seam: a bare pattern matches suit order exactly,
        // while a parenthesized group matches every permutation.
        let exact = HandConstraint::new(&["4432"], HcpRange::new(0, 37)).unwrap();
        assert!(exact.check(Cards::from_str("AKQ2.KQ32.AK2.32").unwrap()));
        assert!(!exact.check(Cards::from_str("AKQ2.KQ32.32.432").unwrap()));
        let grouped = HandConstraint::new(&["(4432)"], HcpRange::new(0, 37)).unwrap();
        assert!(grouped.check(Cards::from_str("AKQ2.KQ32.32.432").unwrap()));
    }

    #[test]
    fn new_with_empty_patterns_accepts_any_shape() {
        let constraint = HandConstraint::new(&[], HcpRange::new(12, 14)).unwrap();
        // Both hands hold 14 HCP; shapes differ (4-4-2-3 vs 5-3-3-2).
        assert!(constraint.check(Cards::from_str("AKQ2.KQ32.32.432").unwrap()));
        assert!(constraint.check(Cards::from_str("AQJ32.K32.A32.32").unwrap()));
        // Strength still filters: 19 HCP is rejected.
        assert!(!constraint.check(Cards::from_str("AKQJ2.KQ32.A2.32").unwrap()));
    }

    #[test]
    fn new_rejects_malformed_pattern_with_cause() {
        let err = HandConstraint::new(&["not a shape!!"], HcpRange::new(0, 37)).unwrap_err();
        match &err {
            ConstraintError::InvalidPattern { pattern, cause } => {
                assert_eq!(pattern, "not a shape!!");
                // The parser cause survives instead of being flattened to a string.
                let _ = cause.to_string();
                assert!(std::error::Error::source(&err).is_some());
            }
            other => panic!("expected InvalidPattern, got {other:?}"),
        }
    }

    #[test]
    fn longest_accepts_per_suit() {
        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            let constraint = HandConstraint::longest(suit, HcpRange::new(0, 37)).unwrap();
            let hand = match suit {
                Suit::Spades => spades_longest_hand(),
                Suit::Hearts => hearts_longest_hand(),
                Suit::Diamonds => Cards::from_str("32.32.AKQJ432.32").unwrap(),
                Suit::Clubs => Cards::from_str("32.32.32.AKQJ432").unwrap(),
            };
            assert!(
                constraint.check(hand),
                "longest {suit:?} should accept its 7-card hand"
            );
        }
    }

    #[test]
    fn longest_rejects_other_suit_anchor() {
        let spades = HandConstraint::longest(Suit::Spades, HcpRange::new(0, 37)).unwrap();
        assert!(!spades.check(hearts_longest_hand()));
    }

    #[test]
    fn builder_chains_without_mut() {
        // Consuming builder: no `mut` binding needed, mirroring DealerBuilder.
        let constraint = HandConstraintBuilder::new()
            .add_shape("(4432)")
            .unwrap()
            .add_shape("(4333)")
            .unwrap()
            .remove_shape("4441")
            .unwrap()
            .with_range(11, 14)
            .build()
            .unwrap();
        assert!(constraint.check(Cards::from_str("AK52.K532.A2.432").unwrap()));
        assert!(!constraint.check(Cards::from_str("AKQJ.AKQ2.AK2.32").unwrap()));
    }

    #[test]
    fn builder_remove_before_add_starts_from_all() {
        let constraint = HandConstraintBuilder::new()
            .remove_shape("(7xxx)")
            .unwrap()
            .with_range(0, 37)
            .build()
            .unwrap();
        assert!(constraint.check(Cards::from_str("AKQ2.KQ32.32.432").unwrap()));
        assert!(!constraint.check(Cards::from_str("AKQJ432.32.32.32").unwrap()));
    }

    #[test]
    fn builder_empty_set_is_an_error_not_a_silent_trap() {
        let err = HandConstraintBuilder::new()
            .add_shape("4432")
            .unwrap()
            .remove_shape("4432")
            .unwrap()
            .build()
            .unwrap_err();
        assert!(matches!(err, ConstraintError::EmptyConstraint));
    }

    #[test]
    fn with_longest_same_suit_is_idempotent() {
        let constraint = HandConstraintBuilder::new()
            .with_longest(Suit::Clubs)
            .unwrap()
            .with_longest(Suit::Clubs)
            .unwrap()
            .with_range(0, 37)
            .build()
            .unwrap();
        assert!(constraint.check(Cards::from_str("32.32.32.AKQJ432").unwrap()));
    }

    #[test]
    fn with_longest_conflicting_suit_is_an_error() {
        let err = HandConstraintBuilder::new()
            .with_longest(Suit::Clubs)
            .unwrap()
            .with_longest(Suit::Spades)
            .unwrap_err();
        match err {
            ConstraintError::ConflictingLongest { first, second } => {
                assert_eq!(first, Suit::Clubs);
                assert_eq!(second, Suit::Spades);
            }
            other => panic!("expected ConflictingLongest, got {other:?}"),
        }
    }

    #[test]
    fn any_unions_alternatives() {
        let weak = HandConstraint::new(&["(4432)"], HcpRange::new(11, 14)).unwrap();
        let strong = HandConstraint::hcp(18..=37);
        let polish = HandConstraint::any(vec![weak, strong]).unwrap();
        assert!(polish.check(Cards::from_str("AK52.K532.A2.432").unwrap()));
        assert!(polish.check(Cards::from_str("AKQJ.AKQ2.AK2.32").unwrap()));
        assert!(!polish.check(Cards::from_str("AT632.K6432.T2.2").unwrap()));
    }

    #[test]
    fn any_rejects_empty_list() {
        assert!(matches!(
            HandConstraint::any(Vec::new()).unwrap_err(),
            ConstraintError::EmptyConstraint
        ));
    }

    #[test]
    fn hcp_only_constructor_uses_an_inclusive_window() {
        // "AK52.K532.A2.432" holds 14 HCP: A+K spades (7), K hearts (3),
        // A diamonds (4). An inclusive 12..=14 window accepts it; the
        // exclusive 12..14 range would not (14 excluded).
        let constraint = HandConstraint::hcp(12..=14);
        assert!(constraint.check(Cards::from_str("AK52.K532.A2.432").unwrap()));
        assert!(!HandConstraint::hcp(12..14).check(Cards::from_str("AK52.K532.A2.432").unwrap()));
    }

    #[test]
    fn hcp_range_clamps_and_reports_membership() {
        let range = HcpRange::new(12, 15);
        assert!(range.contains(13));
        assert!(!range.contains(11));
        assert!(!range.contains(16));

        // Test clamping
        let range = HcpRange::new(40, 45);
        assert_eq!(range.range_max(), 37);
    }
}
