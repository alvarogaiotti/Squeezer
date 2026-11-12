// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Cross-crate serialization tests: the model writers against the importers.
//!
//! The model crate covers the writer output itself; what can only be tested
//! here is the round trip through `squeezer::linparser` and
//! `squeezer::pbnparser`, which read the same records back.

use std::num::NonZeroU8;

use quickcheck::{Arbitrary, Gen};

use crate::prelude::*;
use crate::record::position::trick_winner;

/// A board that a LIN round trip can represent.
///
/// Not every [`Board`] is representable. LIN derives the contract from the
/// auction (so a contract without an auction, or one that disagrees with
/// it, cannot survive), and it attributes played cards by seat rotation
/// (so an arbitrary seat sequence cannot). The generator builds only boards
/// that fit the format — exactly the domain the property quantifies over.
#[derive(Clone, Debug)]
struct AnyBoard(Board);

fn arbitrary_seat(g: &mut Gen) -> Seat {
    Seat::from(u8::arbitrary(g) % 4)
}

fn arbitrary_vulnerability(g: &mut Gen) -> Vulnerability {
    match u8::arbitrary(g) % 4 {
        0 => Vulnerability::None,
        1 => Vulnerability::NS,
        2 => Vulnerability::EW,
        _ => Vulnerability::All,
    }
}

fn arbitrary_deal(g: &mut Gen) -> Deal {
    let mut deck: Vec<Card> = Cards::ALL.into_iter().collect();
    let mut hands = [Cards::EMPTY; 4];
    for slot in &mut hands {
        let mut cards = Cards::EMPTY;
        for _ in 0..13 {
            let index = usize::arbitrary(g) % deck.len();
            cards = cards.insert(deck.swap_remove(index));
        }
        *slot = cards;
    }
    Deal::new(hands).expect("the four hands partition the deck")
}

/// A legal auction, built one legal call at a time.
fn arbitrary_auction(g: &mut Gen, dealer: Seat) -> Option<Auction> {
    let len = usize::arbitrary(g) % 13;
    if len == 0 {
        return None;
    }

    let mut calls = Vec::with_capacity(len);
    let mut highest: Option<(u8, Strain)> = None;
    let mut last_non_pass: Option<(Call, Seat)> = None;
    for index in 0..len {
        let seat = dealer + index;
        let mut options = vec![Call::Pass];
        for level in 1..=7u8 {
            for strain in Strain::ALL {
                if highest.is_none_or(|best| (level, strain) > best) {
                    options.push(Call::Bid {
                        level: NonZeroU8::new(level).expect("level is 1..=7"),
                        strain,
                    });
                }
            }
        }
        if let Some((Call::Bid { .. }, bidder)) = last_non_pass
            && !seat.is_same_line(&bidder)
        {
            options.push(Call::Double);
        }
        if let Some((Call::Double, doubler)) = last_non_pass
            && !seat.is_same_line(&doubler)
        {
            options.push(Call::Redouble);
        }

        let call = *g.choose(&options).expect("a pass is always legal");
        match call {
            Call::Bid { level, strain } => {
                highest = Some((level.get(), strain));
                last_non_pass = Some((call, seat));
            }
            Call::Double | Call::Redouble => last_non_pass = Some((call, seat)),
            Call::Pass => {}
        }
        calls.push(RecordedCall {
            call,
            alert: bool::arbitrary(g),
            annotation: arbitrary_annotation(g),
        });
    }
    Some(Auction::new(calls))
}

fn arbitrary_annotation(g: &mut Gen) -> Option<String> {
    if !bool::arbitrary(g) {
        return None;
    }
    // The importer trims leading/trailing whitespace and stops at `|`, so
    // the alphabet excludes both.
    let alphabet = ['a', 'b', 'c', 'X', 'Y', 'Z', '0', '1', '9'];
    let len = usize::arbitrary(g) % 8;
    let mut annotation = String::with_capacity(len);
    for _ in 0..len {
        annotation.push(*g.choose(&alphabet).expect("alphabet is non-empty"));
    }
    Some(annotation)
}

/// A play whose seats follow the contract's leader and `trick_winner`, the
/// way the importer attributes them. The final trick may be short, and the
/// play may be empty (which `Board::new` normalises to no play).
fn arbitrary_play(g: &mut Gen, deal: &Deal, contract: Contract) -> PlayRecord {
    let total = usize::arbitrary(g) % 53;

    let mut remaining: [Vec<Card>; 4] = std::array::from_fn(|_| Vec::new());
    for seat in Seat::iter() {
        remaining[seat as usize] = deal[seat].into_iter().collect();
    }

    let mut leader = contract.leader();
    let mut plays = Vec::with_capacity(total);
    while plays.len() < total {
        let mut trick = Vec::new();
        for offset in 0..4usize {
            if plays.len() + trick.len() >= total {
                break;
            }
            let seat = leader + offset;
            let card = remaining[seat as usize]
                .pop()
                .expect("each seat holds one card per trick");
            trick.push(Play::new(seat, card));
        }
        if trick.len() == 4 {
            leader = trick_winner(&trick, contract.strain());
        }
        plays.extend(trick);
    }
    PlayRecord::new(plays)
}

fn arbitrary_board(g: &mut Gen) -> Board {
    let deal = arbitrary_deal(g);
    let dealer = arbitrary_seat(g);
    let vulnerability = arbitrary_vulnerability(g);
    let number = bool::arbitrary(g).then(|| u8::arbitrary(g));
    let auction = arbitrary_auction(g, dealer);
    let contract = auction
        .as_ref()
        .and_then(|auction| auction.contract(dealer, vulnerability));
    let play = contract
        .as_ref()
        .map(|contract| arbitrary_play(g, &deal, *contract));
    let claim = bool::arbitrary(g).then(|| u8::arbitrary(g) % 14);
    Board::new(
        deal,
        number,
        dealer,
        vulnerability,
        auction,
        contract,
        play,
        claim,
    )
    .expect("the generator only builds boards that satisfy the invariants")
}

impl Arbitrary for AnyBoard {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(arbitrary_board(g))
    }
}

/// An auction with every call's LIN-only metadata stripped.
///
/// PBN writes no alert marker and no annotation, and the importer reads
/// note references and suffixes only to discard them, so a call that
/// carries either cannot survive a round trip. Stripping the metadata
/// after [`arbitrary_auction`] reuses its legality rules exactly instead of
/// duplicating them.
fn plain_auction(auction: &Auction) -> Auction {
    Auction::new(
        auction
            .calls()
            .iter()
            .map(|recorded| RecordedCall::new(recorded.call))
            .collect(),
    )
}

/// A board that a PBN round trip can represent.
///
/// Two fields are outside the format, so the generator leaves them absent:
///
/// - `claim` is `None`: PBN has no tag for a claimed trick count (LIN's
///   `mc|`).
/// - Every call is plain (`alert = false`, `annotation = None`): PBN's
///   alert suffix and note references are dropped by the importer, and call
///   annotations are deferred.
///
/// The contract is derived from the auction and the play is built only when
/// a contract exists, mirroring [`arbitrary_board`]. A play needs a
/// contract because `[Play]` names the leader the contract fixes, and
/// deriving the contract keeps the `[Declarer]`/`[Contract]` tags in
/// agreement with the `[Auction]` that the importer re-derives it from. A
/// contract-only board (no auction) is representable, but is not sampled
/// here so the PBN domain matches the LIN generator's.
///
/// A random legal auction almost always contains a bid — four opening
/// passes need four consecutive picks from up to 36 options — so pass-outs
/// would otherwise never be sampled. One auction in four is therefore a
/// fixed pass-out, to exercise the contractless writer and importer paths
/// too.
#[derive(Clone, Debug)]
struct AnyPbnBoard(Board);

fn pbn_representable_board(g: &mut Gen) -> Board {
    let deal = arbitrary_deal(g);
    let dealer = arbitrary_seat(g);
    let vulnerability = arbitrary_vulnerability(g);
    let number = bool::arbitrary(g).then(|| u8::arbitrary(g));
    let auction = if u8::arbitrary(g) % 4 == 0 {
        Some(Auction::new(vec![RecordedCall::new(Call::Pass); 4]))
    } else {
        arbitrary_auction(g, dealer)
    };
    let auction = auction.as_ref().map(plain_auction);
    let contract = auction
        .as_ref()
        .and_then(|auction| auction.contract(dealer, vulnerability));
    let play = contract
        .as_ref()
        .map(|contract| arbitrary_play(g, &deal, *contract));
    Board::new(
        deal,
        number,
        dealer,
        vulnerability,
        auction,
        contract,
        play,
        None,
    )
    .expect("the generator only builds boards that satisfy the invariants")
}

impl Arbitrary for AnyPbnBoard {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(pbn_representable_board(g))
    }
}

#[cfg(feature = "lin")]
#[quickcheck_macros::quickcheck]
fn lin_round_trips_generated_boards(any: AnyBoard) -> bool {
    let board = any.0;
    // The importer requires a `pn|` line, so wrap the board in a one-board
    // file. The room is not part of the board and stays `None`.
    let lin = format!("pn|N,E,S,W|{}", board.as_lin());
    let Ok(file) = <crate::linparser::LinFile as FromStr>::from_str(&lin) else {
        return false;
    };
    file.boards.len() == 1 && file.boards[0] == board
}

/// `parse(write(board)) == board` for every field PBN models.
///
/// A board's own writer is a complete one-game file: it always emits
/// `[Dealer]`, `[Vulnerable]` and `[Deal]`, the three tags an importer
/// needs to build a board, so no wrapper is required (unlike LIN's `pn|`).
///
/// The panics carry the written PBN so a failure says which part of the
/// format broke; the property still returns `bool`, as quickcheck expects,
/// and a failure is never a silent `false`.
#[cfg(feature = "pbn")]
#[quickcheck_macros::quickcheck]
fn pbn_round_trips_generated_boards(any: AnyPbnBoard) -> bool {
    let board = any.0;
    let pbn = board.as_pbn();
    let file = <crate::pbnparser::PbnFile as FromStr>::from_str(&pbn)
        .unwrap_or_else(|error| panic!("the writer's output must re-import: {error}"));
    assert_eq!(
        file.boards.len(),
        1,
        "a board writes exactly one game:\n{pbn}"
    );
    assert_eq!(
        file.boards[0], board,
        "the board must survive a PBN round trip:\n{pbn}"
    );
    true
}

/// A deal with one whole suit per seat, so a golden hand listing is easy to
/// read and each hand exercises three void suits.
///
/// The model crate's own writer tests use the same fixture internally; this
/// copy serves the container and cross-crate round-trip tests that stay here.
fn one_suit_per_seat() -> Deal {
    Deal::new([
        Cards::from_str("SAKQJT98765432").expect("spades hand parses"),
        Cards::from_str("HAKQJT98765432").expect("hearts hand parses"),
        Cards::from_str("DAKQJT98765432").expect("diamonds hand parses"),
        Cards::from_str("CAKQJT98765432").expect("clubs hand parses"),
    ])
    .expect("one suit per seat is a valid deal")
}

#[cfg(feature = "lin")]
#[test]
fn md_digit_reparses_as_the_stored_dealer() {
    use crate::linparser::LinFile;

    // Board 1 rotates to North, but this board stores West as dealer. The
    // writer's model-side assertion lives in `squeezer-model`; what this test
    // adds is that the importer reads the same `md|` digit back as West, not as
    // the North that board 1 would otherwise rotate to.
    let board = Board::builder(one_suit_per_seat(), Seat::West, Vulnerability::None)
        .number(1)
        .build()
        .expect("a bare deal is a valid board");
    let lin = board.as_lin();

    let parsed = <LinFile as FromStr>::from_str(&format!("pn|N,E,S,W|{lin}")).expect("reparses");
    assert_eq!(parsed.boards[0].dealer(), Seat::West);
    assert_eq!(parsed.boards[0].number(), Some(1));
}

#[cfg(feature = "lin")]
#[test]
fn fixture_board_round_trips_through_lin() {
    use crate::linparser::LinFile;
    let data = std::fs::read_to_string("tests/4207070707.lin").expect("fixture is readable");
    let file = <LinFile as FromStr>::from_str(&data).expect("fixture parses");
    let board = &file.boards[0];
    let lin = board.as_lin();
    let reparsed =
        <LinFile as FromStr>::from_str(&format!("pn|a,b,c,d|{lin}")).expect("output reparses");
    assert_eq!(&reparsed.boards[0], board);
}

/// The fixture's metadata is comma-free, so it survives the comma-joined
/// `vg|`/`rs|`/`pn|` lines. This does not claim arbitrary metadata
/// round-trips; see [`comma_in_metadata_is_documented_as_lossy`].
#[cfg(feature = "lin")]
#[test]
fn lin_file_round_trips_the_comma_free_fixture() {
    use crate::linparser::LinFile;
    let data = std::fs::read_to_string("tests/83469.lin").expect("fixture is readable");
    let file = <LinFile as FromStr>::from_str(&data).expect("fixture parses");
    let reparsed =
        <LinFile as FromStr>::from_str(&file.as_lin()).expect("re-serialized file parses");
    assert_eq!(reparsed.boards, file.boards);
    assert_eq!(reparsed.metadata.rooms, file.metadata.rooms);
    assert_eq!(reparsed.metadata.players, file.metadata.players);
    assert_eq!(
        reparsed.metadata.tournament_info,
        file.metadata.tournament_info
    );
    assert_eq!(reparsed.metadata.results, file.metadata.results);
}

/// LIN has no escaping, so a comma inside a `pn|` name splits into extra
/// players on the way back. This pins the documented lossiness rather than
/// pretending the metadata round-trips.
#[cfg(feature = "lin")]
#[test]
fn comma_in_metadata_is_documented_as_lossy() {
    use crate::linparser::{LinFile, LinMetadata, Players};
    let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
        .build()
        .expect("a bare deal is a valid board");
    let file = LinFile {
        metadata: LinMetadata {
            tournament_info: None,
            results: None,
            players: Players {
                players: vec!["Smith, John".to_string(), "Doe".to_string()],
            },
            rooms: vec![None],
        },
        boards: vec![board],
    };
    let reparsed = <LinFile as FromStr>::from_str(&file.as_lin()).expect("file reparses");
    assert_ne!(reparsed.metadata.players, file.metadata.players);
    assert_eq!(reparsed.metadata.players.players.len(), 3);
}
