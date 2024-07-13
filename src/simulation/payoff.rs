/// This file contains implementations for simulating and analyzing bridge game scenarios.
/// It includes structures for handling payoff matrices, bridge contracts, scoring functions, and utility functions.
/// Key components include the `Payoff` struct for managing a payoff matrix, `Contract` struct representing a bridge contract, and various scoring functions such as `imps` and `matchpoints`.
/// The file provides methods for calculating scores, creating contracts from strings, and reporting results based on simulated data.
use crate::prelude::*;
use fmt::Display;

pub trait Simulable: sealed::PrivateSimulable {}
mod sealed {
    pub trait PrivateSimulable: std::fmt::Display {}
}

///Struct that rapresents a payoff matrix which returns performances of contracs based
///on scoring. Some sort of expected value of the contracts.
pub struct Payoff<T, D>
where
    T: Fn(i32, i32) -> i32,
    D: fmt::Display,
{
    entries: Vec<D>,
    table: Vec<Vec<Vec<i32>>>,
    diff: T,
}

impl<T, D> Payoff<T, D>
where
    T: Fn(i32, i32) -> i32,
    D: fmt::Display,
{
    pub fn new(entries: Vec<D>, diff: T) -> Self {
        let mut table = Vec::with_capacity(entries.len());
        for i in 0..entries.len() {
            table.push(Vec::with_capacity(entries.len()));
            for _ in 0..entries.len() {
                table[i].push(Vec::new());
            }
        }
        Self {
            entries,
            table,
            diff,
        }
    }

    /// Adds data to the payoff matrix based on raw scores and updates the matrix entries.
    ///
    /// # Arguments
    ///
    /// * `raw_scores` - A reference to a hashmap containing raw scores for each contract entry.
    #[allow(clippy::missing_panics_doc)]
    pub fn add_data(&mut self, raw_scores: &HashMap<&str, i32>) {
        let diff = &self.diff;
        for (i, ei) in self.entries.iter().enumerate() {
            for (j, ej) in self.entries.iter().enumerate() {
                self.table[i][j].push(diff(
                    *raw_scores.get(&ei.to_string() as &str).unwrap(),
                    *raw_scores.get(&ej.to_string() as &str).unwrap(),
                ));
            }
        }
    }

    /// This function generates a report displaying the Payoff matrix in the terminal.
    /// It compares the expected value of each option with respect to the others.
    #[allow(clippy::missing_panics_doc, clippy::cast_precision_loss)]
    pub fn report(&self) {
        let mut means_stderrs: Vec<Vec<(f32, f32)>> = Vec::new();
        for (i, line) in self.table.iter().enumerate() {
            means_stderrs.push(Vec::new());
            for score in line {
                means_stderrs[i].push((
                    mean(score).unwrap(),
                    std_deviation(score).unwrap() / (score.len() as f32).sqrt(),
                ));
            }
        }
        println!("\t{:.7}", self.entries.iter().format("\t"));
        for (i, (entry, line)) in self.entries.iter().zip(means_stderrs.iter()).enumerate() {
            print!("\n{entry:.7}");
            for (j, (mean, stderr)) in line.iter().enumerate() {
                print!("\t{}", {
                    if i == j {
                        "-".blue()
                    } else {
                        let output = format!("{mean:.2}");
                        if mean > stderr {
                            output.green()
                        } else if mean < &-stderr {
                            output.red()
                        } else {
                            output.white()
                        }
                    }
                });
            }
            print!("\n       ");
            for (j, (_mean, stderr)) in line.iter().enumerate() {
                print!("\t{}", {
                    let output = format!("{stderr:.2}");
                    if i == j {
                        String::new()
                    } else {
                        output
                    }
                });
            }
            println!();
        }
    }
}

impl<D: Display, T: Fn(i32, i32) -> i32> SimulationResult for Payoff<T, D> {}

pub struct PayoffSimulation {}

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
fn std_deviation(data: &[i32]) -> Option<f32> {
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
            Some(variance.sqrt())
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
    use dds::ContractScorer;
    #[test]
    fn payoff_report_test() {
        let contratto1 = Contract::from_str("3CN", Vulnerable::No).unwrap();
        let contratto2 = Contract::from_str("3DN", Vulnerable::No).unwrap();
        let contratto3 = Contract::from_str("3NN", Vulnerable::No).unwrap();
        let contracts = vec![
            Contract::from_str("3CN", Vulnerable::No).unwrap(),
            Contract::from_str("3DN", Vulnerable::No).unwrap(),
            Contract::from_str("3NN", Vulnerable::No).unwrap(),
        ];
        let mut matrix = Payoff::new(contracts, imps);
        let mut data = HashMap::new();
        let contratto1str = contratto1.to_string();
        let contratto2str = contratto2.to_string();
        let contratto3str = contratto3.to_string();
        for i in 0..14 {
            data.insert(&contratto1str as &str, contratto1.score(i));
            data.insert(&contratto2str as &str, contratto2.score(i));
            data.insert(&contratto3str as &str, contratto3.score(i));
            matrix.add_data(&data);
        }
        matrix.report();
        assert_eq!(7, matrix.table[2][0][9]);
    }
    #[test]
    fn can_create_contract_from_str_test() {
        let contract_c = Contract::from_str("3CN", Vulnerable::No).unwrap();
        let contract_d = Contract::from_str("3DN", Vulnerable::No).unwrap();
        let contract_s = Contract::from_str("3SN", Vulnerable::No).unwrap();
        let contract_h = Contract::from_str("3HN", Vulnerable::No).unwrap();
        let contract_n = Contract::from_str("3NNXX", Vulnerable::No).unwrap();
        assert_eq!(contract_c.to_string(), "3♣N");
        assert_eq!(contract_d.to_string(), "3♦N");
        assert_eq!(contract_h.to_string(), "3♥N");
        assert_eq!(contract_s.to_string(), "3♠N");
        assert_eq!(contract_n.to_string(), "3NTNXX");
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
