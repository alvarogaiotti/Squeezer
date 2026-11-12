// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Lead Simulations
//!
//! Opening lead analysis as a view over [`CardEvaluation`] at the opening
//! position (ADR-0006). [`LeadSimulation::run`] samples the deals and returns
//! a [`LeadAnalysis`]; every lead-specific quantity is derived from the one
//! all-legal solve per deal, never from a second solver pass.
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
//! let analysis = LeadSimulation::builder()
//!     .num_deals(10)
//!     .dealer(StandardDealer::default())
//!     .contract(Contract::from_str("5CN")?)
//!     .build()?
//!     .run()?;
//!
//! for value in analysis.rankings() {
//!     println!("Card: {}", value.card);
//!     println!("  Set: {:.2}%", value.statistics.set_percentage);
//!     println!("  Avg tricks: {:.2}", value.statistics.mean_dd_tricks());
//! }
//! # Ok(())
//! # }
//! ```

use super::{missing_solver_factory, resolve_solver_factory};
use crate::prelude::*;
use squeezer_solver::ContextFactory;
use std::collections::HashMap;

// ============================================================================
// LEAD ANALYSIS VIEW
// ============================================================================

/// A lead-specific view over a [`CardEvaluation`] at the opening position.
///
/// The evaluation is the primitive; this view owns it and adds the lead
/// concepts that only make sense for an opening lead — `tricks_to_beat`, the
/// contract, set percentage, cumulative trick distributions and per-suit
/// grouping — without solving anything again.
#[derive(Debug)]
pub struct LeadAnalysis {
    evaluation: CardEvaluation,
    /// The contract being led against.
    pub contract: Contract,
}

impl LeadAnalysis {
    /// Wraps `evaluation` as a lead analysis for `contract`.
    #[must_use]
    pub fn new(evaluation: CardEvaluation, contract: Contract) -> Self {
        Self {
            evaluation,
            contract,
        }
    }

    /// The underlying card evaluation.
    #[must_use]
    pub fn evaluation(&self) -> &CardEvaluation {
        &self.evaluation
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

    /// The lead at the top of [`Self::rankings`], i.e. the highest set
    /// percentage, with ties broken by mean side-to-play tricks.
    #[must_use]
    pub fn best_by_set_percentage(&self) -> Option<&CardValue> {
        self.rankings().into_iter().next()
    }

    /// The lead with the highest mean side-to-play trick count, among the
    /// [`Self::rankings`] order (the last of the equal maxima).
    #[must_use]
    pub fn best_by_avg_tricks(&self) -> Option<&CardValue> {
        self.rankings().into_iter().max_by(|a, b| {
            a.statistics
                .mean_dd_tricks()
                .total_cmp(&b.statistics.mean_dd_tricks())
        })
    }

    /// The leads ranked by set percentage (descending), then by mean
    /// side-to-play tricks (descending), with ties keeping the evaluation's
    /// card order.
    #[must_use]
    pub fn rankings(&self) -> Vec<&CardValue> {
        let mut cards: Vec<&CardValue> = self.evaluation.values.iter().collect();
        cards.sort_by(|a, b| {
            b.statistics
                .set_percentage
                .total_cmp(&a.statistics.set_percentage)
                .then_with(|| {
                    b.statistics
                        .mean_dd_tricks()
                        .total_cmp(&a.statistics.mean_dd_tricks())
                })
        });
        cards
    }

    /// The 1-based position of `card` in [`Self::rankings`].
    #[must_use]
    pub fn rank_by_set_percentage(&self, card: Card) -> Option<usize> {
        self.rankings()
            .iter()
            .position(|value| value.card == card)
            .map(|position| position + 1)
    }

    /// Summarises the leads from `suit`, in [`Self::rankings`] order.
    ///
    /// Returns `None` when `suit` has no legal lead. The best-by fields keep
    /// the ranking's tie behaviour: the last of the equal maxima.
    #[must_use]
    pub fn analyze_suit(&self, suit: Suit) -> Option<SuitLeadAnalysis<'_>> {
        let cards: Vec<&CardValue> = self
            .rankings()
            .into_iter()
            .filter(|value| value.card.suit() == suit)
            .collect();

        if cards.is_empty() {
            return None;
        }

        let best_by_avg_tricks = cards.iter().copied().max_by(|a, b| {
            a.statistics
                .mean_dd_tricks()
                .total_cmp(&b.statistics.mean_dd_tricks())
        })?;
        let best_by_set_percentage = cards.iter().copied().max_by(|a, b| {
            a.statistics
                .set_percentage
                .total_cmp(&b.statistics.set_percentage)
        })?;

        #[allow(clippy::cast_precision_loss)]
        let suit_avg_tricks = cards
            .iter()
            .map(|value| value.statistics.mean_dd_tricks())
            .sum::<f64>()
            / cards.len() as f64;

        #[allow(clippy::cast_precision_loss)]
        let suit_avg_set_percentage = cards
            .iter()
            .map(|value| value.statistics.set_percentage)
            .sum::<f64>()
            / cards.len() as f64;

        Some(SuitLeadAnalysis {
            suit,
            cards,
            best_by_avg_tricks,
            best_by_set_percentage,
            suit_avg_tricks,
            suit_avg_set_percentage,
        })
    }

    /// Summarises every suit that has at least one legal lead.
    #[must_use]
    pub fn analyze_all_suits(&self) -> Vec<SuitLeadAnalysis<'_>> {
        [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
            .iter()
            .filter_map(|&suit| self.analyze_suit(suit))
            .collect()
    }

    /// Compares two leads by mean tricks and set percentage.
    ///
    /// Returns `None` unless both cards were legal leads. The verdicts keep the
    /// original heuristic: IMPs favour a set-percentage edge, matchpoints
    /// favour a mean-trick edge.
    #[must_use]
    pub fn compare_cards(&self, card_a: Card, card_b: Card) -> Option<LeadComparison> {
        let value_a = self.evaluation.value_for(card_a)?;
        let value_b = self.evaluation.value_for(card_b)?;

        let avg_tricks_diff =
            value_a.statistics.mean_dd_tricks() - value_b.statistics.mean_dd_tricks();
        let set_percentage_diff =
            value_a.statistics.set_percentage - value_b.statistics.set_percentage;

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

/// The comparison of two leads by mean tricks and set percentage.
#[derive(Debug, Clone)]
pub struct LeadComparison {
    /// First card.
    pub card_a: Card,

    /// Second card.
    pub card_b: Card,

    /// Difference in mean side-to-play tricks (a - b).
    pub avg_tricks_diff: f64,

    /// Difference in set percentage (a - b).
    pub set_percentage_diff: f64,

    /// Which card the IMP heuristic favours.
    pub better_card_imps: Option<Card>,

    /// Which card the matchpoint heuristic favours.
    pub better_card_matchpoints: Option<Card>,
}

/// Analysis of all leads from a particular suit.
#[derive(Debug)]
pub struct SuitLeadAnalysis<'a> {
    /// The suit being analyzed.
    pub suit: Suit,

    /// The suit's cards, in [`LeadAnalysis::rankings`] order.
    pub cards: Vec<&'a CardValue>,

    /// Best card in this suit by mean side-to-play tricks.
    pub best_by_avg_tricks: &'a CardValue,

    /// Best card in this suit by set percentage.
    pub best_by_set_percentage: &'a CardValue,

    /// Mean of the suit's per-card mean tricks.
    pub suit_avg_tricks: f64,

    /// Mean of the suit's per-card set percentages.
    pub suit_avg_set_percentage: f64,
}

// ============================================================================
// LEAD SIMULATION - Builder and implementation
// ============================================================================

/// Builder for configuring a lead simulation
pub struct LeadSimulationBuilder<D: Dealer = StandardDealer> {
    num_deals: Option<usize>,
    dealer: Option<D>,
    contract: Option<Contract>,
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
            solver_factory,
        })
    }
}

impl<D: Dealer> Default for LeadSimulationBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// Samples deals and returns their opening-lead [`LeadAnalysis`].
pub struct LeadSimulation<D: Dealer = StandardDealer> {
    num_deals: usize,
    dealer: D,
    contract: Contract,
    solver_factory: Box<dyn ContextFactory>,
}

impl<D: Dealer> LeadSimulation<D> {
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
            solver_factory: Box::new(DdsSolver::new()),
        }
    }

    /// Runs the lead analysis: one all-legal solve per sampled deal at the
    /// opening position, returned as a [`LeadAnalysis`] view.
    ///
    /// # Errors
    ///
    /// Propagates dealer and solver errors, and reports a board whose opening
    /// position cannot be derived.
    pub fn run(self) -> Result<LeadAnalysis, SimulationError> {
        let Self {
            num_deals,
            dealer,
            contract,
            solver_factory,
        } = self;

        let evaluation = CardEvaluation::builder()
            .num_deals(num_deals)
            .dealer(dealer)
            .contract(contract)
            .solver_factory(solver_factory)
            .build()?
            .run()?;

        Ok(LeadAnalysis::new(evaluation, contract))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::test_support::scripted_lead_analysis;

    fn three_spades() -> Contract {
        Contract::new(
            3,
            Strain::Spades,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled,
        )
    }

    #[test]
    fn rankings_order_by_set_percentage_then_mean_tricks() {
        let contract = three_spades();
        // Five defender tricks set 3S. All but C3 set; S4 takes the most.
        let view = scripted_lead_analysis(
            contract,
            &[
                (Card::D5, &[5, 5]),
                (Card::H2, &[6, 6]),
                (Card::C3, &[3, 3]),
                (Card::S4, &[7, 7]),
            ],
        );

        let ranking: Vec<Card> = view.rankings().iter().map(|value| value.card).collect();
        assert_eq!(ranking, vec![Card::S4, Card::H2, Card::D5, Card::C3]);
        assert_eq!(view.rank_by_set_percentage(Card::S4), Some(1));
        assert_eq!(view.rank_by_set_percentage(Card::D5), Some(3));
        assert_eq!(view.rank_by_set_percentage(Card::C3), Some(4));
        assert_eq!(view.rank_by_set_percentage(Card::C10), None);
        assert_eq!(
            view.best_by_set_percentage().map(|value| value.card),
            Some(Card::S4)
        );
    }

    #[test]
    fn suit_analysis_reports_best_cards_and_suit_averages() {
        let contract = three_spades();
        let view = scripted_lead_analysis(
            contract,
            &[
                (Card::S4, &[7, 7]),
                (Card::S3, &[5, 5]),
                (Card::H2, &[6, 6]),
                (Card::H3, &[3, 3]),
                (Card::D5, &[5, 5]),
            ],
        );

        let spades = view.analyze_suit(Suit::Spades).expect("spades are led");
        assert_eq!(spades.suit, Suit::Spades);
        assert_eq!(
            spades
                .cards
                .iter()
                .map(|value| value.card)
                .collect::<Vec<_>>(),
            vec![Card::S4, Card::S3]
        );
        assert_eq!(spades.best_by_avg_tricks.card, Card::S4);
        assert_eq!(spades.best_by_set_percentage.card, Card::S3);
        assert!((spades.suit_avg_tricks - 6.0).abs() < 1.0e-9);
        assert!((spades.suit_avg_set_percentage - 100.0).abs() < 1.0e-9);

        let hearts = view.analyze_suit(Suit::Hearts).expect("hearts are led");
        assert_eq!(hearts.best_by_avg_tricks.card, Card::H2);
        assert_eq!(hearts.best_by_set_percentage.card, Card::H2);
        assert!((hearts.suit_avg_tricks - 4.5).abs() < 1.0e-9);
        assert!((hearts.suit_avg_set_percentage - 50.0).abs() < 1.0e-9);

        assert!(view.analyze_suit(Suit::Clubs).is_none());
        assert_eq!(view.analyze_all_suits().len(), 3);
    }

    #[test]
    fn comparing_leads_keeps_the_imps_and_matchpoint_heuristics() {
        let contract = three_spades();
        let view = scripted_lead_analysis(contract, &[(Card::S4, &[7, 7]), (Card::H3, &[3, 3])]);

        let comparison = view
            .compare_cards(Card::S4, Card::H3)
            .expect("both cards are legal leads");
        assert!((comparison.avg_tricks_diff - 4.0).abs() < 1.0e-9);
        assert!((comparison.set_percentage_diff - 100.0).abs() < 1.0e-9);
        assert_eq!(comparison.better_card_imps, Some(Card::S4));
        assert_eq!(comparison.better_card_matchpoints, Some(Card::S4));

        assert!(view.compare_cards(Card::S4, Card::C10).is_none());
    }

    #[cfg(feature = "dds")]
    #[test]
    fn test_lead_simulation_builder_validation() {
        let dealer = StandardDealer::default();
        let contract = Contract::from_str("3CN").expect("the fixed contract parses");

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

    /// `LeadSimulation::run` returns the lead view directly, over the card
    /// evaluation it sampled, so there is no second result type to read.
    #[cfg(feature = "dds")]
    #[test]
    fn lead_simulation_run_returns_the_lead_view() {
        let contract = three_spades();

        let analysis = LeadSimulation::new(1, example_dealer(), contract)
            .run()
            .expect("the example hand solves");

        assert_eq!(analysis.contract, contract);
        assert_eq!(analysis.tricks_to_beat(), 5);
        // Every physical card is still a lead, and the primitive stays
        // reachable through the view.
        assert_eq!(analysis.cards().len(), 13);
        assert_eq!(analysis.rankings().len(), 13);
        assert_eq!(analysis.evaluation().position.to_play(), Seat::North);
    }

    /// The lead view reproduces the hand-computed fixture for the DDS example
    /// hand: a diamond lead sets 3S, a heart lead lets it make.
    #[cfg(feature = "dds")]
    #[test]
    fn lead_view_matches_the_hand_computed_fixture() {
        let analysis = LeadSimulation::new(1, example_dealer(), three_spades())
            .run()
            .expect("the example hand solves");

        assert_eq!(analysis.tricks_to_beat(), 5);
        assert!((analysis.set_percentage(Card::D5).expect("D5 is legal") - 100.0).abs() < 1.0e-9);
        assert!(
            analysis
                .set_percentage(Card::H2)
                .expect("H2 is legal")
                .abs()
                < 1.0e-9
        );

        // The cumulative distribution is reachable from the view.
        let cumulative = analysis
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
        let analysis = LeadSimulation::new(1, example_dealer(), three_spades())
            .run()
            .expect("the example hand solves");

        // North holds 13 cards and all are legal opening leads.
        assert_eq!(analysis.cards().len(), 13);
        assert_eq!(analysis.rankings().len(), 13);
        // The class collapsed by DDS is still fully represented.
        for card in [Card::C10, Card::C9, Card::C8] {
            assert!(
                analysis.evaluation().value_for(card).is_some(),
                "{card} is a physical lead and must appear"
            );
        }
    }
}
