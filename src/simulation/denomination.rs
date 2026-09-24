// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Denomination
//!
//! How an outcome is valued. Raw bridge points are the default; a comparison
//! against a baseline under a scoring (IMPs, matchpoints) is opt-in.
//!
//! The design (ticket 06, ADR-0006) intends one `Denomination` mechanism for
//! both card-EV and the contract payoff matrix. Card-EV values each option
//! against the *other options on the same deal*; contract payoff compares
//! contracts *pairwise*, so [`Denomination::difference`] applies the same
//! scoring to the two contracts in the pair. Both paths share this type.
//!
//! ## Why the closures are not serializable
//!
//! `Baseline::Custom` and `Scoring::Custom` carry a boxed closure, which has no
//! serialized form. They are `#[serde(skip)]`, so serializing a value that uses
//! one fails at runtime and deserializing cannot produce one; the standard
//! variants round-trip normally.

use std::fmt;

/// How an outcome is valued.
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Denomination {
    /// The raw bridge score itself (`3NT` making = 400, `2H` = 110).
    #[default]
    RawPoints,
    /// A comparison of the score against a per-deal baseline, under a scoring.
    Compared {
        /// What the option's score is compared against on each deal.
        baseline: Baseline,
        /// How the score-to-baseline difference becomes a value.
        scoring: Scoring,
    },
}

impl Denomination {
    /// Values one option's `score` given the scores of the other options on the
    /// same deal.
    ///
    /// `RawPoints` ignores the alternatives and returns the score unchanged;
    /// `Compared` resolves the baseline over `other_scores` and applies the
    /// scoring.
    #[must_use]
    pub fn value(&self, score: i32, other_scores: &[i32]) -> i32 {
        match self {
            Self::RawPoints => score,
            Self::Compared { baseline, scoring } => {
                scoring.value(score, baseline.resolve(other_scores))
            }
        }
    }

    /// The value of `score_a` in a pairwise comparison against `score_b`.
    ///
    /// A pairwise comparison is [`Self::value`] with the opponent as the only
    /// alternative, so [`Baseline::AverageOfOptions`] and [`Baseline::BestOption`]
    /// both resolve to the opponent's score and the standard [`Scoring`]s
    /// (`Imps`, `Matchpoints`) make the result antisymmetric
    /// (`difference(a, b) == -difference(b, a)`). That reproduces the old bare
    /// pairwise `scoring_fn(a, b)` used by the contract payoff matrix, which
    /// surfaces the denomination only through
    /// [`PairwiseComparison::difference`](crate::simulation::PairwiseComparison);
    /// [`DealResult::score`](crate::simulation::DealResult) stays the raw score.
    ///
    /// # `Baseline::Fixed` is not a pairwise baseline
    ///
    /// `Fixed` ignores the opponent: both `difference(a, b)` and
    /// `difference(b, a)` score against the fixed score, so the result is *not*
    /// antisymmetric. It is unsupported in the contract payoff matrix, whose
    /// rankings aggregate only the `option_a` side of each directed pair and so
    /// assume antisymmetry; use it only to score each option against a reference
    /// (par), not to compare two options with each other. A `Custom` baseline
    /// that ignores its input has the same caveat.
    #[must_use]
    pub fn difference(&self, score_a: i32, score_b: i32) -> i32 {
        match self {
            Self::RawPoints => score_a - score_b,
            Self::Compared { .. } => self.value(score_a, &[score_b]),
        }
    }
}

impl fmt::Debug for Denomination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawPoints => write!(f, "RawPoints"),
            Self::Compared { baseline, scoring } => f
                .debug_struct("Compared")
                .field("baseline", baseline)
                .field("scoring", scoring)
                .finish(),
        }
    }
}

/// A caller-supplied baseline over the other options' scores.
pub type CustomBaseline = Box<dyn Fn(&[i32]) -> i32 + Send + Sync>;

/// A caller-supplied scoring of `(score, baseline)`.
pub type CustomScoring = Box<dyn Fn(i32, i32) -> i32 + Send + Sync>;

/// What an option's score is compared against on each deal.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Baseline {
    /// The mean score of the other candidate options.
    AverageOfOptions,
    /// The best other candidate option's score.
    BestOption,
    /// A fixed reference score (par, a reference contract, ...).
    Fixed(i32),
    /// A caller-supplied function of the other options' scores.
    #[cfg_attr(feature = "serde", serde(skip))]
    Custom(CustomBaseline),
}

impl Baseline {
    /// Resolves the baseline against the other options' scores.
    ///
    /// An empty slice is the no-alternatives case: the average and best are
    /// both zero, so a comparison against "nothing" is a comparison against par
    /// of zero rather than a panic.
    #[must_use]
    pub fn resolve(&self, other_scores: &[i32]) -> i32 {
        match self {
            Self::AverageOfOptions => {
                if other_scores.is_empty() {
                    0
                } else {
                    let count = i32::try_from(other_scores.len()).unwrap_or(i32::MAX);
                    other_scores.iter().sum::<i32>() / count
                }
            }
            Self::BestOption => other_scores.iter().copied().max().unwrap_or(0),
            Self::Fixed(score) => *score,
            Self::Custom(f) => f(other_scores),
        }
    }
}

impl fmt::Debug for Baseline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AverageOfOptions => write!(f, "AverageOfOptions"),
            Self::BestOption => write!(f, "BestOption"),
            Self::Fixed(score) => write!(f, "Fixed({score})"),
            Self::Custom(_) => write!(f, "Custom(<closure>)"),
        }
    }
}

/// How a score-to-baseline difference becomes a value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Scoring {
    /// International match points, via [`imps`](crate::simulation::imps).
    Imps,
    /// Matchpoints: +1 for a better score, -1 for a worse one, 0 for equal.
    Matchpoints,
    /// A caller-supplied function of `(score, baseline)`.
    #[cfg_attr(feature = "serde", serde(skip))]
    Custom(CustomScoring),
}

impl Scoring {
    /// Applies the scoring to `score` against `baseline`.
    #[must_use]
    pub fn value(&self, score: i32, baseline: i32) -> i32 {
        match self {
            Self::Imps => crate::simulation::imps(score, baseline),
            Self::Matchpoints => crate::simulation::matchpoints(score, baseline),
            Self::Custom(f) => f(score, baseline),
        }
    }
}

impl fmt::Debug for Scoring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Imps => write!(f, "Imps"),
            Self::Matchpoints => write!(f, "Matchpoints"),
            Self::Custom(_) => write!(f, "Custom(<closure>)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_points_returns_the_score_unchanged() {
        let denomination = Denomination::default();
        assert_eq!(denomination.value(400, &[90, 110]), 400);
        assert_eq!(denomination.value(-50, &[]), -50);
    }

    #[test]
    fn compared_baselines_resolve_as_documented() {
        let others = [80, 80, 110];
        assert_eq!(Baseline::AverageOfOptions.resolve(&others), 90);
        assert_eq!(Baseline::BestOption.resolve(&others), 110);
        assert_eq!(Baseline::Fixed(120).resolve(&others), 120);
        assert_eq!(
            Baseline::Custom(Box::new(|scores: &[i32]| scores.iter().sum())).resolve(&others),
            270
        );
        assert_eq!(Baseline::AverageOfOptions.resolve(&[]), 0);
        assert_eq!(Baseline::BestOption.resolve(&[]), 0);
    }

    #[test]
    fn pairwise_difference_uses_the_opponent_as_the_baseline() {
        // Raw points is the raw score difference.
        assert_eq!(Denomination::RawPoints.difference(110, 90), 20);
        assert_eq!(Denomination::RawPoints.difference(90, 110), -20);

        // The opponent is the only alternative, so an average/best baseline
        // resolves to its score and the scoring becomes the old pairwise one.
        let imps_denomination = Denomination::Compared {
            baseline: Baseline::AverageOfOptions,
            scoring: Scoring::Imps,
        };
        assert_eq!(
            imps_denomination.difference(110, 90),
            crate::simulation::imps(110, 90)
        );
        assert_eq!(
            imps_denomination.difference(90, 110),
            crate::simulation::imps(90, 110)
        );

        let matchpoints_denomination = Denomination::Compared {
            baseline: Baseline::BestOption,
            scoring: Scoring::Matchpoints,
        };
        assert_eq!(matchpoints_denomination.difference(110, 90), 1);
        assert_eq!(matchpoints_denomination.difference(90, 110), -1);
        assert_eq!(matchpoints_denomination.difference(90, 90), 0);

        // A fixed baseline ignores the opponent: both contracts are scored
        // against the fixed score, so the pair is not antisymmetric.
        let fixed = Denomination::Compared {
            baseline: Baseline::Fixed(0),
            scoring: Scoring::Imps,
        };
        assert_eq!(fixed.difference(50, 100), crate::simulation::imps(50, 0));
        assert_eq!(fixed.difference(50, 999), crate::simulation::imps(50, 0));
        assert_ne!(
            fixed.difference(50, 100),
            -fixed.difference(100, 50),
            "a fixed baseline is not a pairwise baseline"
        );
    }

    #[test]
    fn compared_values_apply_the_scoring() {
        let denomination = Denomination::Compared {
            baseline: Baseline::Fixed(0),
            scoring: Scoring::Imps,
        };
        // 80 raw points against par 0 is a 2-IMP swing.
        assert_eq!(denomination.value(80, &[]), 2);
        assert_eq!(denomination.value(110, &[]), 3);

        let denomination = Denomination::Compared {
            baseline: Baseline::BestOption,
            scoring: Scoring::Custom(Box::new(|score, baseline| score - baseline)),
        };
        // Best other option is 110, so an 80 card is 30 points worse.
        assert_eq!(denomination.value(80, &[80, 110]), -30);
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;

    #[test]
    fn standard_denominations_round_trip() {
        let raw = Denomination::RawPoints;
        let json = serde_json::to_string(&raw).expect("raw points serialize");
        let back: Denomination = serde_json::from_str(&json).expect("raw points deserialize");
        assert!(matches!(back, Denomination::RawPoints));

        let compared = Denomination::Compared {
            baseline: Baseline::AverageOfOptions,
            scoring: Scoring::Imps,
        };
        let json = serde_json::to_string(&compared).expect("compared serializes");
        let back: Denomination = serde_json::from_str(&json).expect("compared deserializes");
        assert!(matches!(
            back,
            Denomination::Compared {
                baseline: Baseline::AverageOfOptions,
                scoring: Scoring::Imps,
            }
        ));
    }

    #[test]
    fn custom_denominations_are_not_serializable() {
        let baseline = Baseline::Custom(Box::new(|_| 0));
        assert!(serde_json::to_string(&baseline).is_err());
        let scoring = Scoring::Custom(Box::new(|score, _| score));
        assert!(serde_json::to_string(&scoring).is_err());
    }
}
