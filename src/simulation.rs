use crate::prelude::{Card, Cards, Contract, Dealer, DealerBuilder, SqueezerError};
use std::collections::HashMap;
use crate::AcceptFunction;
use dds::*;

pub trait SimulationResult {
    fn report(&self) {}
}

pub trait Simulation<T: SimulationResult> {
    fn run(&self) -> Result<T, SqueezerError>;
}

pub struct LeadSimulation<T: Dealer> {
    num_of_boards: usize,
    leader_hand: Cards,
    dealer: T,
    contract: Contract,
}

pub struct LeadSimulationResult<const N: usize> {
    leads: Cards,
    leads_res: Vec<Vec<u8>>,
    contract: Contract
}

impl<const N: usize> LeadSimulationResult<N> {
    #[inline]
    pub fn new(leads: Cards, contract: Contract) -> Self {
        Self {
            leads: leads.dedup(),
            leads_res: Vec::new(),
            contract,
        }
    }
}
impl<const N: usize> SimulationResult for LeadSimulationResult<N> {}

impl<T: Dealer, const N: usize> Simulation<LeadSimulationResult<N>> for LeadSimulation<T> {
    fn run(&self) -> Result<LeadSimulationResult<N>, SqueezerError> {
        let analyzer = DDSPlayAnalyzer::new();
        let mut suits = Vec::new();
        let mut ranks = Vec::new();
        let dedup = self.leader_hand.dedup();
        for _ in 0..(200 / dedup.len() - 1) {
            for card in dedup {
                let suit = card.suit();
                let rank = card.rank();
                suits.push(SuitSeq::try_from([suit as i32]).unwrap());
                ranks.push(RankSeq::try_from([rank]).unwrap());
            }
        }
        let mut plays = PlayTracesBin::from_sequences(suits, ranks).unwrap();
        let mut sim_result = LeadSimulationResult::new(self.leader_hand, self.contract);
        for _ in (0..self.num_of_boards).step_by(plays.len()) {
                let deal = self.dealer.deal()?;
                let deals = vec![&deal; plays.len()];
                let contracts = vec![&self.contract; plays.len()];
                let solved_plays = analyzer.analyze_all_plays(deals, contracts, &mut plays)?;
                // for (solved_play, lead) in solved_plays.into_iter().zip(dedup.into_iter().cycle()) {
                //     sim_result.push(solved_play.lead_result(), lead);
                // }
            
        }

        Ok(sim_result)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn lead_simulation_ok() {
        let dealer = DealerBuilder::new()
            .predeal(Seat::South, Hand::from_str("AQJT KQ32").unwrap())
            .with_function(|cards| {
                let east = cards.east();
                let west = cards.west();
                east.hlen() + west.hlen() >= 8
                    && east.hcp() + west.hcp() >= 20
                    && east.hcp() + west.hcp() <= 24
            })
            .build();
        let contract = Contract::from_str("2HE", Vulnerable::No).unwrap();
        let simulation = LeadSimulation {
            num_of_boards: 1000,
            leader_hand: Hand::from_str("AQJT KQ32").unwrap(),
            dealer,
            contract,
        };
        let result = simulation.run();
        result.report();
    }
}
