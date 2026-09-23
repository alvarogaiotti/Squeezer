// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    ddserror::DdsError,
    deal::AsDDSDeal,
    traits::{AsDDSContract, IntoRawDDS},
    utils::{RankSeq, SuitSeq},
};
use core::slice::Iter;

/// Module to keep namespace clean. We use it to disambiguate between [`IntoIter`] which iterates
/// over [`SolvedPlay`] and [`crate::analyseplay::solved_play::IntoIter`], which iterates over
/// single cards' results.
pub mod solved_play {
    use super::SolvedPlay;
    pub struct IntoIter {
        counter: u8,
        tricks: [i32; 53],
    }

    impl IntoIterator for SolvedPlay {
        type Item = i32;
        type IntoIter = IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter {
                counter: 0,
                tricks: self.tricks,
            }
        }
    }

    impl Iterator for IntoIter {
        type Item = i32;

        fn next(&mut self) -> Option<Self::Item> {
            let value = self.tricks[self.counter as usize];
            self.counter += 1;
            if value == -1 { None } else { Some(value) }
        }
    }
}

impl SolvedPlay {
    #[inline]
    #[must_use]
    /// Get the double dummy values for par + all the 52 cards played
    pub const fn tricks(&self) -> &[i32; 53usize] {
        &self.tricks
    }

    #[inline]
    #[must_use]
    /// Double dummy result after the lead.
    pub fn lead_result(&self) -> Option<i32> {
        self.tricks().get(1).copied()
    }

    #[inline]
    #[must_use]
    /// Number of cards played
    pub const fn number(&self) -> i32 {
        self.number
    }

    /// Function for testing purposes. I advise not to use it since you need to provide a Vec of
    /// valid cards for DDS.
    /// # Panics
    /// Panics when seq is empty
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn from_seq(mut seq: Vec<i32>) -> Self {
        assert!(!seq.is_empty());
        let number = seq.len().clamp(1, 53) as i32;
        seq.resize(53, -1i32);
        Self {
            number,
            tricks: seq.try_into().expect("just resized"),
        }
    }

    /// Creates a new [`SolvedPlay`] instance
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
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

impl Default for SolvedPlay {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a SolvedPlay {
    type Item = &'a i32;
    type IntoIter = Iter<'a, i32>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Default for PlayTraceBin {
    #[inline]
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
            suit: suit.into_raw(),
            rank: rank.into_raw(),
        }
    }

    #[inline]
    #[must_use]
    /// Creates a new [`PlayTraceBin`]
    pub const fn new() -> Self {
        Self {
            number: 0,
            suit: [-1i32; 52],
            rank: [-1i32; 52],
        }
    }

    #[must_use]
    #[inline]
    #[allow(clippy::cast_sign_loss)]
    pub fn iter(&self) -> PlayTraceBinIter<'_> {
        let number = self.number as usize;
        PlayTraceBinIter {
            index: 0,
            suit: &self.suit[..number],
            rank: &self.rank[..number],
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlayTraceBinIter<'a> {
    index: usize,
    suit: &'a [i32],
    rank: &'a [i32],
}

impl PlayTraceBinIter<'_> {
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.suit.len()
    }
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> IntoIterator for &'a PlayTraceBin {
    type Item = (&'a i32, &'a i32);
    type IntoIter = PlayTraceBinIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> Iterator for PlayTraceBinIter<'a> {
    type Item = (&'a i32, &'a i32);
    fn next(&mut self) -> Option<Self::Item> {
        // Guaranteed to have the same length
        if let Some(suit) = self.suit.get(self.index) {
            if let Some(rank) = self.rank.get(self.index) {
                self.index += 1;
                Some((suit, rank))
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

/// A trait which can be implemented by any stuct capable of doing
/// DD analysis. Simple interface so we can eventually swap other DD solvers
/// in the future. Kinda like a Strategy Pattern. Now depends on dds for the
/// generics with traits used but the idea is to create marker traits for deals and
/// contracts to swap them in.
///
/// Batch analysis (`AnalyseAllPlaysBin`) was removed with the batch surface;
/// per-hand `AnalysePlayBin` builds its own context and is safe to call
/// concurrently, so our pool drives the parallelism instead.
pub trait PlayAnalyzer {
    /// Analyzes a single hand
    /// # Errors
    /// Will return an Error when DDS fails in some way.
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
        play: PlayTraceBin,
    ) -> Result<SolvedPlay, DdsError>;
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Hash)]
/// DDS struct, saving the DD result for every card played. `number` stores the actual lenght of
/// `tricks`, which is the list of DD result for the cards played.
pub struct SolvedPlay {
    pub number: ::std::os::raw::c_int,
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    pub tricks: [::std::os::raw::c_int; 53usize],
}

#[allow(clippy::cast_sign_loss)]
impl AsRef<[::std::os::raw::c_int]> for SolvedPlay {
    fn as_ref(&self) -> &[::std::os::raw::c_int] {
        &self.tricks[..self.number as usize]
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Hash)]
/// The [`PlayTraceBin`] struct stores two arrays of 52 element each representing played card's rank and
/// suit, then an integer stating the real lenght of the play sequence.
pub struct PlayTraceBin {
    pub number: ::std::os::raw::c_int,
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    pub suit: [::std::os::raw::c_int; 52usize],
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    pub rank: [::std::os::raw::c_int; 52usize],
}

#[allow(clippy::pedantic, unused_imports, deref_nullptr)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn play_trace_bin_iter_yields_rank_not_suit() {
        let mut play = PlayTraceBin::new();
        play.number = 2;
        play.suit[0] = 3;
        play.suit[1] = 1;
        play.rank[0] = 10;
        play.rank[1] = 4;

        let pairs: Vec<(i32, i32)> = play.iter().map(|(suit, rank)| (*suit, *rank)).collect();
        assert_eq!(pairs, vec![(3, 10), (1, 4)]);
    }

    #[test]
    fn play_trace_bin_iter_advances_index() {
        let mut play = PlayTraceBin::new();
        play.number = 3;
        play.suit[..3].copy_from_slice(&[0, 1, 2]);
        play.rank[..3].copy_from_slice(&[14, 13, 12]);

        assert_eq!(play.iter().collect::<Vec<_>>().len(), 3);
    }

    #[test]
    fn bindgen_test_layout_solved_play() {
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
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<SolvedPlay>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).number) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(solvedPlay),
                "::",
                stringify!(number)
            )
        );
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<SolvedPlay>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).tricks) as usize - ptr as usize
            },
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
    fn bindgen_test_layout_play_trace_bin() {
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
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<PlayTraceBin>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).number) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(playTraceBin),
                "::",
                stringify!(number)
            )
        );
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<PlayTraceBin>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).suit) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(playTraceBin),
                "::",
                stringify!(suit)
            )
        );
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<PlayTraceBin>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).rank) as usize - ptr as usize
            },
            212usize,
            concat!(
                "Offset of field: ",
                stringify!(playTraceBin),
                "::",
                stringify!(rank)
            )
        );
    }
}
