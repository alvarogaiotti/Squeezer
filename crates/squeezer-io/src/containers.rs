// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Writers for the importer-owned file containers.
//!
//! The board and deal writers live in `squeezer-model`; a `LinFile` or
//! `PbnFile` can only be written where its type exists, so these impls live
//! here with the importers. Each one is gated on the feature that defines its
//! container type.

#[cfg(feature = "lin")]
use crate::linparser::{LinFile, Room};
#[cfg(feature = "pbn")]
use crate::pbnparser::PbnFile;
use std::fmt;

/// Renders a container writer into a fresh `String`.
///
/// The two `as_*` methods share this shape; the `expect` is sound because
/// [`fmt::Write`] for [`String`] never fails.
fn written_string(capacity: usize, write: impl FnOnce(&mut String) -> fmt::Result) -> String {
    let mut out = String::with_capacity(capacity);
    write(&mut out).expect("writing to a String is infallible");
    out
}

#[cfg(feature = "lin")]
impl LinFile {
    /// Writes the file metadata and its boards as LIN.
    ///
    /// The importer requires a `pn|` line, so one is always emitted; `vg|` and
    /// `rs|` are emitted only when present. Each board's `qx|` room (kept in
    /// [`LinMetadata`](crate::linparser::LinMetadata), not on the board) is
    /// written before the board itself.
    ///
    /// # Lossy metadata
    ///
    /// `vg|`, `rs|` and `pn|` are comma-separated lists and LIN has no escaping,
    /// so a player name, team or result string that itself contains a comma
    /// re-parses as extra fields. Such values are not round-trippable; the
    /// board data is unaffected.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_lin(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if let Some(tournament) = &self.metadata.tournament_info {
            write!(
                writer,
                "vg|{},{},{},{},{},{},{},{},{}|",
                tournament.tournament,
                tournament.round,
                tournament.board_range,
                tournament.board_start,
                tournament.board_end,
                tournament.team1,
                tournament.team1_score,
                tournament.team2,
                tournament.team2_score,
            )?;
        }

        if let Some(results) = &self.metadata.results {
            write!(writer, "rs|{}|", results.results.join(","))?;
        }

        write!(writer, "pn|{}|", self.metadata.players.players.join(","))?;
        writer.write_str("pg||")?;

        for (index, board) in self.boards.iter().enumerate() {
            if let Some(room) = self.metadata.rooms.get(index).copied().flatten() {
                let code = match room {
                    Room::Open => 'o',
                    Room::Closed => 'c',
                };
                match board.number() {
                    Some(number) => write!(writer, "qx|{code}{number}|")?,
                    None => write!(writer, "qx|{code}|")?,
                }
            }
            board.write_lin(writer)?;
        }

        Ok(())
    }

    /// Formats the file metadata and its boards as LIN.
    #[must_use]
    #[inline]
    pub fn as_lin(&self) -> String {
        written_string(0, |lin| self.write_lin(lin))
    }
}

#[cfg(feature = "pbn")]
impl PbnFile {
    /// Writes every board as PBN, with one empty line between games.
    ///
    /// PBN separates games with a semi-empty line. Each board's own writer
    /// already ends with a newline, so writing one more before every board
    /// after the first produces exactly one blank line and no trailing one.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_pbn(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        for (index, board) in self.boards.iter().enumerate() {
            if index != 0 {
                writer.write_char('\n')?;
            }
            board.write_pbn(writer)?;
        }
        Ok(())
    }

    /// Formats every board as PBN, with one empty line between games.
    #[must_use]
    pub fn as_pbn(&self) -> String {
        let capacity = 512 * self.boards.len().max(1);
        written_string(capacity, |pbn| self.write_pbn(pbn))
    }
}
