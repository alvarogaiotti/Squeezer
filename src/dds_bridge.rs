// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Adapters from the format-independent core records to the DDS bindings.
//!
//! The `record` module stays free of the solver: the trait implementations
//! that mention `dds` types live here, behind the `dds` feature, so a
//! no-default-features build never links the C++ library.

use crate::prelude::*;
use dds::analyseplay::PlayTraceBin;
use dds::deal::{AsDDSDeal, DDSDealRepr, DdsHandEncoding};
use dds::utils::{BuildSequenceError, RankSeq, SEQUENCE_LENGTH, SuitSeq};

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

impl TryFrom<&PlayRecord> for PlayTraceBin {
    type Error = BuildSequenceError;

    /// Builds DDS's flat suit/rank play trace from the recorded cards.
    fn try_from(record: &PlayRecord) -> Result<Self, Self::Error> {
        let len = record.len();
        if len == 0 {
            return Err(BuildSequenceError::SequenceTooShort);
        }
        if len > SEQUENCE_LENGTH {
            return Err(BuildSequenceError::SequenceTooLong);
        }

        let (suitseq, rankseq): (Vec<_>, Vec<_>) = record
            .cards()
            .map(|card| (i32::from(card.suit() as u8), i32::from(card.rank())))
            .unzip();
        Ok(Self::from_sequences(
            SuitSeq::try_from(suitseq.as_slice())?,
            RankSeq::try_from(rankseq.as_slice())?,
        ))
    }
}
