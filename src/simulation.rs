use std::{array, fmt::Display};

use crate::{
    prelude::{Card, Contract, Dealer, SqueezerError},
    Deal,
};
use dds::MAXNOOFBOARDS;
use itertools::Itertools;

pub trait SimulationResult
where
    Self: Sized,
{
    fn report(self) {}
}

pub trait Simulation<T: SimulationResult> {
    /// # Errors
    /// - `DDSError`
    fn run(&self) -> Result<T, SqueezerError>;
}

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
}

pub struct LeadSimulationResult {
    lead_results: Vec<LeadCard>,
    deals_run: usize,
    contract: Contract,
}

impl LeadSimulationResult {
    #[inline]
    #[must_use]
    pub fn new(contract: Contract, deals_run: usize) -> Self {
        Self {
            lead_results: Vec::with_capacity(8),
            contract,
            deals_run,
        }
    }

    #[inline]
    /// # Panics
    ///
    /// Panics when DDS returns a negative result for the tricks
    pub fn add_results(&mut self, _results: dds::SolvedBoards) {}
}

impl SimulationResult for LeadSimulationResult {
    fn report(self) {
        todo!()
    }
}

impl Display for LeadSimulationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let single_width = 6;
        write!(
            f,
            "Double Dummy analysis completed for {} deals\n\n",
            self.deals_run
        )?;
        write!(f, "Ld   Avg   %Set    ")?;
        writeln!(
            f,
            "{}",
            (0u8..14u8).format_with(" ", |elem, formatter| formatter(&format_args!(
                "{elem:>single_width$}",
            )))
        )?;
        todo!()
    }
}

impl<T: Dealer> Simulation<LeadSimulationResult> for LeadSimulation<T> {
    #[allow(clippy::integer_division)]
    fn run(&self) -> Result<LeadSimulationResult, SqueezerError> {
        let mut sim_result = LeadSimulationResult::new(self.contract, self.num_of_boards);

        let contracts = [self.contract; MAXNOOFBOARDS];
        let targets = [dds::Target::default(); MAXNOOFBOARDS];
        let solutions = [dds::Solutions::default(); MAXNOOFBOARDS];
        let modes = [dds::Mode::default(); MAXNOOFBOARDS];

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
        let hand = Cards::from_str("AQT KQ732").unwrap();
        builder
            .predeal(Seat::South, hand.try_into().unwrap())
            .with_function(|cards| {
                let east = cards.east();
                let west = cards.west();
                east.hlen() + west.hlen() >= 8
                    && east.hcp() + west.hcp() >= 20
                    && east.hcp() + west.hcp() <= 24
            });
        let dealer = builder.build().unwrap();
        let contract = Contract::from_str("2HE", Vulnerable::No).unwrap();
        let simulation = LeadSimulation {
            num_of_boards: 10,
            dealer,
            contract,
        };
        //let result = simulation.run().unwrap();
        //result.report();
    }
}
