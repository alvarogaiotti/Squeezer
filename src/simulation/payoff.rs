// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use std::io::Write;

/// This module contains implementations for simulating and analyzing bridge game scenarios.
/// It includes structures for handling payoff matrices, bridge contracts, scoring functions, and utility functions.
/// Key components include the `Payoff` struct for managing a payoff matrix, `Contract` struct representing a bridge contract, and various scoring functions such as `imps` and `matchpoints`.
/// The file provides methods for calculating scores, creating contracts from strings, and reporting results based on simulated data.
use crate::prelude::*;
use dds::{doubledummy::DoubleDummySolver, solver::BridgeSolver, traits::ContractScorer};
use fmt::Display;
use itertools::Itertools;

pub trait DifferenceMaker {}
impl DifferenceMaker for Card {}
impl DifferenceMaker for Contract {}

/// Struct for running payoff simulation: is it better to risk a 3NT or better to play 4C in
/// terms of expected value?
///
/// # Example
///
/// ```
/// use squeezer::prelude::*;
/// use squeezer::prelude::imps;
///
/// let to_compare = vec![
///     Contract::from_str("3CN", Vulnerable::No).unwrap(),
///     Contract::from_str("3HS", Vulnerable::No).unwrap(),
///     Contract::from_str("3NN", Vulnerable::No).unwrap(),
///     ];
/// let simulation = PayoffSimulation::new(100, StandardDealer::new(), to_compare, imps);
/// let payoff = simulation.run().unwrap();
/// payoff.report();
/// ```
#[derive(Debug, Clone)]
pub struct PayoffSimulation<E: Fn(i32, i32) -> i32, D: Dealer, P: DifferenceMaker + Display> {
    no_of_runs: usize,
    dealer: D,
    to_compare: Vec<P>,
    diff: E,
}

impl<E: Fn(i32, i32) -> i32, D: Dealer, P: DifferenceMaker + Display> PayoffSimulation<E, D, P> {
    pub fn new(no_of_runs: usize, dealer: D, to_compare: Vec<P>, diff: E) -> Self {
        Self {
            no_of_runs,
            dealer,
            to_compare,
            diff,
        }
    }
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

struct PayoffAccumulator {
    entry: Contract,
    results: Vec<i32>,
}

struct RecurringDealer<D: Dealer> {
    dealer: D,
    repetitions: usize,
    counter: usize,
    deal: Deal,
}

impl<D: Dealer> Iterator for RecurringDealer<D> {
    type Item = Deal;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.repetitions {
            self.counter += 1;
            Some(self.deal)
        } else {
            self.deal = self.dealer.deal().unwrap();
            self.counter = 1;
            Some(self.deal)
        }
    }
}

impl<E: Fn(i32, i32) -> i32, D: Dealer> Simulation<Payoff<Contract>>
    for PayoffSimulation<E, D, Contract>
{
    fn run(&self) -> Result<Payoff<Contract>, SqueezerError> {
        let no_of_entries = self.to_compare.len();
        let mut payoff = Payoff::new(self.to_compare.clone());
        let mut entries: Vec<PayoffAccumulator> = self
            .to_compare
            .iter()
            .map(|elem| PayoffAccumulator {
                entry: *elem,
                results: Vec::with_capacity(self.no_of_runs),
            })
            .collect();
        let solver_array_len = 200 - (200 % no_of_entries);
        let contracts: Vec<Contract> = self
            .to_compare
            .iter()
            .copied()
            .cycle()
            .take(solver_array_len)
            .collect();
        let solver = DoubleDummySolver::new();

        let mut deal_buffer = Vec::with_capacity(solver_array_len);
        (0..self.no_of_runs)
            .into_iter()
            .chunks(solver_array_len / no_of_entries)
            .into_iter()
            .for_each(|chunk| {
                let len = chunk.count();
                for _ in 0..len {
                    for _ in 0..no_of_entries {
                        let deal = self.dealer.deal().unwrap();
                        deal_buffer.push(deal)
                    }
                }
                let solver_results = solver
                    .dd_tricks_parallel(solver_array_len as i32, &deal_buffer, &contracts)
                    .unwrap();
                let scores = solver_results
                    .into_iter()
                    .zip(contracts.iter())
                    .map(|(n_tricks, contract)| contract.score(n_tricks));
                for (index, score) in scores.enumerate() {
                    entries[index % no_of_entries].results.push(score)
                }
                deal_buffer.clear();
            });
        for (starting_entry_index, entry) in entries.iter().enumerate() {
            if starting_entry_index == no_of_entries - 1 {
                break;
            }
            for next_entry in &entries[starting_entry_index + 1..] {
                let diffs: Vec<i32> = entry
                    .results
                    .iter()
                    .zip(next_entry.results.iter())
                    .map(|(first_res, second_res)| (self.diff)(*first_res, *second_res))
                    .collect();
                payoff
                    .results
                    .push(std_deviation_and_mean(&diffs).or(Some((0.0, 0.0))).unwrap());
            }
        }
        Ok(payoff)
    }
}

pub struct PayoffEntry<P>
where
    P: fmt::Display + DifferenceMaker,
{
    difference_makers: (P, P),
    results: Option<(f32, f32)>,
}

impl<P> PayoffEntry<P>
where
    P: fmt::Display + DifferenceMaker,
{
    pub fn new(difference_maker1: P, difference_maker2: P, results: Option<(f32, f32)>) -> Self {
        Self {
            difference_makers: (difference_maker1, difference_maker2),
            results,
        }
    }
    pub fn set_result(&mut self, results: Option<(f32, f32)>) {
        self.results = results;
    }
}

/// Struct that rapresents a payoff matrix which returns performances of contracs based
/// on scoring. Some sort of expected value of the contracts.
pub struct Payoff<P>
where
    P: fmt::Display + DifferenceMaker,
{
    entries: Vec<P>,
    results: Vec<(f32, f32)>,
}

impl<P> Payoff<P>
where
    P: fmt::Display + DifferenceMaker,
{
    #[must_use]
    pub fn new(entries: Vec<P>) -> Self {
        Self {
            entries,
            results: Vec::new(),
        }
    }
}

impl<D: Display + DifferenceMaker> SimulationResult for Payoff<D> {
    /// This function generates a report displaying the Payoff matrix in the terminal.
    /// It compares the expected value of each option with respect to the others.
    #[allow(clippy::missing_panics_doc, clippy::cast_precision_loss)]
    fn report(&self) {
        let mut buffer = Vec::with_capacity(500);
        let mut stderr_buffer = Vec::with_capacity(self.entries.len());

        write!(&mut buffer, "\t{:.7}", self.entries.iter().format("\t")).unwrap();
        for (index, entry) in self.entries.iter().enumerate() {
            write!(&mut buffer, "\n{entry:.7}").unwrap();
            for (second_index, _entry) in self.entries.iter().enumerate() {
                if second_index == index {
                    write!(&mut buffer, "\t-").unwrap();
                } else {
                    let (mean, stderr) = {
                        let data = *self.results.get(index + second_index - 1).unwrap();
                        if index < second_index {
                            data
                        } else {
                            (-data.0, data.1)
                        }
                    };
                    stderr_buffer.push(stderr);
                    write!(&mut buffer, "\t{mean:.2}").unwrap();
                    //if mean > stderr {
                    //    output.green()
                    //} else if mean < &-stderr {
                    //    output.red()
                    //} else {
                    //    output.white()
                    //}
                }
            }
            write!(&mut buffer, "\n\t{:.2}", stderr_buffer.iter().format("\t")).unwrap();
            stderr_buffer.clear();
        }
        println!("{}", String::from_utf8(buffer).unwrap());
    }
}

#[allow(clippy::cast_precision_loss)]
fn mean(data: &[i32]) -> Option<f32> {
    let sum = data.iter().sum::<i32>() as f32;
    let count = data.len();
    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

#[allow(clippy::cast_precision_loss)]
fn std_deviation_and_mean(data: &[i32]) -> Option<(f32, f32)> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f32);
                    diff * diff
                })
                .sum::<f32>()
                / count as f32;
            Some((variance.sqrt(), data_mean))
        }
        _ => None,
    }
}

#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
fn bisect_right(value: i32, lista: &[i32]) -> i32 {
    for (i, &x) in lista.iter().enumerate() {
        if x < value {
            continue;
        }
        return i as i32;
    }
    lista.len() as i32
}
#[must_use]
pub fn imps(my: i32, other: i32) -> i32 {
    let imp_table: [i32; 24] = [
        15, 45, 85, 125, 165, 215, 265, 315, 365, 425, 495, 595, 745, 895, 1095, 1295, 1495, 1745,
        1995, 2245, 2495, 2995, 3495, 3995,
    ];
    bisect_right((my - other).abs(), &imp_table) * (if my > other { 1 } else { -1 })
}
#[must_use]
pub fn matchpoints(my: i32, other: i32) -> i32 {
    i32::from(my > other) - i32::from(my < other)
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use dds::traits::ContractScorer;
    #[test]
    fn payoff_report_test() {
        let contracts = vec![
            Contract::from_str("3NN", Vulnerable::No).unwrap(),
            Contract::from_str("3HN", Vulnerable::No).unwrap(),
            Contract::from_str("3DN", Vulnerable::No).unwrap(),
            Contract::from_str("4HN", Vulnerable::No).unwrap(),
        ];
        let sim = PayoffSimulation::new(100, StandardDealer::new(), contracts, imps);
        let matrix = sim.run().unwrap();
        matrix.report();
    }

    #[test]
    #[should_panic(expected = "Wrong contract level")]
    fn create_contract_wrong_level_test() {
        let _contract = Contract::from_str("8CS", Vulnerable::No).unwrap();
    }
    #[test]
    fn contract_computes_correct_scores_test() {
        let contract_c = Contract::from_str("6CN", Vulnerable::No).unwrap();
        let contract_d = Contract::from_str("5DNX", Vulnerable::Yes).unwrap();
        let contract_s = Contract::from_str("4SN", Vulnerable::No).unwrap();
        let contract_h = Contract::from_str("3HN", Vulnerable::No).unwrap();
        let contract_n = Contract::from_str("3NN", Vulnerable::No).unwrap();
        assert_eq!(400_i32, contract_n.score(9));
        assert_eq!(140_i32, contract_h.score(9));
        assert_eq!(420_i32, contract_s.score(10));
        assert_eq!(750_i32, contract_d.score(11));
        assert_eq!(920_i32, contract_c.score(12));
        assert_eq!(-200, contract_d.score(10));
    }
}
