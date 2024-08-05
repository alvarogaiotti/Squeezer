// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    bindings::{ddsffi::RETURN_UNKNOWN_FAULT, MAXNOOFBOARDS},
    ddserror::DDSError,
    deal::AsDDSDeal,
    traits::{AsDDSContract, AsRawDDS},
    utils::{RankSeq, SuitSeq},
};
use core::{ffi::c_int, slice::Iter};

/// Number of consecutive boards in a sequence a thread gets when we call
/// [`super::PlayAnalyzer::analyze_play()`].
/// 1 means thread1 takes number 1, thread2 takes number 2 and so on
/// 10 means thread1 takes 1..10, thread2 takes 11..20 etc.
pub const CHUNK_SIZE: i32 = 10;

#[non_exhaustive]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SolvedPlays {
    pub no_of_boards: c_int,
    pub solved: [SolvedPlay; 200usize],
}

impl<'a> IntoIterator for &'a SolvedPlays {
    type Item = &'a SolvedPlay;
    type IntoIter = Iter<'a, SolvedPlay>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl SolvedPlays {
    /// Standard iteration
    fn iter(&self) -> Iter<'_, SolvedPlay> {
        self.solved[..self.no_of_boards as usize].iter()
    }
}

impl IntoIterator for SolvedPlay {
    type Item = i32;
    type IntoIter = core::array::IntoIter<Self::Item, 53>;

    fn into_iter(self) -> Self::IntoIter {
        self.tricks.into_iter()
    }
}

impl SolvedPlay {
    #[inline]
    #[must_use]
    pub const fn tricks(&self) -> &[i32; 53usize] {
        &self.tricks
    }

    #[inline]
    pub fn lead_result(&self) -> Option<i32> {
        self.tricks().get(1).copied()
    }

    #[inline]
    #[must_use]
    pub const fn number(&self) -> i32 {
        self.number
    }

    /// Function for testing purposes and should not be used.
    #[must_use]
    pub fn from_seq(mut seq: Vec<i32>) -> Self {
        let number = seq.len() as i32;
        seq.resize(53, -1i32);
        Self {
            number,
            tricks: seq.try_into().expect("just resized"),
        }
    }

    /// Creates a new [`SolvedPlay`] instance
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            number: 0i32,
            tricks: [-1i32; 53],
        }
    }

    #[inline]
    /// Returns a [`core::slice::Iter`] over the tricks.
    fn iter(&self) -> Iter<'_, i32> {
        self.tricks[..self
            .number
            .try_into()
            .expect("it's a lenght so it's always positive")]
            .iter()
    }
}

impl PlayTracesBin {
    #[allow(
        clippy::unwrap_in_result,
        clippy::unwrap_used,
        clippy::missing_panics_doc
    )]
    #[inline]
    /// Provide length of the sequence you want to be analyzed against double dummy, the suit of the
    /// cards played and their's rank.
    /// # Errors
    /// Returns an error if suits and ranks have different length
    pub fn from_sequences(suits: Vec<SuitSeq>, ranks: Vec<RankSeq>) -> Result<Self, DDSError> {
        let (suits_len, ranks_len) = (
            suits.len().clamp(0, MAXNOOFBOARDS),
            ranks.len().clamp(0, MAXNOOFBOARDS),
        );
        if suits_len != ranks_len {
            return Err(RETURN_UNKNOWN_FAULT.into());
        }
        let mut plays: Vec<PlayTraceBin> = suits
            .into_iter()
            .zip(ranks)
            .map(|(suit, rank)| PlayTraceBin::from_sequences(suit, rank))
            .collect();
        plays.resize(MAXNOOFBOARDS, PlayTraceBin::new());
        Ok(Self {
            // SAFETY: capped at 200
            noOfBoards: suits_len.try_into().unwrap(),
            // SAFETY: We now the length of the Vec
            plays: plays.try_into().unwrap(),
        })
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.noOfBoards.try_into().unwrap()
    }
}

impl Default for PlayTraceBin {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayTraceBin {
    #[inline]
    #[must_use]
    /// Provide length of the sequence you want to be analyzed against double dummy, the suit of the
    /// cards played and their's rank.
    /// Will silently evaluate until the shortest sequence end if their length is different
    pub fn from_sequences(suit: SuitSeq, rank: RankSeq) -> Self {
        let length = if suit.length() <= rank.length() {
            suit.length()
        } else {
            rank.length()
        };
        let number = length;
        Self {
            number,
            suit: suit.as_raw(),
            rank: rank.as_raw(),
        }
    }
    #[inline]
    #[must_use]
    /// Creates a new [`PlayTraceBin`] from data
    const fn from(number: c_int, suit: [c_int; 52], rank: [c_int; 52]) -> Self {
        Self { number, suit, rank }
    }
    /// Creates a new [`PlayTraceBin`]
    pub const fn new() -> Self {
        Self {
            number: 0,
            suit: [-1i32; 52],
            rank: [-1i32; 52],
        }
    }
}

/// A trait which can be implemented by any stuct capable of doing
/// DD analysis. Simple interface so we can eventually swap other DD solvers
/// in the future. Kinda like a Strategy Pattern. Now depends on dds for the
/// generics with traits used but the idea is to create marker traits for deals and
/// contracts to swap them in.
pub trait PlayAnalyzer {
    /// Analyzes a single hand
    /// # Errors
    /// Will return an Error when DDS fails in some way.
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
        play: PlayTraceBin,
    ) -> Result<SolvedPlay, DDSError>;
    /// Analyzes a bunch of hands in paraller.
    /// # Errors
    /// Will return an Error when DDS fails in some way or the deals and contracts vecs have
    /// different length.
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deals: &Vec<D>,
        contracts: &Vec<C>,
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError>;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SolvedPlay {
    pub number: ::std::os::raw::c_int,
    pub tricks: [::std::os::raw::c_int; 53usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// Wrapper around DDS [`PlayTracesBin`] type.
/// The [`PlayTraceBin`] stores two arrays
/// of 52 element each representing played card's rank
/// and suit, then an integer stating the real lenght of the play sequence.
pub struct PlayTracesBin {
    pub noOfBoards: ::std::os::raw::c_int,
    pub plays: [PlayTraceBin; 200usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PlayTracesPBN {
    pub noOfBoards: ::std::os::raw::c_int,
    pub plays: [PlayTracePBN; 200usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// Wrapper around DDS [`PlayTraceBin`] type.
/// The [`PlayTraceBin`] stores two arrays
/// of 52 element each representing played card's rank
/// and suit, then an integer stating the real lenght of the play sequence.
pub struct PlayTraceBin {
    pub number: ::std::os::raw::c_int,
    pub suit: [::std::os::raw::c_int; 52usize],
    pub rank: [::std::os::raw::c_int; 52usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PlayTracePBN {
    pub number: ::std::os::raw::c_int,
    pub cards: [::std::os::raw::c_char; 106usize],
}

#[cfg(test)]
mod test {
    use crate::bindings::ddsffi::DDSInfo;

    use super::*;
    #[test]
    fn bindgen_test_layout_solvedPlays() {
        assert_eq!(
            core::mem::size_of::<SolvedPlays>(),
            43204usize,
            concat!("Size of: ", stringify!(solvedPlays))
        );
        assert_eq!(
            ::std::mem::align_of::<SolvedPlays>(),
            4usize,
            concat!("Alignment of ", stringify!(solvedPlays))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SolvedPlays>())).no_of_boards as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(solvedPlays),
                "::",
                stringify!(noOfBoards)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SolvedPlays>())).solved as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(solvedPlays),
                "::",
                stringify!(solved)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_solvedPlay() {
        assert_eq!(
            ::std::mem::size_of::<SolvedPlay>(),
            216usize,
            concat!("Size of: ", stringify!(solvedPlay))
        );
        assert_eq!(
            ::std::mem::align_of::<SolvedPlay>(),
            4usize,
            concat!("Alignment of ", stringify!(solvedPlay))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SolvedPlay>())).number as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(solvedPlay),
                "::",
                stringify!(number)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SolvedPlay>())).tricks as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(solvedPlay),
                "::",
                stringify!(tricks)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_playTracesBin() {
        assert_eq!(
            ::std::mem::size_of::<PlayTracesBin>(),
            84004usize,
            concat!("Size of: ", stringify!(playTracesBin))
        );
        assert_eq!(
            ::std::mem::align_of::<PlayTracesBin>(),
            4usize,
            concat!("Alignment of ", stringify!(playTracesBin))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTracesBin>())).noOfBoards as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(playTracesBin),
                "::",
                stringify!(noOfBoards)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTracesBin>())).plays as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(playTracesBin),
                "::",
                stringify!(plays)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_playTracesPBN() {
        assert_eq!(
            ::std::mem::size_of::<PlayTracesPBN>(),
            22404usize,
            concat!("Size of: ", stringify!(playTracesPBN))
        );
        assert_eq!(
            ::std::mem::align_of::<PlayTracesPBN>(),
            4usize,
            concat!("Alignment of ", stringify!(playTracesPBN))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTracesPBN>())).noOfBoards as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(playTracesPBN),
                "::",
                stringify!(noOfBoards)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTracesPBN>())).plays as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(playTracesPBN),
                "::",
                stringify!(plays)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_playTraceBin() {
        assert_eq!(
            ::std::mem::size_of::<PlayTraceBin>(),
            420usize,
            concat!("Size of: ", stringify!(playTraceBin))
        );
        assert_eq!(
            ::std::mem::align_of::<PlayTraceBin>(),
            4usize,
            concat!("Alignment of ", stringify!(playTraceBin))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTraceBin>())).number as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(playTraceBin),
                "::",
                stringify!(number)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTraceBin>())).suit as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(playTraceBin),
                "::",
                stringify!(suit)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTraceBin>())).rank as *const _ as usize },
            212usize,
            concat!(
                "Offset of field: ",
                stringify!(playTraceBin),
                "::",
                stringify!(rank)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_playTracePBN() {
        assert_eq!(
            ::std::mem::size_of::<PlayTracePBN>(),
            112usize,
            concat!("Size of: ", stringify!(playTracePBN))
        );
        assert_eq!(
            ::std::mem::align_of::<PlayTracePBN>(),
            4usize,
            concat!("Alignment of ", stringify!(playTracePBN))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTracePBN>())).number as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(playTracePBN),
                "::",
                stringify!(number)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<PlayTracePBN>())).cards as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(playTracePBN),
                "::",
                stringify!(cards)
            )
        );
    }
}
