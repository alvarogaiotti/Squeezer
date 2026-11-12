// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Adapters from the format-independent core records to the DDS bindings.
//!
//! The `record` module stays free of the solver: the trait implementations
//! that mention `dds` types live here, behind the `dds` feature, so a
//! no-default-features build never links the C++ library.

use crate::prelude::*;
use dds::analyseplay::PlayTraceBin;
use dds::deal::{
    AsDDSDeal, ConstructDdsDealError, DDSDealRepr, DdsDeal, DdsDealBuilder, DdsHandEncoding,
    DdsRank, DdsSuit,
};
use dds::traits::AsDDSCard;
use dds::utils::{BuildSequenceError, RankSeq, SEQUENCE_LENGTH, SuitSeq};

/// Translates a card suit into DDS's trump/suit enum.
///
/// `squeezer-cards` must not name `dds`, so the conversion cannot be an
/// `impl From<Suit> for DdsSuit` on the card type; it lives here, in the one
/// module that is allowed to know both vocabularies.
pub(crate) fn dds_suit(suit: Suit) -> DdsSuit {
    match suit {
        Suit::Spades => DdsSuit::Spades,
        Suit::Hearts => DdsSuit::Hearts,
        Suit::Diamonds => DdsSuit::Diamonds,
        Suit::Clubs => DdsSuit::Clubs,
    }
}

/// A [`Card`] in the shape the DDS deal builder reads.
///
/// `AsDDSCard` is foreign to this crate, so the card type cannot implement it
/// once it lives in `squeezer-cards`. The local newtype carries the
/// implementation instead, and conversion is a copy of the packed card.
#[derive(Clone, Copy)]
struct DdsCardAdapter(Card);

impl AsDDSCard for DdsCardAdapter {
    fn as_card(&self) -> (DdsRank, DdsSuit) {
        // A deck and every parsed hand hold ranks 2..=14; the `JOKER` and
        // `NONCE` sentinels are array fillers that never enter a `Deal`.
        (
            DdsRank::new(i32::from(self.0.rank())).expect("a deck card has a rank in 2..=14"),
            dds_suit(self.0.suit()),
        )
    }
}

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
    /// a [`Deal`] with [`Deal::from_cards_unchecked`] before converting.
    fn as_dds_deal(&self) -> DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, cards) in self.into_iter().enumerate() {
            // `Cards: IntoIterator` yields individual cards, so name the four
            // suits explicitly to keep one DDS row per suit.
            let suits = [
                cards.spades(),
                cards.hearts(),
                cards.diamonds(),
                cards.clubs(),
            ];
            for (index, suit) in suits.into_iter().enumerate() {
                remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
            }
        }
        remain_cards.into()
    }
}

impl TryFrom<&Position> for DdsDeal {
    type Error = ConstructDdsDealError;

    /// Builds DDS's raw solver input for a partially played [`Position`].
    ///
    /// DDS's `first` names the *leader of the current trick*, not the seat to
    /// play next: it attributes `currentTrickSuit`/`currentTrickRank` entries
    /// to `first`, `first + 1`, ... (`HAND_ID` in `init.cpp`). For an opening
    /// lead the two coincide; for a partial trick the leader is the seat that
    /// played the first card still on the table.
    #[allow(clippy::cast_possible_truncation)]
    fn try_from(position: &Position) -> Result<Self, Self::Error> {
        let mut remain_cards = [[0u32; 4]; 4];
        for (seat, cards) in position.hands().iter().enumerate() {
            // `Card` stores its rank at `rank + 16 * suit`, so each 16-bit
            // lane of the hand bitboard is already the rank mask DDS expects.
            // Copy four lanes directly instead of walking every set card. Keep
            // the full lane mask rather than `0x7ffc`: malformed rank bits were
            // passed through by the old card-by-card conversion too.
            const LANE_MASK: u64 = 0xffff;
            let bits = cards.as_bits();
            remain_cards[seat] = [
                (bits & LANE_MASK) as u32,
                ((bits >> 16) & LANE_MASK) as u32,
                ((bits >> 32) & LANE_MASK) as u32,
                ((bits >> 48) & LANE_MASK) as u32,
            ];
        }

        let mut current_trick = [None; 3];
        for (index, play) in position.table().iter().enumerate() {
            current_trick[index] = Some(DdsCardAdapter(play.card));
        }
        let leader = position
            .table()
            .first()
            .map_or(position.to_play(), |play| play.seat);

        DdsDealBuilder::new()
            .trump(position.trump().into())
            .first(leader.into())
            .remain_cards(DDSDealRepr::from(remain_cards))
            .current_trick(current_trick)
            .build()
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
