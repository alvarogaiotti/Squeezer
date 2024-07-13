use super::{Simulation, SimulationResult};
use colored::Colorize;
use std::{array, collections::HashMap, fmt::Display};

use crate::{
    prelude::{Card, Contract, Dealer, SqueezerError},
    Deal, Suit,
};
use dds::MAXNOOFBOARDS;
use itertools::Itertools;

pub struct LeadSimulation<T: Dealer> {
    num_of_boards: usize,
    dealer: T,
    contract: Contract,
}

#[derive(Debug)]
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
    fn update(&mut self, tricks_beating: u8, runs: usize) {
        self.average_tricks = self
            .number_of_tricks
            .iter()
            .enumerate()
            .map(|(tricks, times)| tricks * times)
            .sum::<usize>() as f32
            / runs as f32;
        self.set_percentage = (self.number_of_tricks[tricks_beating as usize..]
            .iter()
            .sum::<usize>() as f32
            / runs as f32)
            * 100.0;
    }
}

pub struct LeadSimulationResult {
    lead_results: HashMap<Card, LeadCard>,
    deals_run: usize,
    contract: Contract,
}

impl LeadSimulationResult {
    #[inline]
    #[must_use]
    pub fn new(contract: Contract, deals_run: usize) -> Self {
        Self {
            lead_results: HashMap::with_capacity(8),
            contract,
            deals_run,
        }
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[inline]
    /// # Panics
    ///
    /// Panics when DDS returns a negative result for the tricks
    pub fn add_results(&mut self, results: dds::SolvedBoards) {
        let mut results_iter = results.into_iter();
        if let Some(solved) = results_iter.next() {
            for index in 0..solved.cards as usize {
                let rank = solved.rank[index];
                let suit = solved.suit[index];
                let card = Card::new(Suit::try_from(suit).unwrap(), rank as u8);
                self.lead_results
                    .entry(card)
                    .and_modify(|lead_card| {
                        lead_card.number_of_tricks[solved.score[index] as usize] += 1;
                    })
                    .or_insert_with(|| {
                        let mut lead_card = LeadCard::new(card);
                        let tricks = solved.score[index] as usize;
                        lead_card.number_of_tricks[tricks] += 1;
                        lead_card
                    });
            }
        }
        for solved in results_iter {
            for index in 0..solved.cards as usize {
                let rank = solved.rank[index];
                let suit = solved.suit[index];
                let card = Card::new(Suit::try_from(suit).unwrap(), rank as u8);
                self.lead_results
                    .entry(card)
                    .and_modify(|lead_card| {
                        lead_card.number_of_tricks[solved.score[index] as usize] += 1;
                    })
                    .or_insert_with(|| {
                        let mut lead_card = LeadCard::new(card);
                        lead_card.number_of_tricks[solved.score[index] as usize] += 1;
                        lead_card
                    });
            }
        }
    }

    fn update(&mut self, tricks_beating: u8, runs: usize) {
        for lead in self.lead_results.values_mut() {
            lead.update(tricks_beating, runs);
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
        let mut res_iter = self
            .lead_results
            .iter()
            .sorted_by(|a, b| b.1.set_percentage.total_cmp(&a.1.set_percentage));
        let first = res_iter.next().unwrap();
        writeln!(
            f,
            "{}",
            format_args!(
                "{}  {:>4.2} {:>5.2}  [{:>width$} ]",
                first.0,
                first.1.average_tricks,
                first.1.set_percentage,
                first.1.number_of_tricks.iter().format("")
            )
        )?;
        for lead in res_iter {
            writeln!(
                f,
                "{}  {:>4.2} {:>5.2}  [{:>width$} ]",
                lead.0,
                lead.1.average_tricks,
                lead.1.set_percentage,
                lead.1.number_of_tricks.iter().format("")
            )?;
        }
        Ok(())
    }
}

impl<T: Dealer> Simulation<LeadSimulationResult> for LeadSimulation<T> {
    #[allow(clippy::integer_division)]
    fn run(&self) -> Result<LeadSimulationResult, SqueezerError> {
        let mut sim_result = LeadSimulationResult::new(self.contract, self.num_of_boards);

        let contracts = [self.contract; MAXNOOFBOARDS];
        let targets = [dds::Target::MaxTricks; MAXNOOFBOARDS];
        let solutions = [dds::Solutions::AllLegal; MAXNOOFBOARDS];
        let modes = [dds::Mode::Auto; MAXNOOFBOARDS];

        // For the number of boads, stepped by the number of deals we use per analysis
        for _ in (0..self.num_of_boards).step_by(MAXNOOFBOARDS) {
            // We take from the deal producer the number we need
            let solvedb =
                self.solve_boards(MAXNOOFBOARDS, &contracts, &targets, &solutions, &modes)?;
            sim_result.add_results(solvedb);
        }

        let num = self.num_of_boards % MAXNOOFBOARDS;
        let solvedb = self.solve_boards(num, &contracts, &targets, &solutions, &modes)?;
        sim_result.add_results(solvedb);

        sim_result.update(8 - self.contract.level(), self.num_of_boards);
        Ok(sim_result)
    }
}

impl<T: Dealer> LeadSimulation<T> {
    fn solve_boards(
        &self,
        num: usize,
        contracts: &[Contract; MAXNOOFBOARDS],
        targets: &[dds::Target; MAXNOOFBOARDS],
        solutions: &[dds::Solutions; MAXNOOFBOARDS],
        modes: &[dds::Mode; MAXNOOFBOARDS],
    ) -> Result<dds::SolvedBoards, SqueezerError> {
        // We take from the deal producer the number we need
        let deals = array::from_fn(|_| self.dealer.deal().unwrap());
        let mut boards = dds::Boards::new(
            i32::try_from(num).unwrap(),
            &deals,
            contracts,
            targets,
            solutions,
            modes,
        );
        let mut solvedb = dds::SolvedBoards::new(i32::try_from(num).unwrap());
        // TODO: Implement the right interface on `DoubleDummySolver` for this free
        // function.
        if (unsafe {
            dds::SolveAllBoardsBin(
                std::ptr::from_mut::<dds::Boards>(&mut boards),
                std::ptr::from_mut::<dds::SolvedBoards>(&mut solvedb),
            )
        } < 0)
        {
            Err(SqueezerError::DDSError(21.into()))
        } else {
            Ok(solvedb)
        }
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
    fn new(dealer: &'simulation D, number_repeats: usize) -> Self {
        let deal = dealer
            .deal()
            .expect("unable to deal the first deal inside the simulation.");
        Self {
            number_repeats,
            dealer,
            deal,
            counter: number_repeats,
        }
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
                .expect("error while dealing inside the simulation");
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
        builder
            .predeal(Seat::South, hand.try_into().unwrap())
            .with_function(|cards| {
                let east = cards.east();
                let west = cards.west();
                east.hlen() + west.hlen() == 8
                    && east.hcp() + west.hcp() >= 20
                    && east.hcp() + west.hcp() <= 24
            });
        let dealer = builder.build().unwrap();
        let contract = Contract::from_str("2HE", Vulnerable::No).unwrap();
        let simulation = LeadSimulation {
            num_of_boards: 100,
            dealer,
            contract,
        };
        let result = simulation.run().unwrap();
        result.report();
    }
}
