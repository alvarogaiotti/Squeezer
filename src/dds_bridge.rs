// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Adapters from the format-independent model records to the DDS bindings.
//!
//! `squeezer-model` must not name `dds`, so the trait implementations that
//! mention DDS types cannot live there. The types are also foreign to this
//! crate now, so `impl AsDDSDeal for Deal` (and the two `TryFrom` impls) would
//! violate the orphan rule. The adapters below are ordinary local items: a
//! borrowing newtype for [`Deal`], and free functions for [`Position`] and
//! [`PlayRecord`]. The `record` modules stay free of the solver, so a
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

/// Translates a contract strain into DDS's trump enum.
///
/// This was an `impl From<Strain> for DdsSuit` while `Strain` lived in this
/// crate. `squeezer-model` must not name `dds`, and both the trait and the
/// enum are foreign here now, so the conversion is an ordinary function.
fn dds_strain(strain: Strain) -> DdsSuit {
    strain.as_suit().map_or(DdsSuit::NoTrump, dds_suit)
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

/// Translates a seat into DDS's hand encoding.
///
/// `Seat` lives in `squeezer-model` now, so this is a function rather than an
/// `impl From<Seat> for DdsHandEncoding`: both the trait and the enum are
/// foreign here.
fn dds_hand_encoding(seat: Seat) -> DdsHandEncoding {
    match seat {
        Seat::North => DdsHandEncoding::North,
        Seat::East => DdsHandEncoding::East,
        Seat::South => DdsHandEncoding::South,
        Seat::West => DdsHandEncoding::West,
    }
}

/// A borrowed [`Deal`] that can be read as DDS's rank-bitmask matrix.
///
/// This is a local newtype because `AsDDSDeal` and `Deal` are both foreign:
/// the orphan rule leaves the implementation no home other than a type this
/// crate defines. The solver converts a [`Position`] directly, so this adapter
/// exists for the `Deal`-based path rather than for a current call site.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DdsDealAdapter<'a>(pub(crate) &'a Deal);

impl AsDDSDeal for DdsDealAdapter<'_> {
    /// Packs the four hands into DDS's 4x4 `rank-bitmask` matrix.
    fn as_dds_deal(&self) -> DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, cards) in self.0.into_iter().enumerate() {
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

/// Builds DDS's raw solver input for a partially played [`Position`].
///
/// DDS's `first` names the *leader of the current trick*, not the seat to play
/// next: it attributes `currentTrickSuit`/`currentTrickRank` entries to
/// `first`, `first + 1`, ... (`HAND_ID` in `init.cpp`). For an opening lead the
/// two coincide; for a partial trick the leader is the seat that played the
/// first card still on the table.
///
/// # Errors
///
/// Returns [`ConstructDdsDealError`] when the position violates a DDS input
/// invariant (for example a current trick longer than three cards).
#[allow(clippy::cast_possible_truncation)]
pub fn position_to_dds_deal(position: &Position) -> Result<DdsDeal, ConstructDdsDealError> {
    let mut remain_cards = [[0u32; 4]; 4];
    for (seat, cards) in position.hands().iter().enumerate() {
        // `Card` stores its rank at `rank + 16 * suit`, so each 16-bit lane of
        // the hand bitboard is already the rank mask DDS expects. Copy four
        // lanes directly instead of walking every set card. Keep the full lane
        // mask rather than `0x7ffc`: malformed rank bits were passed through by
        // the old card-by-card conversion too.
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
        .trump(dds_strain(position.trump()))
        .first(dds_hand_encoding(leader))
        .remain_cards(DDSDealRepr::from(remain_cards))
        .current_trick(current_trick)
        .build()
}

/// Builds DDS's flat suit/rank play trace from a recorded [`PlayRecord`].
///
/// # Errors
///
/// Returns [`BuildSequenceError`] for an empty record or one longer than the
/// 52-card sequence DDS accepts.
pub fn play_record_to_trace(record: &PlayRecord) -> Result<PlayTraceBin, BuildSequenceError> {
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
    Ok(PlayTraceBin::from_sequences(
        SuitSeq::try_from(suitseq.as_slice())?,
        RankSeq::try_from(rankseq.as_slice())?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One whole suit per seat, so every row of the DDS matrix is either the
    /// full rank mask (`1 << 2..=1 << 14`) or zero.
    #[test]
    fn deal_adapter_packs_one_rank_mask_per_occupied_suit() {
        let deal = Deal::new([
            Cards::from_str("SAKQJT98765432").expect("spades hand parses"),
            Cards::from_str("HAKQJT98765432").expect("hearts hand parses"),
            Cards::from_str("DAKQJT98765432").expect("diamonds hand parses"),
            Cards::from_str("CAKQJT98765432").expect("clubs hand parses"),
        ])
        .expect("one suit per seat is a valid deal");
        let full_rank_mask: u32 = (2..=14).map(|rank| 1u32 << rank).sum();

        let expected = [
            [full_rank_mask, 0, 0, 0],
            [0, full_rank_mask, 0, 0],
            [0, 0, full_rank_mask, 0],
            [0, 0, 0, full_rank_mask],
        ];
        let packed = DdsDealAdapter(&deal).as_dds_deal();
        assert_eq!(packed.as_array(), expected);
    }
}
