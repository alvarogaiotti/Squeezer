// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Portable Bridge Notation (PBN) hand records.
//!
//! Only the four-hand `[Deal "N:..."]` record is written here. A full recorded
//! board also needs a `[Board "..."]` line (and the auction and play tags), but
//! the PBN board writer is deferred (design ticket 07), so `Board` composes
//! nothing from this module yet.

use crate::prelude::*;

impl Deal {
    /// Writes the four hands as a PBN `[Deal "N:..."]` hand record.
    ///
    /// Hands are emitted North, East, South, West; each hand lists its suits
    /// spades, hearts, diamonds, clubs, separated by `.` and with ranks high to
    /// low. An empty suit is an empty field, so a void shows as the conventional
    /// leading/interior dot.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_pbn(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        writer.write_str("[Deal \"N:")?;
        for (hand_index, hand) in self.into_iter().enumerate() {
            if hand_index != 0 {
                writer.write_char(' ')?;
            }
            for (suit_index, holding) in hand.into_iter().enumerate() {
                if suit_index != 0 {
                    writer.write_char('.')?;
                }
                for card in holding {
                    writer.write_char(card.rankchar())?;
                }
            }
        }
        writer.write_str("\"]")
    }

    /// Formats the four hands as a PBN `[Deal "N:..."]` hand record.
    #[must_use]
    #[inline]
    pub fn as_pbn(&self) -> String {
        super::written_string(128, |pbn| self.write_pbn(pbn))
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::one_suit_per_seat;

    #[test]
    fn pbn_hand_record_golden_string() {
        // One whole suit per seat: each hand is a single suit, so the other
        // three suits show as the conventional empty `.` fields.
        assert_eq!(
            one_suit_per_seat().as_pbn(),
            "[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]"
        );
    }
}
