// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Bridge Simulation API - Core Types
//!
//! This module provides a redesigned API for Monte Carlo simulations in Bridge.
//! The design focuses on:
//! - Separation of raw data from computed statistics
//! - GUI-friendly structured data access
//! - Flexible result consumption patterns
//! - Clear documentation and type safety

use crate::prelude::*;
use std::collections::HashMap;

// ============================================================================
// TRAIT DEFINITIONS - Core abstractions
// ============================================================================

// NOTE: a `Comparable` marker trait (`Clone + Eq + Hash`, implemented only
// for `Card` and `Contract`) used to live here. It added generic-parameter
// noise without abstraction value, so the bounds are spelled out at each
// use site instead.

/// Core trait for running simulations.
/// Simulations produce results that can be consumed in various ways.
pub trait Simulation {
    /// The type of raw result data this simulation produces
    type RawResult;

    /// The type of computed statistics this simulation produces
    type Statistics;

    /// Run the simulation and return both raw data and computed statistics
    /// # Errors
    /// Errors if any of the parameter is not liked by DDS. This could happen if the
    /// deal is wrongly set up (cards not in the right hands, cards already played present, etc).
    /// This function tries to do it's best to realize if something is wrong before calling the DDS
    /// library to ensure informative errors, but somethings might slip through and
    /// you'll have to rely on DDS errors to understand what went wrong.
    fn run(self) -> Result<SimulationOutput<Self::RawResult, Self::Statistics>, SqueezerError>;
}

// ============================================================================
// SIMULATION OUTPUT - Wrapper for results
// ============================================================================

/// Container for simulation results, providing both raw data and computed statistics.
///
/// This is the primary return type from all simulations. It provides:
/// - Access to raw simulation data for custom analysis and visualization
/// - Pre-computed statistics for common use cases
/// - Methods to extract specific data subsets
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimulationOutput<R, S> {
    /// Raw data from the simulation (e.g., individual deal results)
    pub raw: R,

    /// Computed statistics (e.g., means, standard deviations, percentages)
    pub statistics: S,

    /// Metadata about the simulation run
    pub metadata: SimulationMetadata,
}

impl<R, S> SimulationOutput<R, S> {
    /// Create a new simulation output with raw data, statistics, and metadata
    pub fn new(raw: R, statistics: S, metadata: SimulationMetadata) -> Self {
        Self {
            raw,
            statistics,
            metadata,
        }
    }

    /// Consume the output and return just the raw data
    pub fn into_raw(self) -> R {
        self.raw
    }

    /// Consume the output and return just the statistics
    pub fn into_statistics(self) -> S {
        self.statistics
    }

    /// Decompose into all components
    pub fn into_parts(self) -> (R, S, SimulationMetadata) {
        (self.raw, self.statistics, self.metadata)
    }
}

/// Metadata about a simulation run, useful for displaying context in a GUI
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimulationMetadata {
    /// Total number of deals simulated
    pub num_deals: usize,

    /// Number of items being compared (contracts, cards, etc.)
    pub num_comparisons: usize,

    /// Timestamp when the simulation was run
    // #[cfg(feature = "chrono")]
    // pub timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

impl SimulationMetadata {
    /// Create new metadata with required fields
    #[must_use]
    #[inline]
    pub fn new(num_deals: usize, num_comparisons: usize) -> Self {
        Self {
            num_deals,
            num_comparisons,
            // #[cfg(feature = "chrono")]
            // timestamp: Some(chrono::Utc::now()),
            custom: HashMap::new(),
        }
    }

    /// Add custom metadata field
    #[must_use]
    #[inline]
    pub fn with_custom(mut self, key: String, value: String) -> Self {
        self.custom.insert(key, value);
        self
    }
}

// ============================================================================
// DEAL RESULT - Individual deal outcome
// ============================================================================

/// Result from a single deal in the simulation.
/// This is the atomic unit of raw data.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DealResult {
    /// Index of this deal in the simulation
    pub deal_index: usize,

    /// Number of tricks taken
    pub tricks: u8,

    /// Score achieved
    pub score: i32,
}

impl DealResult {
    /// Create a new deal result
    #[must_use]
    #[inline]
    pub fn new(deal_index: usize, tricks: u8, score: i32) -> Self {
        Self {
            deal_index,
            tricks,
            score,
        }
    }
}

// ============================================================================
// COMPARISON RESULT - Pairwise comparison outcome
// ============================================================================

/// Result of comparing two options (contracts or cards) on a single deal.
/// This represents the raw difference data that can be used for detailed analysis.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComparisonResult {
    /// Index of the deal this comparison is for
    pub deal_index: usize,

    /// Score for the first option
    pub score_a: i32,

    /// Score for the second option
    pub score_b: i32,

    /// Difference in score (`score_a` `score_b`) after applying the scoring function
    pub difference: i32,
}

impl ComparisonResult {
    /// Create a new comparison result
    #[must_use]
    #[inline]
    pub fn new(deal_index: usize, score_a: i32, score_b: i32, difference: i32) -> Self {
        Self {
            deal_index,
            score_a,
            score_b,
            difference,
        }
    }

    /// Get the absolute difference
    #[must_use]
    #[inline]
    pub fn abs_difference(&self) -> i32 {
        self.difference.abs()
    }
}

// ============================================================================
// PAIRWISE STATISTICS - Statistics for a single comparison
// ============================================================================

/// Computed statistics for comparing two options across all deals.
/// This is what most GUIs will want to display.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PairwiseStatistics {
    /// Mean difference across all deals
    pub mean: f64,

    /// Standard deviation of differences
    pub std_dev: f64,

    /// Standard error of the mean
    pub std_error: f64,

    /// Minimum difference observed
    pub min: i32,

    /// Maximum difference observed
    pub max: i32,

    /// 25th percentile
    pub q1: f64,

    /// Median (50th percentile)
    pub median: f64,

    /// 75th percentile
    pub q3: f64,

    /// Number of deals where option A was better
    pub a_better_count: usize,

    /// Number of deals where option B was better
    pub b_better_count: usize,

    /// Number of deals where they were equal
    pub equal_count: usize,
}

impl PairwiseStatistics {
    /// Calculate statistics from a set of comparison results
    #[must_use]
    #[inline]
    pub fn from_comparisons(comparisons: &[ComparisonResult]) -> Self {
        let n = comparisons.len();

        if n == 0 {
            return Self::default();
        }

        // Single pass: Welford's online algorithm, min/max, counts and
        // collection of the differences to sort for the percentiles.
        let mut mean = 0.0;
        let mut m2 = 0.0;
        let mut min = i32::MAX;
        let mut max = i32::MIN;
        let mut a_better = 0;
        let mut b_better = 0;
        let mut equal = 0;
        let mut sorted_diffs: Vec<i32> = Vec::with_capacity(n);

        #[allow(clippy::cast_precision_loss)]
        for (i, comp) in comparisons.iter().enumerate() {
            let difference = comp.difference;
            let diff = f64::from(difference);
            let delta = diff - mean;

            mean += delta / (i + 1) as f64;

            let delta2 = diff - mean;
            m2 += delta * delta2;

            min = min.min(difference);
            max = max.max(difference);

            match difference.cmp(&0) {
                std::cmp::Ordering::Greater => a_better += 1,
                std::cmp::Ordering::Less => b_better += 1,
                std::cmp::Ordering::Equal => equal += 1,
            }

            sorted_diffs.push(difference);
        }

        #[allow(clippy::cast_precision_loss)]
        let variance = if n > 1 { m2 / n as f64 } else { 0.0 };
        let std_dev = variance.sqrt();
        #[allow(clippy::cast_precision_loss)]
        let std_error = std_dev / (n as f64).sqrt();

        // Calculate percentiles
        sorted_diffs.sort_unstable();

        let q1 = percentile(&sorted_diffs, 0.25);
        let median = percentile(&sorted_diffs, 0.50);
        let q3 = percentile(&sorted_diffs, 0.75);

        Self {
            mean,
            std_dev,
            std_error,
            min,
            max,
            q1,
            median,
            q3,
            a_better_count: a_better,
            b_better_count: b_better,
            equal_count: equal,
        }
    }

    /// Is option A significantly better than option B?
    /// Uses a simple heuristic: `mean`> `std_error`
    #[must_use]
    #[inline]
    pub fn is_a_significantly_better(&self) -> bool {
        self.mean > self.std_error
    }

    /// Is option B significantly better than option A?
    #[must_use]
    #[inline]
    pub fn is_b_significantly_better(&self) -> bool {
        self.mean < -self.std_error
    }

    /// Are the options statistically indistinguishable?
    #[must_use]
    #[inline]
    pub fn are_equivalent(&self) -> bool {
        self.mean.abs() <= self.std_error
    }
}

impl Default for PairwiseStatistics {
    fn default() -> Self {
        Self {
            mean: 0.0,
            std_dev: 0.0,
            std_error: 0.0,
            min: 0,
            max: 0,
            q1: 0.0,
            median: 0.0,
            q3: 0.0,
            a_better_count: 0,
            b_better_count: 0,
            equal_count: 0,
        }
    }
}

/// Calculate percentile from sorted data
#[must_use]
#[inline]
fn percentile(sorted_data: &[i32], p: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }

    #[allow(clippy::cast_precision_loss)]
    let index = (sorted_data.len() as f64 - 1.0) * p;

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let lower = index.floor() as usize;

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let upper = index.ceil() as usize;

    if lower == upper {
        f64::from(sorted_data[lower])
    } else {
        #[allow(clippy::cast_precision_loss)]
        let weight = index - lower as f64;
        f64::from(sorted_data[lower]) * (1.0 - weight) + f64::from(sorted_data[upper]) * weight
    }
}

// ============================================================================
// DISTRIBUTION DATA - Histogram data for visualization
// ============================================================================

/// Distribution data for creating histograms and density plots.
/// Useful for visualizing the spread of results in a GUI.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DistributionData {
    /// Histogram bins: (`bin_start`, `bin_end`, `count`)
    pub bins: Vec<(f64, f64, usize)>,

    /// Bin width used
    pub bin_width: f64,
}

impl DistributionData {
    /// Create distribution data from differences
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    #[inline]
    pub fn from_differences(differences: &[i32], num_bins: usize) -> Self {
        Self::from_differences_iter(differences.iter().copied(), num_bins)
    }

    /// Create distribution data from comparison results directly, without
    /// materialising an intermediate `Vec` of differences and without
    /// requiring a cloneable iterator.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    #[inline]
    pub fn from_comparison_differences(comparisons: &[ComparisonResult], num_bins: usize) -> Self {
        Self::from_differences_iter(comparisons.iter().map(|c| c.difference), num_bins)
    }

    /// Create distribution data from an iterator of differences without
    /// materialising an intermediate `Vec`.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    #[inline]
    pub(crate) fn from_differences_iter<I>(differences: I, num_bins: usize) -> Self
    where
        I: Iterator<Item = i32> + Clone,
    {
        let mut probe = differences.clone();
        let Some(first) = probe.next() else {
            return Self {
                bins: Vec::new(),
                bin_width: 0.0,
            };
        };

        let mut min_i = first;
        let mut max_i = first;
        for diff in probe {
            min_i = min_i.min(diff);
            max_i = max_i.max(diff);
        }

        let min = f64::from(min_i);
        let max = f64::from(max_i);
        let range = max - min;

        #[allow(clippy::cast_precision_loss)]
        let bin_width = range / num_bins as f64;

        let mut bins = vec![(0.0, 0.0, 0); num_bins];

        for (i, bin) in bins.iter_mut().enumerate().take(num_bins) {
            #[allow(clippy::cast_precision_loss)]
            let bin_start = min + i as f64 * bin_width;
            let bin_end = bin_start + bin_width;
            bin.0 = bin_start;
            bin.1 = bin_end;
        }

        for diff in differences {
            let diff_f64 = f64::from(diff);

            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let bin_idx = ((diff_f64 - min) / bin_width).floor() as usize;

            let bin_idx = bin_idx.min(num_bins - 1);
            bins[bin_idx].2 += 1;
        }

        Self { bins, bin_width }
    }
}

#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
fn bisect_right(value: i32, lista: &[i32]) -> i32 {
    lista.partition_point(|&x| x < value) as i32
}
#[must_use]
#[inline]
pub fn imps(my: i32, other: i32) -> i32 {
    let imp_table: [i32; 24] = [
        15, 45, 85, 125, 165, 215, 265, 315, 365, 425, 495, 595, 745, 895, 1095, 1295, 1495, 1745,
        1995, 2245, 2495, 2995, 3495, 3995,
    ];
    bisect_right((my - other).abs(), &imp_table) * (if my > other { 1 } else { -1 })
}
#[must_use]
#[inline]
pub fn matchpoints(my: i32, other: i32) -> i32 {
    i32::from(my > other) - i32::from(my < other)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::all, clippy::pedantic)]
    use super::*;

    #[test]
    fn test_deal_result_creation() {
        let result = DealResult::new(5, 9, 420);
        assert_eq!(result.deal_index, 5);
        assert_eq!(result.tricks, 9);
        assert_eq!(result.score, 420);
    }

    #[test]
    fn test_comparison_result_creation() {
        let comp = ComparisonResult::new(10, 100, 90, 10);
        assert_eq!(comp.deal_index, 10);
        assert_eq!(comp.score_a, 100);
        assert_eq!(comp.score_b, 90);
        assert_eq!(comp.difference, 10);
        assert_eq!(comp.abs_difference(), 10);

        let comp_neg = ComparisonResult::new(11, 90, 100, -10);
        assert_eq!(comp_neg.abs_difference(), 10);
    }

    #[test]
    fn test_pairwise_statistics_empty() {
        let stats = PairwiseStatistics::from_comparisons(&[]);
        assert!(stats.mean - 0.0 < 1.0e-6);
        assert!(stats.std_dev - 0.0 < 1.0e-6);
        assert_eq!(stats.a_better_count, 0);
        assert_eq!(stats.b_better_count, 0);
    }

    #[test]
    fn test_pairwise_statistics_basic() {
        let comparisons = vec![
            ComparisonResult::new(0, 100, 90, 10),
            ComparisonResult::new(1, 95, 100, -5),
            ComparisonResult::new(2, 110, 95, 15),
            ComparisonResult::new(3, 100, 100, 0),
        ];

        let stats = PairwiseStatistics::from_comparisons(&comparisons);

        // Mean: (10 - 5 + 15 + 0) / 4 = 5.0
        assert!((stats.mean - 5.0).abs() < 0.01);

        // Counts
        assert_eq!(stats.a_better_count, 2); // 10 and 15
        assert_eq!(stats.b_better_count, 1); // -5
        assert_eq!(stats.equal_count, 1); // 0

        // Min/Max
        assert_eq!(stats.min, -5);
        assert_eq!(stats.max, 15);

        // Standard error should be non-zero
        assert!(stats.std_error > 0.0);
    }

    #[test]
    fn test_pairwise_statistics_all_positive() {
        let comparisons = vec![
            ComparisonResult::new(0, 100, 90, 10),
            ComparisonResult::new(1, 95, 85, 10),
            ComparisonResult::new(2, 110, 95, 15),
        ];

        let stats = PairwiseStatistics::from_comparisons(&comparisons);

        assert!(stats.mean > 0.0);
        assert_eq!(stats.a_better_count, 3);
        assert_eq!(stats.b_better_count, 0);
    }

    #[test]
    fn test_pairwise_statistics_significance() {
        // Large positive mean with small error = A significantly better
        let comparisons: Vec<_> = (0..100)
            .map(|i| ComparisonResult::new(i, 100, 90, 10))
            .collect();

        let stats = PairwiseStatistics::from_comparisons(&comparisons);
        assert!(stats.is_a_significantly_better());
        assert!(!stats.is_b_significantly_better());
        assert!(!stats.are_equivalent());

        // Large negative mean = B significantly better
        let comparisons: Vec<_> = (0..100)
            .map(|i| ComparisonResult::new(i, 90, 100, -10))
            .collect();

        let stats = PairwiseStatistics::from_comparisons(&comparisons);
        assert!(!stats.is_a_significantly_better());
        assert!(stats.is_b_significantly_better());

        // Mean near zero = equivalent
        let comparisons = vec![
            ComparisonResult::new(0, 100, 99, 1),
            ComparisonResult::new(1, 99, 100, -1),
        ];

        let stats = PairwiseStatistics::from_comparisons(&comparisons);
        assert!(stats.are_equivalent());
    }

    #[test]
    fn test_percentile_calculation() {
        let comparisons = vec![
            ComparisonResult::new(0, 100, 90, 10),
            ComparisonResult::new(1, 95, 100, -5),
            ComparisonResult::new(2, 110, 95, 15),
            ComparisonResult::new(3, 105, 90, 15),
            ComparisonResult::new(4, 100, 100, 0),
        ];

        let stats = PairwiseStatistics::from_comparisons(&comparisons);

        // Sorted: [-5, 0, 10, 15, 15]

        assert!(stats.min <= stats.q1 as i32);
        assert!(stats.q1 <= stats.median);
        assert!(stats.median <= stats.q3);
        assert!(stats.q3 <= f64::from(stats.max));
    }

    #[test]
    fn test_distribution_data_empty() {
        let dist = DistributionData::from_differences(&[], 10);
        assert_eq!(dist.bins.len(), 0);
        assert_eq!(dist.bin_width, 0.0);
    }

    #[test]
    fn test_distribution_data_basic() {
        let diffs = vec![-10, -5, 0, 5, 10, 15, 20];
        let dist = DistributionData::from_differences(&diffs, 3);

        assert_eq!(dist.bins.len(), 3);
        assert!(dist.bin_width > 0.0);

        // Total count should equal input count
        let total_count: usize = dist.bins.iter().map(|(_, _, c)| c).sum();
        assert_eq!(total_count, diffs.len());
    }

    #[test]
    fn test_simulation_metadata() {
        let metadata = SimulationMetadata::new(1000, 3);
        assert_eq!(metadata.num_deals, 1000);
        assert_eq!(metadata.num_comparisons, 3);

        let metadata = metadata
            .with_custom("key1".into(), "value1".into())
            .with_custom("key2".into(), "value2".into());

        assert_eq!(metadata.custom.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.custom.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_simulation_output_accessors() {
        let raw = vec![1, 2, 3];
        let stats = "statistics";
        let metadata = SimulationMetadata::new(100, 2);

        let output = SimulationOutput::new(raw.clone(), stats, metadata.clone());

        // Test accessors
        assert_eq!(output.raw, raw);
        assert_eq!(output.statistics, stats);
        assert_eq!(output.metadata.num_deals, 100);

        // Test into_raw
        let output2 = SimulationOutput::new(raw.clone(), stats, metadata.clone());
        let extracted_raw = output2.into_raw();
        assert_eq!(extracted_raw, raw);

        // Test into_statistics
        let output3 = SimulationOutput::new(raw.clone(), stats, metadata.clone());
        let extracted_stats = output3.into_statistics();
        assert_eq!(extracted_stats, stats);

        // Test into_parts
        let output4 = SimulationOutput::new(raw.clone(), stats, metadata.clone());
        let (r, s, m) = output4.into_parts();
        assert_eq!(r, raw);
        assert_eq!(s, stats);
        assert_eq!(m.num_deals, 100);
    }
}
