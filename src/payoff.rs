use crate::prelude::*;

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
    pub fn add_data(&mut self, raw_scores: &HashMap<String, i32>) {
        let diff = &self.diff;
        for (i, ei) in self.entries.iter().enumerate() {
            for (j, ej) in self.entries.iter().enumerate() {
                //println!(
                //    "{:?}\nv[i]:{:?}\nv[i][j]:{:?}",
                //    self.table, self.table[i], self.table[i][j]
                //);
                //println!("i:{i}, j:{j}",);
                self.table[i][j].push(diff(
                    *raw_scores.get(&ei.to_string()).unwrap(),
                    *raw_scores.get(&ej.to_string()).unwrap(),
                ));
            }
        }
    }
    pub fn report(&self) {
        let mut means_stderrs: Vec<Vec<(f32, f32)>> = Vec::new();
        for (i, line) in self.table.iter().enumerate() {
            means_stderrs.push(Vec::new());
            for (_j, score) in line.iter().enumerate() {
                means_stderrs[i].push((
                    mean(score).unwrap(),
                    std_deviation(score).unwrap() / (score.len() as f32).sqrt(),
                ))
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
                })
            }
            println!()
        }
    }
}

fn mean(data: &[i32]) -> Option<f32> {
    let sum = data.iter().sum::<i32>() as f32;
    let count = data.len();
    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}
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
///A struct rapresenting a contract
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Contract {
    vuln: bool,
    level: usize,
    doubled: Doubled,
    strain: Strain,
    declarer: Seat,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum Doubled {
    NotDoubled = 0,
    Doubled = 1,
    Redoubled = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum Strain {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
    NoTrumps = 4,
}

impl Contract {
    #[allow(clippy::cast_possible_truncation)]
    pub fn strain(&self) -> Strain {
        self.strain
    }
    pub fn leader(&self) -> Seat {
        self.declarer().next()
    }
    pub fn declarer(&self) -> Seat {
        self.declarer
    }
    pub fn from_str(s: &str, vuln: bool) -> Result<Self, DealerError> {
        let doubled = match s.len() - s.trim_end_matches('X').len() {
            0 => Doubled::NotDoubled,
            1 => Doubled::Doubled,
            2 => Doubled::Redoubled,
            _ => unreachable!("too many `X`"),
        };
        let mut chars = s.chars();
        let level = chars.next().unwrap().to_digit(10).unwrap() as usize;
        if !(1..=7).contains(&level) {
            return Err(DealerError::new("Wrong contract level"));
        };
        Ok(Self {
            vuln,
            doubled,
            level,
            strain: Strain::from_char(chars.next().unwrap())?,
            declarer: Seat::from_char(chars.next().unwrap())?,
        })
    }
    pub fn score(&self, tricks: u8) -> i32 {
        let target: i32 = self.level as i32 + 6i32;
        let overtricks: i32 = tricks as i32 - target;
        if overtricks >= 0 {
            let per_trick: i32 = match self.strain {
                Strain::Clubs | Strain::Diamonds => 20,
                _ => 30,
            };
            let mut per_overtrick: i32 = per_trick;
            let mut base_score: i32 = per_trick * self.level as i32;
            let mut bonus: i32 = 0;
            if self.strain == Strain::NoTrumps {
                base_score += 10
            };

            match self.doubled {
                Doubled::Doubled => {
                    base_score *= 2;
                    bonus += 50;
                    per_overtrick = 100;
                }
                Doubled::Redoubled => {
                    base_score *= 4;
                    bonus += 100;
                    per_overtrick = 200;
                }
                Doubled::NotDoubled => {}
            };
            bonus += if base_score >= 100 {
                if self.vuln {
                    500
                } else {
                    300
                }
            } else {
                50
            };
            bonus += if self.level == 6 {
                match self.vuln {
                    true => 750,
                    false => 500,
                }
            } else {
                0
            };
            bonus += if self.level == 7 {
                match self.vuln {
                    true => 1500,
                    false => 1000,
                }
            } else {
                0
            };
            bonus += overtricks * per_overtrick;
            base_score + bonus
        } else {
            let mut score: i32;
            if matches!(self.doubled, Doubled::NotDoubled) {
                let per_undertrick = if self.vuln { 100 } else { 50 };
                score = overtricks * per_undertrick;
            } else {
                match overtricks {
                    -1 => score = if self.vuln { -200 } else { -100 },
                    -2 => score = if self.vuln { -500 } else { -300 },
                    _ => {
                        score = if self.vuln {
                            300 * overtricks + 100
                        } else {
                            300 * overtricks + 400
                        }
                    }
                }
                if matches!(self.doubled, Doubled::Redoubled) {
                    score *= 2
                }
            }
            score
        }
    }
    pub fn not_unicode_str(&self) -> String {
        format!(
            "{}{}{}{}",
            self.level,
            self.strain.not_unicode_str(),
            self.declarer,
            if matches!(self.doubled, Doubled::NotDoubled) {
                String::new()
            } else {
                let mut stringa = String::new();
                for _ in 0..(self.doubled as usize) {
                    stringa.push('X')
                }
                stringa
            }
        )
    }
}
impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.level,
            self.strain,
            self.declarer,
            if matches!(self.doubled, Doubled::NotDoubled) {
                String::new()
            } else {
                let mut stringa = String::new();
                for _ in 0..(self.doubled as usize) {
                    stringa.push('X')
                }
                stringa
            }
        )
    }
}
impl Strain {
    fn from_char(c: char) -> Result<Self, DealerError> {
        match c {
            'S' => Ok(Self::Spades),
            'H' => Ok(Self::Hearts),
            'D' => Ok(Self::Diamonds),
            'C' => Ok(Self::Clubs),
            'N' => Ok(Self::NoTrumps),
            _ => Err(DealerError::new("Not a strain.")),
        }
    }
    fn not_unicode_str(self) -> String {
        match self {
            Self::Spades => String::from("S"),
            Self::Hearts => String::from("H"),
            Self::Diamonds => String::from("D"),
            Self::NoTrumps => String::from("NT"),
            Self::Clubs => String::from("C"),
        }
    }
}
impl fmt::Display for Strain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spades => write!(f, "♠"),
            Self::Hearts => write!(f, "♥"),
            Self::Diamonds => write!(f, "♦"),
            Self::NoTrumps => write!(f, "NT"),
            Self::Clubs => write!(f, "♣"),
        }
    }
}

fn bisect_right(value: i32, lista: &[i32]) -> i32 {
    for (i, &x) in lista.iter().enumerate() {
        if x < value {
            continue;
        }
        return i as i32;
    }
    lista.len() as i32
}
pub fn imps(my: i32, other: i32) -> i32 {
    let imp_table: [i32; 24] = [
        15, 45, 85, 125, 165, 215, 265, 315, 365, 425, 495, 595, 745, 895, 1095, 1295, 1495, 1745,
        1995, 2245, 2495, 2995, 3495, 3995,
    ];
    bisect_right((my - other).abs(), &imp_table) * (if my > other { 1 } else { -1 })
}
pub fn matchpoints(my: i32, other: i32) -> i32 {
    (my > other) as i32 - (my < other) as i32
}

#[cfg(test)]
#[test]
fn payoff_report_test() {
    let contratto1 = Contract::from_str("3CN", false).unwrap();
    let contratto2 = Contract::from_str("3DN", false).unwrap();
    let contratto3 = Contract::from_str("3NN", false).unwrap();
    let contracts = vec![
        Contract::from_str("3CN", false).unwrap(),
        Contract::from_str("3DN", false).unwrap(),
        Contract::from_str("3NN", false).unwrap(),
    ];
    let mut matrix = Payoff::new(contracts, imps);
    let mut data: HashMap<String, i32> = HashMap::new();
    for i in 0..14 {
        data.insert(contratto1.to_string(), contratto1.score(i));
        data.insert(contratto2.to_string(), contratto2.score(i));
        data.insert(contratto3.to_string(), contratto3.score(i));
        matrix.add_data(&data)
    }
    matrix.report();
    assert_eq!(7, matrix.table[2][0][9]);
}
#[test]
fn can_create_contract_from_str_test() {
    let contract_c = Contract::from_str("3CN", false).unwrap();
    let contract_d = Contract::from_str("3DN", false).unwrap();
    let contract_s = Contract::from_str("3SN", false).unwrap();
    let contract_h = Contract::from_str("3HN", false).unwrap();
    let contract_n = Contract::from_str("3NNXX", false).unwrap();
    assert_eq!(contract_c.to_string(), "3♣N");
    assert_eq!(contract_d.to_string(), "3♦N");
    assert_eq!(contract_h.to_string(), "3♥N");
    assert_eq!(contract_s.to_string(), "3♠N");
    assert_eq!(contract_n.to_string(), "3NTNXX");
}
#[test]
#[should_panic(expected = "Wrong contract level")]
fn create_contract_wrong_level_test() {
    let _contract = Contract::from_str("8CS", false).unwrap();
}
#[test]
fn contract_computes_correct_scores_test() {
    let contract_c = Contract::from_str("6CN", false).unwrap();
    let contract_d = Contract::from_str("5DNX", true).unwrap();
    let contract_s = Contract::from_str("4SN", false).unwrap();
    let contract_h = Contract::from_str("3HN", false).unwrap();
    let contract_n = Contract::from_str("3NN", false).unwrap();
    assert_eq!(400_i32, contract_n.score(9));
    assert_eq!(140_i32, contract_h.score(9));
    assert_eq!(420_i32, contract_s.score(10));
    assert_eq!(750_i32, contract_d.score(11));
    assert_eq!(920_i32, contract_c.score(12));
    assert_eq!(-200, contract_d.score(10));
}
