// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Adapters from the format-independent core records to the DDS bindings.
//!
//! The `record` module stays free of the solver: the trait implementations
//! that mention `dds` types live here, behind the `dds` feature, so a
//! no-default-features build never links the C++ library.

use crate::prelude::*;
use dds::deal::{AsDDSDeal, DDSDealRepr, DdsHandEncoding};

impl From<Seat> for DdsHandEncoding {
    fn from(value: Seat) -> Self {
        match value {
            Seat::North => Self::North,
            Seat::East => Self::East,
            Seat::South => Self::South,
            Seat::West => Self::West,
        }
    }
}

impl AsDDSDeal for Deal {
    /// Packs the four hands into DDS's 4x4 `rank-bitmask` matrix.
    ///
    /// Callers that mutate a copy of the hands (the payoff simulation) rebuild
    /// a [`Deal`] with [`Deal::from_hands_unchecked`] before converting.
    fn as_dds_deal(&self) -> DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, hand) in self.into_iter().enumerate() {
            for (index, suit) in hand.into_iter().enumerate() {
                remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
            }
        }
        remain_cards.into()
    }
}
