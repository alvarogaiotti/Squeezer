// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Card expected value
//!
//! The card expected-value primitive: one all-legal solve per sampled deal
//! returns every legal card's double-dummy value at once, and everything else
//! is derived (design ticket 06, ADR-0006). Lead analysis is a view over the
//! same primitive at the opening position, not a second solver path.
//!
//! ## Perspective
//!
//! DDS reports tricks for the *side to play* at the solved position. Scoring a
//! contract needs the declarer's tricks, so the outcome is complemented unless
//! the candidate's side *is* the declaring side:
//!
//! ```text
//! declarer_tricks = if to_play.is_same_line(&declarer) { dd_tricks } else { 13 - dd_tricks }
//! ```
//!
//! Because the primitive solves the position *before* the candidate is played,
//! `to_play` is always the candidate's own side, which removes the trick-winner
//! edge that the deleted after-the-card path had to reason about.
//!
//! ## Endgame
//!
//! With four or fewer cards remaining DDS returns only the lead card; that is
//! also the only legal play, so the card-EV surface stays consistent without a
//! special case.

use super::resolve_solver_factory;
use super::solve_all_cards_pool;
use crate::prelude::*;
use squeezer_solver::ContextFactory;
use std::collections::{BTreeMap, HashMap};

/// One option's outcome on one sampled deal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardDealOutcome {
    /// Index of the deal in `0..num_deals`.
    pub deal_index: usize,
    /// Tricks for the side to play, exactly as DDS reported them.
    pub dd_tricks: u8,
    /// The same result from the declarer's perspective.
    pub declarer_tricks: u8,
    /// `Contract::score(declarer_tricks)`.
    pub score: i32,
    /// The score valued per the evaluation's [`Denomination`].
    pub value: i32,
}

/// Summary statistics for one card's per-deal outcomes.
///
/// `mean` and the spread statistics are over [`CardDealOutcome::value`], i.e.
/// the quantity the denomination ranks. The lead-derived fields are over the
/// declarer's tricks: `set_percentage` is the fraction of deals where
/// `declarer_tricks < level + 6`, and the histograms let a consumer rebuild
/// trick distributions without the raw outcomes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardStatistics {
    /// Number of deals this card was evaluated on.
    pub deals: usize,
    /// Mean value.
    pub mean: f64,
    /// Population standard deviation of the value.
    pub std_dev: f64,
    /// Standard error of the mean.
    pub std_error: f64,
    /// 95% confidence interval of the mean.
    pub confidence_interval: (f64, f64),
    /// Minimum value observed.
    pub min: i32,
    /// Maximum value observed.
    pub max: i32,
    /// 25th percentile of the value.
    pub q1: f64,
    /// Median (50th percentile) of the value.
    pub median: f64,
    /// 75th percentile of the value.
    pub q3: f64,
    /// Score histogram: score to number of deals.
    pub score_distribution: BTreeMap<i32, usize>,
    /// Fraction of deals, in percent, where the contract is set.
    pub set_percentage: f64,
    /// Tricks for the side to play, bucketed `0..=13`.
    pub dd_tricks: [usize; 14],
    /// Declarer tricks, bucketed `0..=13`.
    pub declarer_tricks: [usize; 14],
}

impl CardStatistics {
    /// Computes the statistics of `outcomes` for a contract of `level`.
    ///
    /// # Panics
    ///
    /// Panics only if `outcomes` is non-empty but `values.first()` and
    /// `values.last()` disagree on non-empty, which they cannot.
    #[must_use]
    pub fn from_outcomes(outcomes: &[CardDealOutcome], level: u8) -> Self {
        let deals = outcomes.len();
        if deals == 0 {
            return Self::default();
        }

        let mut values: Vec<i32> = outcomes.iter().map(|outcome| outcome.value).collect();
        values.sort_unstable();

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_lossless
        )]
        let n = (deals as u32) as f64;
        #[allow(clippy::cast_precision_loss)]
        let mean = values.iter().map(|&value| f64::from(value)).sum::<f64>() / n;

        #[allow(clippy::cast_precision_loss)]
        let variance = values
            .iter()
            .map(|&value| {
                let diff = f64::from(value) - mean;
                diff * diff
            })
            .sum::<f64>()
            / n;
        let std_dev = variance.sqrt();
        let std_error = std_dev / n.sqrt();

        let min = *values.first().expect("non-empty values");
        let max = *values.last().expect("non-empty values");
        let q1 = percentile(&values, 0.25);
        let median = percentile(&values, 0.50);
        let q3 = percentile(&values, 0.75);

        let mut score_distribution: BTreeMap<i32, usize> = BTreeMap::new();
        let mut dd_tricks = [0usize; 14];
        let mut declarer_tricks = [0usize; 14];
        let mut set_count = 0usize;
        for outcome in outcomes {
            *score_distribution.entry(outcome.score).or_insert(0) += 1;
            dd_tricks[outcome.dd_tricks as usize] += 1;
            declarer_tricks[outcome.declarer_tricks as usize] += 1;
            if outcome.declarer_tricks < level + 6 {
                set_count += 1;
            }
        }

        #[allow(clippy::cast_precision_loss)]
        let set_percentage = set_count as f64 / n * 100.0;

        Self {
            deals,
            mean,
            std_dev,
            std_error,
            confidence_interval: (mean - 1.96 * std_error, mean + 1.96 * std_error),
            min,
            max,
            q1,
            median,
            q3,
            score_distribution,
            set_percentage,
            dd_tricks,
            declarer_tricks,
        }
    }

    /// Cumulative declarer-trick distribution: `(tricks, deals at most tricks)`.
    #[must_use]
    pub fn cumulative_declarer_tricks(&self) -> [(usize, usize); 14] {
        cumulative(&self.declarer_tricks)
    }

    /// Cumulative side-to-play trick distribution.
    #[must_use]
    pub fn cumulative_dd_tricks(&self) -> [(usize, usize); 14] {
        cumulative(&self.dd_tricks)
    }

    /// Mean tricks for the side to play, derived from the histogram.
    #[must_use]
    pub fn mean_dd_tricks(&self) -> f64 {
        if self.deals == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let total: usize = self
            .dd_tricks
            .iter()
            .enumerate()
            .map(|(tricks, &count)| tricks * count)
            .sum();
        #[allow(clippy::cast_precision_loss)]
        let mean = total as f64 / self.deals as f64;
        mean
    }
}

impl Default for CardStatistics {
    fn default() -> Self {
        Self {
            deals: 0,
            mean: 0.0,
            std_dev: 0.0,
            std_error: 0.0,
            confidence_interval: (0.0, 0.0),
            min: 0,
            max: 0,
            q1: 0.0,
            median: 0.0,
            q3: 0.0,
            score_distribution: BTreeMap::new(),
            set_percentage: 0.0,
            dd_tricks: [0; 14],
            declarer_tricks: [0; 14],
        }
    }
}

/// One physical legal card, its per-deal outcomes and summary statistics.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardValue {
    /// The card this value is for.
    pub card: Card,
    /// The outcome on every deal the card was legal on, in deal order.
    pub deals: Vec<CardDealOutcome>,
    /// Summary statistics over `deals`.
    pub statistics: CardStatistics,
}

/// The card expected-value surface over one all-legal solve per sampled deal.
#[derive(Debug)]
pub struct CardEvaluation {
    /// The position solved for the representative (first) sampled deal.
    pub position: Position,
    /// How outcomes were valued.
    pub denomination: Denomination,
    /// One entry per physical legal card, sorted by suit then rank.
    pub values: Vec<CardValue>,
    /// The DD-optimal equivalence classes of the representative deal.
    pub dd_optimal: Vec<Vec<Card>>,
}

impl CardEvaluation {
    /// Starts configuring a card evaluation.
    #[must_use]
    pub fn builder<D: Dealer>() -> CardEvBuilder<D> {
        CardEvBuilder::new()
    }

    /// The evaluated value of `card`, if it was legal on any sampled deal.
    #[must_use]
    pub fn value_for(&self, card: Card) -> Option<&CardValue> {
        self.values.iter().find(|value| value.card == card)
    }

    /// Groups the evaluated cards by suit.
    #[must_use]
    pub fn by_suit(&self) -> HashMap<Suit, Vec<&CardValue>> {
        let mut by_suit: HashMap<Suit, Vec<&CardValue>> = HashMap::new();
        for value in &self.values {
            by_suit.entry(value.card.suit()).or_default().push(value);
        }
        by_suit
    }

    /// The union of the DD-optimal equivalence classes.
    #[must_use]
    pub fn dd_optimal_cards(&self) -> Cards {
        self.dd_optimal
            .iter()
            .flat_map(|class| class.iter().copied())
            .fold(Cards::EMPTY, Cards::insert)
    }

    /// Whether `card` is in a DD-optimal equivalence class.
    #[must_use]
    pub fn is_dd_optimal(&self, card: Card) -> bool {
        self.dd_optimal.iter().any(|class| class.contains(&card))
    }
}

/// Builder for [`CardEvaluation`].
pub struct CardEvBuilder<D: Dealer> {
    num_deals: Option<usize>,
    dealer: Option<D>,
    contract: Option<Contract>,
    seat: Option<Seat>,
    play_trace: Vec<Card>,
    denomination: Denomination,
    solver_factory: Option<Box<dyn ContextFactory>>,
}

impl<D: Dealer> CardEvBuilder<D> {
    /// Creates an empty builder; raw points and the opening lead are the
    /// defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            num_deals: None,
            dealer: None,
            contract: None,
            seat: None,
            play_trace: Vec::new(),
            denomination: Denomination::RawPoints,
            solver_factory: None,
        }
    }

    /// Sets the number of deals to sample.
    #[must_use]
    pub fn num_deals(mut self, num: usize) -> Self {
        self.num_deals = Some(num);
        self
    }

    /// Sets the dealer whose conditioning shapes the sampled deals.
    #[must_use]
    pub fn dealer(mut self, dealer: D) -> Self {
        self.dealer = Some(dealer);
        self
    }

    /// Sets the contract the position is solved under.
    #[must_use]
    pub fn contract(mut self, contract: Contract) -> Self {
        self.contract = Some(contract);
        self
    }

    /// Sets the seat to move. Defaults to the contract's opening leader.
    #[must_use]
    pub fn seat(mut self, seat: Seat) -> Self {
        self.seat = Some(seat);
        self
    }

    /// Sets the cards already played, in play order.
    ///
    /// The last card was played by `seat`'s predecessor, the one before by
    /// theirs, and so on; an empty trace is the opening lead.
    #[must_use]
    pub fn play_trace(mut self, trace: Vec<Card>) -> Self {
        self.play_trace = trace;
        self
    }

    /// Sets how outcomes are valued. Defaults to [`Denomination::RawPoints`].
    #[must_use]
    pub fn denomination(mut self, denomination: Denomination) -> Self {
        self.denomination = denomination;
        self
    }

    /// Uses a specific solver backend; defaults to DDS.
    #[must_use]
    pub fn solver_factory(mut self, factory: Box<dyn ContextFactory>) -> Self {
        self.solver_factory = Some(factory);
        self
    }

    /// Builds the evaluation, validating the configuration.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidInput`] when the number of deals, the
    /// dealer or the contract is missing, when `num_deals` is zero (there would
    /// be no representative position), or when the play trace leaves no card to
    /// play.
    pub fn build(self) -> Result<CardEvSimulation<D>, SimulationError> {
        let num_deals = self
            .num_deals
            .ok_or_else(|| SimulationError::invalid_input("card ev", "num_deals must be set"))?;
        let dealer = self
            .dealer
            .ok_or_else(|| SimulationError::invalid_input("card ev", "dealer must be set"))?;
        let contract = self
            .contract
            .ok_or_else(|| SimulationError::invalid_input("card ev", "contract must be set"))?;

        if num_deals == 0 {
            return Err(SimulationError::invalid_input(
                "card ev",
                "num_deals must be at least 1",
            ));
        }
        if self.play_trace.len() >= 52 {
            return Err(SimulationError::invalid_input(
                "card ev",
                "the play trace leaves no card to play",
            ));
        }

        Ok(CardEvSimulation {
            num_deals,
            dealer,
            contract,
            seat: self.seat.unwrap_or_else(|| contract.leader()),
            play_trace: self.play_trace,
            denomination: self.denomination,
            solver_factory: resolve_solver_factory(self.solver_factory, "card ev")?,
        })
    }
}

impl<D: Dealer> Default for CardEvBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// A configured card evaluation ready to run.
pub struct CardEvSimulation<D: Dealer> {
    num_deals: usize,
    dealer: D,
    contract: Contract,
    seat: Seat,
    play_trace: Vec<Card>,
    denomination: Denomination,
    solver_factory: Box<dyn ContextFactory>,
}

impl<D: Dealer> CardEvSimulation<D> {
    /// Runs the evaluation: one all-legal solve per sampled deal.
    ///
    /// # Errors
    ///
    /// Propagates dealer and solver errors, and reports a board the play trace
    /// cannot be validated against.
    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> Result<CardEvaluation, SimulationError> {
        let Self {
            num_deals,
            dealer,
            contract,
            seat,
            play_trace,
            denomination,
            solver_factory,
        } = self;

        let declarer = contract.declarer();
        let prefix = prefix_plays(seat, &play_trace);

        // The shared engine generates the deals sequentially and solves every
        // legal card of each deal in one call; only the derivation below
        // remains here. The opening position is the empty-prefix case, which
        // `from_deal_prefix` handles with the contract's leader.
        let solved = solve_all_cards_pool(
            solver_factory,
            &dealer,
            num_deals,
            |deal, deal_index| -> Result<Position, SimulationError> {
                Position::from_deal_prefix_with_contract(deal, &prefix, contract).map_err(|cause| {
                    SimulationError::invalid_input(
                        "card ev",
                        format!("cannot derive position for deal {deal_index}: {cause}"),
                    )
                })
            },
        )?;

        let position = solved
            .first()
            .map(|entry| entry.position.clone())
            .ok_or_else(|| SimulationError::invalid_input("card ev", "no deals were solved"))?;

        // Expand DDS's equivalence classes into physical cards; each physical
        // card carries its class's value. The per-deal `Compared` baseline is
        // resolved over the physical alternatives that were legal that deal.
        let mut accumulators: [Option<(Card, Vec<CardDealOutcome>)>; 64] =
            std::array::from_fn(|_| None);

        // These three buffers are reused across every sampled deal, so their
        // allocations and growth are paid once per simulation instead of once
        // per deal (and, for `other_scores`, once per physical card). The mock
        // profile attributed roughly half of the non-native cost to that
        // churn. `physical` holds the expanded equivalence classes of the
        // current deal, `scores` mirrors their `score` fields for the baseline
        // lookup, and `other_scores` is the "every other legal card" view that
        // `Denomination::value` consumes.
        let mut physical: Vec<(Card, CardDealOutcome)> = Vec::new();
        let mut scores: Vec<i32> = Vec::new();
        let mut other_scores: Vec<i32> = Vec::new();

        for (deal_index, entry) in solved.iter().enumerate() {
            let to_play = entry.position.to_play();
            physical.clear();
            for solve in &entry.results.cards {
                let dd_tricks = solve.tricks;
                let declarer_tricks = declarer_tricks(to_play, declarer, dd_tricks);
                let score = contract.score(declarer_tricks);
                for card in solve.equivalents {
                    physical.push((
                        card,
                        CardDealOutcome {
                            deal_index,
                            dd_tricks,
                            declarer_tricks,
                            score,
                            value: 0,
                        },
                    ));
                }
            }

            scores.clear();
            scores.extend(physical.iter().map(|(_, outcome)| outcome.score));
            for (index, (card, outcome)) in physical.iter().enumerate() {
                let mut outcome = *outcome;
                other_scores.clear();
                other_scores.extend(
                    scores
                        .iter()
                        .enumerate()
                        .filter(|(other, _)| *other != index)
                        .map(|(_, &score)| score),
                );
                outcome.value = denomination.value(outcome.score, &other_scores);
                accumulators[card.as_u8() as usize]
                    .get_or_insert_with(|| (*card, Vec::with_capacity(num_deals)))
                    .1
                    .push(outcome);
            }
        }

        let mut values: Vec<CardValue> = accumulators
            .into_iter()
            .flatten()
            .map(|(card, deals)| {
                let statistics = CardStatistics::from_outcomes(&deals, contract.level());
                CardValue {
                    card,
                    deals,
                    statistics,
                }
            })
            .collect();
        values.sort_by(|a, b| compare_cards_by_suit_then_rank(a.card, b.card));

        // The representative deal's DD-optimal equivalence classes: the classes
        // whose side-to-play tricks equal the maximum returned.
        let dd_optimal = solved.first().map_or_else(Vec::new, |entry| {
            let max = entry.results.cards.iter().map(|solve| solve.tricks).max();
            match max {
                Some(max) => entry
                    .results
                    .cards
                    .iter()
                    .filter(|solve| solve.tricks == max)
                    .map(|solve| solve.equivalents.into_iter().collect())
                    .collect(),
                None => Vec::new(),
            }
        });

        Ok(CardEvaluation {
            position,
            denomination,
            values,
            dd_optimal,
        })
    }
}

/// Orders cards by suit, then by descending rank (bridge display order).
///
/// Shared by card-EV's `values` and the lead view's raw data so the two never
/// drift apart.
#[must_use]
pub(crate) fn compare_cards_by_suit_then_rank(a: Card, b: Card) -> std::cmp::Ordering {
    a.suit()
        .cmp(&b.suit())
        .then_with(|| b.rank().cmp(&a.rank()))
}

/// Converts side-to-play tricks into declarer tricks.
///
/// DDS reports the side to play; if that side is the declaring side the count
/// is already declarer-relative, otherwise it is its complement.
#[must_use]
pub(crate) fn declarer_tricks(to_play: Seat, declarer: Seat, dd_tricks: u8) -> u8 {
    if to_play.is_same_line(&declarer) {
        dd_tricks
    } else {
        13 - dd_tricks
    }
}

/// Reconstructs the seats of a play trace ending with `seat` to move.
fn prefix_plays(seat: Seat, play_trace: &[Card]) -> Vec<Play> {
    let mut player = seat;
    let mut prefix = Vec::with_capacity(play_trace.len());
    for &card in play_trace.iter().rev() {
        player = player.previous();
        prefix.push(Play::new(player, card));
    }
    prefix.reverse();
    prefix
}

/// Linear-interpolated percentile over sorted data.
fn percentile(sorted: &[i32], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let index = (sorted.len() as f64 - 1.0) * p;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let lower = index.floor() as usize;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let upper = index.ceil() as usize;
    if lower == upper {
        f64::from(sorted[lower])
    } else {
        #[allow(clippy::cast_precision_loss)]
        let weight = index - lower as f64;
        f64::from(sorted[lower]) * (1.0 - weight) + f64::from(sorted[upper]) * weight
    }
}

/// Cumulative histogram over a fixed 14-bucket trick histogram.
fn cumulative(histogram: &[usize; 14]) -> [(usize, usize); 14] {
    let mut out = [(0usize, 0usize); 14];
    let mut running = 0usize;
    for (tricks, &count) in histogram.iter().enumerate() {
        running += count;
        out[tricks] = (tricks, running);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use squeezer_solver::test_fixtures::example_deal;

    /// A dealer that always returns DDS's example hand 0.
    fn example_dealer() -> impl Dealer {
        let deal = example_deal();
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

    fn example_contract() -> Contract {
        Contract::new(
            1,
            Strain::Spades,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled,
        )
    }

    fn evaluate(contract: Contract, denomination: Denomination) -> CardEvaluation {
        CardEvaluation::builder()
            .num_deals(1)
            .dealer(example_dealer())
            .contract(contract)
            .denomination(denomination)
            .build()
            .expect("valid configuration")
            .run()
            .expect("example hand 0 solves")
    }

    #[test]
    fn perspective_conversion_uses_the_declarer_side() {
        // North is a defender, so the side-to-play count is complemented.
        assert_eq!(declarer_tricks(Seat::North, Seat::West, 5), 8);
        // West is the declarer, so the count is already declarer-relative.
        assert_eq!(declarer_tricks(Seat::West, Seat::West, 8), 8);
        // Partners count as the declarer's side.
        assert_eq!(declarer_tricks(Seat::East, Seat::West, 8), 8);
    }

    #[cfg(feature = "dds")]
    #[test]
    fn card_ev_matches_dds_example_hand_perspective() {
        let evaluation = evaluate(example_contract(), Denomination::RawPoints);

        // North is on lead against 1S by West.
        assert_eq!(evaluation.position.to_play(), Seat::North);

        // DDS reports 5 tricks for the side to play with a diamond lead; the
        // declarer therefore takes 13 - 5 = 8 tricks and scores 110 (1S + 1).
        let diamond_five = evaluation.value_for(Card::D5).expect("D5 is a legal lead");
        let outcome = diamond_five.deals.first().expect("one deal");
        assert_eq!(outcome.dd_tricks, 5);
        assert_eq!(outcome.declarer_tricks, 8);
        assert_eq!(outcome.score, 110);
        assert_eq!(outcome.value, 110);

        // A heart lead holds the side to play to 4 tricks, so the declarer
        // takes 9 and scores 140.
        let heart_two = evaluation.value_for(Card::H2).expect("H2 is a legal lead");
        let outcome = heart_two.deals.first().expect("one deal");
        assert_eq!(outcome.dd_tricks, 4);
        assert_eq!(outcome.declarer_tricks, 9);
        assert_eq!(outcome.score, 140);
    }

    #[cfg(feature = "dds")]
    #[test]
    fn equals_expansion_makes_every_physical_card_carry_its_class_value() {
        let evaluation = evaluate(example_contract(), Denomination::RawPoints);

        // All 13 cards in North's hand are legal opening leads.
        assert_eq!(evaluation.values.len(), 13);

        // DDS collapses C10/C9/C8 into one class; each physical card carries
        // the class's value.
        let value_of = |card| {
            evaluation
                .value_for(card)
                .expect("card is a legal lead")
                .deals
                .first()
                .expect("one deal")
                .dd_tricks
        };
        assert_eq!(value_of(Card::C10), 5);
        assert_eq!(value_of(Card::C9), 5);
        assert_eq!(value_of(Card::C8), 5);

        // The classes stay available separately.
        let club_class = evaluation
            .dd_optimal
            .iter()
            .find(|class| class.contains(&Card::C10))
            .expect("C10 is DD-optimal");
        assert!(club_class.contains(&Card::C9));
        assert!(club_class.contains(&Card::C8));
        assert!(evaluation.is_dd_optimal(Card::C9));
        assert!(!evaluation.is_dd_optimal(Card::H2));
    }

    #[cfg(feature = "dds")]
    #[test]
    fn raw_points_and_compared_baselines_on_the_example_hand() {
        // Raw points is the default and returns the score unchanged.
        let raw = evaluate(example_contract(), Denomination::default());
        assert_eq!(raw.value_for(Card::D5).expect("legal").deals[0].value, 110);
        assert_eq!(raw.value_for(Card::H2).expect("legal").deals[0].value, 140);

        // 110 against par 0 is a 3-IMP swing; 140 is 4 IMPs.
        let imps_against_par = evaluate(
            example_contract(),
            Denomination::Compared {
                baseline: Baseline::Fixed(0),
                scoring: Scoring::Imps,
            },
        );
        assert_eq!(
            imps_against_par.value_for(Card::D5).expect("legal").deals[0].value,
            3
        );
        assert_eq!(
            imps_against_par.value_for(Card::H2).expect("legal").deals[0].value,
            4
        );

        // Average of the other 12 physical options: 120 for a 110 card,
        // 117 (integer mean) for a 140 card.
        let average = evaluate(
            example_contract(),
            Denomination::Compared {
                baseline: Baseline::AverageOfOptions,
                scoring: Scoring::Custom(Box::new(|score, baseline| score - baseline)),
            },
        );
        assert_eq!(
            average.value_for(Card::D5).expect("legal").deals[0].value,
            -10
        );
        assert_eq!(
            average.value_for(Card::H2).expect("legal").deals[0].value,
            23
        );

        // Best of the other options is a heart (140) for a diamond card, and
        // another heart (140) for a heart card.
        let best = evaluate(
            example_contract(),
            Denomination::Compared {
                baseline: Baseline::BestOption,
                scoring: Scoring::Custom(Box::new(|score, baseline| score - baseline)),
            },
        );
        assert_eq!(best.value_for(Card::D5).expect("legal").deals[0].value, -30);
        assert_eq!(best.value_for(Card::H2).expect("legal").deals[0].value, 0);
    }

    #[cfg(feature = "dds")]
    #[test]
    fn compared_card_ev_values_remain_correct_across_deals() {
        let evaluation = CardEvaluation::builder()
            .num_deals(2)
            .dealer(example_dealer())
            .contract(example_contract())
            .denomination(Denomination::Compared {
                baseline: Baseline::AverageOfOptions,
                scoring: Scoring::Custom(Box::new(|score, baseline| score - baseline)),
            })
            .build()
            .expect("valid configuration")
            .run()
            .expect("example hand solves twice");

        for (card, expected) in [(Card::D5, -10), (Card::H2, 23)] {
            let outcomes = &evaluation
                .value_for(card)
                .expect("card is a legal lead")
                .deals;
            assert_eq!(outcomes.len(), 2);
            assert!(outcomes.iter().all(|outcome| outcome.value == expected));
        }
    }

    #[cfg(feature = "dds")]
    #[test]
    fn by_suit_groups_the_evaluated_cards() {
        let evaluation = evaluate(example_contract(), Denomination::RawPoints);
        let by_suit = evaluation.by_suit();
        assert_eq!(by_suit[&Suit::Spades].len(), 3);
        assert_eq!(by_suit[&Suit::Hearts].len(), 4);
        assert_eq!(by_suit[&Suit::Diamonds].len(), 3);
        assert_eq!(by_suit[&Suit::Clubs].len(), 3);
    }

    #[cfg(feature = "dds")]
    #[test]
    fn set_percentage_counts_declarer_failures() {
        // 3S by West needs 9 tricks; the diamond/club/spade leads hold the
        // declarer to 8 (set), the heart leads give 9 (making).
        let contract = Contract::new(
            3,
            Strain::Spades,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled,
        );
        let evaluation = evaluate(contract, Denomination::RawPoints);
        let stats = &evaluation.value_for(Card::D5).expect("legal").statistics;
        assert!((stats.set_percentage - 100.0).abs() < 1.0e-9);
        let stats = &evaluation.value_for(Card::H2).expect("legal").statistics;
        assert!(stats.set_percentage.abs() < 1.0e-9);
    }
}
