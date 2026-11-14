// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Importer tests: the `tests/fixtures/*.lin` fixtures become `Board`s, and
//! malformed input fails at import with a `ParseLinError` instead of yielding a
//! corrupt board or a silent default.
//!
//! Fixtures are resolved from the crate manifest directory, so the tests run
//! whatever the process working directory is.

use std::str::FromStr;

use squeezer_cards::{Card, Cards};
use squeezer_io::linparser::{LinFile, ParseLinError, ParseLinErrorKind, Room};
use squeezer_model::{
    Board, Call, Contract, Doubled, Play, Seat, Strain, Vulnerability, Vulnerable,
};

mod common;

fn parse_fixture(path: &str) -> LinFile {
    let data = common::read_fixture(path);
    LinFile::from_str(&data).unwrap_or_else(|error| panic!("{path} should parse: {error}"))
}

fn only_board(path: &str) -> Board {
    let mut boards = parse_fixture(path).boards;
    assert_eq!(boards.len(), 1, "{path} holds one board");
    boards.remove(0)
}

fn parse_lin(lin: &str) -> LinFile {
    LinFile::from_str(lin).unwrap_or_else(|error| panic!("valid LIN should parse: {error}"))
}

fn assert_hand(board: &Board, seat: Seat, hand: &str) {
    assert_eq!(
        board.deal()[seat],
        Cards::from_str(hand).expect("expected hand parses"),
        "{seat} hand mismatch"
    );
}

#[test]
fn parses_every_fixture() {
    for path in [
        "4207070707.lin",
        "4207076395.lin",
        "4207079732.lin",
        "4207083254.lin",
        "4207093356.lin",
        "4207094241.lin",
        "83469.lin",
    ] {
        let file = parse_fixture(path);
        assert!(!file.boards.is_empty(), "{path} has boards");
    }
}

#[test]
fn parses_83469_into_28_boards_with_tournament_metadata() {
    let file = parse_fixture("83469.lin");
    assert_eq!(file.boards.len(), 28);

    let tournament = file
        .metadata
        .tournament_info
        .as_ref()
        .expect("83469 has a vg| line");
    assert_eq!(tournament.tournament, "2nd Super College Cup");
    assert_eq!(tournament.round, "Final 2");
    assert_eq!(tournament.board_start, 15);
    assert_eq!(tournament.board_end, 28);
    assert_eq!(tournament.team1, "NCCU");
    assert_eq!(tournament.team2, "CGU");

    assert_eq!(file.players().players.len(), 8);
    assert_eq!(file.metadata.rooms.len(), file.boards.len());
}

#[test]
fn parses_83469_open_room_board() {
    let board = &parse_fixture("83469.lin").boards[0];

    assert_eq!(board.number(), Some(15));
    assert_eq!(board.dealer(), Seat::South);
    assert_eq!(board.vulnerability(), Vulnerability::NS);
    assert_eq!(board.claim(), None);
    assert_eq!(board.play().expect("board 15 has a full play").len(), 52);

    // LIN lists hands South, West, North, East; the core stores [N, E, S, W].
    assert_hand(board, Seat::North, "SKJ62HKT8642DQ4C3");
    assert_hand(board, Seat::East, "SAT75HAQ3DAKJ53CQ");
    assert_hand(board, Seat::South, "SQ8H9DT87CAKT8765");
    assert_hand(board, Seat::West, "S943HJ75D962CJ942");

    let auction = board.auction().expect("board 15 has an auction");
    assert_eq!(auction.len(), 11);
    assert_eq!(
        auction.calls()[0].call,
        Call::Bid {
            level: std::num::NonZeroU8::new(3).expect("3 is non-zero"),
            strain: Strain::Clubs,
        }
    );

    let contract = board.contract().expect("board 15 has a contract");
    assert_eq!(contract.level(), 4);
    assert_eq!(contract.strain(), Strain::Hearts);
    assert_eq!(contract.declarer(), Seat::West);
    assert_eq!(contract.vulnerable(), Vulnerable::No);
    assert_eq!(
        contract,
        Contract::new(
            4,
            Strain::Hearts,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled
        )
    );

    // The opening leader is declarer's LHO, North.
    let play = board.play().expect("play");
    assert_eq!(play.plays()[0], Play::new(Seat::North, Card::C3));
}

#[test]
fn parses_83469_closed_room_board() {
    let file = parse_fixture("83469.lin");
    assert_eq!(file.metadata.rooms[0], Some(Room::Open));
    assert_eq!(file.metadata.rooms[1], Some(Room::Closed));

    let board = &file.boards[1];
    assert_eq!(board.claim(), Some(8));
    assert_eq!(
        board.contract().expect("board has a contract"),
        Contract::new(
            4,
            Strain::Hearts,
            Seat::West,
            Vulnerable::No,
            Doubled::Doubled
        )
    );
}

#[test]
fn parses_a_claimed_board_with_a_short_final_trick() {
    // Board 16's play is cut off after one card of the last trick, so the final
    // `pg||` group holds a single card.
    let file = parse_fixture("83469.lin");
    let board = &file.boards[2];
    assert_eq!(board.number(), Some(16));
    assert_eq!(board.play().expect("play").len(), 17);
    assert_eq!(board.claim(), Some(11));
}

#[test]
fn parses_4207070707_board() {
    let board = only_board("4207070707.lin");

    assert_eq!(board.number(), Some(1));
    assert_eq!(board.dealer(), Seat::North);
    assert_eq!(board.vulnerability(), Vulnerability::None);
    assert_eq!(board.claim(), Some(8));
    assert_eq!(board.auction().expect("auction").len(), 16);
    assert_eq!(board.play().expect("play").len(), 40);

    assert_hand(&board, Seat::North, "SA754HJ5DT97632C7");
    assert_hand(&board, Seat::East, "SKQH96DKQJ85CT532");
    assert_hand(&board, Seat::South, "ST92HAKT32DCKQJ86");
    assert_hand(&board, Seat::West, "SJ863HQ874DA4CA94");

    assert_eq!(
        board.contract().expect("contract"),
        Contract::new(
            4,
            Strain::Spades,
            Seat::North,
            Vulnerable::No,
            Doubled::NotDoubled
        )
    );
    // Dealer North, so the opening leader is East.
    assert_eq!(
        board.play().expect("play").plays()[0],
        Play::new(Seat::East, Card::SK)
    );
}

#[test]
fn parses_the_4207_fixture_contracts() {
    let cases = [
        (
            "4207076395.lin",
            Contract::new(
                6,
                Strain::Diamonds,
                Seat::North,
                Vulnerable::Yes,
                Doubled::NotDoubled,
            ),
        ),
        (
            "4207079732.lin",
            Contract::new(
                3,
                Strain::NoTrumps,
                Seat::West,
                Vulnerable::Yes,
                Doubled::NotDoubled,
            ),
        ),
        (
            "4207083254.lin",
            Contract::new(
                3,
                Strain::Clubs,
                Seat::South,
                Vulnerable::Yes,
                Doubled::NotDoubled,
            ),
        ),
    ];
    for (path, expected) in cases {
        assert_eq!(only_board(path).contract(), Some(expected), "{path}");
    }
}

#[test]
fn parses_a_passed_out_board() {
    let board = only_board("4207094241.lin");
    assert_eq!(board.dealer(), Seat::East);
    assert_eq!(board.vulnerability(), Vulnerability::EW);
    assert_eq!(board.contract(), None);
    assert_eq!(board.play(), None);
    assert_eq!(board.claim(), None);
    assert_eq!(board.auction().expect("auction").len(), 4);
}

#[test]
fn a_completed_lin_auction_keeps_its_contract() {
    // North 1S and three passes: the auction is complete.
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|");
    let board = &parse_lin(&lin).boards[0];
    let auction = board.auction().expect("the auction was recorded");
    assert_eq!(auction.len(), 4);
    assert!(auction.is_complete());
    assert_eq!(
        board.contract(),
        Some(Contract::new(
            1,
            Strain::Spades,
            Seat::North,
            Vulnerable::No,
            Doubled::NotDoubled,
        ))
    );
}

#[test]
fn a_doubled_lin_auction_completes_after_three_passes() {
    // North 1S, East doubles, then three passes: the double resets the
    // closing pass count, so this is a complete auction.
    let lin = format!("{VALID_HEAD}mb|1S|mb|d|mb|p|mb|p|mb|p|");
    let board = &parse_lin(&lin).boards[0];
    assert_eq!(board.auction().expect("auction").len(), 5);
    assert_eq!(
        board.contract(),
        Some(Contract::new(
            1,
            Strain::Spades,
            Seat::North,
            Vulnerable::No,
            Doubled::Doubled,
        ))
    );
}

#[test]
fn a_cut_lin_auction_is_recorded_as_incomplete() {
    // 1S, pass, 2S and then the record stops: a cut auction, not a completed
    // one, but still a valid board.
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|2S|");
    let board = &parse_lin(&lin).boards[0];
    let auction = board.auction().expect("the auction was recorded");
    assert_eq!(auction.len(), 3);
    assert!(!auction.is_complete());
}

// ============================================================================
// Malformed input fails at import
// ============================================================================

const VALID_HEAD: &str = "pn|a,b,c,d|st||md|3SAKQJT98765432,HAKQJT98765432,DAKQJT98765432,CAKQJT98765432|rh||ah|Board 1|sv|o|";

fn import_error(lin: &str) -> ParseLinError {
    LinFile::from_str(lin).expect_err("malformed LIN must fail")
}

#[test]
fn malformed_bid_is_an_error() {
    let lin = format!("{VALID_HEAD}mb|1X|mb|p|mb|p|mb|p|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Bidding);
}

#[test]
fn malformed_card_is_an_error() {
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|pg||pc|XY|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Card);
}

#[test]
fn malformed_claim_is_an_error() {
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|pg||mc|abc|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Claim);
}

#[test]
fn disagreeing_board_numbers_are_an_error() {
    // `qx|o5|` says board 5, `rh||ah|Board 1|` says board 1.
    let lin = "pn|a,b,c,d|qx|o5|st||md|3SAKQJT98765432,HAKQJT98765432,DAKQJT98765432,CAKQJT98765432|rh||ah|Board 1|sv|o|mb|1S|mb|p|mb|p|mb|p|";
    let error = import_error(lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Number);
}

#[test]
fn malformed_board_number_is_an_error() {
    let lin = "pn|a,b,c,d|st||md|3SAKQJT98765432,HAKQJT98765432,DAKQJT98765432,CAKQJT98765432|rh||ah|Board X|sv|o|mb|1S|mb|p|mb|p|mb|p|";
    let error = import_error(lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Number);
}

#[test]
fn malformed_hands_are_an_error() {
    // South holds only twelve cards, so the deal cannot be a 52-card partition.
    let lin = "pn|a,b,c,d|st||md|3SAKQJT9876543,HAKQJT98765432,DAKQJT98765432|rh||ah|Board 1|sv|o|mb|1S|mb|p|mb|p|mb|p|";
    let error = import_error(lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Hands);
}

#[test]
fn malformed_vugraph_line_is_an_error() {
    let lin = "vg|too,few,fields|\npn|a,b,c,d|st||md|3SAKQJT98765432,HAKQJT98765432,DAKQJT98765432,CAKQJT98765432|rh||ah|Board 1|sv|o|mb|1S|mb|p|mb|p|mb|p|";
    let error = import_error(lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Metadata);
}

#[test]
fn illegal_auction_is_an_error() {
    // An auction cannot open with a double.
    let lin = format!("{VALID_HEAD}mb|d|mb|p|mb|p|mb|p|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Board);
}

#[test]
fn a_bid_after_four_opening_passes_is_an_error() {
    // The auction passed out after four passes; the following 1S cannot turn
    // the board into a 1S contract.
    let lin = format!("{VALID_HEAD}mb|p|mb|p|mb|p|mb|p|mb|1S|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Board);
    assert!(
        error
            .detail()
            .is_some_and(|detail| detail.contains("after the auction")),
        "the error should say the auction ended: {error}"
    );
}

#[test]
fn a_call_after_the_third_closing_pass_is_an_error() {
    // 1S and three passes close the auction; the later 2S is illegal.
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|mb|2S|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Board);
    assert!(
        error
            .detail()
            .is_some_and(|detail| detail.contains("after the auction")),
        "the error should say the auction ended: {error}"
    );
}

#[test]
fn foreign_played_card_is_an_error() {
    // East is on lead (dealer North, contract by North) but holds only clubs,
    // so leading a spade is not in their hand.
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|pg||pc|S2|pc|H2|pc|D2|pc|C2|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Board);
}

#[test]
fn short_non_final_trick_is_an_error() {
    // The first `pg||` group holds two cards, but a short group is only legal
    // for the final (claimed) trick.
    let lin = format!(
        "{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|pg||pc|S2|pc|H2|pg||pc|D2|pc|C2|pc|S3|pc|H3|pg||"
    );
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Card);
}

#[test]
fn trailing_garbage_is_an_error() {
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|not-a-tag|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Trailing);
}

// ============================================================================
// Error causes
// ============================================================================

#[test]
fn unparseable_hand_card_keeps_the_exact_cause() {
    let lin = "pn|a,b,c,d|st||md|3SAKQJT9876543X,HAKQJT98765432,DAKQJT98765432,CAKQJT98765432|rh||ah|Board 1|sv|o|mb|1S|mb|p|mb|p|mb|p|";
    let error = import_error(lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Hands);
    assert_eq!(error.detail(), Some("invalid card \"SAKQJT9876543X\""));
}

#[test]
fn short_hand_keeps_the_exact_cause() {
    let lin = "pn|a,b,c,d|st||md|3SAKQJT9876543,HAKQJT98765432,DAKQJT98765432,CAKQJT98765432|rh||ah|Board 1|sv|o|mb|1S|mb|p|mb|p|mb|p|";
    let error = import_error(lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Hands);
    assert_eq!(error.detail(), Some("S holds 12 cards, expected 13"));
}

#[test]
fn unparseable_played_card_keeps_the_exact_cause() {
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|pg||pc|XY|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Card);
    assert_eq!(error.detail(), Some("invalid card \"XY\""));
}

#[test]
fn board_errors_keep_the_exact_cause() {
    // East is on lead (dealer North, contract by North) but holds only clubs,
    // so leading a spade is not in their hand.
    let lin = format!("{VALID_HEAD}mb|1S|mb|p|mb|p|mb|p|pg||pc|S2|pc|H2|pc|D2|pc|C2|");
    let error = import_error(&lin);
    assert_eq!(error.kind(), &ParseLinErrorKind::Board);
    assert_eq!(
        error.detail(),
        Some("E played 2♠, which is not in their hand")
    );
}

#[test]
fn every_error_kind_renders_as_prose() {
    // The kind is what callers match on, but its `Display` is what a person
    // reads, so every variant spells the failure out instead of printing the
    // variant name.
    let cases = [
        (ParseLinErrorKind::Syntax, "LIN syntax error"),
        (ParseLinErrorKind::Player, "missing or malformed `pn|` line"),
        (ParseLinErrorKind::Hands, "malformed or missing `md|` deal"),
        (ParseLinErrorKind::Number, "malformed numeric field"),
        (ParseLinErrorKind::Bidding, "malformed `mb|` call"),
        (ParseLinErrorKind::Card, "malformed `pc|` card"),
        (ParseLinErrorKind::Claim, "malformed `mc|` claim"),
        (
            ParseLinErrorKind::Vulnerability,
            "unknown `sv|` vulnerability code",
        ),
        (ParseLinErrorKind::Board, "board parts did not fit together"),
        (
            ParseLinErrorKind::Metadata,
            "malformed file-level metadata line",
        ),
        (ParseLinErrorKind::Trailing, "unparsed trailing input"),
    ];
    for (kind, prose) in cases {
        assert_eq!(kind.to_string(), prose, "{kind:?}");
    }
}

#[test]
fn display_includes_the_kind_prose_and_the_detail() {
    let lin = "pn|a,b,c,d|st||md|3SAKQJT9876543X,HAKQJT98765432,DAKQJT98765432,CAKQJT98765432|rh||ah|Board 1|sv|o|mb|1S|mb|p|mb|p|mb|p|";
    let error = import_error(lin);
    // The whole rendering is pinned, input tail included, because the ticket
    // fixes the full error string and the detail must not silently drop out.
    assert_eq!(
        error.to_string(),
        format!(
            "unable to parse LIN input: malformed or missing `md|` deal: \
             invalid card \"SAKQJT9876543X\":\n{lin}"
        )
    );
}
