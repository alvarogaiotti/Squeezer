// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use core::ffi::c_int;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// A DDS struct populated by its Solve* functions.
/// This contains all the data regarding a single situation of play.
/// It represents the result of every card that a player can play at a specific point of a deal
pub struct FutureTricks {
    /// Number of nodes searched
    pub nodes: ::std::os::raw::c_int,
    pub cards: ::std::os::raw::c_int,
    /// Series of suit of cards played
    pub suit: [::std::os::raw::c_int; 13usize],
    /// Series of rank of cards played
    pub rank: [::std::os::raw::c_int; 13usize],
    /// Equivalent cards not used for analysis purposes
    pub equals: [::std::os::raw::c_int; 13usize],
    /// Score of every single card evaluated
    pub score: [::std::os::raw::c_int; 13usize],
}

impl FutureTricks {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        FutureTricks::default()
    }
    #[inline]
    #[must_use]
    pub fn score(&self) -> &[c_int; 13] {
        &self.score
    }
}

impl Default for FutureTricks {
    fn default() -> Self {
        FutureTricks {
            nodes: 0,
            cards: 0,
            suit: [-1; 13],
            rank: [-1; 13],
            equals: [0; 13],
            score: [0; 13],
        }
    }
}

#[cfg(test)]
#[allow(deref_nullptr, clippy::ref_as_ptr)]
mod test {
    use super::FutureTricks;
    #[test]
    fn bindgen_test_layout_future_tricks() {
        assert_eq!(
            ::std::mem::size_of::<FutureTricks>(),
            216usize,
            concat!("Size of: ", stringify!(futureTricks))
        );
        assert_eq!(
            ::std::mem::align_of::<FutureTricks>(),
            4usize,
            concat!("Alignment of ", stringify!(futureTricks))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<FutureTricks>())).nodes as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(futureTricks),
                "::",
                stringify!(nodes)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<FutureTricks>())).cards as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(futureTricks),
                "::",
                stringify!(cards)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<FutureTricks>())).suit as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(futureTricks),
                "::",
                stringify!(suit)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<FutureTricks>())).rank as *const _ as usize },
            60usize,
            concat!(
                "Offset of field: ",
                stringify!(futureTricks),
                "::",
                stringify!(rank)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<FutureTricks>())).equals as *const _ as usize },
            112usize,
            concat!(
                "Offset of field: ",
                stringify!(futureTricks),
                "::",
                stringify!(equals)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<FutureTricks>())).score as *const _ as usize },
            164usize,
            concat!(
                "Offset of field: ",
                stringify!(futureTricks),
                "::",
                stringify!(score)
            )
        );
    }
}
