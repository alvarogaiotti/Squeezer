// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use super::{Simulation, SimulationResult};
use colored::Colorize;
use std::{collections::HashMap, fmt::Display};

use crate::{
    prelude::{Card, Contract, Dealer, SqueezerError},
    Deal, Suit,
};
use dds::{
    solver::{BridgeSolver, SolvedBoards},
    MAXNOOFBOARDS,
};
use itertools::Itertools;

/// The struct you will fire up when you want to run a lead simulation.
/// You will provide the number of boards you want to run the simulation
/// for; a structure implementing the [`crate::prelude::Dealer`] trait,
/// which will handle the deal creation; and a contract to run the simulation over.
///
/// This structure implements the [`super::Simulation`] trait, so you can run it and it will
/// generate `num_of_boards` deals, finding the best lead for every single deal and then computing
/// some statistics for the whole deals, finding the best lead from a tricks perspective and from the
/// % of contract setting perspective.
///
/// # Example
///
/// ```
/// use squeezer::prelude::*;
/// let dealer = StandardDealer::default(); // Not very useful but you get the idea
/// let lead_sim = LeadSimulation::new(
///     100,
///     dealer,
///     Contract::from_str("5CN", Vulnerable::No).unwrap(),
/// );
/// let results = lead_sim.run().expect("unable to run simulation");
/// results.report();
/// ```
pub struct LeadSimulation<D: Dealer> {
    num_of_boards: usize,
    dealer: D,
    contract: Contract,
}

impl<D: Dealer> LeadSimulation<D> {
    pub fn new(num_of_boards: usize, dealer: D, contract: Contract) -> Self {
        Self {
            num_of_boards,
            dealer,
            contract,
        }
    }

    fn solve_boards<S: BridgeSolver>(
        &self,
        num: usize,
        solver: &S,
        contracts: &[Contract],
    ) -> Result<SolvedBoards, SqueezerError> {
        // We take from the deal producer the number we need
        let mut deals: Vec<Deal> = Vec::with_capacity(num);
        for _ in 0..num {
            deals.push(self.dealer.deal()?);
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        solver
            .dd_tricks_all_cards_parallel(num as i32, &deals, contracts)
            .map_err(Into::into)
    }
}

impl<T: Dealer> Simulation<LeadSimulationResult> for LeadSimulation<T> {
    #[allow(clippy::integer_division)]
    fn run(&self) -> Result<LeadSimulationResult, SqueezerError> {
        let mut sim_result = LeadSimulationResult::new(self.contract, self.num_of_boards);
        let mut counter = self.num_of_boards;

        let contracts = [self.contract; MAXNOOFBOARDS];
        let solver = dds::doubledummy::MultiThreadDoubleDummySolver::new();
        // For the number of boards, stepped by the number of deals we use per analysis
        while counter != 0 {
            if let Some(new_counter) = counter.checked_sub(MAXNOOFBOARDS) {
                let solvedb = self.solve_boards(MAXNOOFBOARDS, &solver, &contracts)?;
                sim_result.add_results(solvedb);
                counter = new_counter;
            } else {
                let solvedb = self.solve_boards(counter, &solver, &contracts[0..counter])?;
                sim_result.add_results(solvedb);
                counter = 0;
            }
        }

        sim_result.finish(8 - self.contract.level(), self.num_of_boards);
        Ok(sim_result)
    }
}

#[derive(Debug, Clone, Copy)]
/// A single lead card, storing the number of times it will make
/// `number_of_tricks` in a array. The `average_tricks` and `set_percentage`
/// will be calculate at the end of the simulation.
pub struct LeadCard {
    card: Card,
    number_of_tricks: [usize; 14],
    average_tricks: f32,
    set_percentage: f32,
}

impl LeadCard {
    #[must_use]
    pub fn new(card: Card) -> Self {
        Self {
            card,
            number_of_tricks: [0; 14],
            average_tricks: 0.0,
            set_percentage: 0.0,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    /// This will compute the statistics for the card.
    /// You will need to provide the number of tricks able to beat the
    /// contract (e.g. 5 for a 3 level contract, formula: 8 - level_of_contract), and the number of runs.
    fn finish(&mut self, tricks_beating: u8, runs: usize) {
        // FIXME: Evaluate if providing runs is faster than calculating a running
        // sum in this loop and using it. I assumed it was but we really have this
        // information already.
        // Try to get this to compile in a SIMD friendly way.
        self.average_tricks = self
            .number_of_tricks
            .iter()
            .enumerate()
            .skip(1) // Skip zero since will add up to zero
            .map(|(times, tricks)| times * tricks)
            .sum::<usize>() as f32
            / runs as f32;
        self.set_percentage = (self.number_of_tricks[tricks_beating as usize..]
            .iter()
            .sum::<usize>() as f32
            / runs as f32)
            * 100.0;
    }
}

/// The simulation results, containing a `HashMap` from `Card` to `LeadCard` for
/// storing the result and being able to update the data.
pub struct LeadSimulationResult {
    lead_results: HashMap<Card, LeadCard>,
    deals_run: usize,
    contract: Contract,
}

impl LeadSimulationResult {
    #[inline]
    #[must_use]
    fn new(contract: Contract, deals_run: usize) -> Self {
        Self {
            lead_results: HashMap::with_capacity(10),
            contract,
            deals_run,
        }
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[inline]
    /// # Panics
    ///
    /// Panics when DDS is in unstable state and returns a negative numbers for the suit of a card
    fn add_results(&mut self, results: SolvedBoards) {
        for future_tricks in results.into_iter() {
            for index in 0..future_tricks.cards as usize {
                let rank = future_tricks.rank[index];
                let suit = future_tricks.suit[index];
                let card = Card::new(Suit::try_from(suit).unwrap(), rank as u8);
                self.lead_results
                    .entry(card)
                    .and_modify(|lead_card| {
                        lead_card.number_of_tricks[future_tricks.score[index] as usize] += 1;
                    })
                    .or_insert_with(|| {
                        let mut lead_card = LeadCard::new(card);
                        lead_card.number_of_tricks[future_tricks.score[index] as usize] += 1;
                        lead_card
                    });
            }
        }
    }

    fn finish(&mut self, tricks_beating: u8, runs: usize) {
        for lead in self.lead_results.values_mut() {
            lead.finish(tricks_beating, runs);
        }
    }
}

impl SimulationResult for LeadSimulationResult {
    fn report(&self) {
        let string = self.to_string();
        for (index, line) in string.lines().enumerate() {
            if index == 3 {
                println!("{}", line.green());
            } else {
                println!("{line}");
            }
        }
    }
}

impl Display for LeadSimulationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.deals_run.to_string().len();
        let width = if width < 4 { 4 } else { width };
        writeln!(f, "Simulated {} deals:", self.deals_run)?;
        writeln!(f, "{:^1$}", "Frequency of tricks taken", width * 14 + 16)?;
        writeln!(f,
            "Ld   Avg  %Set   {:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}{:>width$}",
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
        )?;
        let res_iter = self
            .lead_results
            .iter()
            .sorted_by(|a, b| b.1.set_percentage.total_cmp(&a.1.set_percentage))
            .collect_vec();
        let max_average_tricks_position = res_iter
            .iter()
            .position_max_by(|a, b| a.1.average_tricks.total_cmp(&b.1.average_tricks))
            .unwrap();
        let mut res_iter = res_iter.into_iter();
        let first = res_iter.next().unwrap();
        writeln!(
            f,
            "{}",
            format_args!(
                "{} {} {:>5.2}  [{:>width$} ]",
                first.0,
                if max_average_tricks_position != 0 {
                    format!("{:>5.2}", first.1.average_tricks)
                } else {
                    format!("*{:<4.2}", first.1.average_tricks)
                },
                first.1.set_percentage,
                first.1.number_of_tricks.iter().format("")
            )
        )?;
        for (index, lead) in res_iter.enumerate() {
            writeln!(
                f,
                "{} {} {:>5.2}  [{:>width$} ]",
                lead.0,
                if max_average_tricks_position != index + 1 {
                    format!("{:>5.2}", lead.1.average_tricks)
                } else {
                    format!("*{:<4.2}", lead.1.average_tricks)
                },
                lead.1.set_percentage,
                lead.1.number_of_tricks.iter().format("")
            )?;
        }
        Ok(())
    }
}

pub struct DealProducer<'simulation, D: Dealer> {
    number_repeats: usize,
    dealer: &'simulation D,
    deal: Deal,
    counter: usize,
}

impl<'simulation, D: Dealer> DealProducer<'simulation, D> {
    #[inline]
    #[must_use]
    fn new(dealer: &'simulation D, number_repeats: usize) -> Result<Self, SqueezerError> {
        let deal = dealer.deal()?;

        Ok(Self {
            number_repeats,
            dealer,
            deal,
            counter: number_repeats,
        })
    }
}

#[allow(clippy::expect_used, clippy::unwrap_in_result)]
impl<D: Dealer> Iterator for DealProducer<'_, D> {
    type Item = Deal;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter >= self.number_repeats {
            self.deal = self
                .dealer
                .deal()
                .expect("error while dealing inside the simulation: first deal was dealt successfully, then something broke");
            self.counter = 0;
        }
        self.counter += 1;
        Some(self.deal)
    }
}
#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn lead_simulation_ok() {
        let mut builder = DealerBuilder::new();
        let hand = Cards::from_str("AQT KQ732 432 43").unwrap();
        builder.predeal(Seat::South, hand).with_function(|cards| {
            let east = cards.east();
            let west = cards.west();
            east.hlen() + west.hlen() == 8
                && east.hcp() + west.hcp() >= 20
                && east.hcp() + west.hcp() <= 24
        });
        let dealer = builder.build().unwrap();
        let contract = Contract::from_str("2HE", Vulnerable::No).unwrap();
        let simulation = LeadSimulation {
            num_of_boards: 1000,
            dealer,
            contract,
        };
        let result = simulation.run().unwrap();
        result.report();
    }
}
