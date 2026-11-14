// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! File-level metadata and errors for the LIN importer.
//!
//! The importer produces [`squeezer_model::record::Board`]s and keeps only the
//! information that is not part of a board here: tournament details, results,
//! player names and the room each board was played in. `Room` is a LIN notion
//! and deliberately does not leak into the core record types.

use crate::prelude::*;

/// Which room a board was played in (LIN `qx|`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Room {
    /// The open room.
    Open,
    /// The closed room.
    Closed,
}

/// What went wrong while importing a LIN file.
///
/// This is the importer's public taxonomy: [`ParseLinError::kind`] returns it
/// and callers may match on it. It is deliberately public and
/// `#[non_exhaustive]`, mirroring the PBN importer's kind, so a new category
/// can be added without breaking a caller; match with a wildcard arm.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseLinErrorKind {
    /// A tag or token did not match the LIN grammar.
    Syntax,
    /// A `pn|` line was missing or malformed.
    Player,
    /// An `md|` line was missing or its hands were not a valid deal.
    Hands,
    /// A numeric field (board number, claim, tournament score) did not parse.
    Number,
    /// An `mb|` line carried a call that is not a bid, pass, double or redouble.
    Bidding,
    /// A `pc|` line carried an unparseable card.
    Card,
    /// An `mc|` line carried an unparseable claim count.
    Claim,
    /// An `sv|` line carried an unknown vulnerability code.
    Vulnerability,
    /// The parts of a board did not fit together.
    Board,
    /// A file-level line (`vg|`, `rs|`, `pn|`) was malformed.
    Metadata,
    /// Parsing stopped before the end of the input.
    Trailing,
}

impl std::fmt::Display for ParseLinErrorKind {
    // Renders the category on its own, so callers can log or label a failure
    // without the offending input that `ParseLinError` also carries.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Each arm restates the variant's doc comment; where one LIN tag
        // identifies the bad line, the message names it so the reader can find
        // it in the file they supplied.
        let text = match self {
            Self::Syntax => "LIN syntax error",
            Self::Player => "missing or malformed `pn|` line",
            Self::Hands => "malformed or missing `md|` deal",
            Self::Number => "malformed numeric field",
            Self::Bidding => "malformed `mb|` call",
            Self::Card => "malformed `pc|` card",
            Self::Claim => "malformed `mc|` claim",
            Self::Vulnerability => "unknown `sv|` vulnerability code",
            Self::Board => "board parts did not fit together",
            Self::Metadata => "malformed file-level metadata line",
            Self::Trailing => "unparsed trailing input",
        };
        f.write_str(text)
    }
}

/// The error returned when a LIN file cannot be imported.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseLinError {
    lin: String,
    kind: ParseLinErrorKind,
    detail: Option<String>,
}

impl ParseLinError {
    pub(crate) fn new(kind: ParseLinErrorKind, lin: String) -> Self {
        Self {
            lin,
            kind,
            detail: None,
        }
    }

    /// Builds the public error, carrying the parser's typed cause when one
    /// reached the top level.
    pub(crate) fn with_detail(
        kind: ParseLinErrorKind,
        lin: String,
        detail: Option<String>,
    ) -> Self {
        Self { lin, kind, detail }
    }

    /// The category of the failure.
    #[must_use]
    pub const fn kind(&self) -> &ParseLinErrorKind {
        &self.kind
    }

    /// The exact message explaining the failure, when the parser recorded one.
    ///
    /// It is usually the underlying model error's own wording, such as
    /// `"S holds 12 cards, expected 13"` for a short hand, but a parser rule
    /// may supply the message too.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl std::fmt::Display for ParseLinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to parse LIN input: {}", self.kind)?;
        if let Some(detail) = &self.detail {
            write!(f, ": {detail}")?;
        }
        write!(f, ":\n{}", self.lin)
    }
}

impl std::error::Error for ParseLinError {}

/// A parsed LIN file: its metadata and the boards it records.
#[derive(Debug, Clone)]
pub struct LinFile {
    /// File-level metadata shared by the boards.
    pub metadata: LinMetadata,
    /// The boards, in file order.
    pub boards: Vec<Board>,
}

impl LinFile {
    /// The player names declared by the `pn|` line.
    #[must_use]
    #[inline]
    pub fn players(&self) -> &Players {
        &self.metadata.players
    }
}

/// Metadata that is not part of an individual [`Board`].
///
/// `rooms` is parallel to [`LinFile::boards`]: `rooms[i]` is the room recorded
/// for `boards[i]`, or `None` when the board carried no `qx|` line. It lives in
/// the metadata rather than on [`Board`] because a room is a LIN notion, not a
/// property of the played board.
#[derive(Debug, Clone)]
pub struct LinMetadata {
    /// The `vg|` tournament line, if present.
    pub tournament_info: Option<TournamentInfo>,
    /// The `rs|` results line, if present.
    pub results: Option<Results>,
    /// The `pn|` player names.
    pub players: Players,
    /// One entry per board, in board order.
    pub rooms: Vec<Option<Room>>,
}

/// The `vg|` tournament line.
#[derive(Debug, Clone, PartialEq)]
pub struct TournamentInfo {
    /// Tournament name.
    pub tournament: String,
    /// Round name.
    pub round: String,
    /// Board range label.
    pub board_range: String,
    /// First board number of the range.
    pub board_start: u8,
    /// Last board number of the range.
    pub board_end: u8,
    /// First team's name.
    pub team1: String,
    /// First team's score.
    pub team1_score: u32,
    /// Second team's name.
    pub team2: String,
    /// Second team's score.
    pub team2_score: u32,
}

/// The `rs|` results line.
#[derive(Debug, Clone, PartialEq)]
pub struct Results {
    /// One result string per board.
    pub results: Vec<String>,
}

/// The `pn|` player names.
#[derive(Debug, Clone, PartialEq)]
pub struct Players {
    /// Player names, in file order.
    pub players: Vec<String>,
}

impl Players {
    /// Iterates over the player names.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.players.iter()
    }
}
