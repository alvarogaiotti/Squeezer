// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The PBN importer: nom parsers that build [`crate::record::Board`]s.
//!
//! PBN is not a line grammar like LIN: a game is a run of `[Name "value"]` tag
//! pairs followed by two *sections* (`[Auction]` and `[Play]`) whose bodies are
//! bare token runs, not tag pairs. The importer therefore scans a game's lines
//! tag by tag and, when a section tag is seen, collects the body tokens until
//! the next tag or the end of the game. Ticket 04 parses those tokens; this
//! ticket only collects them.
//!
//! Malformed identification data is reported as a [`ParsePbnError`] carrying a
//! [`ParsePbnErrorKind`]; nothing is given a silent default, so a bad tag fails
//! at import instead of yielding a corrupt board. The one deliberate exception
//! is a section body: it is collected as opaque tokens (including the `*`
//! terminator and anything after it) and ticket 04 validates it. The
//! identification tags are assembled through `Board::new`, which inherits the
//! core cross-part validation (illegal auctions, contract/auction disagreement
//! and vulnerability disagreement).
//!
//! # Comments and games
//!
//! `%` escape lines, `{ … }` brace comments (which may span lines) and `;`
//! rest-of-line comments are skipped before parsing, with quoted tag values
//! protected from comment stripping. Games are separated by semi-empty lines (a
//! line holding only spaces or tabs); leading and trailing semi-empty lines are
//! ignored.
//!
//! # Missing identification tags
//!
//! PBN import format has no mandatory tags (PBN 2.1 §3.4). `[Deal]` is still
//! required because a board cannot be built without cards, but the other
//! identification tags are optional:
//!
//! - A missing `[Dealer]` falls back to the first seat of the `[Deal]` value,
//!   which PBN's export format defines as the dealer. An explicit `[Dealer]`
//!   still wins, even when it differs from the `[Deal]` first letter.
//! - A missing `[Vulnerable]` means [`Vulnerability::None`].
//!
//! # Documented drops
//!
//! - A `[Declarer]` with no `[Contract]` has no contract to attach to and is
//!   dropped; a `[Contract]` with no `[Declarer]` is an error because the
//!   contract has no seat.
//! - `;` is a rest-of-line comment, as this ticket specifies, not PBN 2.1's
//!   supplemental-section marker. The `[Tag]` lines after it are therefore read
//!   as ordinary tags; first-occurrence-wins keeps the main section's values for
//!   any tag the main section already provided.
//! - Every unmodeled tag (`Event`, `Site`, players, `Result`, `Note`, …) is
//!   dropped.

use nom::{
    IResult, Parser,
    bytes::complete::{take_while, take_while1},
    character::complete::{char, multispace0, multispace1},
    multi::many0,
};
use std::str::FromStr;

use crate::prelude::*;

use super::types::{ParsePbnError, ParsePbnErrorKind, PbnFile};

/// The nom error threaded through the tag parsers: just the category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PbnError {
    pub(crate) kind: ParsePbnErrorKind,
}

impl PbnError {
    const fn new(kind: ParsePbnErrorKind) -> Self {
        Self { kind }
    }
}

impl<'a> nom::error::ParseError<&'a str> for PbnError {
    fn from_error_kind(_input: &'a str, _kind: nom::error::ErrorKind) -> Self {
        Self::new(ParsePbnErrorKind::Syntax)
    }

    fn append(_input: &'a str, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

type PbnResult<'a, O> = IResult<&'a str, O, PbnError>;

fn failure(kind: ParsePbnErrorKind) -> nom::Err<PbnError> {
    nom::Err::Failure(PbnError::new(kind))
}

// ============================================================================
// Comments and line structure
// ============================================================================

/// One source line with its comments blanked out.
///
/// `blank` records whether the *source* line was semi-empty. A line that holds
/// only a comment is not blank: it must not split a game the way a genuinely
/// empty line does.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CleanLine {
    text: String,
    blank: bool,
}

/// Replaces `%` escape lines, `{ … }` comments and `;` comments with spaces,
/// preserving every newline so the line structure survives.
///
/// Markup inside a quoted tag value is literal: a value may contain `;`, `%` or
/// braces without becoming a comment. `"` and `\` are the only escapes.
///
/// `%` is recognised at the first non-whitespace character of a line, slightly
/// more lenient than PBN 2.1's column-0 rule; a `%` after other content is
/// literal and usually fails tag parsing.
fn strip_comments(input: &str) -> Result<Vec<CleanLine>, PbnError> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut line_blank = true;
    let mut at_content_start = true;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_brace = false;
    let mut chars = input.chars().peekable();

    let push_line = |lines: &mut Vec<CleanLine>, current: &mut String, blank: bool| {
        lines.push(CleanLine {
            text: std::mem::take(current),
            blank,
        });
    };

    while let Some(character) = chars.next() {
        // Inside a brace comment everything is discarded, but newlines still
        // separate lines and a line touched by a comment is never a separator.
        if in_brace {
            match character {
                '}' => {
                    in_brace = false;
                    current.push(' ');
                    line_blank = false;
                    at_content_start = false;
                }
                '\n' => {
                    push_line(&mut lines, &mut current, false);
                    line_blank = true;
                    at_content_start = true;
                }
                _ => {
                    current.push(' ');
                    line_blank = false;
                    at_content_start = false;
                }
            }
            continue;
        }

        // Inside a quoted tag value the comment markers are ordinary text.
        if in_string {
            if character == '\n' {
                return Err(PbnError::new(ParsePbnErrorKind::Syntax));
            }
            current.push(character);
            if character == '\\' && !escaped {
                escaped = true;
            } else if character == '"' && !escaped {
                in_string = false;
                escaped = false;
            } else {
                escaped = false;
            }
            continue;
        }

        match character {
            '\n' => {
                push_line(&mut lines, &mut current, line_blank);
                line_blank = true;
                at_content_start = true;
            }
            ' ' | '\t' | '\r' => current.push(character),
            '%' if at_content_start => {
                line_blank = false;
                at_content_start = false;
                while chars.peek().is_some_and(|next| *next != '\n') {
                    chars.next();
                }
            }
            ';' => {
                line_blank = false;
                at_content_start = false;
                while chars.peek().is_some_and(|next| *next != '\n') {
                    chars.next();
                }
            }
            '{' => {
                line_blank = false;
                at_content_start = false;
                in_brace = true;
                current.push(' ');
            }
            '}' => return Err(PbnError::new(ParsePbnErrorKind::Syntax)),
            '"' => {
                line_blank = false;
                at_content_start = false;
                in_string = true;
                escaped = false;
                current.push(character);
            }
            _ => {
                line_blank = false;
                at_content_start = false;
                current.push(character);
            }
        }
    }

    if in_brace || in_string {
        return Err(PbnError::new(ParsePbnErrorKind::Syntax));
    }
    if !current.is_empty() || !line_blank {
        push_line(&mut lines, &mut current, line_blank);
    }

    Ok(lines)
}

/// Groups the source lines into games at semi-empty lines.
///
/// Comment-only lines have empty text but `blank == false`, so they neither
/// join nor split a game.
fn split_games(lines: &[CleanLine]) -> Vec<Vec<&str>> {
    let mut games = Vec::new();
    let mut current = Vec::new();
    for line in lines {
        if line.blank {
            if !current.is_empty() {
                games.push(std::mem::take(&mut current));
            }
        } else if !line.text.trim().is_empty() {
            current.push(line.text.as_str());
        }
    }
    if !current.is_empty() {
        games.push(current);
    }
    games
}

// ============================================================================
// Tag pairs
// ============================================================================

/// A tag value: everything up to the closing `"`, with `\"` and `\\` unescaped.
fn parse_tag_value(input: &str) -> PbnResult<'_, String> {
    let mut value = String::new();
    let mut rest = input;
    loop {
        let (after, chunk) = take_while(|character| character != '"' && character != '\\')(rest)?;
        value.push_str(chunk);
        rest = after;

        let mut chars = rest.chars();
        match chars.next() {
            Some('\\') => match chars.next() {
                Some('"') => value.push('"'),
                Some('\\') => value.push('\\'),
                _ => return Err(failure(ParsePbnErrorKind::Syntax)),
            },
            Some('"') => return Ok((rest, value)),
            Some(_) | None => return Err(failure(ParsePbnErrorKind::Syntax)),
        }
        rest = chars.as_str();
    }
}

/// One `[Name "value"]` pair.
fn parse_tag_pair(input: &str) -> PbnResult<'_, (String, String)> {
    let (input, _) = multispace0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = take_while1(|character: char| character.is_alphanumeric())(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = char('"')(input)?;
    let (input, value) = parse_tag_value(input)?;
    let (input, _) = char('"')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, (name.to_string(), value)))
}

/// Every tag pair on one line. PBN writes one per line, but nothing forbids
/// several, and the writer's output is one per line either way.
fn parse_tag_pairs(input: &str) -> PbnResult<'_, Vec<(String, String)>> {
    let (input, first) = parse_tag_pair(input)?;
    let (input, rest) = many0(parse_tag_pair).parse(input)?;
    let mut pairs = Vec::with_capacity(rest.len() + 1);
    pairs.push(first);
    pairs.extend(rest);
    Ok((input, pairs))
}

// ============================================================================
// Games
// ============================================================================

/// Which section body the following non-tag lines belong to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Auction,
    Play,
}

/// A game's tag pairs plus the body tokens of its two sections.
///
/// The body tokens are collected here so ticket 04 can parse them without
/// rescanning the file. They are opaque to this ticket, including the `*`
/// terminator and anything after it; ticket 04 decides where a body really
/// ends. Tags keep their source order and `tag` returns the first occurrence,
/// which is PBN's duplicate-tag rule.
// TODO(ticket 04): parse `auction`/`play` into `Auction`/`PlayRecord` instead
// of leaving the bodies as raw tokens.
#[derive(Debug, Clone, Default)]
pub(crate) struct PbnGame {
    tags: Vec<(String, String)>,
    pub(crate) auction: Option<Vec<String>>,
    pub(crate) play: Option<Vec<String>>,
}

impl PbnGame {
    /// The first value of `name`; names are case-sensitive and later
    /// occurrences are ignored.
    fn tag(&self, name: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(tag_name, _)| tag_name == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Splits one game's lines into tags and section-body tokens.
fn collect_game(lines: &[&str]) -> Result<PbnGame, PbnError> {
    let mut game = PbnGame::default();
    let mut section = None;
    let mut saw_tag = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('[') {
            let (rest, pairs) =
                parse_tag_pairs(trimmed).map_err(|_| PbnError::new(ParsePbnErrorKind::Syntax))?;
            if !rest.trim().is_empty() {
                return Err(PbnError::new(ParsePbnErrorKind::Syntax));
            }
            for (name, value) in pairs {
                saw_tag = true;
                section = match name.as_str() {
                    "Auction" => {
                        game.auction.get_or_insert_with(Vec::new);
                        Some(Section::Auction)
                    }
                    "Play" => {
                        game.play.get_or_insert_with(Vec::new);
                        Some(Section::Play)
                    }
                    _ => None,
                };
                game.tags.push((name, value));
            }
        } else {
            let tokens = trimmed.split_whitespace().map(str::to_owned);
            match section {
                Some(Section::Auction) => game.auction.get_or_insert_with(Vec::new).extend(tokens),
                Some(Section::Play) => game.play.get_or_insert_with(Vec::new).extend(tokens),
                // A body line with no section is leftover input after the
                // game: report it as trailing rather than as a syntax error in
                // a tag, matching `parse_bridge_file`.
                None if saw_tag => return Err(PbnError::new(ParsePbnErrorKind::Trailing)),
                None => return Err(PbnError::new(ParsePbnErrorKind::Syntax)),
            }
        }
    }

    Ok(game)
}

// ============================================================================
// Identification values
// ============================================================================

/// A single seat letter, either case.
fn parse_seat(value: &str, kind: ParsePbnErrorKind) -> Result<Seat, PbnError> {
    let mut chars = value.trim().chars();
    let seat = chars
        .next()
        .and_then(|character| Seat::try_from(character).ok());
    if chars.next().is_some() {
        return Err(PbnError::new(kind));
    }
    seat.ok_or_else(|| PbnError::new(kind))
}

fn parse_number(value: &str) -> Result<u8, PbnError> {
    value
        .trim()
        .parse()
        .map_err(|_| PbnError::new(ParsePbnErrorKind::Number))
}

/// The `[Vulnerable]` spellings, including PBN's synonyms and lowercase.
fn parse_vulnerability(value: &str) -> Result<Vulnerability, PbnError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" | "love" | "-" => Ok(Vulnerability::None),
        "ns" => Ok(Vulnerability::NS),
        "ew" => Ok(Vulnerability::EW),
        "all" | "both" => Ok(Vulnerability::All),
        _ => Err(PbnError::new(ParsePbnErrorKind::Vulnerability)),
    }
}

/// One PBN hand: four `.`-separated suit fields, ranks in any order.
///
/// The field order is fixed (spades, hearts, diamonds, clubs), so this is the
/// positional grammar [`Cards::from_str`] already understands, with one
/// deliberate difference: a repeated rank is an error here instead of being
/// silently deduplicated by `Cards::insert`.
fn parse_hand(value: &str) -> Result<Hand, PbnError> {
    let fields: Vec<&str> = value.split('.').collect();
    if fields.len() != 4 {
        return Err(PbnError::new(ParsePbnErrorKind::Hands));
    }

    let mut cards = Cards::EMPTY;
    for (suit_index, field) in fields.iter().enumerate() {
        let suit = match suit_index {
            0 => Suit::Spades,
            1 => Suit::Hearts,
            2 => Suit::Diamonds,
            3 => Suit::Clubs,
            _ => unreachable!("the field count was checked"),
        };
        let mut ranks = field.chars().peekable();
        while let Some(character) = ranks.next() {
            let rank = match character {
                'A' | 'a' => 14,
                'K' | 'k' => 13,
                'Q' | 'q' => 12,
                'J' | 'j' => 11,
                'T' | 't' => 10,
                '1' => match ranks.next() {
                    Some('0') => 10,
                    _ => return Err(PbnError::new(ParsePbnErrorKind::Hands)),
                },
                '2'..='9' => u8::try_from(
                    character
                        .to_digit(10)
                        .expect("a decimal digit has a digit value"),
                )
                .expect("a decimal digit is at most 9"),
                _ => return Err(PbnError::new(ParsePbnErrorKind::Hands)),
            };
            let card = Card::new(suit, rank);
            if cards.has_card(card) {
                return Err(PbnError::new(ParsePbnErrorKind::Hands));
            }
            cards = cards.insert(card);
        }
    }

    Hand::try_from(cards).map_err(|_| PbnError::new(ParsePbnErrorKind::Hands))
}

/// The `[Deal]` value: a first seat and four hands clockwise from it.
///
/// At most one hand may be `-`; it is filled from the cards the other three
/// hands leave. The returned seat places the hands and is the dealer fallback
/// when `[Dealer]` is absent; an explicit `[Dealer]` still wins.
fn parse_deal(value: &str) -> Result<(Seat, Deal), PbnError> {
    let (first, hands) = value
        .split_once(':')
        .ok_or_else(|| PbnError::new(ParsePbnErrorKind::Hands))?;
    let first = parse_seat(first, ParsePbnErrorKind::Hands)?;

    let listed: Vec<&str> = hands.split_whitespace().collect();
    if listed.len() != 4 {
        return Err(PbnError::new(ParsePbnErrorKind::Hands));
    }

    let mut hands: [Option<Hand>; 4] = [None; 4];
    for (offset, hand) in listed.iter().enumerate() {
        if *hand == "-" {
            continue;
        }
        hands[(first + offset) as usize] = Some(parse_hand(hand)?);
    }

    let unknown: Vec<usize> = hands
        .iter()
        .enumerate()
        .filter_map(|(index, hand)| hand.is_none().then_some(index))
        .collect();
    if unknown.len() > 1 {
        return Err(PbnError::new(ParsePbnErrorKind::Hands));
    }
    if let Some(&index) = unknown.first() {
        let mut deck = Cards::ALL;
        for hand in hands.iter().flatten() {
            deck -= hand.cards();
        }
        hands[index] =
            Some(Hand::try_from(deck).map_err(|_| PbnError::new(ParsePbnErrorKind::Hands))?);
    }

    let hands = hands.map(|hand| hand.expect("exactly one empty slot was filled"));
    let deal = Deal::new(hands).map_err(|_| PbnError::new(ParsePbnErrorKind::Hands))?;
    Ok((first, deal))
}

/// The `[Contract]` value: `Pass` is handled by the caller; otherwise
/// `<level><denom><risk>`.
fn parse_contract(
    value: &str,
    declarer: Seat,
    vulnerability: Vulnerability,
) -> Result<Contract, PbnError> {
    let mut chars = value.trim().chars();
    let level = chars
        .next()
        .and_then(|character| character.to_digit(10))
        .filter(|level| (1..=7).contains(level))
        .and_then(|level| u8::try_from(level).ok())
        .ok_or_else(|| PbnError::new(ParsePbnErrorKind::Contract))?;

    let strain = match chars
        .next()
        .ok_or_else(|| PbnError::new(ParsePbnErrorKind::Contract))?
        .to_ascii_uppercase()
    {
        'C' => Strain::Clubs,
        'D' => Strain::Diamonds,
        'H' => Strain::Hearts,
        'S' => Strain::Spades,
        'N' => match chars.next() {
            Some(next) if next.eq_ignore_ascii_case(&'T') => Strain::NoTrumps,
            _ => return Err(PbnError::new(ParsePbnErrorKind::Contract)),
        },
        _ => return Err(PbnError::new(ParsePbnErrorKind::Contract)),
    };

    let risk: String = chars.collect();
    let doubled = match risk.to_ascii_uppercase().as_str() {
        "" => Doubled::NotDoubled,
        "X" => Doubled::Doubled,
        "XX" => Doubled::Redoubled,
        _ => return Err(PbnError::new(ParsePbnErrorKind::Contract)),
    };

    Ok(Contract::new(
        level,
        strain,
        declarer,
        Vulnerable::new(vulnerability, declarer),
        doubled,
    ))
}

/// Assembles one game's identification tags into a board.
///
/// The `Auction` and `Play` bodies are deliberately not parsed here, so both
/// stay `None`; ticket 04 fills them in. A `[Contract]` without a `[Declarer]`
/// has no seat to name and is an error, except for `Pass`, which means a
/// passed-out board and therefore carries no contract.
///
/// Import format has no mandatory tags: a missing `[Dealer]` falls back to the
/// `[Deal]` first letter and a missing `[Vulnerable]` to [`Vulnerability::None`].
/// An explicit tag always wins. `[Deal]` itself is still required.
fn build_board(game: &PbnGame) -> Result<Board, PbnError> {
    let (deal_first, deal) = parse_deal(
        game.tag("Deal")
            .ok_or_else(|| PbnError::new(ParsePbnErrorKind::Tag))?,
    )?;
    let dealer = match game.tag("Dealer") {
        Some(value) => parse_seat(value, ParsePbnErrorKind::Dealer)?,
        None => deal_first,
    };
    let vulnerability = match game.tag("Vulnerable") {
        Some(value) => parse_vulnerability(value)?,
        None => Vulnerability::None,
    };
    let number = game.tag("Board").map(parse_number).transpose()?;
    let declarer = game
        .tag("Declarer")
        .map(|value| parse_seat(value, ParsePbnErrorKind::Declarer))
        .transpose()?;

    let contract = match game.tag("Contract").map(str::trim) {
        None => None,
        Some(token) if token.eq_ignore_ascii_case("pass") => None,
        Some(token) => {
            let declarer = declarer.ok_or_else(|| PbnError::new(ParsePbnErrorKind::Declarer))?;
            Some(parse_contract(token, declarer, vulnerability)?)
        }
    };

    // TODO(ticket 04): pass the parsed `game.auction`/`game.play` here, and
    // derive the contract from the auction when the tags are absent. Until then
    // both are `None`, so `Board::new`'s auction cross-checks cannot fire.
    Board::new(
        deal,
        number,
        dealer,
        vulnerability,
        None,
        contract,
        None,
        None,
    )
    .map_err(|_| PbnError::new(ParsePbnErrorKind::Board))
}

// ============================================================================
// Whole file
// ============================================================================

/// Parses a whole PBN file.
///
/// # Errors
///
/// Returns [`ParsePbnError`] when a tag or section is malformed, `[Deal]` is
/// missing, the deal is not a 52-card partition, or parsing stops before the
/// end of the input.
pub fn parse_pbn_file(input: &str) -> Result<PbnFile, ParsePbnError> {
    match parse_pbn_file_inner(input) {
        Ok((rest, file)) if rest.trim().is_empty() => Ok(file),
        // The line scanner consumes the whole input, so a non-empty rest is
        // unreachable today; the arm mirrors `parse_bridge_file`'s contract for
        // any future parser that stops before the end. Leftover *lines* that
        // are not tags or section bodies are reported as `Trailing` while the
        // games are collected.
        Ok((rest, _)) => Err(ParsePbnError::new(
            ParsePbnErrorKind::Trailing,
            rest.to_string(),
        )),
        Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
            Err(ParsePbnError::new(error.kind, input.to_string()))
        }
        Err(nom::Err::Incomplete(_)) => {
            unreachable!("complete parsers never return Incomplete")
        }
    }
}

fn parse_pbn_file_inner(input: &str) -> PbnResult<'_, PbnFile> {
    let lines = strip_comments(input).map_err(nom::Err::Failure)?;
    let games = split_games(&lines);
    if games.is_empty() {
        return Err(failure(ParsePbnErrorKind::Syntax));
    }

    let games = games
        .iter()
        .map(|lines| collect_game(lines))
        .collect::<Result<Vec<_>, _>>()
        .map_err(nom::Err::Failure)?;
    let boards = games
        .iter()
        .map(build_board)
        .collect::<Result<Vec<_>, _>>()
        .map_err(nom::Err::Failure)?;

    Ok(("", PbnFile { boards }))
}

impl FromStr for PbnFile {
    type Err = ParsePbnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_pbn_file(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_SUIT_DEAL: &str =
        "[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]";

    const WIRE_EXAMPLE: &str = "\
[Board \"1\"]
[Dealer \"N\"]
[Vulnerable \"None\"]
[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]
[Declarer \"N\"]
[Contract \"4S\"]
[Auction \"N\"]
1S Pass 2S Pass
4S Pass Pass Pass
[Play \"E\"]
HK DA C2 SA
*
";

    fn identification() -> String {
        format!("[Dealer \"N\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n")
    }

    fn parse(input: &str) -> PbnFile {
        parse_pbn_file(input).unwrap_or_else(|error| panic!("valid PBN should parse: {error}"))
    }

    fn error_kind(input: &str) -> ParsePbnErrorKind {
        parse_pbn_file(input)
            .expect_err("malformed PBN must fail")
            .kind()
            .clone()
    }

    fn collected(input: &str) -> PbnGame {
        let lines = strip_comments(input).expect("comments are well formed");
        let games = split_games(&lines);
        assert_eq!(games.len(), 1, "the input is one game");
        collect_game(&games[0]).expect("the game collects")
    }

    fn assert_one_suit_per_seat(board: &Board) {
        assert_eq!(
            board.deal()[Seat::North],
            Hand::from_str("SAKQJT98765432").expect("hand parses")
        );
        assert_eq!(
            board.deal()[Seat::East],
            Hand::from_str("HAKQJT98765432").expect("hand parses")
        );
        assert_eq!(
            board.deal()[Seat::South],
            Hand::from_str("DAKQJT98765432").expect("hand parses")
        );
        assert_eq!(
            board.deal()[Seat::West],
            Hand::from_str("CAKQJT98765432").expect("hand parses")
        );
    }

    #[test]
    fn deal_only_game_builds_a_board() {
        let board = &parse(&identification()).boards[0];
        assert_eq!(board.number(), None);
        assert_eq!(board.dealer(), Seat::North);
        assert_eq!(board.vulnerability(), Vulnerability::None);
        assert_eq!(board.contract(), None);
        assert_eq!(board.auction(), None);
        assert_eq!(board.play(), None);
        assert_one_suit_per_seat(board);
    }

    #[test]
    fn board_tag_sets_the_number() {
        let pbn = format!("[Board \"12\"]\n{}", identification());
        assert_eq!(parse(&pbn).boards[0].number(), Some(12));
    }

    #[test]
    fn malformed_board_number_is_a_number_error() {
        let pbn = format!("[Board \"twelve\"]\n{}", identification());
        assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Number);
    }

    #[test]
    fn vulnerability_spellings_are_accepted() {
        let cases = [
            ("None", Vulnerability::None),
            ("Love", Vulnerability::None),
            ("-", Vulnerability::None),
            ("NS", Vulnerability::NS),
            ("EW", Vulnerability::EW),
            ("All", Vulnerability::All),
            ("Both", Vulnerability::All),
        ];
        for (spelling, expected) in cases {
            let pbn = format!("[Dealer \"N\"]\n[Vulnerable \"{spelling}\"]\n{ONE_SUIT_DEAL}\n");
            assert_eq!(
                parse(&pbn).boards[0].vulnerability(),
                expected,
                "{spelling}"
            );
        }
    }

    #[test]
    fn malformed_vulnerability_is_a_vulnerability_error() {
        let pbn = format!("[Dealer \"N\"]\n[Vulnerable \"North-South\"]\n{ONE_SUIT_DEAL}\n");
        assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Vulnerability);
    }

    #[test]
    fn dealer_letter_is_case_insensitive() {
        for spelling in ["n", "N"] {
            let pbn = format!("[Dealer \"{spelling}\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n");
            assert_eq!(parse(&pbn).boards[0].dealer(), Seat::North);
        }
    }

    #[test]
    fn malformed_dealer_is_a_dealer_error() {
        for spelling in ["X", "NN", ""] {
            let pbn = format!("[Dealer \"{spelling}\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n");
            assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Dealer, "{spelling:?}");
        }
    }

    #[test]
    fn deal_only_game_falls_back_to_north_and_not_vulnerable() {
        // PBN import format has no mandatory tags: a bare `[Deal]` is a real
        // shape (DDS's example exports), with the deal's first seat acting as
        // the dealer.
        let board = &parse(&format!("{ONE_SUIT_DEAL}\n")).boards[0];
        assert_eq!(board.number(), None);
        assert_eq!(board.dealer(), Seat::North);
        assert_eq!(board.vulnerability(), Vulnerability::None);
        assert_eq!(board.contract(), None);
        assert_one_suit_per_seat(board);
    }

    #[test]
    fn deal_only_east_game_falls_back_to_east() {
        let pbn =
            "[Deal \"E:.AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432 AKQJT98765432...\"]\n";
        let board = &parse(pbn).boards[0];
        assert_eq!(board.dealer(), Seat::East);
        assert_eq!(board.vulnerability(), Vulnerability::None);
        assert_one_suit_per_seat(board);
    }

    #[test]
    fn missing_vulnerability_defaults_to_none() {
        let pbn = format!("[Dealer \"N\"]\n{ONE_SUIT_DEAL}\n");
        assert_eq!(parse(&pbn).boards[0].vulnerability(), Vulnerability::None);
    }

    #[test]
    fn game_without_a_deal_is_a_tag_error() {
        assert_eq!(
            error_kind("[Dealer \"N\"]\n[Vulnerable \"None\"]\n"),
            ParsePbnErrorKind::Tag
        );
    }

    #[test]
    fn lowercase_and_out_of_order_ranks_are_accepted() {
        let pbn = "\
[Dealer \"e\"]
[Vulnerable \"both\"]
[Deal \"e:.akqjt98765432.. ..AKQJT98765432. ...AKQJT98765432 32jkqt987654a...\"]
";
        let board = &parse(pbn).boards[0];
        assert_eq!(board.dealer(), Seat::East);
        assert_eq!(board.vulnerability(), Vulnerability::All);
        assert_one_suit_per_seat(board);
    }

    #[test]
    fn ten_may_be_spelled_with_two_digits() {
        let pbn = "[Dealer \"N\"]\n[Vulnerable \"None\"]\n[Deal \"N:AKQJ1098765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]\n";
        assert_one_suit_per_seat(&parse(pbn).boards[0]);
    }

    #[test]
    fn unknown_hand_is_filled_from_the_remaining_deck() {
        let pbn = "[Dealer \"N\"]\n[Vulnerable \"None\"]\n[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. -\"]\n";
        assert_one_suit_per_seat(&parse(pbn).boards[0]);
    }

    #[test]
    fn more_than_one_unknown_hand_is_a_hands_error() {
        let pbn = "[Dealer \"N\"]\n[Vulnerable \"None\"]\n[Deal \"N:AKQJT98765432... - - ...AKQJT98765432\"]\n";
        assert_eq!(error_kind(pbn), ParsePbnErrorKind::Hands);
    }

    #[test]
    fn duplicate_rank_is_a_hands_error() {
        // North lists the ace twice, so the hand has 14 listed cards but only
        // 13 distinct ones. `Cards` would silently deduplicate it.
        let pbn = "[Dealer \"N\"]\n[Vulnerable \"None\"]\n[Deal \"N:AAKQJT9876543... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]\n";
        assert_eq!(error_kind(pbn), ParsePbnErrorKind::Hands);
    }

    #[test]
    fn card_duplicated_across_hands_is_a_hands_error() {
        // North and East both list the spade ace; the spade two is missing
        // instead. Every hand still has 13 cards, so only `Deal::new`'s
        // duplicate check can reject this.
        let pbn = "[Dealer \"N\"]\n[Vulnerable \"None\"]\n[Deal \"N:AKQJT9876543.2.. A.AKQJT9876543.. ..AKQJT98765432. ...AKQJT98765432\"]\n";
        assert_eq!(error_kind(pbn), ParsePbnErrorKind::Hands);
    }

    #[test]
    fn short_hand_is_a_hands_error() {
        let pbn = "[Dealer \"N\"]\n[Vulnerable \"None\"]\n[Deal \"N:AKQJT9876543... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]\n";
        assert_eq!(error_kind(pbn), ParsePbnErrorKind::Hands);
    }

    #[test]
    fn deal_needs_a_first_letter_a_colon_and_four_hands() {
        for deal in [
            "AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432",
            "N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432.",
            "X:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432",
            "N:AKQJT98765432.. .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432",
        ] {
            let pbn = format!("[Dealer \"N\"]\n[Vulnerable \"None\"]\n[Deal \"{deal}\"]\n");
            assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Hands, "{deal:?}");
        }
    }

    #[test]
    fn explicit_dealer_overrides_the_deal_first_letter() {
        // The DDS fixtures write `N:` regardless of the dealer. When both tags
        // are present the explicit `[Dealer]` wins, while the `[Deal]` letter
        // still places the hands.
        let pbn = format!("[Dealer \"W\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n");
        let board = &parse(&pbn).boards[0];
        assert_eq!(board.dealer(), Seat::West);
        assert_one_suit_per_seat(board);
    }

    #[test]
    fn contract_forms_build_the_expected_contract() {
        let cases = [
            ("4S", 4, Strain::Spades, Doubled::NotDoubled),
            ("3NT", 3, Strain::NoTrumps, Doubled::NotDoubled),
            ("2HX", 2, Strain::Hearts, Doubled::Doubled),
            ("7CXX", 7, Strain::Clubs, Doubled::Redoubled),
            ("4sx", 4, Strain::Spades, Doubled::Doubled),
        ];
        for (token, level, strain, doubled) in cases {
            let pbn = format!(
                "{}[Declarer \"N\"]\n[Contract \"{token}\"]\n",
                identification()
            );
            let contract = parse(&pbn).boards[0]
                .contract()
                .expect("the contract tag built a contract");
            assert_eq!(contract.level(), level, "{token}");
            assert_eq!(contract.strain(), strain, "{token}");
            assert_eq!(contract.declarer(), Seat::North, "{token}");
            assert_eq!(contract.vulnerable(), Vulnerable::No, "{token}");
            // `Contract` exposes no doubled accessor, so compare whole values.
            assert_eq!(
                contract,
                Contract::new(level, strain, Seat::North, Vulnerable::No, doubled),
                "{token}"
            );
        }
    }

    #[test]
    fn contract_vulnerability_is_derived_from_the_board() {
        // The contract carries the declarer's vulnerability under the board's
        // `[Vulnerable]`; there is no separate contract vulnerability in PBN.
        let pbn = format!(
            "[Dealer \"N\"]\n[Vulnerable \"NS\"]\n{ONE_SUIT_DEAL}\n[Declarer \"N\"]\n[Contract \"4S\"]\n"
        );
        let contract = parse(&pbn).boards[0]
            .contract()
            .expect("the contract tag built a contract");
        assert_eq!(contract.vulnerable(), Vulnerable::Yes);

        let pbn = format!(
            "[Dealer \"N\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n[Declarer \"N\"]\n[Contract \"4S\"]\n"
        );
        let contract = parse(&pbn).boards[0]
            .contract()
            .expect("the contract tag built a contract");
        assert_eq!(contract.vulnerable(), Vulnerable::No);
    }

    #[test]
    fn pass_contract_yields_no_contract() {
        for token in ["Pass", "pass"] {
            let pbn = format!(
                "{}[Declarer \"N\"]\n[Contract \"{token}\"]\n",
                identification()
            );
            assert_eq!(parse(&pbn).boards[0].contract(), None, "{token}");
        }
    }

    #[test]
    fn declarer_without_contract_is_dropped() {
        // Documented drop: a declarer names a seat in a contract, so without a
        // `[Contract]` there is nothing to attach it to.
        let pbn = format!("{}[Declarer \"N\"]\n", identification());
        assert_eq!(parse(&pbn).boards[0].contract(), None);
    }

    #[test]
    fn contract_without_declarer_is_a_declarer_error() {
        let pbn = format!("{}[Contract \"4S\"]\n", identification());
        assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Declarer);
    }

    #[test]
    fn malformed_declarer_is_a_declarer_error() {
        let pbn = format!("{}[Declarer \"X\"]\n[Contract \"4S\"]\n", identification());
        assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Declarer);
    }

    #[test]
    fn malformed_contract_is_a_contract_error() {
        for token in ["4X", "8S", "4SXXX", "4", "S4"] {
            let pbn = format!(
                "{}[Declarer \"N\"]\n[Contract \"{token}\"]\n",
                identification()
            );
            assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Contract, "{token:?}");
        }
    }

    #[test]
    fn unknown_tags_are_dropped() {
        let pbn = format!(
            "[Event \"a club game\"]\n[Site \"somewhere\"]\n{}[Result \"4S=\"]\n",
            identification()
        );
        assert_eq!(parse(&pbn).boards.len(), 1);
    }

    #[test]
    fn first_tag_occurrence_wins() {
        let pbn =
            format!("[Dealer \"N\"]\n[Dealer \"E\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n");
        assert_eq!(parse(&pbn).boards[0].dealer(), Seat::North);
    }

    #[test]
    fn tag_names_are_case_sensitive() {
        // `DEAL` is not `Deal`, so the one required tag is still missing.
        let pbn =
            "[DEAL \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]\n";
        assert_eq!(error_kind(pbn), ParsePbnErrorKind::Tag);
    }

    #[test]
    fn tag_values_unescape_quotes_and_backslashes() {
        let (rest, pairs) = parse_tag_pairs("[Note \"a \\\"quoted\\\" value and a \\\\\"]")
            .expect("the tag parses");
        assert!(rest.is_empty());
        assert_eq!(pairs[0].0, "Note");
        assert_eq!(pairs[0].1, "a \"quoted\" value and a \\");
    }

    #[test]
    fn unknown_tag_escape_is_a_syntax_error() {
        let error = parse_tag_pairs("[Note \"a \\q\"]").expect_err("invalid escape");
        assert_eq!(
            error,
            nom::Err::Failure(PbnError::new(ParsePbnErrorKind::Syntax))
        );
    }

    #[test]
    fn comment_forms_are_skipped_without_splitting_the_game() {
        let pbn = "\
% an escape line: [Board \"7\"]
[Dealer \"N\"] ; a rest-of-line comment
{ a brace comment with a tag [Vulnerable \"All\"] }
[Vulnerable \"None\"]
{ a brace comment
  spanning several lines
  with another [Board \"8\"] }
[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]
";
        let board = &parse(pbn).boards[0];
        assert_eq!(board.number(), None);
        assert_eq!(board.vulnerability(), Vulnerability::None);
    }

    #[test]
    fn comment_markers_inside_a_tag_value_are_literal() {
        let lines = strip_comments("[Note \"a ; b % c { d }\"]").expect("quoted markup is literal");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "[Note \"a ; b % c { d }\"]");
    }

    #[test]
    fn unterminated_brace_or_string_is_a_syntax_error() {
        assert_eq!(
            error_kind("[Dealer \"N\" { unterminated"),
            ParsePbnErrorKind::Syntax
        );
        assert_eq!(error_kind("[Dealer \"N"), ParsePbnErrorKind::Syntax);
    }

    #[test]
    fn malformed_tag_line_is_a_syntax_error() {
        for line in ["[Dealer N]", "[Dealer \"N\"", "[Dealer \"N\"] junk", "[]"] {
            let pbn = format!("{line}\n{}", identification());
            assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Syntax, "{line:?}");
        }
    }

    #[test]
    fn auction_and_play_bodies_are_collected_without_being_parsed() {
        let game = collected(WIRE_EXAMPLE);
        assert_eq!(
            game.auction
                .as_ref()
                .expect("the auction body was collected"),
            &vec!["1S", "Pass", "2S", "Pass", "4S", "Pass", "Pass", "Pass"]
        );
        assert_eq!(
            game.play.as_ref().expect("the play body was collected"),
            &vec!["HK", "DA", "C2", "SA", "*"]
        );
        assert_eq!(game.tag("Contract"), Some("4S"));
    }

    #[test]
    fn section_bodies_are_opaque_token_runs_until_the_next_tag() {
        // Ticket 03 only collects bodies; ticket 04 owns the `*` terminator and
        // decides what follows it. The collector must not rescan or interpret
        // these tokens, so an unknown call/card token is kept verbatim.
        let game = collected("[Auction \"N\"]\n1S Pass * trailing\n[Play \"E\"]\nHK - DA\n");
        assert_eq!(
            game.auction.as_ref().expect("auction body"),
            &vec!["1S", "Pass", "*", "trailing"]
        );
        assert_eq!(
            game.play.as_ref().expect("play body"),
            &vec!["HK", "-", "DA"]
        );
    }

    #[test]
    fn a_section_body_ends_at_the_next_tag() {
        let pbn = "\
[Dealer \"N\"]
[Vulnerable \"None\"]
[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]
[Auction \"N\"]
1S Pass
[Contract \"1S\"]
[Declarer \"N\"]
";
        let game = collected(pbn);
        assert_eq!(game.auction.expect("auction body"), vec!["1S", "Pass"]);
        assert_eq!(game.play, None);
        // The board itself is contract-only in this ticket; the auction body
        // is not parsed yet.
        let board = &parse(pbn).boards[0];
        assert_eq!(board.auction(), None);
        assert_eq!(board.contract().expect("contract").level(), 1);
    }

    #[test]
    fn two_games_parse_in_order_with_leading_and_trailing_blank_lines() {
        let first = identification();
        let second = "[Board \"2\"]\n[Dealer \"E\"]\n[Vulnerable \"NS\"]\n[Deal \"E:.AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432 AKQJT98765432...\"]\n";
        // Semi-empty means only spaces/tabs, not only a bare empty line.
        let pbn = format!("  \n\t\n{first}  \t\n{second}\n \n");
        let file = parse(&pbn);
        assert_eq!(file.boards.len(), 2);
        assert_eq!(file.boards[0].dealer(), Seat::North);
        assert_eq!(file.boards[1].dealer(), Seat::East);
        assert_eq!(file.boards[1].number(), Some(2));
    }

    #[test]
    fn trailing_junk_is_a_trailing_error() {
        let pbn = format!("{}not-a-tag\n", identification());
        assert_eq!(error_kind(&pbn), ParsePbnErrorKind::Trailing);
    }

    #[test]
    fn a_comment_only_file_has_no_games() {
        assert_eq!(
            error_kind("% only a comment\n{ and another }\n"),
            ParsePbnErrorKind::Syntax
        );
    }
}
