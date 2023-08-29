use rusty_dealer_macros::*;
use std::{error::Error, ffi::c_int};
mod analyseplay;
mod ddserror;
mod ddsffi;
mod deal;
mod future_tricks;
mod solver;
pub use analyseplay::*;
pub use ddserror::DDSError;
pub use ddsffi::*;
pub use deal::*;
pub use solver::*;

enum ThreadIndex {
    Auto,
    NumThreads(NonZeroI32),
}

impl From<ThreadIndex> for std::ffi::c_int {
    fn from(value: ThreadIndex) -> Self {
        match value {
            ThreadIndex::Auto => 0,
            ThreadIndex::NumThreads(value) => value.into(),
        }
    }
}

enum Target {
    MaxTricks,
    LegalNoScore,
    Goal(NonZeroI32),
}

impl From<Target> for std::ffi::c_int {
    fn from(value: Target) -> Self {
        match value {
            Target::MaxTricks => -1,
            Target::LegalNoScore => 0,
            Target::Goal(goal) => std::ffi::c_int::max(13, goal.into()),
        }
    }
}

enum Solutions {
    Best,
    AllOptimal,
    AllLegal,
}

impl From<Solutions> for std::ffi::c_int {
    fn from(value: Solutions) -> Self {
        match value {
            Solutions::Best => 1,
            Solutions::AllOptimal => 2,
            Solutions::AllLegal => 3,
        }
    }
}

enum Mode {
    Auto,
    AutoSearchAlways,
    Always,
}

impl From<Mode> for std::ffi::c_int {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Auto => 0,
            Mode::AutoSearchAlways => 1,
            Mode::Always => 2,
        }
    }
}

pub struct DoubleDummySolver {}

pub trait BridgeTableCalculator {}
struct DDSCalc {}
impl BridgeTableCalculator for DDSCalc {}

impl DoubleDummySolver {
    pub fn solver() -> DDSSolver {
        DDSSolver {}
    }

    pub fn play_analyzer() -> impl PlayAnalyzer {
        DDSPlayAnalyzer {}
    }

    pub fn calculator() -> impl BridgeTableCalculator {
        DDSCalc {}
    }

    fn set_max_threads(user_threads: i32) {
        unsafe { ddsffi::SetMaxThreads(user_threads) }
    }

    fn set_resources(max_memory_mb: i32, max_threads: i32) {
        unsafe { ddsffi::SetResources(max_memory_mb, max_threads) }
    }
}

#[must_use]
fn dd_score<D: AsDDSDeal, C: AsDDSContract + ContractScorer>(
    deal: &D,
    contract: &C,
) -> Result<i32, Box<dyn std::error::Error>> {
    let solver = DoubleDummySolver::solver();
    let tricks = solver.dd_tricks(deal, contract)?;
    Ok(contract.score(tricks))
}

pub trait RawDDS {
    type Raw;

    fn get_raw(&self) -> Self::Raw;
}

/// Models a side: either North-South or East-West
pub enum Side {
    NS = 0,
    EW = 1,
}

pub trait AsDDSContract {
    fn as_dds_contract(&self) -> (i32, i32);
}

pub trait ContractScorer {
    fn score(&self, tricks: u8) -> i32;
}

#[derive(RawDDS)]
pub struct SolvedPlays {
    #[raw]
    solved_play: ddsffi::solvedPlays,
}

#[derive(RawDDS)]
pub struct SolvedPlay {
    #[raw]
    solved_play: ddsffi::solvedPlay,
}

impl SolvedPlay {
    pub fn new() -> Self {
        Self {
            solved_play: ddsffi::solvedPlay {
                number: 0,
                tricks: [0; 53],
            },
        }
    }
    pub fn tricks(&self) -> &[i32; 53usize] {
        self.get_tricks()
    }

    fn get_tricks(&self) -> &[i32; 53usize] {
        &self.solved_play.tricks
    }

    pub fn number(&self) -> i32 {
        self.get_number()
    }
    fn get_number(&self) -> i32 {
        self.get_raw().number
    }
}

impl Default for SolvedPlay {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(RawDDS)]
pub struct PlayTraceBin {
    #[raw]
    play_trace_bin: playTraceBin,
}

impl PlayTraceBin {
    pub fn new(number: c_int, suit: [c_int; 52], rank: [c_int; 52]) -> Self {
        Self {
            play_trace_bin: playTraceBin::new(number, suit, rank),
        }
    }
}

pub trait AsDDSCard {
    fn as_card(&self) -> (i32, i32);
}

pub trait AsDDSPlayTrace<I, C>
where
    I: IntoIterator,
    I::Item: std::borrow::Borrow<C>,
    C: AsDDSCard,
{
    fn as_play_trace(&self) -> I;
}

impl ddsffi::playTraceBin {
    /// Provide length of the sequence you want to be analyzed against double dummy, the suit of the
    /// cards played and their's rank.
    pub fn new(number: c_int, suit: [c_int; 52], rank: [c_int; 52]) -> Self {
        Self { number, suit, rank }
    }
}

#[cfg(test)]
mod test {
    use super::BridgeSolver;
    use std::cell::OnceCell;

    const DEAL: OnceCell<DealMock> = OnceCell::new();

    #[derive(Debug, Clone)]
    struct DealMock {
        hands: [[usize; 4]; 4],
    }

    impl IntoIterator for DealMock {
        type Item = [usize; 4];
        type IntoIter = std::array::IntoIter<[usize; 4], 4>;
        fn into_iter(self) -> Self::IntoIter {
            self.hands.into_iter()
        }
    }

    impl super::deal::AsDDSDeal for DealMock {
        fn as_dds_deal(&self) -> super::deal::DDSDealRepr {
            let mut remain_cards = [[0; 4]; 4];
            for (seat, hand) in self.clone().into_iter().enumerate() {
                for (index, suit) in hand.into_iter().enumerate() {
                    remain_cards[seat][index] = (suit >> (16 * index)) as u32;
                }
            }
            crate::DDSDealRepr::new(remain_cards)
        }
    }

    struct ContractMock {}

    impl crate::ContractScorer for ContractMock {
        fn score(&self, tricks: u8) -> i32 {
            0
        }
    }

    impl crate::AsDDSContract for ContractMock {
        fn as_dds_contract(&self) -> (i32, i32) {
            (2, 2)
        }
    }

    fn initialize_test() -> DealMock {
        DEAL.get_or_init(|| DealMock {
            hands: [
                [1580, 3145728, 71468255805440, 5215168368495034368],
                [26624, 1233649664, 171798691840, 3459890413727383552],
                [80, 608436224, 9431748182016, 364791569817010176],
                [4480, 301989888, 59648505806848, 182395784908505088],
            ],
        })
        .clone()
    }

    #[test]
    fn test_linkage() {
        let deal = initialize_test();
        let contract = ContractMock {};
        let solver = crate::bindings::DoubleDummySolver::solver();
        println!("{}", solver.dd_tricks(&deal, &contract).unwrap());
        println!("{}", crate::bindings::dd_score(&deal, &contract).unwrap());
    }
}
