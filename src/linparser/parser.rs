// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The LIN importer: nom parsers that build [`crate::record::Board`]s.
//!
//! Every failure is reported as a [`LinError`] carrying a
//! [`ParseLinErrorKind`]; nothing is swallowed with `unwrap_or` defaults, so a
//! malformed file fails at import instead of yielding a corrupt board. The
//! parser attributes played cards to seats by walking the recorded play with
//! the contract's trump and the core `trick_winner` rule, then hands the
//! assembled board to `Board::new` for cross-part validation.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0},
    combinator::opt,
    multi::{many0, many1},
};
use std::num::NonZeroU8;
use std::str::FromStr;

use crate::prelude::*;
use crate::record::play::{attribute_plays, validate_trick_lengths};

use super::types::{
    LinFile, LinMetadata, ParseLinError, ParseLinErrorKind, Players, Results, Room, TournamentInfo,
};

/// The nom error threaded through the parsers: just the category of failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinError {
    pub(crate) kind: ParseLinErrorKind,
}

impl LinError {
    const fn new(kind: ParseLinErrorKind) -> Self {
        Self { kind }
    }
}

impl<'a> nom::error::ParseError<&'a str> for LinError {
    fn from_error_kind(_input: &'a str, _kind: nom::error::ErrorKind) -> Self {
        Self::new(ParseLinErrorKind::Syntax)
    }

    fn append(_input: &'a str, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

type LinResult<'a, O> = IResult<&'a str, O, LinError>;

/// A parser combinator that skips surrounding whitespace.
fn ws<'a, O, F>(mut inner: F) -> impl FnMut(&'a str) -> LinResult<'a, O>
where
    F: Parser<&'a str, Output = O, Error = LinError>,
{
    move |input: &'a str| {
        let (input, _) = multispace0(input)?;
        let (input, result) = inner.parse(input)?;
        let (input, _) = multispace0(input)?;
        Ok((input, result))
    }
}

/// Consumes a `pg||` separator if present.
fn pg(input: &str) -> LinResult<'_, ()> {
    let (input, _) = ws(tag("pg||"))(input)?;
    Ok((input, ()))
}

fn parse_number<T: FromStr>(value: &str) -> Result<T, nom::Err<LinError>> {
    value
        .trim()
        .parse::<T>()
        .map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Number)))
}

// ============================================================================
// File-level lines
// ============================================================================

/// `vg|tournament,round,range,start,end,team1,score1,team2,score2|`
fn parse_vugraph_data(input: &str) -> LinResult<'_, TournamentInfo> {
    let (input, _) = ws(tag("vg|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let parts: Vec<&str> = content.split(',').collect();
    if parts.len() < 9 {
        return Err(nom::Err::Failure(LinError::new(
            ParseLinErrorKind::Metadata,
        )));
    }
    Ok((
        input,
        TournamentInfo {
            tournament: parts[0].to_string(),
            round: parts[1].to_string(),
            board_range: parts[2].to_string(),
            board_start: parse_number::<u8>(parts[3])?,
            board_end: parse_number::<u8>(parts[4])?,
            team1: parts[5].to_string(),
            team1_score: parse_number::<u32>(parts[6])?,
            team2: parts[7].to_string(),
            team2_score: parse_number::<u32>(parts[8])?,
        },
    ))
}

/// `rs|result1,result2,...|`
fn parse_results(input: &str) -> LinResult<'_, Results> {
    let (input, _) = ws(tag("rs|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    Ok((
        input,
        Results {
            results: content.split(',').map(ToString::to_string).collect(),
        },
    ))
}

/// `pn|player1,player2,...|`
fn parse_players(input: &str) -> LinResult<'_, Players> {
    let (input, _) = ws(tag("pn|"))
        .parse(input)
        .map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Player)))?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    Ok((
        input,
        Players {
            players: content.split(',').map(ToString::to_string).collect(),
        },
    ))
}

// ============================================================================
// Per-board lines
// ============================================================================

/// `qx|<room><number>|`; the room is optional metadata, the number may be absent.
fn parse_room(input: &str) -> LinResult<'_, (Option<Room>, Option<u8>)> {
    let (input, _) = ws(tag("qx|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let mut chars = content.chars();
    let room = match chars.next() {
        Some('o' | 'O') => Some(Room::Open),
        Some('c' | 'C') => Some(Room::Closed),
        _ => None,
    };
    let number_str: String = chars.collect();
    let number = if number_str.is_empty() {
        None
    } else {
        Some(parse_number::<u8>(&number_str)?)
    };
    Ok((input, (room, number)))
}

/// `st||`
fn consume_start_table_tag(input: &str) -> LinResult<'_, ()> {
    let (input, _) = ws(tag("st||"))(input)?;
    Ok((input, ()))
}

/// `md|<dealer><hand1>,<hand2>,<hand3>[,<hand4>]|`.
///
/// LIN lists the hands South, West, North, East; the fourth may be omitted and
/// is then the remainder of the deck. The core stores hands `[N, E, S, W]`.
fn parse_dealer_and_hands(input: &str) -> LinResult<'_, (Seat, [Cards; 4])> {
    let (input, _) = ws(tag("md|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let mut chars = content.chars();
    let dealer = match chars.next() {
        Some('1') => Seat::South,
        Some('2') => Seat::West,
        Some('3') => Seat::North,
        Some('4') => Seat::East,
        _ => return Err(nom::Err::Failure(LinError::new(ParseLinErrorKind::Hands))),
    };
    let hands_str: String = chars.collect();

    let parts: Vec<&str> = hands_str.split(',').collect();
    if !(3..=4).contains(&parts.len()) {
        return Err(nom::Err::Failure(LinError::new(ParseLinErrorKind::Hands)));
    }

    let mut listed: Vec<Option<Cards>> = Vec::with_capacity(4);
    for part in &parts {
        if part.is_empty() {
            listed.push(None);
        } else {
            let cards = Cards::from_str(part)
                .map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Hands)))?;
            listed.push(Some(cards));
        }
    }
    if listed.len() == 3 {
        listed.push(None);
    }

    // Subtract the listed hands from the deck, rejecting a card listed twice.
    let mut deck = Cards::ALL;
    for cards in listed.iter().flatten() {
        if !deck.contains(*cards) {
            return Err(nom::Err::Failure(LinError::new(ParseLinErrorKind::Hands)));
        }
        deck -= *cards;
    }

    // At most one hand may be omitted; it is the remaining deck.
    let empty_indices: Vec<usize> = listed
        .iter()
        .enumerate()
        .filter_map(|(index, cards)| cards.is_none().then_some(index))
        .collect();
    if empty_indices.len() > 1 {
        return Err(nom::Err::Failure(LinError::new(ParseLinErrorKind::Hands)));
    }
    if let Some(&index) = empty_indices.first() {
        listed[index] = Some(deck);
    }

    let cards: Vec<Cards> = listed
        .into_iter()
        .map(|cards| cards.expect("exactly one empty slot was filled"))
        .collect();
    // The partition is validated once, in `parse_deal`, where the deal is built.
    let hands = [cards[2], cards[3], cards[0], cards[1]];

    Ok((input, (dealer, hands)))
}

/// `rh||ah|Board N|` (or `ah|Board N|`), the board-number tag that LIN places
/// between `md|` and `sv|`.
fn parse_board_number_tag(input: &str) -> LinResult<'_, u8> {
    let (input, _) = alt((tag("rh||ah|Board "), tag("ah|Board "))).parse(input)?;
    let (input, digits) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;
    Ok((input, parse_number::<u8>(digits)?))
}

/// `sv|<vulnerability>|`
fn parse_vulnerability(input: &str) -> LinResult<'_, Vulnerability> {
    let (input, _) = ws(tag("sv|"))(input)?;
    let (input, code) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let vulnerability = match code {
        "n" => Vulnerability::NS,
        "e" => Vulnerability::EW,
        "o" => Vulnerability::None,
        "b" => Vulnerability::All,
        _ => {
            return Err(nom::Err::Failure(LinError::new(
                ParseLinErrorKind::Vulnerability,
            )));
        }
    };
    Ok((input, vulnerability))
}

/// `mb|<call>[!]|` with an optional following `an|...|` explanation.
fn parse_bid(input: &str) -> LinResult<'_, RecordedCall> {
    let (input, _) = ws(tag("mb|"))(input)?;
    let (input, raw) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let alert = raw.ends_with('!');
    let call = parse_call(raw.trim_end_matches('!'))?;
    let (input, annotation) = opt(parse_annotation).parse(input)?;
    Ok((
        input,
        RecordedCall {
            call,
            alert,
            annotation,
        },
    ))
}

/// `an|<explanation>|`
fn parse_annotation(input: &str) -> LinResult<'_, String> {
    let (input, _) = ws(tag("an|"))(input)?;
    let (input, content) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;
    Ok((input, content.to_string()))
}

/// Maps a LIN call token onto the core [`Call`].
fn parse_call(bid: &str) -> Result<Call, nom::Err<LinError>> {
    match bid {
        "p" | "P" | "pass" | "Pass" => Ok(Call::Pass),
        "d" | "D" | "x" | "X" => Ok(Call::Double),
        "r" | "R" | "xx" | "XX" => Ok(Call::Redouble),
        _ => {
            let mut chars = bid.chars();
            let level = chars.next().and_then(|c| c.to_digit(10));
            let strain = chars.next().and_then(|c| match c {
                'c' | 'C' => Some(Strain::Clubs),
                'd' | 'D' => Some(Strain::Diamonds),
                'h' | 'H' => Some(Strain::Hearts),
                's' | 'S' => Some(Strain::Spades),
                'n' | 'N' => Some(Strain::NoTrumps),
                _ => None,
            });
            // A bid is exactly a level and a strain; reject trailing junk.
            if chars.next().is_some() {
                return Err(nom::Err::Failure(LinError::new(ParseLinErrorKind::Bidding)));
            }
            let level = level
                .and_then(|level| u8::try_from(level).ok())
                .and_then(NonZeroU8::new);
            match (level, strain) {
                (Some(level), Some(strain)) => Ok(Call::Bid { level, strain }),
                _ => Err(nom::Err::Failure(LinError::new(ParseLinErrorKind::Bidding))),
            }
        }
    }
}

/// `pc|<card>|`
fn parse_played_card(input: &str) -> LinResult<'_, Card> {
    let (input, _) = ws(tag("pc|"))(input)?;
    let (input, card) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;

    let card = Card::from_str(card)
        .map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Card)))?;
    Ok((input, card))
}

/// A trick: the cards up to the `pg||` separator (a partial last trick is fine).
fn parse_trick(input: &str) -> LinResult<'_, Vec<Card>> {
    let (input, cards) = many1(parse_played_card).parse(input)?;
    if cards.len() > 4 {
        return Err(nom::Err::Failure(LinError::new(ParseLinErrorKind::Card)));
    }
    let (input, _) = opt(pg).parse(input)?;
    Ok((input, cards))
}

/// `mc|<tricks>|`
fn parse_claim(input: &str) -> LinResult<'_, u8> {
    let (input, _) = ws(tag("mc|"))(input)?;
    let (input, value) = ws(take_until("|"))(input)?;
    let (input, _) = ws(char('|'))(input)?;
    let claim = value
        .trim()
        .parse::<u8>()
        .map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Claim)))?;
    Ok((input, claim))
}

/// A full board: `qx|`, `st||`, `md|`, `rh|`, `sv|`, bids, play and claim.
fn parse_deal(input: &str) -> LinResult<'_, (Board, Option<Room>)> {
    let (input, qx) = opt(parse_room).parse(input)?;
    let (room, qx_number) = qx.unwrap_or((None, None));

    let (input, ()) = consume_start_table_tag(input)?;
    let (input, (dealer, hands)) = parse_dealer_and_hands(input)?;
    let (input, rh_number) = opt(parse_board_number_tag).parse(input)?;
    let (input, vulnerability) = parse_vulnerability(input)?;
    let (input, _) = opt(pg).parse(input)?;

    let (input, calls) = many0(parse_bid).parse(input)?;
    let auction = if calls.is_empty() {
        None
    } else {
        Some(Auction::new(calls))
    };
    let contract = auction
        .as_ref()
        .and_then(|auction| auction.contract(dealer, vulnerability));

    let (input, _) = opt(pg).parse(input)?;
    let (input, groups) = many0(parse_trick).parse(input)?;
    validate_trick_lengths(&groups)
        .map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Card)))?;
    let (input, claim) = opt(parse_claim).parse(input)?;
    let (input, _) = opt(pg).parse(input)?;

    // The `qx|` and `rh||ah|Board N|` numbers must agree when both are present.
    let number = match (qx_number, rh_number) {
        (Some(qx), Some(rh)) if qx != rh => {
            return Err(nom::Err::Failure(LinError::new(ParseLinErrorKind::Number)));
        }
        (Some(qx), _) => Some(qx),
        (None, rh) => rh,
    };
    let deal =
        Deal::new(hands).map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Hands)))?;

    // Play attribution needs the contract's opening leader and trump.
    let play = if groups.is_empty() {
        None
    } else {
        let contract =
            contract.ok_or_else(|| nom::Err::Failure(LinError::new(ParseLinErrorKind::Board)))?;
        Some(
            attribute_plays(&groups, contract)
                .map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Card)))?,
        )
    };

    let board = Board::new(
        deal,
        number,
        dealer,
        vulnerability,
        auction,
        contract,
        play,
        claim,
    )
    .map_err(|_| nom::Err::Failure(LinError::new(ParseLinErrorKind::Board)))?;

    Ok((input, (board, room)))
}

// ============================================================================
// Whole file
// ============================================================================

/// Parses a whole LIN file.
///
/// # Errors
///
/// Returns [`ParseLinError`] when any line is malformed, the parts of a board
/// do not fit together, or parsing stops before the end of the input.
pub fn parse_bridge_file(input: &str) -> Result<LinFile, ParseLinError> {
    match parse_bridge_file_inner(input) {
        Ok((rest, file)) if rest.trim().is_empty() => Ok(file),
        Ok((rest, _)) => Err(ParseLinError::new(
            ParseLinErrorKind::Trailing,
            rest.to_string(),
        )),
        Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
            Err(ParseLinError::new(error.kind, input.to_string()))
        }
        Err(nom::Err::Incomplete(_)) => {
            unreachable!("complete parsers never return Incomplete")
        }
    }
}

fn parse_bridge_file_inner(input: &str) -> LinResult<'_, LinFile> {
    let (input, _) = multispace0(input)?;
    let (input, tournament_info) = opt(parse_vugraph_data).parse(input)?;
    let (input, results) = opt(parse_results).parse(input)?;
    let (input, players) = parse_players(input)?;
    let (input, _) = opt(pg).parse(input)?;
    let (input, parsed) = many1(parse_deal).parse(input)?;
    let (input, _) = opt(pg).parse(input)?;
    let (input, _) = multispace0(input)?;

    let (boards, rooms): (Vec<_>, Vec<_>) = parsed.into_iter().unzip();
    Ok((
        input,
        LinFile {
            metadata: LinMetadata {
                tournament_info,
                results,
                players,
                rooms,
            },
            boards,
        },
    ))
}

impl FromStr for LinFile {
    type Err = ParseLinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_bridge_file(s)
    }
}
