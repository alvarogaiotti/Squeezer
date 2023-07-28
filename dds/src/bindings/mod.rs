use rusty_dealer_macros::*;
use std::{error::Error, ffi::c_int};
mod ddsffi;
mod deal;
mod future_tricks;
mod solver;
pub use ddsffi::*;
pub use deal::*;
pub use solver::*;

#[derive(Debug)]
pub struct DDSError {
    kind: DDSErrorKind,
}

impl std::fmt::Display for DDSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "something went wrong while solving boards:\n\t{}",
            self.kind
        )
    }
}
impl std::error::Error for DDSError {}

#[derive(Debug)]
pub enum DDSErrorKind {
    ReturnUnknownFault,
    ReturnZeroCards,
    ReturnTargetTooHigh,
    ReturnDuplicateCards,
    ReturnTargetWrongLo,
    ReturnTargetWrongHi,
    ReturnSolnsWrongLo,
    ReturnSolnsWrongHi,
    ReturnTooManyCards,
    ReturnSuitOrRank,
    ReturnPlayedCard,
    ReturnCardCount,
    ReturnThreadIndex,
    ReturnModeWrongLo,
    ReturnModeWrongHi,
    ReturnTrumpWrong,
    ReturnFirstWrong,
    ReturnPlayFault,
    ReturnPbnFault,
    ReturnThreadCreate,
    ReturnThreadWait,
    ReturnNoSuit,
    ReturnTooManyTables,
    ReturnChunkSize,
}

impl From<std::ffi::c_int> for DDSErrorKind {
    fn from(value: std::ffi::c_int) -> Self {
        match value {
            RETURN_UNKNOWN_FAULT => Self::ReturnUnknownFault,
            RETURN_ZERO_CARDS => Self::ReturnZeroCards,
            RETURN_TARGET_TOO_HIGH => Self::ReturnTargetTooHigh,
            RETURN_DUPLICATE_CARDS => Self::ReturnDuplicateCards,
            RETURN_TARGET_WRONG_LO => Self::ReturnTargetWrongLo,
            RETURN_TARGET_WRONG_HI => Self::ReturnTargetWrongHi,
            RETURN_SOLNS_WRONG_LO => Self::ReturnSolnsWrongLo,
            RETURN_SOLNS_WRONG_HI => Self::ReturnSolnsWrongHi,
            RETURN_TOO_MANY_CARDS => Self::ReturnTooManyCards,
            RETURN_SUIT_OR_RANK => Self::ReturnSuitOrRank,
            RETURN_PLAYED_CARD => Self::ReturnPlayedCard,
            RETURN_CARD_COUNT => Self::ReturnCardCount,
            RETURN_THREAD_INDEX => Self::ReturnThreadIndex,
            RETURN_MODE_WRONG_LO => Self::ReturnModeWrongLo,
            RETURN_MODE_WRONG_HI => Self::ReturnModeWrongHi,
            RETURN_TRUMP_WRONG => Self::ReturnTrumpWrong,
            RETURN_FIRST_WRONG => Self::ReturnFirstWrong,
            RETURN_PLAY_FAULT => Self::ReturnPlayFault,
            RETURN_PBN_FAULT => Self::ReturnPbnFault,
            RETURN_THREAD_CREATE => Self::ReturnThreadCreate,
            RETURN_THREAD_WAIT => Self::ReturnThreadWait,
            RETURN_NO_SUIT => Self::ReturnNoSuit,
            RETURN_TOO_MANY_TABLES => Self::ReturnTooManyTables,
            RETURN_CHUNK_SIZE => Self::ReturnChunkSize,
            // SAFETY: Return from DDS cannot be different from its defined constants
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for DDSErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReturnUnknownFault => write!(f, "fopen() failed or wrong number of boards"),
            Self::ReturnZeroCards => write!(f, "zero cards"),
            Self::ReturnTargetTooHigh => write!(f, "target higher than number of tricks remaining"),
            Self::ReturnDuplicateCards => write!(f, "duplicated card"),
            Self::ReturnTargetWrongLo => write!(f, "target is less than -1"),
            Self::ReturnTargetWrongHi => write!(f, "target is higher than 13"),
            Self::ReturnSolnsWrongLo => write!(f, "solutions is less than 1"),
            Self::ReturnSolnsWrongHi => write!(f, "solutions is more than 3"),
            Self::ReturnTooManyCards => write!(f, "too many cards"),
            Self::ReturnSuitOrRank => write!(f, "currentTrickSuit or currentTrickRank have wrong data"),
            Self::ReturnPlayedCard => write!(f, "card already played"),
            Self::ReturnCardCount => write!(f, "wrong number of remining cards for a hand"),
            Self::ReturnThreadIndex => write!(f, "thread number is less than 0 or higher than the maximum permitted"),
            Self::ReturnModeWrongLo => write!(f, "mode is less than 0"),
            Self::ReturnModeWrongHi => write!(f, "mode is greater than 2"),
            Self::ReturnTrumpWrong => write!(f, "trump is not one of 0,1,2,3 or 4"),
            Self::ReturnFirstWrong => write!(f, "first is not one of 0,1,2 or 3"),
            Self::ReturnPlayFault => write!(f, "less than 0 or more than 52 cards supplied, invalid suit or rank supplied or played card is not held by the right player"),
            Self::ReturnPbnFault => write!(f, "PBN string is malformed"),
            Self::ReturnThreadCreate => write!(f, "thread created"),
            Self::ReturnThreadWait => write!(f, "something went wrong while waiting for threads to complete"),
            Self::ReturnNoSuit => write!(f, "denomination filter vector has no entries"),
            Self::ReturnTooManyTables => write!(f, "too many tables requested"),
            Self::ReturnChunkSize => write!(f, "chunk size is less than 1"),
        }
    }
}

pub struct DoubleDummySolver {}

pub trait BridgePlayAnalyzer {
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: &D,
        contract: C,
        play: &PlayTraceBin,
    ) -> SolvedPlay;
    fn analyze_all_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: &D,
        contract: C,
        play: &PlayTraceBin,
    ) -> SolvedPlay;
}
struct DDSPlayAnalyzer {}
impl BridgePlayAnalyzer for DDSPlayAnalyzer {
    fn analyze_all_play<D: AsDDSDeal, C: AsDDSContract>(
        _deal: &D,
        _contract: C,
        _play: &PlayTraceBin,
    ) -> SolvedPlay {
        todo!()
    }
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: &D,
        contract: C,
        play: &PlayTraceBin,
    ) -> SolvedPlay {
        let (trump, first) = contract.as_dds_contract();
        let c_deal = ddsffi::deal {
            trump: trump as c_int,
            first: first as c_int,
            currentTrickSuit: [0; 3],
            currentTrickRank: [0; 3],
            remainCards: deal.as_dds_deal().as_slice(),
        };
        let solved_play = SolvedPlay::new();
        {
            let solved: *mut ddsffi::solvedPlay = &mut solved_play.get_raw();
            let play_trace: *const ddsffi::playTraceBin = &play.get_raw();
            unsafe { ddsffi::AnalysePlayBin(c_deal, *play_trace, solved, 0) };
        }
        solved_play
    }
}
pub trait BridgeTableCalculator {}
struct DDSCalc {}
impl BridgeTableCalculator for DDSCalc {}

impl DoubleDummySolver {
    pub fn solver() -> DDSSolver {
        DDSSolver {}
    }

    pub fn play_analyzer() -> impl BridgePlayAnalyzer {
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
    fn as_dds_contract(&self) -> (u8, u8);
}

pub trait ContractScorer {
    fn score(&self, tricks: u8) -> i32;
}

#[derive(RawDDS)]
pub struct SolvedPlay {
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
        fn as_dds_contract(&self) -> (u8, u8) {
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
