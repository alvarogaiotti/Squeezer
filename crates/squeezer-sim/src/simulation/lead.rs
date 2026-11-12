// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Lead Simulations
//!
//! This module implements opening lead analysis with structured data access.
//!
//! ## Example
//!
//! ```no_run
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use squeezer_model::*;
//! use squeezer_sim::simulation::*;
//! use std::str::FromStr;
//!
//! let sim = LeadSimulation::builder()
//!     .num_deals(10)
//!     .dealer(StandardDealer::default())
//!     .contract(Contract::from_str("5CN")?)
//!     .build()?;
//!
//! let results = sim.run()?;
//!
//! // Access raw data for each card
//! for card_result in &results.raw.cards {
//!     println!("Card: {}", card_result.card);
//!     println!("  Deals where it took {} tricks:", card_result.tricks_distribution.len());
//!     for (tricks, count) in card_result.tricks_distribution.iter().enumerate() {
//!         println!("    {} tricks: {} times", tricks, count);
//!     }
//! }
//!
//! // Or use computed statistics
//! let stats = &results.statistics;
//! println!("Best lead: {}", stats.best_by_avg_tricks.card);
//! println!("  Avg tricks: {:.2}", stats.best_by_avg_tricks.average_tricks);
//! println!("  Set %: {:.2}", stats.best_by_avg_tricks.set_percentage);
//! # Ok(())
//! # }
//! ```

use super::{missing_solver_factory, resolve_solver_factory};
use crate::prelude::*;
use squeezer_solver::ContextFactory;
use std::collections::HashMap;

// ============================================================================
// RAW LEAD DATA - Complete per-card results
// ============================================================================

/// Raw data from a lead simulation.
/// Contains complete trick distribution for each card.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawLeadData {
    /// Results for each card that can be led
    pub cards: Vec<CardLeadResult>,

    /// The contract being analyzed
    pub contract: Contract,
}

impl RawLeadData {
    /// Create a new raw lead data container
    #[must_use]
    #[inline]
    pub fn new(contract: Contract) -> Self {
        Self {
            cards: Vec::new(),
            contract,
        }
    }

    /// Derives lead results from a card evaluation at the opening position.
    ///
    /// Each card's distribution is the side-to-play trick histogram (the
    /// opening leader is a defender, so this is the defenders' trick count).
    /// `store_deal_details` keeps one detail entry per deal.
    #[must_use]
    pub fn from_evaluation(
        evaluation: &CardEvaluation,
        contract: Contract,
        store_deal_details: bool,
    ) -> Self {
        let mut raw = Self::new(contract);
        raw.cards = evaluation
            .values
            .iter()
            .map(|value| {
                let mut result = if store_deal_details {
                    CardLeadResult::with_deal_details(value.card, value.deals.len())
                } else {
                    CardLeadResult::new(value.card)
                };
                for outcome in &value.deals {
                    let deal_index = store_deal_details.then_some(outcome.deal_index);
                    result.record_tricks(outcome.dd_tricks, deal_index);
                }
                result
            })
            .collect();
        // Sort cards for consistent ordering (by suit and rank).
        raw.cards
            .sort_by(|a, b| super::card_ev::compare_cards_by_suit_then_rank(a.card, b.card));
        raw
    }

    /// Get results for a specific card
    #[must_use]
    #[inline]
    pub fn result_for(&self, card: Card) -> Option<&CardLeadResult> {
        self.cards.iter().find(|r| r.card == card)
    }

    /// Get all cards sorted by a criterion
    #[must_use]
    #[inline]
    pub fn cards_sorted_by<F>(&self, mut f: F) -> Vec<&CardLeadResult>
    where
        F: FnMut(&CardLeadResult, &CardLeadResult) -> std::cmp::Ordering,
    {
        let mut cards: Vec<_> = self.cards.iter().collect();
        cards.sort_by(|a, b| f(a, b));
        cards
    }
}

/// Complete results for a single lead card across all deals
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardLeadResult {
    /// The card being led
    pub card: Card,

    /// Distribution of tricks: maps number of tricks to count of deals
    pub tricks_distribution: [usize; 14],

    /// Individual deal results (optional, for detailed analysis)
    pub deal_details: Option<Vec<LeadDealDetail>>,
}

impl CardLeadResult {
    /// Create a new card lead result
    #[must_use]
    #[inline]
    pub fn new(card: Card) -> Self {
        Self {
            card,
            tricks_distribution: [0; 14],
            deal_details: None,
        }
    }

    /// Create with capacity for deal details
    #[must_use]
    #[inline]
    pub fn with_deal_details(card: Card, capacity: usize) -> Self {
        Self {
            card,
            tricks_distribution: [0; 14],
            deal_details: Some(Vec::with_capacity(capacity)),
        }
    }

    /// Record a result
    #[inline]
    pub fn record_tricks(&mut self, tricks: u8, deal_index: Option<usize>) {
        self.tricks_distribution[tricks as usize] += 1;

        if let (Some(details), Some(idx)) = (&mut self.deal_details, deal_index) {
            details.push(LeadDealDetail {
                deal_index: idx,
                tricks_taken: tricks,
            });
        }
    }

    /// Get the total number of deals for this card
    #[must_use]
    #[inline]
    pub fn total_deals(&self) -> usize {
        self.tricks_distribution.iter().sum()
    }

    /// Get tricks as a sorted vector for plotting: Vec<(tricks, count)>
    ///
    /// Prefer [`tricks_histogram_array`](Self::tricks_histogram_array) in hot
    /// paths to avoid the 14-element heap allocation.
    #[must_use]
    #[inline]
    pub fn tricks_histogram(&self) -> Vec<(usize, usize)> {
        self.tricks_histogram_array().to_vec()
    }

    /// Stack-allocated variant of [`tricks_histogram`](Self::tricks_histogram).
    #[must_use]
    #[inline]
    pub fn tricks_histogram_array(&self) -> [(usize, usize); 14] {
        std::array::from_fn(|i| (i, self.tricks_distribution[i]))
    }

    /// Get cumulative distribution: Vec<(`tricks`, `cumulative_count`)>
    ///
    /// Prefer [`cumulative_distribution_array`](Self::cumulative_distribution_array)
    /// in hot paths to avoid the heap allocation.
    #[must_use]
    #[inline]
    pub fn cumulative_distribution(&self) -> Vec<(usize, usize)> {
        self.cumulative_distribution_array().to_vec()
    }

    /// Stack-allocated variant of [`cumulative_distribution`](Self::cumulative_distribution).
    #[must_use]
    #[inline]
    pub fn cumulative_distribution_array(&self) -> [(usize, usize); 14] {
        let mut out = [(0usize, 0usize); 14];
        let mut cumulative = 0usize;
        for (i, &count) in self.tricks_distribution.iter().enumerate() {
            cumulative += count;
            out[i] = (i, cumulative);
        }
        out
    }
}

/// Detail for a single deal result
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LeadDealDetail {
    /// Index of the deal
    pub deal_index: usize,

    /// Number of tricks taken when leading this card
    pub tricks_taken: u8,
}

// ============================================================================
// LEAD STATISTICS - Computed statistics for each card
// ============================================================================

/// Computed statistics for all cards in a lead simulation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LeadStatistics {
    /// Statistics for each card, sorted by set percentage (best first)
    pub cards: Vec<CardLeadStatistics>,

    /// The best card by average tricks taken
    pub best_by_avg_tricks: CardLeadStatistics,

    /// The best card by contract set percentage
    pub best_by_set_percentage: CardLeadStatistics,

    /// The contract being analyzed
    pub contract: Contract,

    /// Number of tricks needed to beat the contract
    pub tricks_to_beat: u8,
}

impl LeadStatistics {
    /// Get statistics for a specific card
    #[must_use]
    #[inline]
    pub fn stats_for(&self, card: Card) -> Option<&CardLeadStatistics> {
        self.cards.iter().find(|s| s.card == card)
    }

    /// Get the ranking (1-based) of a card by set percentage
    #[must_use]
    #[inline]
    pub fn rank_by_set_percentage(&self, card: Card) -> Option<usize> {
        self.cards
            .iter()
            .position(|s| s.card == card)
            .map(|pos| pos + 1)
    }

    /// Get cards grouped by suit
    #[must_use]
    #[inline]
    pub fn cards_by_suit(&self) -> HashMap<Suit, Vec<&CardLeadStatistics>> {
        let mut by_suit: HashMap<Suit, Vec<_>> = HashMap::new();
        for card_stat in &self.cards {
            by_suit
                .entry(card_stat.card.suit())
                .or_default()
                .push(card_stat);
        }
        by_suit
    }

    /// Create from raw data
    #[must_use]
    #[inline]
    pub fn from_raw_data(raw: &RawLeadData, tricks_to_beat: u8) -> Self {
        let mut cards: Vec<_> = Vec::with_capacity(raw.cards.len());
        cards.extend(
            raw.cards.iter().map(|card_result| {
                CardLeadStatistics::from_card_result(card_result, tricks_to_beat)
            }),
        );

        // Sort by set percentage (descending), then by average tricks.
        // `total_cmp` avoids panics on NaN that `partial_cmp().unwrap()` has.
        cards.sort_by(|a, b| {
            b.set_percentage
                .total_cmp(&a.set_percentage)
                .then_with(|| b.average_tricks.total_cmp(&a.average_tricks))
        });

        // Reuse the sorted order: the first element is the best by set %.
        let best_by_set_percentage = cards
            .first()
            .cloned()
            .unwrap_or_else(CardLeadStatistics::default);
        // Best by average still needs a scan, but with total_cmp (no unwrap).
        let best_by_avg_tricks = cards
            .iter()
            .max_by(|a, b| a.average_tricks.total_cmp(&b.average_tricks))
            .cloned()
            .unwrap_or_else(CardLeadStatistics::default);

        Self {
            cards,
            best_by_avg_tricks,
            best_by_set_percentage,
            contract: raw.contract,
            tricks_to_beat,
        }
    }
}

/// Statistics for a single lead card
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardLeadStatistics {
    /// The card
    pub card: Card,

    /// Average number of tricks taken
    pub average_tricks: f64,

    /// Percentage of deals where this lead beats the contract
    pub set_percentage: f64,

    /// Standard deviation of tricks taken
    pub std_dev: f64,

    /// Minimum tricks taken across all deals
    pub min_tricks: u8,

    /// Maximum tricks taken across all deals
    pub max_tricks: u8,

    /// Median tricks taken
    pub median_tricks: f64,

    /// Distribution summary: (tricks, percentage of deals)
    pub distribution_percentages: [f64; 14],
}

impl CardLeadStatistics {
    /// Calculate statistics from a card result
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    #[inline]
    pub fn from_card_result(result: &CardLeadResult, tricks_to_beat: u8) -> Self {
        let total_deals = result.total_deals();

        if total_deals == 0 {
            return Self::default();
        }

        // Single pass: total tricks, set count, min/max and percentages.
        let mut total_tricks: usize = 0;
        let mut deals_beating: usize = 0;
        let mut min_tricks: usize = 14;
        let mut max_tricks: usize = 0;
        let mut distribution_percentages = [0.0f64; 14];
        #[allow(clippy::cast_precision_loss)]
        let total_f = total_deals as f64;
        for (tricks, &count) in result.tricks_distribution.iter().enumerate() {
            if count == 0 {
                continue;
            }
            total_tricks += tricks * count;
            if tricks >= tricks_to_beat as usize {
                deals_beating += count;
            }
            if tricks < min_tricks {
                min_tricks = tricks;
            }
            if tricks > max_tricks {
                max_tricks = tricks;
            }
            #[allow(clippy::cast_precision_loss)]
            {
                distribution_percentages[tricks] = (count as f64 / total_f) * 100.0;
            }
        }
        if min_tricks == 14 {
            min_tricks = 0;
        }

        #[allow(clippy::cast_precision_loss)]
        let average_tricks = total_tricks as f64 / total_f;

        #[allow(clippy::cast_precision_loss)]
        let set_percentage = (deals_beating as f64 / total_f) * 100.0;

        // Second pass for variance (needs the mean).
        #[allow(clippy::cast_precision_loss)]
        let variance: f64 = result
            .tricks_distribution
            .iter()
            .enumerate()
            .map(|(tricks, &count)| {
                let diff = tricks as f64 - average_tricks;
                diff * diff * count as f64
            })
            .sum::<f64>()
            / total_f;
        let std_dev = variance.sqrt();

        // Median (approximate)
        let median_tricks = calculate_median(&result.tricks_distribution, total_deals);

        #[allow(clippy::cast_possible_truncation)]
        Self {
            card: result.card,
            average_tricks,
            set_percentage,
            std_dev,
            min_tricks: min_tricks as u8,
            max_tricks: max_tricks as u8,
            median_tricks,
            distribution_percentages,
        }
    }
}

impl Default for CardLeadStatistics {
    fn default() -> Self {
        Self {
            card: Card::new(Suit::Spades, 2),
            average_tricks: 0.0,
            set_percentage: 0.0,
            std_dev: 0.0,
            min_tricks: 0,
            max_tricks: 0,
            median_tricks: 0.0,
            distribution_percentages: [0.0; 14],
        }
    }
}

/// Calculate median from a trick distribution
///
/// Single cumulative walk over the fixed `[usize; 14]` histogram — no
/// allocation. For even totals this returns the lower-middle value, which
/// is exact enough for lead ordering.
fn calculate_median(distribution: &[usize; 14], total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let mid = total.div_ceil(2);
    let mut cumulative = 0usize;

    for (tricks, &count) in distribution.iter().enumerate() {
        cumulative += count;

        #[allow(clippy::cast_precision_loss)]
        if cumulative >= mid {
            return tricks as f64;
        }
    }

    0.0
}

// ============================================================================
// LEAD SIMULATION - Builder and implementation
// ============================================================================

/// Builder for configuring a lead simulation
pub struct LeadSimulationBuilder<D: Dealer = StandardDealer> {
    num_deals: Option<usize>,
    dealer: Option<D>,
    contract: Option<Contract>,
    store_deal_details: bool,
    solver_factory: Option<Box<dyn ContextFactory>>,
}

impl<D: Dealer> LeadSimulationBuilder<D> {
    /// Create a new builder
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            num_deals: None,
            dealer: None,
            contract: None,
            store_deal_details: false,
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

    /// Set the contract
    #[must_use]
    #[inline]
    pub fn contract(mut self, contract: Contract) -> Self {
        self.contract = Some(contract);
        self
    }

    /// Store individual deal details (uses more memory but enables detailed analysis)
    #[must_use]
    #[inline]
    pub fn with_deal_details(mut self) -> Self {
        self.store_deal_details = true;
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
    #[inline]
    /// # Errors
    /// ATM errors if some field is not set. TODO: evolve into typestate
    pub fn build(self) -> Result<LeadSimulation<D>, SimulationError> {
        let num_deals = self.num_deals.ok_or_else(|| {
            SimulationError::invalid_input("lead simulation", "num_deals must be set")
        })?;
        let dealer = self.dealer.ok_or_else(|| {
            SimulationError::invalid_input("lead simulation", "dealer must be defined")
        })?;
        let contract = self.contract.ok_or_else(|| {
            SimulationError::invalid_input("lead simulation", "contract must be set")
        })?;

        let solver_factory = resolve_solver_factory(self.solver_factory)
            .ok_or_else(|| missing_solver_factory("lead simulation"))?;

        Ok(LeadSimulation {
            num_deals,
            dealer,
            contract,
            store_deal_details: self.store_deal_details,
            solver_factory,
        })
    }
}

impl<D: Dealer> Default for LeadSimulationBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulation for analyzing opening leads
pub struct LeadSimulation<D: Dealer = StandardDealer> {
    num_deals: usize,
    dealer: D,
    contract: Contract,
    store_deal_details: bool,
    solver_factory: Box<dyn ContextFactory>,
}

impl<D: Dealer + Default> LeadSimulation<D> {
    /// Create a builder for configuring the simulation
    #[must_use]
    #[inline]
    pub fn builder() -> LeadSimulationBuilder<D> {
        LeadSimulationBuilder::new()
    }
}

impl<D: Dealer> LeadSimulation<D> {
    /// Legacy constructor for backwards compatibility.
    ///
    /// This convenience constructor exists only with the `dds` feature; without
    /// it, build through [`Self::builder`] and pass a factory.
    #[cfg(feature = "dds")]
    pub fn new(num_deals: usize, dealer: D, contract: Contract) -> Self {
        Self {
            num_deals,
            dealer,
            contract,
            store_deal_details: false,
            solver_factory: Box::new(DdsSolver::new()),
        }
    }
}

impl<D: Dealer> Simulation for LeadSimulation<D> {
    type RawResult = RawLeadData;
    type Statistics = LeadStatistics;

    /// Runs the lead analysis as a view over [`CardEvaluation`] at the opening
    /// position: one all-legal solve per deal, then the lead-specific view
    /// (`tricks_to_beat`, set percentage, distributions, per-suit) is derived.
    fn run(self) -> Result<SimulationOutput<Self::RawResult, Self::Statistics>, SimulationError> {
        let LeadSimulation {
            num_deals,
            dealer,
            contract,
            store_deal_details,
            solver_factory,
        } = self;

        let evaluation = CardEvaluation::builder()
            .num_deals(num_deals)
            .dealer(dealer)
            .contract(contract)
            .solver_factory(solver_factory)
            .build()?
            .run()?;

        let raw_data = RawLeadData::from_evaluation(&evaluation, contract, store_deal_details);
        let tricks_to_beat = 8u8.saturating_sub(contract.level());
        let statistics = LeadStatistics::from_raw_data(&raw_data, tricks_to_beat);

        let metadata = SimulationMetadata::new(num_deals, raw_data.cards.len())
            .with_custom("simulation_type".into(), "lead".into())
            .with_custom("contract".into(), contract.to_string())
            .with_custom("tricks_to_beat".into(), tricks_to_beat.to_string());

        Ok(SimulationOutput::new(raw_data, statistics, metadata))
    }
}

// ============================================================================
// LEAD ANALYSIS VIEW
// ============================================================================

/// A lead-specific view over a [`CardEvaluation`] at the opening position.
///
/// The evaluation is the primitive; this view adds the lead concepts that only
/// make sense for an opening lead — `tricks_to_beat`, the contract, set
/// percentage, cumulative trick distributions and per-suit grouping — without
/// solving anything again.
pub struct LeadAnalysis<'a> {
    /// The underlying card evaluation.
    pub evaluation: &'a CardEvaluation,
    /// The contract being led against.
    pub contract: Contract,
}

impl<'a> LeadAnalysis<'a> {
    /// Wraps `evaluation` as a lead analysis for `contract`.
    #[must_use]
    pub fn new(evaluation: &'a CardEvaluation, contract: Contract) -> Self {
        Self {
            evaluation,
            contract,
        }
    }

    /// The number of defensive tricks needed to beat the contract.
    #[must_use]
    pub fn tricks_to_beat(&self) -> u8 {
        8u8.saturating_sub(self.contract.level())
    }

    /// The cards that can be led, in the evaluation's order.
    #[must_use]
    pub fn cards(&self) -> &[CardValue] {
        &self.evaluation.values
    }

    /// The set percentage of `card`, if it was a legal lead.
    #[must_use]
    pub fn set_percentage(&self, card: Card) -> Option<f64> {
        self.evaluation
            .value_for(card)
            .map(|value| value.statistics.set_percentage)
    }

    /// Cumulative declarer-trick distribution for `card`.
    #[must_use]
    pub fn cumulative_declarer_tricks(&self, card: Card) -> Option<[(usize, usize); 14]> {
        self.evaluation
            .value_for(card)
            .map(|value| value.statistics.cumulative_declarer_tricks())
    }

    /// Groups the leads by suit.
    #[must_use]
    pub fn cards_by_suit(&self) -> HashMap<Suit, Vec<&CardValue>> {
        self.evaluation.by_suit()
    }

    /// The lead with the highest set percentage.
    #[must_use]
    pub fn best_by_set_percentage(&self) -> Option<&CardValue> {
        self.evaluation.values.iter().max_by(|a, b| {
            a.statistics
                .set_percentage
                .total_cmp(&b.statistics.set_percentage)
        })
    }

    /// The lead with the highest mean side-to-play trick count.
    #[must_use]
    pub fn best_by_avg_tricks(&self) -> Option<&CardValue> {
        self.evaluation.values.iter().max_by(|a, b| {
            a.statistics
                .mean_dd_tricks()
                .total_cmp(&b.statistics.mean_dd_tricks())
        })
    }
}

// ============================================================================
// CONVENIENCE ITERATORS
// ============================================================================

/// Iterator over card lead results, useful for streaming processing
pub struct CardLeadResultIter<'a> {
    inner: std::slice::Iter<'a, CardLeadResult>,
}

impl<'a> Iterator for CardLeadResultIter<'a> {
    type Item = &'a CardLeadResult;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl RawLeadData {
    /// Iterate over card results
    #[must_use]
    #[inline]
    pub fn iter(&self) -> CardLeadResultIter<'_> {
        CardLeadResultIter {
            inner: self.cards.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a RawLeadData {
    type IntoIter = CardLeadResultIter<'a>;
    type Item = &'a CardLeadResult;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over card statistics
pub struct CardLeadStatsIter<'a> {
    inner: std::slice::Iter<'a, CardLeadStatistics>,
}

impl<'a> Iterator for CardLeadStatsIter<'a> {
    type Item = &'a CardLeadStatistics;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl LeadStatistics {
    /// Iterate over card statistics
    #[must_use]
    #[inline]
    pub fn iter(&self) -> CardLeadStatsIter<'_> {
        CardLeadStatsIter {
            inner: self.cards.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a LeadStatistics {
    type IntoIter = CardLeadStatsIter<'a>;
    type Item = &'a CardLeadStatistics;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ============================================================================
// COMPARISON UTILITIES
// ============================================================================

/// Compare two cards based on their lead performance
#[derive(Debug, Clone)]
pub struct LeadComparison {
    /// First card
    pub card_a: Card,

    /// Second card
    pub card_b: Card,

    /// Difference in average tricks (a - b)
    pub avg_tricks_diff: f64,

    /// Difference in set percentage (a - b)
    pub set_percentage_diff: f64,

    /// Which card is statistically better at imps
    pub better_card_imps: Option<Card>,
    /// Which card is statistically better at matchpoints
    pub better_card_matchpoints: Option<Card>,
}

impl LeadStatistics {
    /// Compare two cards
    #[must_use]
    #[inline]
    pub fn compare_cards(&self, card_a: Card, card_b: Card) -> Option<LeadComparison> {
        let stats_a = self.stats_for(card_a)?;
        let stats_b = self.stats_for(card_b)?;

        let avg_tricks_diff = stats_a.average_tricks - stats_b.average_tricks;
        let set_percentage_diff = stats_a.set_percentage - stats_b.set_percentage;

        // Simple heuristic: card is better if both metrics favor it
        let better_card_imps = if set_percentage_diff > 0.1 {
            Some(card_a)
        } else if avg_tricks_diff < -0.1 && set_percentage_diff < -0.1 {
            Some(card_b)
        } else {
            None
        };

        let better_card_matchpoints = if avg_tricks_diff > 0.1 {
            Some(card_a)
        } else if avg_tricks_diff < -0.1 {
            Some(card_b)
        } else {
            None
        };

        Some(LeadComparison {
            card_a,
            card_b,
            avg_tricks_diff,
            set_percentage_diff,
            better_card_imps,
            better_card_matchpoints,
        })
    }
}

// ============================================================================
// SUIT ANALYSIS
// ============================================================================

/// Analysis of all leads from a particular suit
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SuitLeadAnalysis {
    /// The suit being analyzed
    pub suit: Suit,

    /// Statistics for each card in this suit
    pub cards: Vec<CardLeadStatistics>,

    /// Best card in this suit by average tricks
    pub best_by_avg_tricks: CardLeadStatistics,

    /// Best card in this suit by set percentage
    pub best_by_set_percentage: CardLeadStatistics,

    /// Average performance of all cards in this suit
    pub suit_avg_tricks: f64,

    /// Average set percentage of all cards in this suit
    pub suit_avg_set_percentage: f64,
}

impl LeadStatistics {
    /// Analyze all leads from a specific suit
    #[must_use]
    #[inline]
    pub fn analyze_suit(&self, suit: Suit) -> Option<SuitLeadAnalysis> {
        let cards: Vec<_> = self
            .cards
            .iter()
            .filter(|s| s.card.suit() == suit)
            .cloned()
            .collect();

        if cards.is_empty() {
            return None;
        }

        let best_by_avg_tricks = cards
            .iter()
            .max_by(|a, b| {
                a.average_tricks
                    .partial_cmp(&b.average_tricks)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()?;

        let best_by_set_percentage = cards
            .iter()
            .max_by(|a, b| {
                a.set_percentage
                    .partial_cmp(&b.set_percentage)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()?;

        #[allow(clippy::cast_precision_loss)]
        let suit_avg_tricks =
            cards.iter().map(|c| c.average_tricks).sum::<f64>() / cards.len() as f64;

        #[allow(clippy::cast_precision_loss)]
        let suit_avg_set_percentage =
            cards.iter().map(|c| c.set_percentage).sum::<f64>() / cards.len() as f64;

        Some(SuitLeadAnalysis {
            suit,
            cards,
            best_by_avg_tricks,
            best_by_set_percentage,
            suit_avg_tricks,
            suit_avg_set_percentage,
        })
    }

    /// Analyze all suits
    #[must_use]
    #[inline]
    pub fn analyze_all_suits(&self) -> Vec<SuitLeadAnalysis> {
        [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
            .iter()
            .filter_map(|&suit| self.analyze_suit(suit))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_lead_result_creation() {
        let card = Card::new(Suit::Spades, 14);
        let mut result = CardLeadResult::new(card);

        assert_eq!(result.card, card);
        assert!(result.deal_details.is_none());

        result.record_tricks(9, None);
        result.record_tricks(9, None);
        result.record_tricks(10, None);

        assert_eq!(*result.tricks_distribution.get(9).unwrap(), 2);
        assert_eq!(*result.tricks_distribution.get(10).unwrap(), 1);
        assert_eq!(result.total_deals(), 3);
    }

    #[test]
    fn test_card_lead_result_with_details() {
        let card = Card::new(Suit::Hearts, 13);
        let mut result = CardLeadResult::with_deal_details(card, 10);

        result.record_tricks(9, Some(0));
        result.record_tricks(10, Some(1));

        assert!(result.deal_details.is_some());
        let details = result.deal_details.as_ref().unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].deal_index, 0);
        assert_eq!(details[0].tricks_taken, 9);
    }

    #[test]
    fn test_card_lead_result_histogram() {
        let card = Card::new(Suit::Diamonds, 12);
        let mut result = CardLeadResult::new(card);

        result.record_tricks(8, None);
        result.record_tricks(9, None);
        result.record_tricks(9, None);
        result.record_tricks(10, None);

        let histogram = result.tricks_histogram();

        // Should be sorted by tricks
        assert_eq!(histogram[8], (8, 1));
        assert_eq!(histogram[9], (9, 2));
        assert_eq!(histogram[10], (10, 1));
    }

    #[test]
    fn test_card_lead_result_cumulative() {
        let card = Card::new(Suit::Clubs, 11);
        let mut result = CardLeadResult::new(card);

        result.record_tricks(8, None);
        result.record_tricks(9, None);
        result.record_tricks(9, None);
        result.record_tricks(10, None);

        let cumulative = result.cumulative_distribution();

        assert_eq!(cumulative[8], (8, 1)); // 1 deal with 8 or less
        assert_eq!(cumulative[9], (9, 3)); // 3 deals with 9 or less
        assert_eq!(cumulative[10], (10, 4)); // 4 deals with 10 or less
    }

    #[test]
    fn test_card_lead_statistics_from_result() {
        let card = Card::new(Suit::Spades, 14);
        let mut result = CardLeadResult::new(card);

        // Simulate 10 deals: 5 times 8 tricks, 5 times 10 tricks
        for _ in 0..5 {
            result.record_tricks(8, None);
        }
        for _ in 0..5 {
            result.record_tricks(10, None);
        }

        let tricks_to_beat = 7;
        let stats = CardLeadStatistics::from_card_result(&result, tricks_to_beat);

        assert_eq!(stats.card, card);
        assert!((stats.average_tricks - 9.0).abs() < 0.01); // (5*8 + 5*10)/10 = 9
        assert_eq!(stats.min_tricks, 8);
        assert_eq!(stats.max_tricks, 10);
        assert!(stats.set_percentage - 100.0 < 1.0e-5); // All deals beat contract
    }

    #[test]
    fn test_card_lead_statistics_set_percentage() {
        let card = Card::new(Suit::Hearts, 13);
        let mut result = CardLeadResult::new(card);

        // 3 deals with 6 tricks (don't beat), 7 deals with 8 tricks (beat)
        for _ in 0..3 {
            result.record_tricks(6, None);
        }
        for _ in 0..7 {
            result.record_tricks(8, None);
        }

        let tricks_to_beat = 7;
        let stats = CardLeadStatistics::from_card_result(&result, tricks_to_beat);

        assert!(stats.set_percentage - 70.0 < 1.0e-5); // 7 out of 10 beat contract
    }

    #[test]
    fn test_raw_lead_data_creation() {
        let contract = Contract::from_str("3CN").unwrap();
        let mut raw = RawLeadData::new(contract);

        assert_eq!(raw.cards.len(), 0);
        assert_eq!(raw.contract, contract);

        let card = Card::new(Suit::Spades, 14);
        let mut card_result = CardLeadResult::new(card);
        card_result.record_tricks(9, None);
        raw.cards.push(card_result);

        // Test result_for
        let result = raw.result_for(card);
        assert!(result.is_some());
        assert_eq!(result.unwrap().card, card);

        // Test result_for non-existent card
        let other_card = Card::new(Suit::Hearts, 13);
        assert!(raw.result_for(other_card).is_none());
    }

    #[test]
    fn test_lead_statistics_from_raw() {
        let contract = Contract::from_str("4HS").unwrap();
        let mut raw = RawLeadData::new(contract);

        // Add results for two cards
        let card1 = Card::new(Suit::Spades, 14);
        let mut result1 = CardLeadResult::new(card1);
        for _ in 0..7 {
            result1.record_tricks(4, None); // Beats 4H (needs 4 tricks)
        }
        for _ in 0..3 {
            result1.record_tricks(3, None); // Doesn't beat
        }
        raw.cards.push(result1);

        let card2 = Card::new(Suit::Hearts, 13);
        let mut result2 = CardLeadResult::new(card2);
        for _ in 0..5 {
            result2.record_tricks(4, None); // Exactly beats
        }
        for _ in 0..5 {
            result2.record_tricks(3, None); // Doesn't beat
        }
        raw.cards.push(result2);

        let tricks_to_beat = 4;
        let stats = LeadStatistics::from_raw_data(&raw, tricks_to_beat);

        assert_eq!(stats.cards.len(), 2);
        assert_eq!(stats.contract, contract);
        assert_eq!(stats.tricks_to_beat, tricks_to_beat);

        // card1 should be better (70% vs 50% set percentage)
        assert_eq!(stats.best_by_set_percentage.card, card1);

        // Test stats_for
        let card1_stats = stats.stats_for(card1);
        assert!(card1_stats.is_some());
        assert!(card1_stats.unwrap().set_percentage - 70.0 < 1.0e-5);
    }

    #[test]
    fn test_lead_statistics_suit_analysis() {
        let contract = Contract::from_str("3NN").unwrap();
        let mut raw = RawLeadData::new(contract);

        // Add cards from different suits
        for suit in &[Suit::Spades, Suit::Hearts, Suit::Diamonds] {
            for rank in 13..=14 {
                let card = Card::new(*suit, rank);
                let mut result = CardLeadResult::new(card);
                for _ in 0..10 {
                    result.record_tricks(7, None);
                }
                raw.cards.push(result);
            }
        }

        let stats = LeadStatistics::from_raw_data(&raw, 6);

        // Test analyze_suit
        let spade_analysis = stats.analyze_suit(Suit::Spades);
        assert!(spade_analysis.is_some());
        let spade_analysis = spade_analysis.unwrap();
        assert_eq!(spade_analysis.suit, Suit::Spades);
        assert_eq!(spade_analysis.cards.len(), 2); // King and Ace

        // Test analyze_all_suits
        let all_suits = stats.analyze_all_suits();
        assert_eq!(all_suits.len(), 3); // Spades, Hearts, Diamonds
    }

    #[test]
    fn test_lead_statistics_compare_cards() {
        let contract = Contract::from_str("3NN").unwrap();
        let mut raw = RawLeadData::new(contract);

        let card1 = Card::new(Suit::Spades, 14);
        let mut result1 = CardLeadResult::new(card1);
        for _ in 0..10 {
            result1.record_tricks(8, None);
        }
        raw.cards.push(result1);

        let card2 = Card::new(Suit::Hearts, 13);
        let mut result2 = CardLeadResult::new(card2);
        for _ in 0..10 {
            result2.record_tricks(4, None);
        }
        raw.cards.push(result2);

        let stats = LeadStatistics::from_raw_data(&raw, 6);

        let comparison = stats.compare_cards(card1, card2);
        assert!(comparison.is_some());

        let comp = comparison.unwrap();
        assert_eq!(comp.card_a, card1);
        assert_eq!(comp.card_b, card2);
        assert!(comp.avg_tricks_diff > 0.0); // card1 takes more tricks
        assert_eq!(comp.better_card_imps, Some(card1));
        assert_eq!(comp.better_card_matchpoints, Some(card1));
    }

    #[cfg(feature = "dds")]
    #[test]
    fn test_lead_simulation_builder_validation() {
        let dealer = StandardDealer::default();
        let contract = Contract::from_str("3CN").unwrap();

        // Missing num_deals
        let result = LeadSimulationBuilder::new()
            .dealer(dealer)
            .contract(contract)
            .build();
        assert!(result.is_err());

        // Missing dealer
        let result = LeadSimulationBuilder::<StandardDealer>::default()
            .num_deals(100)
            .contract(contract)
            .build();
        assert!(result.is_err());

        // Missing contract
        let result = LeadSimulationBuilder::<StandardDealer>::default()
            .num_deals(100)
            .build();
        assert!(result.is_err());

        // Valid configuration
        let result = LeadSimulationBuilder::new()
            .dealer(StandardDealer::default())
            .num_deals(100)
            .contract(contract)
            .build();
        assert!(result.is_ok());

        // With deal details
        let result = LeadSimulationBuilder::new()
            .num_deals(100)
            .dealer(StandardDealer::default())
            .contract(contract)
            .with_deal_details()
            .build();
        assert!(result.is_ok());
    }

    /// A dealer that always returns DDS's example hand 0.
    fn example_dealer() -> impl Dealer {
        let deal = squeezer_solver::test_fixtures::example_deal();
        DealerBuilder::new()
            .predeal(Seat::North, deal[Seat::North])
            .expect("north parses")
            .predeal(Seat::East, deal[Seat::East])
            .expect("east parses")
            .predeal(Seat::South, deal[Seat::South])
            .expect("south parses")
            .predeal(Seat::West, deal[Seat::West])
            .expect("west parses")
            .build()
            .expect("the example deal is a partition")
    }

    /// The lead view over `CardEvaluation` reproduces `LeadSimulation`'s set
    /// percentage, and both agree with the hand-computed fixture.
    #[cfg(feature = "dds")]
    #[test]
    fn lead_view_reproduces_lead_simulation_set_percentage() {
        // 3S by West needs 9 tricks. A diamond lead holds declarer to 8 (set);
        // a heart lead gives 9 (making).
        let contract = Contract::new(
            3,
            Strain::Spades,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled,
        );

        let output = LeadSimulation::new(1, example_dealer(), contract)
            .run()
            .expect("the example hand solves");
        let d5 = output.statistics.stats_for(Card::D5).expect("D5 is led");
        let h2 = output.statistics.stats_for(Card::H2).expect("H2 is led");
        assert!((d5.set_percentage - 100.0).abs() < 1.0e-9);
        assert!(h2.set_percentage.abs() < 1.0e-9);

        let evaluation = CardEvaluation::builder()
            .num_deals(1)
            .dealer(example_dealer())
            .contract(contract)
            .build()
            .expect("valid configuration")
            .run()
            .expect("the example hand solves");
        let view = LeadAnalysis::new(&evaluation, contract);

        assert_eq!(view.tricks_to_beat(), 5);
        assert!(
            (view.set_percentage(Card::D5).expect("D5 is legal") - d5.set_percentage).abs()
                < 1.0e-9
        );
        assert!(
            (view.set_percentage(Card::H2).expect("H2 is legal") - h2.set_percentage).abs()
                < 1.0e-9
        );

        // The cumulative distribution is reachable from the view.
        let cumulative = view
            .cumulative_declarer_tricks(Card::D5)
            .expect("D5 is legal");
        assert_eq!(cumulative[8].1, 1);
        assert_eq!(cumulative[13].1, 1);
    }

    /// The lead view is per **physical** card, not per DDS equivalence-class
    /// representative.
    ///
    /// DDS returns 9 class representatives for example hand 0's opening lead
    /// (e.g. only C10 for the C10/C9/C8 class); `CardEvaluation` expands each
    /// class, so the lead view must list all 13 physical cards of the opening
    /// leader's hand. This is the intended design-ticket-06 behaviour, so it is
    /// pinned here rather than left implicit.
    #[cfg(feature = "dds")]
    #[test]
    fn lead_view_lists_every_physical_card_not_just_class_representatives() {
        let contract = Contract::new(
            1,
            Strain::Spades,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled,
        );
        let output = LeadSimulation::new(1, example_dealer(), contract)
            .run()
            .expect("the example hand solves");

        // North holds 13 cards and all are legal opening leads.
        assert_eq!(output.raw.cards.len(), 13);
        assert_eq!(output.statistics.cards.len(), 13);
        // The class collapsed by DDS is still fully represented.
        for card in [Card::C10, Card::C9, Card::C8] {
            assert!(
                output.statistics.stats_for(card).is_some(),
                "{card} is a physical lead and must appear"
            );
        }
    }
}
