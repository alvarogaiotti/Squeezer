use std::{array, fmt::Display};

use crate::{
    imps,
    prelude::{Card, Cards, Contract, Dealer, SqueezerError},
    Deal, Hand,
};
use dds::{
    ContractScorer, DDSPlayAnalyzer, PlayAnalyzer, PlayTracesBin, RankSeq, SolvedPlays, SuitSeq,
};
use itertools::*;

pub trait SimulationResult
where
    Self: Sized,
{
    fn report(self) {}
}

pub trait Simulation<T: SimulationResult> {
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
    pub fn new(leads: Cards, contract: Contract, deals_run: usize) -> Self {
        let lead_results = leads
            .dedup()
            .into_iter()
            .map(|card| LeadCard::new(card))
            .collect();
        Self {
            lead_results,
            contract,
            deals_run,
        }
    }

    #[inline]
    /// # Panics
    ///
    /// Panics when DDS returns a negative result for the tricks
    pub fn add_results(&mut self, results: dds::SolvedBoards) {
        for trick in results {}
    }
}

impl SimulationResult for LeadSimulationResult {
    fn report(self) {
        let tricks = self.lead_results;
        let leads_results = self
            .cards
            .into_iter()
            .zip(tricks)
            .map(|(card, results_array)| {
                let mut sum = 0;
                let mut times_beaten = 0;
                let setting_tricks = 8 - self.contract.level() as usize;

                for (ntricks, times_made) in results_array.iter().enumerate() {
                    sum += ntricks * *times_made;
                    if setting_tricks > ntricks {
                        continue;
                    }
                    times_beaten += times_made;
                }
                LeadCard {
                    card,
                    number_of_tricks: results_array,
                    average_tricks: sum as f32 / self.deals_run as f32,
                    set_percentage: times_beaten as f32 / self.deals_run as f32,
                }
            });
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
        let dedup = self.leader_hand.dedup();
        let number_of_leads = dedup.len() as usize;
        let number_of_deals_per_analysis = 200 / number_of_leads;
        let length_of_play_traces = number_of_deals_per_analysis * number_of_leads;

        // Create the Seq vectors to feed to PlayTraces
        let (suits, ranks): (Vec<SuitSeq>, Vec<RankSeq>) = dedup
            .into_iter()
            .map(|card| {
                (
                    [card.suit() as i32].try_into().unwrap(),
                    [card.rank()].try_into().unwrap(),
                )
            })
            .cycle()
            .take(length_of_play_traces)
            .unzip();

        let mut plays = PlayTracesBin::from_sequences(suits, ranks)?;
        let mut deal_producer = DealProducer::new(&self.dealer, number_of_leads);
        let mut sim_result =
            LeadSimulationResult::new(self.leader_hand, self.contract, self.num_of_boards);
        let contracts = vec![self.contract; plays.len()];

        // For the number of boads, stepped by the number of deals we use per analysis
        for _ in (0..self.num_of_boards).step_by(200) {
            // We take from the deal producer the number we need
            let deals = array::from_fn(|_| self.dealer.deal().unwrap());
            let mut boards = dds::Boards::new(
                200,
                &deals,
                &[self.contract; 200],
                &[dds::Target::default(); 200],
                &[dds::Solutions::default(); 200],
                &[dds::Mode::default(); 200],
            );
            let mut solvedb = dds::SolvedBoards::new(200);
            unsafe {
                dds::SolveAllBoardsBin(
                    &mut boards as *mut dds::Boards,
                    &mut solvedb as *mut dds::SolvedBoards,
                )
            };
            sim_result.add_results(solvedb);
        }

        // We do the same for the remaining deals
        let rest = self.num_of_boards % number_of_deals_per_analysis;
        let (suits_rest, rank_rest) = dedup
            .into_iter()
            .map(|card| {
                (
                    [card.suit() as i32].try_into().unwrap(),
                    [card.rank()].try_into().unwrap(),
                )
            })
            .cycle()
            .take(rest)
            .unzip();

        let mut rest_playtrace = PlayTracesBin::from_sequences(suits_rest, rank_rest)?;
        let deals = (&mut deal_producer).take(rest).collect();
        let contracts_rest = vec![self.contract; rest];
        let solved_plays =
            analyzer.analyze_all_plays(&deals, &contracts_rest, &mut rest_playtrace)?;
        sim_result.add_results(solved_plays);

        Ok(sim_result)
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
        builder.predeal(Seat::South, hand).with_function(|cards| {
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
            leader_hand: hand,
            dealer,
            contract,
        };
        let result = simulation.run().unwrap();
        result.report();
    }
}
