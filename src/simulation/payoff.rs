// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Payoff Simulations
//!
//! This module implements contract payoff simulations with a focus on
//! providing structured, GUI-friendly data access. Card expected value lives
//! in [`crate::simulation::card_ev`].
//!
//! ## Example: Contract Payoff
//!
//! ```
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use squeezer::prelude::*;
//!
//! let contracts = vec![
//!     Contract::from_str("3CN")?,
//!     Contract::from_str("3HS")?,
//!     Contract::from_str("3NN")?,
//! ];
//!
//! let sim = ContractPayoffSimulation::builder()
//!     .num_deals(1000)
//!     .dealer(StandardDealer::new())
//!     .contracts(contracts)
//!     .denomination(Denomination::Compared {
//!         baseline: Baseline::AverageOfOptions,
//!         scoring: Scoring::Imps,
//!     })
//!     .build()?;
//!
//! let results = sim.run()?;
//!
//! // Access raw data for custom visualization
//! for item in &results.raw.items {
//!     println!("Comparing {} vs {}", item.option_a, item.option_b);
//!     for comp in &item.comparisons {
//!         println!("  Deal {}: {} vs {} (diff: {})",
//!             comp.deal_index, comp.score_a, comp.score_b, comp.difference);
//!     }
//! }
//!
//! // Or use pre-computed statistics
//! let matrix = results.statistics;
//! for item in &matrix.comparisons {
//!     println!("{} vs {}: mean={:.2}, std_dev={:.2}",
//!         item.option_a, item.option_b, item.stats.mean, item.stats.std_dev);
//! }
//! # Ok(())
//! # }
//! ```

use super::core::{
    ComparisonResult, DealResult, DistributionData, PairwiseStatistics, Simulation,
    SimulationMetadata, SimulationOutput,
};
use super::{opening_position, solve_tricks_pool};
use crate::prelude::*;

// ============================================================================
// RAW PAYOFF DATA - Complete simulation results for detailed analysis
// ============================================================================

/// Raw data from a payoff simulation comparing multiple options.
/// Contains all individual deal results for maximum flexibility in analysis.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawPayoffData<T: Clone + Eq + std::hash::Hash> {
    /// The options that were compared
    pub options: Vec<T>,

    /// Results for each option across all deals
    pub option_results: Vec<OptionResults<T>>,

    /// Pairwise comparison data
    pub items: Vec<PairwiseComparison<T>>,
}

impl<T: Clone + Eq + std::hash::Hash + std::fmt::Display> RawPayoffData<T> {
    /// Create a new raw payoff data container
    #[must_use]
    #[inline]
    pub fn new(options: Vec<T>) -> Self {
        let n = options.len();
        Self {
            option_results: options
                .iter()
                .map(|opt| OptionResults::new(opt.clone()))
                .collect(),
            options,
            // Directed pairs (A,B) and (B,A); reserve up front.
            items: Vec::with_capacity(n.saturating_sub(1) * n),
        }
    }

    /// Get results for a specific option
    pub fn results_for(&self, option: &T) -> Option<&OptionResults<T>> {
        self.option_results.iter().find(|r| &r.option == option)
    }

    /// Get comparison between two specific options
    pub fn comparison_between(&self, option_a: &T, option_b: &T) -> Option<&PairwiseComparison<T>> {
        self.items
            .iter()
            .find(|item| &item.option_a == option_a && &item.option_b == option_b)
    }

    /// Get all comparisons involving a specific option
    pub fn comparisons_involving(&self, option: &T) -> Vec<&PairwiseComparison<T>> {
        self.items
            .iter()
            .filter(|item| &item.option_a == option || &item.option_b == option)
            .collect()
    }
}

/// Pairwise builder for contract payoff comparisons.
///
/// Builds the directed comparison `option_a` vs `option_b` from aligned
/// per-deal results, applying the denomination's pairwise difference once per
/// deal. Capacity for `num_deals` comparisons is reserved up front.
fn build_pairwise<T: Clone + Eq + std::hash::Hash>(
    option_a: T,
    option_b: T,
    results_a: &[DealResult],
    results_b: &[DealResult],
    denomination: &Denomination,
) -> PairwiseComparison<T> {
    let capacity = results_a.len().min(results_b.len());
    let mut pairwise = PairwiseComparison::with_capacity(option_a, option_b, capacity);
    for (deal_idx, (result_a, result_b)) in results_a.iter().zip(results_b.iter()).enumerate() {
        let difference = denomination.difference(result_a.score, result_b.score);
        pairwise.add_comparison(ComparisonResult::new(
            deal_idx,
            result_a.score,
            result_b.score,
            difference,
        ));
    }
    pairwise
}

/// Results for a single option across all deals
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionResults<T: Clone + Eq + std::hash::Hash> {
    /// The option these results are for
    pub option: T,

    /// Individual deal results
    pub deals: Vec<DealResult>,
}

impl<T: Clone + Eq + std::hash::Hash> OptionResults<T> {
    /// Create a new option results container
    #[inline]
    pub fn new(option: T) -> Self {
        Self {
            option,
            deals: Vec::new(),
        }
    }

    /// Create a new option results container with deal capacity reserved.
    #[inline]
    pub fn with_capacity(option: T, capacity: usize) -> Self {
        Self {
            option,
            deals: Vec::with_capacity(capacity),
        }
    }

    /// Get scores across all deals
    pub fn scores(&self) -> Vec<i32> {
        self.deals.iter().map(|d| d.score).collect()
    }

    /// Get tricks across all deals
    pub fn tricks(&self) -> Vec<u8> {
        self.deals.iter().map(|d| d.tricks).collect()
    }

    /// Add a deal result
    #[inline]
    pub fn add_deal(&mut self, result: DealResult) {
        self.deals.push(result);
    }
}

/// Pairwise comparison between two options
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PairwiseComparison<T: Clone + Eq + std::hash::Hash> {
    /// First option being compared
    pub option_a: T,

    /// Second option being compared
    pub option_b: T,

    /// Individual comparison results for each deal
    pub comparisons: Vec<ComparisonResult>,
}

impl<T: Clone + Eq + std::hash::Hash> PairwiseComparison<T> {
    /// Create a new pairwise comparison
    #[inline]
    pub fn new(option_a: T, option_b: T) -> Self {
        Self {
            option_a,
            option_b,
            comparisons: Vec::new(),
        }
    }

    /// Create a new pairwise comparison with comparison capacity reserved.
    #[inline]
    pub fn with_capacity(option_a: T, option_b: T, capacity: usize) -> Self {
        Self {
            option_a,
            option_b,
            comparisons: Vec::with_capacity(capacity),
        }
    }

    /// Add a comparison result
    #[inline]
    pub fn add_comparison(&mut self, result: ComparisonResult) {
        self.comparisons.push(result);
    }

    /// Get all differences as a vector (useful for plotting)
    #[inline]
    pub fn differences(&self) -> Vec<i32> {
        self.comparisons.iter().map(|c| c.difference).collect()
    }
}

// ============================================================================
// PAYOFF MATRIX - Computed statistics in matrix form
// ============================================================================

/// A payoff matrix containing computed statistics for all pairwise comparisons.
/// This is the primary structure for displaying results in a GUI.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PayoffMatrix<T: Clone + Eq + std::hash::Hash> {
    /// The options being compared
    pub options: Vec<T>,

    /// Computed statistics for each pairwise comparison
    pub comparisons: Vec<PayoffComparison<T>>,

    /// Overall rankings based on average performance
    pub rankings: Vec<OptionRanking<T>>,
}

impl<T: Clone + Eq + std::hash::Hash> PayoffMatrix<T> {
    /// Get statistics for a specific comparison
    pub fn get_stats(&self, option_a: &T, option_b: &T) -> Option<&PairwiseStatistics> {
        self.comparisons
            .iter()
            .find(|c| &c.option_a == option_a && &c.option_b == option_b)
            .map(|c| &c.stats)
    }

    /// Get the ranking for a specific option
    pub fn rank_of(&self, option: &T) -> Option<usize> {
        self.rankings
            .iter()
            .position(|r| &r.option == option)
            .map(|pos| pos + 1) // Convert to 1-based ranking
    }

    /// Get the best performing option
    #[must_use]
    #[inline]
    pub fn best_option(&self) -> Option<&T> {
        self.rankings.first().map(|r| &r.option)
    }

    /// Get options sorted by performance
    #[must_use]
    #[inline]
    pub fn sorted_options(&self) -> Vec<&T> {
        self.rankings.iter().map(|r| &r.option).collect()
    }

    /// Create matrix from raw data
    #[must_use]
    #[inline]
    pub fn from_raw_data(raw: &RawPayoffData<T>) -> Self {
        let mut comparisons = Vec::with_capacity(raw.items.len());

        // Compute statistics for each pairwise comparison
        for item in &raw.items {
            let stats = PairwiseStatistics::from_comparisons(&item.comparisons);
            let distribution = DistributionData::from_comparison_differences(&item.comparisons, 20);

            comparisons.push(PayoffComparison {
                option_a: item.option_a.clone(),
                option_b: item.option_b.clone(),
                stats,
                distribution,
            });
        }

        // Calculate rankings
        let rankings = Self::calculate_rankings(&raw.options, &comparisons);

        Self {
            options: raw.options.clone(),
            comparisons,
            rankings,
        }
    }

    /// Calculate overall rankings based on average performance against all other options
    ///
    /// Single pass over `comparisons`: aggregates (sum, count, wins, losses)
    /// per option via an index map instead of filtering per option (O(O*C)).
    fn calculate_rankings(
        options: &[T],
        comparisons: &[PayoffComparison<T>],
    ) -> Vec<OptionRanking<T>> {
        #[derive(Clone, Copy, Default)]
        struct Agg {
            sum: f64,
            count: usize,
            wins: usize,
            losses: usize,
        }
        let mut index_of = std::collections::HashMap::with_capacity(options.len() * 2);
        for (idx, opt) in options.iter().enumerate() {
            index_of.entry(opt).or_insert(idx);
        }
        let mut aggs = vec![Agg::default(); options.len()];
        for comparison in comparisons {
            if let Some(&idx) = index_of.get(&comparison.option_a) {
                let a = &mut aggs[idx];
                a.sum += comparison.stats.mean;
                a.count += 1;
                if comparison.stats.is_a_significantly_better() {
                    a.wins += 1;
                }
                if comparison.stats.is_b_significantly_better() {
                    a.losses += 1;
                }
            }
        }

        let mut rankings: Vec<_> = options
            .iter()
            .enumerate()
            .map(|(idx, opt)| {
                let a = aggs[idx];
                #[allow(clippy::cast_precision_loss)]
                let avg_mean = if a.count == 0 {
                    0.0
                } else {
                    a.sum / a.count as f64
                };

                OptionRanking {
                    option: opt.clone(),
                    average_score: avg_mean,
                    wins: a.wins,
                    losses: a.losses,
                }
            })
            .collect();

        // Sort by average score (descending)
        rankings.sort_by(|a, b| b.average_score.total_cmp(&a.average_score));

        rankings
    }
}

/// Statistics for a single pairwise comparison in the matrix
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PayoffComparison<T: Clone + Eq + std::hash::Hash> {
    /// First option
    pub option_a: T,

    /// Second option
    pub option_b: T,

    /// Computed statistics
    pub stats: PairwiseStatistics,

    /// Distribution data for visualization
    pub distribution: DistributionData,
}

/// Ranking information for a single option
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionRanking<T: Clone + Eq + std::hash::Hash> {
    /// The option
    pub option: T,

    /// Average score compared to all other options
    pub average_score: f64,

    /// Number of significant wins against other options
    pub wins: usize,

    /// Number of significant losses against other options
    pub losses: usize,
}

// ============================================================================
// CONTRACT PAYOFF SIMULATION - Builder pattern for configuration
// ============================================================================

/// Builder for configuring a contract payoff simulation
pub struct ContractPayoffBuilder<D: Dealer = StandardDealer> {
    num_deals: Option<usize>,
    dealer: Option<D>,
    contracts: Vec<Contract>,
    denomination: Denomination,
    solver_factory: Option<Box<dyn ContextFactory>>,
}

impl<D: Dealer> ContractPayoffBuilder<D> {
    /// Create a new builder
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            num_deals: None,
            dealer: None,
            contracts: Vec::new(),
            denomination: Denomination::RawPoints,
            solver_factory: None,
        }
    }

    /// Set the number of deals to simulate
    #[must_use]
    #[inline]
    pub fn num_deals(mut self, num: usize) -> Self {
        self.num_deals = Some(num);
        self
    }

    /// Set the dealer
    #[must_use]
    #[inline]
    pub fn dealer(mut self, dealer: D) -> Self {
        self.dealer = Some(dealer);
        self
    }

    /// Set the contracts to compare
    #[must_use]
    #[inline]
    pub fn contracts(mut self, contracts: Vec<Contract>) -> Self {
        self.contracts = contracts;
        self
    }

    /// Add a single contract
    #[must_use]
    #[inline]
    pub fn add_contract(mut self, contract: Contract) -> Self {
        self.contracts.push(contract);
        self
    }

    /// Set how outcomes are valued. Defaults to [`Denomination::RawPoints`].
    ///
    /// The pairwise comparison applies the denomination's
    /// [`difference`](Denomination::difference) to each pair of contracts, so
    /// `Compared` with an average/best baseline reproduces the old bare
    /// `scoring_fn(a, b)` (IMPs, matchpoints, or a custom scoring).
    #[must_use]
    #[inline]
    pub fn denomination(mut self, denomination: Denomination) -> Self {
        self.denomination = denomination;
        self
    }

    /// Use a specific solver backend, by its object-safe factory.
    ///
    /// Defaults to the DDS backend. This is the seam that keeps a solver type
    /// parameter out of the public simulation API and lets tests inject a mock
    /// (ADR-0005).
    #[must_use]
    #[inline]
    pub fn solver_factory(mut self, factory: Box<dyn ContextFactory>) -> Self {
        self.solver_factory = Some(factory);
        self
    }

    /// Build the simulation
    /// # Errors
    /// When you did not define the number of deals or the dealer.
    /// TODO: Fix with typestate
    #[inline]
    pub fn build(self) -> Result<ContractPayoffSimulation<D>, SqueezerError> {
        let num_deals = self
            .num_deals
            .ok_or_else(|| SqueezerError::InvalidInput("num_deals must be set".into()))?;
        let dealer = self
            .dealer
            .ok_or_else(|| SqueezerError::InvalidInput("dealer must be set".into()))?;
        let solver_factory: Box<dyn ContextFactory> = self
            .solver_factory
            .unwrap_or_else(|| Box::new(DdsSolver::new()));

        if self.contracts.len() < 2 {
            return Err(SqueezerError::InvalidInput(
                "at least 2 contracts required".into(),
            ));
        }

        Ok(ContractPayoffSimulation {
            num_deals,
            dealer,
            contracts: self.contracts,
            denomination: self.denomination,
            solver_factory,
        })
    }
}

impl<D: Dealer> Default for ContractPayoffBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulation for comparing contract payoffs
pub struct ContractPayoffSimulation<D: Dealer> {
    num_deals: usize,
    dealer: D,
    contracts: Vec<Contract>,
    denomination: Denomination,
    solver_factory: Box<dyn ContextFactory>,
}

impl<D: Dealer> ContractPayoffSimulation<D> {
    /// Create a builder for configuring the simulation
    #[must_use]
    #[inline]
    pub fn builder() -> ContractPayoffBuilder<D> {
        ContractPayoffBuilder::new()
    }

    /// Constructor taking an explicit [`Denomination`] and the default DDS
    /// solver. Prefer [`Self::builder`] when you need a custom solver factory.
    pub fn new(
        num_deals: usize,
        dealer: D,
        contracts: Vec<Contract>,
        denomination: Denomination,
    ) -> Self {
        Self {
            num_deals,
            dealer,
            contracts,
            denomination,
            solver_factory: Box::new(DdsSolver::new()),
        }
    }
}

impl<D: Dealer> Simulation for ContractPayoffSimulation<D> {
    type RawResult = RawPayoffData<Contract>;
    type Statistics = PayoffMatrix<Contract>;

    fn run(self) -> Result<SimulationOutput<Self::RawResult, Self::Statistics>, SqueezerError> {
        let ContractPayoffSimulation {
            num_deals,
            dealer,
            contracts,
            denomination,
            solver_factory,
        } = self;
        let num_contracts = contracts.len();

        // Order the solves by strain so same-strain contracts are adjacent on
        // the context and DDS keeps the transposition table across them; a
        // strain change resets it. `order[slot]` names the contract solved at
        // `slot`, so results are scattered back to the caller's order after.
        let mut order: Vec<usize> = (0..num_contracts).collect();
        order.sort_by_key(|&index| contracts[index].strain() as u8);

        // Deal-major: one call per deal, with all its contracts solved
        // consecutively on the one context rayon hands to that call.
        let solved = solve_tricks_pool(
            solver_factory,
            &dealer,
            num_deals,
            num_contracts,
            "contract payoff",
            |deal, _deal_index| -> Result<Vec<Position>, SqueezerError> {
                Ok(order
                    .iter()
                    .map(|&index| opening_position(deal, contracts[index]))
                    .collect())
            },
        )?;

        // Initialize accumulators for each contract first (borrows contracts),
        // then move the contracts into raw_data (no second allocation).
        let mut accumulators: Vec<_> = contracts
            .iter()
            .map(|contract| ContractAccumulator {
                contract: *contract,
                results: Vec::with_capacity(num_deals),
            })
            .collect();
        let mut raw_data = RawPayoffData::new(contracts);

        // Scatter each deal's trick counts back to the contract they solved.
        for entry in &solved {
            for (slot, &contract_index) in order.iter().enumerate() {
                let tricks = entry.tricks[slot];
                let accumulator = &mut accumulators[contract_index];
                let score = accumulator.contract.score(tricks);
                accumulator
                    .results
                    .push(DealResult::new(entry.deal_index, tricks, score));
            }
        }

        // Populate raw data with individual results. Both vectors are keyed by
        // the same contract order, so zip directly instead of a linear search.
        for (opt_results, acc) in raw_data.option_results.iter_mut().zip(&accumulators) {
            opt_results.deals.clone_from(&acc.results);
        }

        // Generate all directed pairwise comparisons via the shared helper
        // (capacity reserved; scoring applied once per deal per pair).
        for (i, acc_a) in accumulators.iter().enumerate() {
            for (j, acc_b) in accumulators.iter().enumerate() {
                if i == j {
                    continue;
                }

                raw_data.items.push(build_pairwise(
                    acc_a.contract,
                    acc_b.contract,
                    &acc_a.results,
                    &acc_b.results,
                    &denomination,
                ));
            }
        }

        // Compute statistics
        let statistics = PayoffMatrix::from_raw_data(&raw_data);

        // Create metadata
        let metadata = SimulationMetadata::new(num_deals, num_contracts)
            .with_custom("simulation_type".into(), "contract_payoff".into());

        Ok(SimulationOutput::new(raw_data, statistics, metadata))
    }
}

/// Internal accumulator for contract results during simulation
struct ContractAccumulator {
    contract: Contract,
    results: Vec<DealResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// IMPs against the pairwise opponent: the common `Compared` config that
    /// reproduces the old bare `imps` scoring function.
    fn imps_against_the_opponent() -> Denomination {
        Denomination::Compared {
            baseline: Baseline::AverageOfOptions,
            scoring: Scoring::Imps,
        }
    }

    #[test]
    fn test_option_results_creation() {
        let contract = Contract::from_str("3CN").unwrap();
        let mut results = OptionResults::new(contract);

        assert_eq!(results.deals.len(), 0);
        assert_eq!(results.option, contract);

        results.add_deal(DealResult::new(0, 9, 400));
        results.add_deal(DealResult::new(1, 10, 430));

        assert_eq!(results.deals.len(), 2);

        let scores = results.scores();
        assert_eq!(scores, vec![400, 430]);

        let tricks = results.tricks();
        assert_eq!(tricks, vec![9, 10]);
    }

    #[test]
    fn test_pairwise_comparison_creation() {
        let contract_a = Contract::from_str("3CN").unwrap();
        let contract_b = Contract::from_str("3NN").unwrap();

        let mut pairwise = PairwiseComparison::new(contract_a, contract_b);
        assert_eq!(pairwise.comparisons.len(), 0);

        pairwise.add_comparison(ComparisonResult::new(0, 100, 90, 10));
        pairwise.add_comparison(ComparisonResult::new(1, 95, 100, -5));

        assert_eq!(pairwise.comparisons.len(), 2);

        let diffs = pairwise.differences();
        assert_eq!(diffs, vec![10, -5]);
    }

    #[test]
    fn test_raw_payoff_data_creation() {
        let contracts = vec![
            Contract::from_str("3CN").unwrap(),
            Contract::from_str("3NN").unwrap(),
        ];

        let raw = RawPayoffData::new(contracts.clone());

        assert_eq!(raw.options.len(), 2);
        assert_eq!(raw.option_results.len(), 2);
        assert_eq!(raw.items.len(), 0);

        // Test results_for
        let result = raw.results_for(&contracts[0]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().option, contracts[0]);
    }

    #[test]
    fn test_raw_payoff_data_queries() {
        let contracts = vec![
            Contract::from_str("3CN").unwrap(),
            Contract::from_str("3NN").unwrap(),
        ];

        let mut raw = RawPayoffData::new(contracts.clone());

        // Add a pairwise comparison
        let mut pairwise = PairwiseComparison::new(contracts[0], contracts[1]);
        pairwise.add_comparison(ComparisonResult::new(0, 100, 90, 10));
        raw.items.push(pairwise);

        // Test comparison_between
        let comp = raw.comparison_between(&contracts[0], &contracts[1]);
        assert!(comp.is_some());
        assert_eq!(comp.unwrap().comparisons.len(), 1);

        // Test comparisons_involving
        let involving = raw.comparisons_involving(&contracts[0]);
        assert_eq!(involving.len(), 1);
    }

    #[test]
    fn test_payoff_matrix_from_raw_data() {
        let contracts = vec![
            Contract::from_str("3CN").unwrap(),
            Contract::from_str("3NN").unwrap(),
        ];

        let mut raw = RawPayoffData::new(contracts.clone());

        // Add option results
        for (idx, _) in contracts.iter().enumerate() {
            let opt_result = &mut raw.option_results[idx];
            opt_result.add_deal(DealResult::new(0, 9, 400));
            opt_result.add_deal(DealResult::new(1, 10, 430));
        }

        // Add pairwise comparison
        let mut pairwise = PairwiseComparison::new(contracts[0], contracts[1]);
        pairwise.add_comparison(ComparisonResult::new(0, 400, 390, 10));
        pairwise.add_comparison(ComparisonResult::new(1, 430, 425, 5));
        raw.items.push(pairwise);

        let matrix = PayoffMatrix::from_raw_data(&raw);

        assert_eq!(matrix.options.len(), 2);
        assert_eq!(matrix.comparisons.len(), 1);
        assert_eq!(matrix.rankings.len(), 2);

        // Test get_stats
        let stats = matrix.get_stats(&contracts[0], &contracts[1]);
        assert!(stats.is_some());
        assert!(stats.unwrap().mean > 0.0);

        // Test rank_of
        let rank = matrix.rank_of(&contracts[0]);
        assert!(rank.is_some());
        assert!(rank.unwrap() <= 2);

        // Test best_option
        let best = matrix.best_option();
        assert!(best.is_some());
    }

    #[test]
    fn test_payoff_matrix_rankings() {
        let contracts = vec![
            Contract::from_str("3CN").unwrap(),
            Contract::from_str("3NN").unwrap(),
            Contract::from_str("4CN").unwrap(),
        ];

        let mut raw = RawPayoffData::new(contracts.clone());

        // Setup comparisons where 3CN > 3NN > 4CN
        // 3CN vs 3NN: 3CN wins
        let mut pairwise1 = PairwiseComparison::new(contracts[0], contracts[1]);
        for i in 0..10 {
            pairwise1.add_comparison(ComparisonResult::new(i, 100, 90, 10));
        }
        raw.items.push(pairwise1);

        // 3CN vs 4CN: 3CN wins
        let mut pairwise2 = PairwiseComparison::new(contracts[0], contracts[2]);
        for i in 0..10 {
            pairwise2.add_comparison(ComparisonResult::new(i, 100, 80, 20));
        }
        raw.items.push(pairwise2);

        // 3NN vs 4CN: 3NN wins
        let mut pairwise3 = PairwiseComparison::new(contracts[1], contracts[2]);
        for i in 0..10 {
            pairwise3.add_comparison(ComparisonResult::new(i, 90, 80, 10));
        }
        raw.items.push(pairwise3);

        let matrix = PayoffMatrix::from_raw_data(&raw);

        // Check rankings
        assert_eq!(matrix.rankings.len(), 3);
        assert_eq!(matrix.best_option(), Some(&contracts[0]));
        assert_eq!(matrix.rank_of(&contracts[0]), Some(1));

        // Rankings should be sorted
        assert!(matrix.rankings[0].average_score >= matrix.rankings[1].average_score);
        assert!(matrix.rankings[1].average_score >= matrix.rankings[2].average_score);
    }

    #[test]
    fn test_contract_payoff_builder_validation() {
        // Two contracts, so only the condition named by each case is missing.
        let contracts = vec![
            Contract::from_str("3CN").unwrap(),
            Contract::from_str("3NN").unwrap(),
        ];

        // Missing num_deals
        let result = ContractPayoffBuilder::new()
            .dealer(StandardDealer::new())
            .contracts(contracts.clone())
            .denomination(imps_against_the_opponent())
            .build();
        assert!(result.is_err());

        // Missing dealer
        let result = ContractPayoffBuilder::<StandardDealer>::new()
            .num_deals(100)
            .contracts(contracts.clone())
            .denomination(imps_against_the_opponent())
            .build();
        assert!(result.is_err());

        // Missing contracts
        let result = ContractPayoffBuilder::new()
            .num_deals(100)
            .dealer(StandardDealer::new())
            .denomination(imps_against_the_opponent())
            .build();
        assert!(result.is_err());

        // Too few contracts
        let result = ContractPayoffBuilder::new()
            .num_deals(100)
            .dealer(StandardDealer::new())
            .contracts(vec![Contract::from_str("3CN").unwrap()])
            .denomination(imps_against_the_opponent())
            .build();
        assert!(result.is_err());

        // Valid configuration
        let result = ContractPayoffBuilder::new()
            .num_deals(100)
            .dealer(StandardDealer::new())
            .contracts(contracts)
            .denomination(imps_against_the_opponent())
            .build();
        assert!(result.is_ok());
    }

    /// A fully predealt dealer: one possible deal, so every run is identical.
    fn predealt_dealer() -> impl Dealer {
        DealerBuilder::new()
            .predeal(Seat::North, Cards::from_str("SAKQJT98765432").unwrap())
            .unwrap()
            .predeal(Seat::East, Cards::from_str("HAKQJT98765432").unwrap())
            .unwrap()
            .predeal(Seat::South, Cards::from_str("DAKQJT98765432").unwrap())
            .unwrap()
            .predeal(Seat::West, Cards::from_str("CAKQJT98765432").unwrap())
            .unwrap()
            .build()
            .unwrap()
    }

    /// The `Denomination` reproduces the pre-`Denomination` pairwise scoring.
    ///
    /// The old builder took a bare `scoring_fn(i32, i32) -> i32` and applied it
    /// to each pair of contracts. A `Compared` denomination with the opponent
    /// as its baseline (`AverageOfOptions`/`BestOption` over a single other
    /// contract) resolves to that opponent's score, so `RawPoints` matches the
    /// old `|a, b| a - b` and `Imps`/`Matchpoints` match the old `imps` and
    /// `matchpoints` exactly.
    #[test]
    fn contract_payoff_matches_the_previous_pairwise_scoring() {
        let contracts = vec![
            Contract::from_str("1NN").unwrap(),
            Contract::from_str("2SN").unwrap(),
            Contract::from_str("3HN").unwrap(),
        ];

        let run = |denomination| {
            ContractPayoffBuilder::new()
                .num_deals(4)
                .dealer(predealt_dealer())
                .contracts(contracts.clone())
                .denomination(denomination)
                .build()
                .expect("valid configuration")
                .run()
                .expect("fixed deals solve")
        };

        let raw = run(Denomination::RawPoints);
        let compared_imps = run(imps_against_the_opponent());
        let compared_matchpoints = run(Denomination::Compared {
            baseline: Baseline::BestOption,
            scoring: Scoring::Matchpoints,
        });

        assert!(!raw.raw.items.is_empty());
        for ((raw_item, imps_item), matchpoints_item) in raw
            .raw
            .items
            .iter()
            .zip(&compared_imps.raw.items)
            .zip(&compared_matchpoints.raw.items)
        {
            for ((raw_comp, imps_comp), matchpoints_comp) in raw_item
                .comparisons
                .iter()
                .zip(&imps_item.comparisons)
                .zip(&matchpoints_item.comparisons)
            {
                assert_eq!(raw_comp.difference, raw_comp.score_a - raw_comp.score_b);
                assert_eq!(
                    imps_comp.difference,
                    imps(imps_comp.score_a, imps_comp.score_b)
                );
                assert_eq!(
                    matchpoints_comp.difference,
                    matchpoints(matchpoints_comp.score_a, matchpoints_comp.score_b)
                );
            }
        }
    }

    /// The builder's default denomination is raw points: omitting
    /// `.denomination(...)` yields raw score differences.
    #[test]
    fn contract_payoff_defaults_to_raw_points() {
        let output = ContractPayoffBuilder::new()
            .num_deals(2)
            .dealer(predealt_dealer())
            .contracts(vec![
                Contract::from_str("1NN").unwrap(),
                Contract::from_str("2SN").unwrap(),
            ])
            .build()
            .expect("valid configuration")
            .run()
            .expect("fixed deals solve");

        assert!(!output.raw.items.is_empty());
        for item in &output.raw.items {
            for comp in &item.comparisons {
                assert_eq!(comp.difference, comp.score_a - comp.score_b);
            }
        }
    }

    /// Deterministic end-to-end through the shared pool-solve engine.
    ///
    /// A fully predealt dealer can only produce one deal, so two simulated
    /// deals are identical and the exact tricks/scores are pinned. This is
    /// the no-behavior-change proof for the engine migration: expected
    /// values were captured from the pre-refactor `run()`.
    #[test]
    fn test_contract_payoff_fully_predealt_is_deterministic() {
        let dealer = predealt_dealer();
        let contracts = vec![
            Contract::from_str("1NN").unwrap(),
            Contract::from_str("2SN").unwrap(),
        ];
        let output = ContractPayoffSimulation::new(2, dealer, contracts, Denomination::RawPoints)
            .run()
            .expect("fixed deals solve");

        assert_eq!(output.raw.option_results.len(), 2);
        let first = &output.raw.option_results[0].deals;
        let second = &output.raw.option_results[1].deals;
        assert_eq!(first.len(), 2);
        assert_eq!(second.len(), 2);
        for deal in first {
            assert_eq!((deal.tricks, deal.score), (0, -350));
        }
        for deal in second {
            assert_eq!((deal.tricks, deal.score), (13, 260));
        }
    }

    /// Records the context, deal and strain of every solve, so the scheduling
    /// can be asserted without the native solver.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SolveRecord {
        context: usize,
        north: Cards,
        trump: Strain,
    }

    struct RecordingContext {
        id: usize,
        log: std::sync::Arc<std::sync::Mutex<Vec<SolveRecord>>>,
    }

    struct RecordingFactory {
        log: std::sync::Arc<std::sync::Mutex<Vec<SolveRecord>>>,
    }

    impl ContextFactory for RecordingFactory {
        fn new_context(&self) -> Box<dyn SolverContext> {
            static NEXT_CONTEXT: std::sync::atomic::AtomicUsize =
                std::sync::atomic::AtomicUsize::new(0);
            Box::new(RecordingContext {
                id: NEXT_CONTEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                log: std::sync::Arc::clone(&self.log),
            })
        }
    }

    impl SolverContext for RecordingContext {
        fn solve(
            &mut self,
            position: &Position,
            _mode: SolveMode,
            _solutions: Solutions,
        ) -> Result<SolveResults, SolverError> {
            self.log
                .lock()
                .expect("recording log is never poisoned")
                .push(SolveRecord {
                    context: self.id,
                    north: position.hand(Seat::North),
                    trump: position.trump(),
                });
            let cards = position
                .legal_plays()
                .into_iter()
                .next()
                .map(|card| {
                    vec![CardSolve {
                        card,
                        tricks: 7,
                        equivalents: Cards::EMPTY.insert(card),
                    }]
                })
                .unwrap_or_default();
            Ok(SolveResults { cards })
        }

        fn clear_tt(&mut self) {}

        fn reset(&mut self) {}
    }

    /// The contract payoff must issue all of a deal's solves on one context,
    /// grouped by strain, rather than fanning individual solves across workers.
    ///
    /// With one context per rayon job, a job's recorded sequence is the exact
    /// solve order. We assert two properties per job: a deal's solves are
    /// contiguous (deal-major, never contract-major) and its strains are
    /// non-decreasing (same-strain contracts adjacent, so the TT survives).
    #[test]
    fn contract_payoff_is_deal_major_with_same_strain_adjacent() {
        use std::collections::{HashMap, HashSet};
        use std::sync::{Arc, Mutex};

        let log = Arc::new(Mutex::new(Vec::new()));
        // Deliberately interleave the strains: input order is S, NT, H, S.
        let contracts = vec![
            Contract::from_str("1SN").expect("fixed contract parses"),
            Contract::from_str("1NN").expect("fixed contract parses"),
            Contract::from_str("1HN").expect("fixed contract parses"),
            Contract::from_str("1SN").expect("fixed contract parses"),
        ];
        let output = ContractPayoffBuilder::new()
            .num_deals(16)
            .dealer(StandardDealer::new())
            .contracts(contracts)
            .denomination(imps_against_the_opponent())
            .solver_factory(Box::new(RecordingFactory {
                log: Arc::clone(&log),
            }))
            .build()
            .expect("valid configuration")
            .run()
            .expect("the mock never fails");
        assert_eq!(output.raw.option_results.len(), 4);

        let records = log.lock().expect("log is never poisoned").clone();
        assert_eq!(records.len(), 16 * 4);

        let mut by_context: HashMap<usize, Vec<SolveRecord>> = HashMap::new();
        for record in records {
            by_context.entry(record.context).or_default().push(record);
        }

        for sequence in by_context.values() {
            let mut seen_deals = HashSet::new();
            let mut current: Option<Cards> = None;
            let mut previous_strain: Option<u8> = None;
            for record in sequence {
                if current.as_ref() != Some(&record.north) {
                    assert!(
                        seen_deals.insert(record.north),
                        "a context returned to an earlier deal, so deals are not contiguous"
                    );
                    current = Some(record.north);
                    previous_strain = None;
                }
                let strain = record.trump as u8;
                if let Some(previous) = previous_strain {
                    assert!(
                        strain >= previous,
                        "strains within a deal are not adjacent: {previous} then {strain}"
                    );
                }
                previous_strain = Some(strain);
            }
        }
    }
}
