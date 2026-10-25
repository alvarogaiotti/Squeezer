// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! File-level types and errors for the PBN importer.
//!
//! The importer produces [`crate::record::Board`]s and keeps only the boards
//! here. PBN's unmodeled tags (the export-format mandatory set and the
//! supplemental tags) are read and dropped (documented lossy), so unlike LIN
//! there is no file-level metadata type: the format contract stores everything
//! that survives on [`crate::record::Board`].

use crate::prelude::*;

/// What went wrong while importing a PBN file.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParsePbnErrorKind {
    /// A tag value did not match the PBN grammar.
    Syntax,
    /// The `[Deal]` tag was missing. Every other identification tag is optional
    /// in PBN import format, and unknown tags are dropped rather than rejected.
    Tag,
    /// A numeric field (board number) did not parse.
    Number,
    /// A `[Deal]` value was not four valid hands, or its first letter was not a
    /// seat.
    Hands,
    /// A supplied `[Dealer]` value was not one of the four seats. A missing
    /// `[Dealer]` falls back to the `[Deal]` first letter instead.
    Dealer,
    /// A supplied `[Vulnerable]` value was not a known vulnerability code. A
    /// missing `[Vulnerable]` defaults to [`Vulnerability::None`] instead.
    Vulnerability,
    /// A `[Contract]` value was not a level, denomination, risk triple.
    Contract,
    /// A `[Declarer]` value was not one of the four seats, or a non-`Pass`
    /// `[Contract]` was present without a declarer.
    Declarer,
    /// An `[Auction]` body carried an unknown call token, or the `[Auction]`
    /// tag value was not a seat, or the leading `-` placeholders did not match
    /// the seats between the first column and the dealer, or a `-` placeholder
    /// had a call after it.
    Bidding,
    /// A `[Play]` card token was not a card, or was the `-` unknown card the
    /// model cannot store.
    Card,
    /// A `[Play]` section was present without a contract, its tag value was not
    /// the contract's leader, or its cards could not be attributed to seats.
    Play,
    /// The parts of a board did not fit together.
    Board,
    /// Parsing stopped before the end of the input.
    Trailing,
}

/// The error returned when a PBN file cannot be imported.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParsePbnError {
    pbn: String,
    kind: ParsePbnErrorKind,
}

impl ParsePbnError {
    pub(crate) fn new(kind: ParsePbnErrorKind, pbn: String) -> Self {
        Self { pbn, kind }
    }

    /// The category of the failure.
    #[must_use]
    pub const fn kind(&self) -> &ParsePbnErrorKind {
        &self.kind
    }
}

impl std::fmt::Display for ParsePbnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unable to parse PBN input ({:?}):\n{}",
            self.kind, self.pbn
        )
    }
}

impl std::error::Error for ParsePbnError {}

/// A parsed PBN file: the boards it records.
#[derive(Debug, Clone)]
pub struct PbnFile {
    /// The boards, in file order.
    pub boards: Vec<Board>,
}

impl PbnFile {
    /// The boards, in file order.
    #[must_use]
    #[inline]
    pub fn boards(&self) -> &[Board] {
        &self.boards
    }
}
