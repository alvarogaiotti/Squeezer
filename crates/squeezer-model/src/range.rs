// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! # Single home for inclusive `u8` range clamping
//!
//! `HcpRange` (cap 37) and `LenRange` (cap 13) used to reimplement the same
//! idea in parallel: clamp, order, contains. The one invariant worth owning
//! centrally is `0 <= min <= max <= MAX`, enforced once here in
//! [`Bounded::new`]. Both public types are thin newtypes over it, so a
//! boundary fix lands in exactly one place.

/// Inclusive range with the invariant `0 <= min <= max <= MAX`.
///
/// Construction clamps both ends into `[0, MAX]`, then clamps `min` into
/// `[0, max]`: an inverted input like `(5, 3)` becomes `(3, 3)` rather than
/// silently widening to `(5, 5)` or swapping to `(3, 5)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Bounded<const MAX: u8> {
    min: u8,
    max: u8,
}

impl<const MAX: u8> Bounded<MAX> {
    /// Builds the range, enforcing `0 <= min <= max <= MAX`.
    #[must_use]
    pub(crate) const fn new(min: u8, max: u8) -> Self {
        let max = if max > MAX { MAX } else { max };
        let min = if min > max { max } else { min };
        Self { min, max }
    }

    /// Builds the unrestricted range `0..=MAX`.
    #[must_use]
    pub(crate) const fn full() -> Self {
        Self { min: 0, max: MAX }
    }

    /// Lower bound.
    #[must_use]
    pub(crate) const fn min(self) -> u8 {
        self.min
    }

    /// Upper bound.
    #[must_use]
    pub(crate) const fn max(self) -> u8 {
        self.max
    }

    /// Membership test.
    #[must_use]
    pub(crate) const fn contains(self, value: u8) -> bool {
        self.min <= value && value <= self.max
    }

    /// Converts into the `(min, max)` pair.
    #[must_use]
    pub(crate) const fn as_pair(self) -> (u8, u8) {
        (self.min, self.max)
    }

    /// Converts into an inclusive Rust range.
    #[must_use]
    pub(crate) const fn as_range(self) -> core::ops::RangeInclusive<u8> {
        self.min..=self.max
    }
}

impl<const MAX: u8> Default for Bounded<MAX> {
    fn default() -> Self {
        Self::full()
    }
}
