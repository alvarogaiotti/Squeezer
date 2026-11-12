// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Adapters from the format-independent model records to the DDS bindings.
//!
//! `squeezer-model` must not name `dds`, so the trait implementations that
//! mention DDS types cannot live there. The types are also foreign to this
//! crate now, so `impl AsDDSDeal for Deal` (and the two `TryFrom` impls) would
//! violate the orphan rule. The adapters below are therefore ordinary local
//! items: a borrowing newtype for each converted model record ([`Card`],
//! [`Deal`], [`Position`], [`PlayRecord`]), plus free functions for the suit,
//! strain and seat enums. The published entry points for the position and play
//! conversions stay free functions, since callers and benchmarks name them
//! directly. The model's `record` modules stay free of the solver, so a
//! no-default-features build never links the C++ library. The DD-table reader
//! goes the other way: [`dd_table_from_dds`] turns a populated DDS table into
//! the seam's backend-free `DdTable`.

use crate::prelude::*;
use crate::{DdTable, Par, ParSide, SolverError};
use dds::analyseplay::PlayTraceBin;
use dds::deal::{
    AsDDSDeal, ConstructDdsDealError, DDSDealRepr, DdsDeal, DdsDealBuilder, DdsHandEncoding,
    DdsRank, DdsSuit,
};
use dds::tables::{DdTableDeal, DdTableResults, ParResults, Populated, VulnerabilityEncoding};
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
/// crate defines. The solver converts a [`Position`] directly, while the DD
/// table path consumes a [`Deal`] through this adapter.
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

impl DdsDealAdapter<'_> {
    /// The same four hands in the shape DDS's DD-table entry point reads.
    ///
    /// A DD table takes only the unplayed cards, so unlike the solve input it
    /// carries no trump, leader or current trick.
    pub(crate) fn to_table_deal(self) -> DdTableDeal {
        DdTableDeal {
            cards: self.as_dds_deal().as_array(),
        }
    }
}

/// Reads a populated DDS table as the seam's backend-free [`DdTable`].
///
/// The seam indexes by [`Strain`] and [`Seat`]; DDS indexes by [`DdsSuit`] and
/// [`DdsHandEncoding`]. Translation goes through the same enums the other
/// adapters use, so the two orderings cannot silently drift apart.
pub(crate) fn dd_table_from_dds(
    results: &DdTableResults<Populated>,
) -> Result<DdTable, SolverError> {
    let mut table = DdTable::empty();
    for strain in Strain::ALL {
        let dds_strain = dds_strain(strain);
        for seat in Seat::iter() {
            let count = results.tricks(dds_strain, dds_hand_encoding(seat));
            let count = u8::try_from(count).map_err(|_| {
                SolverError::Backend(format!(
                    "DDS returned an out-of-range DD-table count: {count}"
                ))
            })?;
            table = table.with(strain, seat, count);
        }
    }
    Ok(table)
}

/// Reads the seam's [`DdTable`] as the populated table DDS par consumes.
///
/// This is the reverse of [`dd_table_from_dds`], and it exists for par: DDS
/// derives par from a table, not from a deal, so a caller that already holds a
/// table must be able to hand it back without recalculating one.
pub(crate) fn dd_table_to_dds(table: &DdTable) -> DdTableResults<Populated> {
    let mut tricks = [[0u8; 4]; 5];
    for strain in Strain::ALL {
        // DDS's `res_table` rows follow its `DdsSuit` encoding, the same order
        // the read side uses.
        let row = dds_strain(strain) as usize;
        for seat in Seat::iter() {
            // Reuse the seat adapter rather than `enumerate()`: a reordered
            // `Seat::iter()` must not silently transpose the table.
            tricks[row][dds_hand_encoding(seat) as usize] = table.tricks(strain, seat);
        }
    }
    DdTableResults::from_tricks(tricks)
}

/// Translates the model's vulnerability into DDS's par encoding.
pub(crate) fn vulnerability_encoding(vulnerability: Vulnerability) -> VulnerabilityEncoding {
    match vulnerability {
        Vulnerability::None => VulnerabilityEncoding::None,
        Vulnerability::NS => VulnerabilityEncoding::Ns,
        Vulnerability::EW => VulnerabilityEncoding::Ew,
        Vulnerability::All => VulnerabilityEncoding::Both,
    }
}

/// Reads DDS's par text as the seam's backend-free [`Par`].
///
/// DDS formats each side as `"<side> <score>"` and `"<side>:<contracts>"`, with
/// the side prefix repeated on both. The seam's [`Par`] already names the side,
/// so the prefix is stripped and the score parsed into points. DDS fills both
/// sides even when neither can make anything; the contract list is then empty.
pub(crate) fn par_from_dds(results: &ParResults) -> Result<Par, SolverError> {
    let side = |index: usize, name: &str| -> Result<ParSide, SolverError> {
        let score_text = results.score_for_side(index).ok_or_else(|| {
            SolverError::Backend(format!("DDS returned no readable par score for {name}"))
        })?;
        let (score_side, points) = score_text.split_once(' ').ok_or_else(|| {
            SolverError::Backend(format!(
                "DDS returned a malformed par score: {score_text:?}"
            ))
        })?;
        // DDS prints the side it is scoring; checking it catches a side swap
        // instead of mislabelling both scores.
        if score_side != name {
            return Err(SolverError::Backend(format!(
                "DDS returned a {score_side} par score for the {name} side"
            )));
        }
        let score = points.parse::<i32>().map_err(|_| {
            SolverError::Backend(format!("DDS returned a non-numeric par score: {points:?}"))
        })?;

        let contracts_text = results.contracts_for_side(index).ok_or_else(|| {
            SolverError::Backend(format!("DDS returned no readable par contracts for {name}"))
        })?;
        let (contracts_side, contracts) = contracts_text.split_once(':').ok_or_else(|| {
            SolverError::Backend(format!(
                "DDS returned a malformed par contract list: {contracts_text:?}"
            ))
        })?;
        if contracts_side != name {
            return Err(SolverError::Backend(format!(
                "DDS returned a {contracts_side} par contract list for the {name} side"
            )));
        }

        Ok(ParSide {
            score,
            contracts: contracts.to_owned(),
        })
    };

    // DDS's side encoding is 0 = NS, 1 = EW (`parResults.parScore`).
    Ok(Par {
        ns: side(0, "NS")?,
        ew: side(1, "EW")?,
    })
}

/// A borrowed [`Position`] read as DDS's raw solver input.
///
/// DDS takes a position through `DdsDealBuilder` rather than a conversion
/// trait, so unlike the card and deal adapters this is a plain local type with
/// a method. It still keeps the translation in one place and gives the free
/// [`position_to_dds_deal`] a name to build on.
#[derive(Debug)]
pub(crate) struct DdsPositionAdapter<'a>(pub(crate) &'a Position);

impl DdsPositionAdapter<'_> {
    /// Builds DDS's raw solver input for a partially played position.
    ///
    /// DDS's `first` names the *leader of the current trick*, not the seat to
    /// play next: it attributes `currentTrickSuit`/`currentTrickRank` entries to
    /// `first`, `first + 1`, ... (`HAND_ID` in `init.cpp`). For an opening lead
    /// the two coincide; for a partial trick the leader is the seat that played
    /// the first card still on the table.
    ///
    /// # Errors
    ///
    /// Returns [`ConstructDdsDealError`] when the position violates a DDS input
    /// invariant (for example a current trick longer than three cards).
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn to_deal(&self) -> Result<DdsDeal, ConstructDdsDealError> {
        let position = self.0;
        let mut remain_cards = [[0u32; 4]; 4];
        for (seat, cards) in position.hands().iter().enumerate() {
            // `Card` stores its rank at `rank + 16 * suit`, so each 16-bit lane
            // of the hand bitboard is already the rank mask DDS expects. Copy
            // four lanes directly instead of walking every set card. Keep the
            // full lane mask rather than `0x7ffc`: malformed rank bits were
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
            .trump(dds_strain(position.trump()))
            .first(dds_hand_encoding(leader))
            .remain_cards(DDSDealRepr::from(remain_cards))
            .current_trick(current_trick)
            .build()
    }
}

/// Builds DDS's raw solver input for a partially played [`Position`].
///
/// # Errors
///
/// Returns [`ConstructDdsDealError`] when the position violates a DDS input
/// invariant (for example a current trick longer than three cards).
pub fn position_to_dds_deal(position: &Position) -> Result<DdsDeal, ConstructDdsDealError> {
    DdsPositionAdapter(position).to_deal()
}

/// A borrowed [`PlayRecord`] read as DDS's flat suit/rank trace.
///
/// Like the position adapter this is a plain local type: DDS's
/// `PlayTraceBin::from_sequences` is a constructor, not a trait to implement.
#[derive(Debug)]
pub(crate) struct DdsPlayRecordAdapter<'a>(pub(crate) &'a PlayRecord);

impl DdsPlayRecordAdapter<'_> {
    /// Builds DDS's play trace, checking the sequence length DDS accepts.
    ///
    /// # Errors
    ///
    /// Returns [`BuildSequenceError`] for an empty record or one longer than
    /// the 52-card sequence DDS accepts.
    pub(crate) fn to_trace(&self) -> Result<PlayTraceBin, BuildSequenceError> {
        let record = self.0;
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
}

/// Builds DDS's flat suit/rank play trace from a recorded [`PlayRecord`].
///
/// # Errors
///
/// Returns [`BuildSequenceError`] for an empty record or one longer than the
/// 52-card sequence DDS accepts.
pub fn play_record_to_trace(record: &PlayRecord) -> Result<PlayTraceBin, BuildSequenceError> {
    DdsPlayRecordAdapter(record).to_trace()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

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

    #[test]
    fn vulnerability_maps_to_the_dds_par_encoding() {
        // DDS documents par/table vulnerability as None 0, Both 1, NS 2,
        // EW 3 (`dll.h`). The model orders `All` second, so copying the
        // discriminant would silently turn "both" into "NS".
        for (vulnerability, expected) in [
            (Vulnerability::None, 0),
            (Vulnerability::All, 1),
            (Vulnerability::NS, 2),
            (Vulnerability::EW, 3),
        ] {
            assert_eq!(vulnerability_encoding(vulnerability) as i32, expected);
        }
    }
}
