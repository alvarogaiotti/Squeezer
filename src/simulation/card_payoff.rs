// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;
use dds::{
    deal::{AsDDSDeal, DdsDeal, DdsDealBuilder},
    doubledummy::DoubleDummySolver,
    solver::BridgeSolver,
};
use itertools::Itertools;
use std::io::Write;

/// Struct for running payoff simulation: is it better to risk a 3NT or better to play 4C in
/// terms of expected value?
///
/// # Example
///
/// ```
/// # use squeezer::prelude::*;
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>>{
///
/// let to_compare = vec![
///     Contract::from_str("3CN")?,
///     Contract::from_str("3HS")?,
///     Contract::from_str("3NN")?,
///     ];
/// let simulation = PayoffSimulation::new(100, StandardDealer::new(), to_compare, imps);
/// let payoff = simulation.run()?;
/// payoff.report();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CardPayoffSimulation<E: Fn(i32, i32) -> i32, D: Dealer> {
    no_of_runs: usize,
    dealer: D,
    to_compare: Vec<Card>,
    contract: Contract,
    diff: E,
    seat: Seat,
    play_trace: Cards,
}

impl<E: Fn(i32, i32) -> i32, D: Dealer> CardPayoffSimulation<E, D> {
    /// Creates a new [`PayoffSimulation<E, D, P>`].
    /// # Errors
    /// Errors if the cards to compare are not included in the hand that has to play
    pub fn new(
        no_of_runs: usize,
        dealer: D,
        to_compare: Vec<Card>,
        contract: Contract,
        diff: E,
        seat: Seat,
        play_trace: Cards,
    ) -> Result<Self, DealerError> {
        let hands = dealer.predealt();
        for card in &to_compare {
            if let Some(hand) = hands[seat as usize]
                && !hand.contains(*card)
            {
                return Err(DealerError::new("{card} is not contained in {seat} hand"));
            }
        }
        Ok(Self {
            no_of_runs,
            dealer,
            to_compare,
            contract,
            diff,
            seat,
            play_trace,
        })
    }
}

impl<E, D> Simulation<Payoff<Card>> for CardPayoffSimulation<E, D>
where
    E: Fn(i32, i32) -> i32,
    D: Dealer,
{
    /// Runs the simulation, providing you with the results in the form
    /// a [`Payoff<Contract>`], which compares the EV of the contract pair-wise.
    #[inline]
    fn run(self) -> Result<Payoff<Card>, SqueezerError> {
        let no_of_entries = self.to_compare.len();
        let mut payoff = Payoff::new(self.to_compare.clone());
        let solver_array_len = 200 - (200 % no_of_entries);
        let mut entries = self
            .to_compare
            .iter()
            .map(|&elem| PayoffAccumulator {
                entry: elem,
                results: Vec::with_capacity(self.no_of_runs),
            })
            .collect_vec();
        let solver = DoubleDummySolver::new();

        let mut deal_buffer = Vec::with_capacity(solver_array_len);
        // Take the total number of runs we want to simulate, and divide it into chunks.
        // Eg. 1000 runs, 4 contracts = 1000 split into 20 chunks of 50 elements
        // For the 20 chunks of length 50...
        let already_played: Cards = self.play_trace;
        let number_of_deals = solver_array_len / no_of_entries;
        let hand = DdsDeal::new();
        deal_buffer.resize(number_of_deals * no_of_entries, hand);
        for start in (0..self.no_of_runs).step_by(number_of_deals) {
            let chunk_length = (self.no_of_runs - start).min(number_of_deals);
            // Chunk is 50 elements
            for i in 0..chunk_length {
                // Generate a deal
                let deal = self.dealer.deal()?;
                // push it 4 times
                for (j, entry) in entries.iter().enumerate() {
                    let mut hands = deal.hands();
                    // Consistency for this loop is not guaranteed
                    for hand in &mut hands {
                        hand.set_cards(hand.as_cards() - already_played.insert(entry.entry));
                    }
                    let deal = DdsDealBuilder::new()
                        .remain_cards(hands.as_dds_deal())
                        .current_trick(&[Some(entry.entry), None, None])
                        .contract(&self.contract)
                        .first((self.seat).into())
                        .build()
                        .unwrap();
                    deal_buffer[i * no_of_entries + j] = deal;
                }
                // We end up with a 200 element Vec laid as so:
                //  _______________________________________________
                // |D.1|D.1|D.1|D.1|D.2|D.2|D.2|D.2|D.3|D.3|D.3|D.3| and so on
                // ¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯
            }
            // We run the solver with the above Vec and with a set of contract as so:
            //  _______________________________
            // |C.1|C.2|C.3|C.4|C.1|C.2|C.3|C.4| and so on...
            // ¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯
            let total_len = chunk_length * no_of_entries;
            #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
            let solver_results =
                solver.dd_tricks_parallel(total_len as i32, &deal_buffer[..total_len])?;
            for (index, tricks) in solver_results.into_iter().enumerate() {
                let entry = &mut entries[index % no_of_entries];
                let score = self.contract.score(tricks);
                entry.results.push(score);
            }
        }
        for (starting_entry_index, entry) in entries.iter().enumerate() {
            for (next_entry_index, next_entry) in entries.iter().enumerate() {
                if starting_entry_index == next_entry_index {
                    payoff.results.push((0.0, 0.0));
                    continue;
                }
                let mut mean = 0f64;
                let mut m2 = 0f64;
                let mut count = 0usize;

                #[allow(clippy::cast_precision_loss)]
                for (first_result, second_result) in
                    entry.results.iter().zip(next_entry.results.iter())
                {
                    let diff = f64::from((self.diff)(*first_result, *second_result));
                    count += 1;
                    let delta = diff - mean;
                    mean += delta / (count as f64);
                    let delta2 = diff - mean;
                    m2 += delta * delta2;
                }

                #[allow(clippy::cast_precision_loss)]
                if count > 1 {
                    let variance = m2 / (count as f64);
                    payoff.results.push((mean, variance.sqrt()));
                } else {
                    payoff.results.push((0.0, 0.0));
                }
            }
        }
        Ok(payoff)
    }
}

/// Struct that rapresents a payoff matrix which returns performances of contracs based
/// on scoring. Some sort of expected value of the contracts.
pub struct CardPayoff {
    entries: Cards,
    results: Vec<(f64, f64)>,
}

impl CardPayoff {
    #[must_use]
    pub fn new(entries: Cards) -> Self {
        let size = entries.len() * entries.len();
        Self {
            entries,
            results: Vec::with_capacity(size as usize),
        }
    }
}

impl SimulationResult for CardPayoff {
    /// This function generates a report displaying the Payoff matrix in the terminal.
    /// It compares the expected value of each option with respect to the others.
    #[allow(clippy::missing_panics_doc, clippy::cast_precision_loss)]
    fn report(&self) {
        let mut buffer = Vec::with_capacity(500);
        let mut stderr_buffer = Vec::with_capacity(self.entries.len() as usize);
        let entries_length = self.entries.len();

        write!(
            &mut buffer,
            "\t {}",
            self.entries.into_iter().format("\t  ")
        )
        .unwrap();
        for (index, entry) in self.entries.into_iter().enumerate() {
            write!(&mut buffer, "\n{entry:}").unwrap();
            for (second_index, _second_entry) in self.entries.into_iter().enumerate() {
                if second_index == index {
                    write!(&mut buffer, "\t{:^5}", "-").unwrap();
                    stderr_buffer.push(0.0);
                } else {
                    let (mean, stderr) = {
                        *self
                            .results
                            .get(index * entries_length as usize + second_index)
                            .unwrap()
                        //if index < second_index {
                        //    data
                        //} else {
                        //    (-data.0, data.1)
                        //}
                    };
                    stderr_buffer.push(stderr);
                    write!(&mut buffer, "\t{mean:>5.2}").unwrap();
                    //if mean > stderr {
                    //    output.green()
                    //} else if mean < &-stderr {
                    //    output.red()
                    //} else {
                    //    output.white()
                    //}
                }
            }
            write!(
                &mut buffer,
                "\n\t{:>5.2}\n",
                stderr_buffer.iter().format("\t")
            )
            .unwrap();
            stderr_buffer.clear();
        }
        println!("{}", String::from_utf8(buffer).unwrap());
    }
}

#[cfg(test)]
mod test {
    use crate::{
        linparser::PlaySequence, prelude::*, simulation::card_payoff::CardPayoffSimulation,
    };
    #[test]
    fn cards_payoff_report_test() {
        let contract = Contract::from_str_and_vuln("3NN", Vulnerable::No).unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, Cards::from_str("AKQ AKQ K 9").unwrap())
            .unwrap()
            .predeal(Seat::East, Cards::from_str("DA6543CAKQJ").unwrap())
            .unwrap()
            .predeal(Seat::West, Cards::from_str("D98").unwrap())
            .unwrap()
            .predeal(Seat::South, Cards::from_str("D2").unwrap())
            .unwrap()
            .with_hand_descriptor(
                Seat::South,
                HandDescriptor::new(vec![HandType::new(
                    Shape::new_from_pattern("3433").unwrap(),
                    HcpRange::new(5, 15),
                )]),
            )
            .build()
            .unwrap();
        let play_trace = PlaySequence::new(vec![Card::D3, Card::D2, Card::D8, Card::DK]).into();
        let sim = CardPayoffSimulation::new(
            200,
            dealer,
            vec![Card::SA, Card::HA, Card::C9],
            contract,
            imps,
            Seat::North,
            play_trace,
        )
        .unwrap();
        let matrix = sim.run().unwrap();
        matrix.report();
    }

    #[should_panic(expected = "Wrong contract level")]
    fn create_contract_wrong_level_test() {
        let _contract = Contract::from_str_and_vuln("8CS", Vulnerable::No).unwrap();
    }
    fn contract_computes_correct_scores_test() {
        let contract_c = Contract::from_str_and_vuln("6CN", Vulnerable::No).unwrap();
        let contract_d = Contract::from_str_and_vuln("5DNX", Vulnerable::Yes).unwrap();
        let contract_s = Contract::from_str_and_vuln("4SN", Vulnerable::No).unwrap();
        let contract_h = Contract::from_str_and_vuln("3HN", Vulnerable::No).unwrap();
        let contract_n = Contract::from_str_and_vuln("3NN", Vulnerable::No).unwrap();
        assert_eq!(400_i32, contract_n.score(9));
        assert_eq!(140_i32, contract_h.score(9));
        assert_eq!(420_i32, contract_s.score(10));
        assert_eq!(750_i32, contract_d.score(11));
        assert_eq!(920_i32, contract_c.score(12));
        assert_eq!(-200, contract_d.score(10));
    }
    fn other_payoff_test() {
        let contracts = vec![
            Contract::from_str("4SN").unwrap(),
            Contract::from_str("3NN").unwrap(),
            Contract::from_str("6SN").unwrap(),
        ];
        let ht = HandTypeBuilder::new()
            .with_range(14, 14)
            .add_shape("3(433)")
            .unwrap()
            .add_shape("3(442)")
            .unwrap()
            .build();
        let dealer = DealerBuilder::new()
            .predeal(Seat::South, Cards::from_str("AKQJ2 52 AT83 Q2").unwrap())
            .unwrap()
            .with_hand_descriptor(Seat::North, HandDescriptor::new(vec![ht]))
            .build()
            .unwrap();
        let sim = PayoffSimulation::new(500, dealer, contracts, imps);
        let matrix = sim.run().unwrap();
        matrix.report();
    }
}
