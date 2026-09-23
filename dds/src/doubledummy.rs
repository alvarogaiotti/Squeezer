// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    analyseplay::{PlayAnalyzer, PlayTraceBin, SolvedPlay},
    bindings::SolveBoard,
    context::AnalysePlayBin,
    ddserror::DdsError,
    deal::{AsDDSDeal, DdsDeal},
    future_tricks::FutureTricks,
    solver::BridgeSolver,
    traits::AsDDSContract,
    utils::{Mode, Solutions, Target, ThreadIndex, build_c_deal, if_no_fault_return},
};

/// A single threaded double dummy solver.
///
/// In DDS 3.1 the legacy entry points this wraps draw their solver context from
/// a `thread_local`, so calling one from several threads is safe (each thread
/// gets its own context); the older "SEGFAULT under threads" warning applied to
/// DDS 2.9's global state. For transposition-table reuse, still prefer the
/// `squeezer` solver seam's pool, which keeps one context per worker.
/// This struct implements all the needed traits linked to a double dummy analysis
/// such as:
/// - Play analysis [`PlayAnalyzer`]
/// - Par calculation [`DdTableCalculator`](crate::tables::DdTableCalculator)
/// - Board solution [`BridgeSolver`]
///
/// To use it, import the required trait, instantiate this struct and use it.
///
/// # Examples
/// Since this crate is just a wrapper around a DLL it is hard to provide a good example,
/// but the usage is something about like so:
/// ```
/// # use dds::solver::BridgeSolver;
/// # use std::error::Error;
/// # use dds::doubledummy::DoubleDummySolver;
/// # use dds::deal::DdsDealBuilder;
/// # use dds::deal::AsDDSDeal;
/// # #[derive(Debug, Clone)]
/// # pub struct DealMock {pub hands: [[usize; 4]; 4]}
/// # impl IntoIterator for DealMock {
/// #     type Item = [usize; 4];
/// #     type IntoIter = std::array::IntoIter<[usize; 4], 4>;
/// #     fn into_iter(self) -> Self::IntoIter {
/// #         self.hands.into_iter()
/// #     }
/// # }
/// #
/// # impl dds::deal::AsDDSDeal for DealMock {
/// #     fn as_dds_deal(&self) -> dds::deal::DDSDealRepr {
/// #         let mut remain_cards = [[0; 4]; 4];
/// #         for (seat, hand) in self.clone().into_iter().enumerate() {
/// #             for (index, suit) in hand.into_iter().enumerate() {
/// #                 remain_cards[seat][index] = (suit >> (16 * index)) as u32;
/// #             }
/// #         }
/// #         remain_cards.into()
/// #     }
/// # }
///
/// # #[derive(Debug, Copy, Clone)]
/// # pub struct ContractMock {}
/// #
/// # impl dds::traits::ContractScorer for ContractMock {
/// #     fn score(&self, _tricks: u8) -> i32 {
/// #         0
/// #     }
/// # }
/// #
/// # impl dds::traits::AsDDSContract for ContractMock {
/// #     fn as_dds_contract(&self) -> (dds::deal::DdsSuit, dds::deal::DdsHandEncoding) {
/// #     use dds::deal::{DdsSuit, DdsHandEncoding};
/// #         (DdsSuit::Diamonds, DdsHandEncoding::West)
/// #     }
/// # }
/// # fn main() -> Result<(), Box<dyn Error>>{
///    /*
///           ♠K93
///           ♡JT9862
///           ♢9
///           ♣K73
///
///    ♠T4           ♠AQJ
///    ♡Q            ♡75
///    ♢KQT543       ♢AJ2
///    ♣QT85         ♣J9642
///
///           ♠87652
///           ♡AK43
///           ♢876
///           ♣A
///    */
///
///    let deal = DealMock {
///        hands: [
///            [8712, 256114688, 2199023255552, 2344123606046343168],
///            [22528, 10485760, 79182017069056, 744219838422974464],
///            [484, 1612185600, 1924145348608, 4611686018427387904],
///            [1040, 268435456, 57415122812928, 1522216674051227648],
///        ],
///    };
///    let contract = ContractMock {};
///    let solver = DoubleDummySolver::new();
///    let deal = DdsDealBuilder::new().contract(&contract).remain_cards(deal.as_dds_deal()).build()?;
///    println!("{}", solver.dd_tricks(&deal)?);
/// # Ok(())
/// # }
/// ```
pub struct DoubleDummySolver {
    _marker: std::marker::PhantomData<std::cell::Cell<()>>,
}

impl DoubleDummySolver {
    /// Function to free the dinamically allocated memory of the DDS's DLL.
    /// Use this if you are not gonna use DDS for a long time and want to free some memory.
    pub fn free_memory(&self) {
        unsafe { crate::bindings::FreeMemory() }
    }

    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for DoubleDummySolver {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgeSolver for DoubleDummySolver {
    #[inline]
    fn dd_tricks(&self, deal: &DdsDeal) -> Result<u8, DdsError> {
        let mut future_tricks = FutureTricks::new();
        let futp: *mut FutureTricks = &raw mut future_tricks;
        let result;
        unsafe {
            result = SolveBoard(
                *deal,
                Target::MaxTricks.into(),
                Solutions::Best.into(),
                Mode::AutoReuseAlwaysSearch.into(),
                futp,
                ThreadIndex::Auto.into(),
            );
        };
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss
        )]
        if result == 1 {
            Ok(13 - future_tricks.score()[0] as u8)
        } else {
            Err(DdsError::from(result))
        }
    }

    #[inline]
    fn dd_tricks_all_cards(&self, deal: &DdsDeal) -> Result<FutureTricks, DdsError> {
        let mut future_tricks = FutureTricks::new();
        let futp: *mut FutureTricks = &raw mut future_tricks;
        let result;
        unsafe {
            result = SolveBoard(
                *deal,
                Target::MaxTricks.into(),
                Solutions::AllLegal.into(),
                Mode::AutoReuseAlwaysSearch.into(),
                futp,
                ThreadIndex::Auto.into(),
            );
        };
        if result == 1 {
            Ok(future_tricks)
        } else {
            Err(DdsError::from(result))
        }
    }

    fn solve_with_params(
        &self,
        deal: &DdsDeal,
        mode: Mode,
        solutions: Solutions,
        target: Target,
    ) -> Result<FutureTricks, DdsError> {
        let mut future_tricks = FutureTricks::new();
        let futp: *mut FutureTricks = &raw mut future_tricks;
        let result;
        unsafe {
            result = SolveBoard(
                *deal,
                target.into(),
                solutions as i32,
                mode as i32,
                futp,
                ThreadIndex::Auto.into(),
            );
        };
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss
        )]
        if result == 1 {
            Ok(future_tricks)
        } else {
            Err(DdsError::from(result))
        }
    }
}

impl PlayAnalyzer for DoubleDummySolver {
    #[inline]
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
        play: PlayTraceBin,
    ) -> Result<SolvedPlay, DdsError> {
        let Ok(dds_deal) = build_c_deal((contract, deal)) else {
            return Err(crate::bindings::ddsffi::RETURN_UNKNOWN_FAULT.into());
        };
        let mut solved_play = SolvedPlay::new();
        let solved: *mut SolvedPlay = &raw mut solved_play;
        let play_trace = play;
        // SAFETY: calling an external C function
        let result = unsafe { AnalysePlayBin(dds_deal, play_trace, solved, 0) };
        if_no_fault_return!(result, solved_play);
    }
}
