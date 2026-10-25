// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! PBN importer tests: the tracked `tests/*.pbn` fixtures become `Board`s, and
//! malformed input fails at import with a `ParsePbnError` instead of yielding a
//! corrupt board or a silent default.

use std::str::FromStr;

use squeezer::pbnparser::{ParsePbnErrorKind, PbnFile, parse_pbn_file};
use squeezer::{Board, Contract, Doubled, Hand, Seat, Strain, Vulnerability, Vulnerable};

fn parse_fixture(path: &str) -> PbnFile {
    let data = std::fs::read_to_string(path).expect("fixture is readable");
    PbnFile::from_str(&data).unwrap_or_else(|error| panic!("{path} should parse: {error}"))
}

fn only_board(path: &str) -> Board {
    let mut boards = parse_fixture(path).boards;
    assert_eq!(boards.len(), 1, "{path} holds one board");
    boards.remove(0)
}

fn assert_hand(board: &Board, seat: Seat, hand: &str) {
    assert_eq!(
        board.deal()[seat],
        Hand::from_str(hand).expect("expected hand parses"),
        "{seat} hand mismatch"
    );
}

fn assert_one_suit_per_seat(board: &Board) {
    assert_hand(board, Seat::North, "SAKQJT98765432");
    assert_hand(board, Seat::East, "HAKQJT98765432");
    assert_hand(board, Seat::South, "DAKQJT98765432");
    assert_hand(board, Seat::West, "CAKQJT98765432");
}

#[test]
fn parses_every_valid_fixture() {
    for (path, boards) in [
        ("tests/deal_only.pbn", 1),
        ("tests/numbered_vulnerable.pbn", 1),
        ("tests/contract.pbn", 1),
        ("tests/unknown_hand.pbn", 1),
        ("tests/lowercase.pbn", 1),
        ("tests/two_games.pbn", 2),
        ("tests/wire_example.pbn", 1),
    ] {
        assert_eq!(parse_fixture(path).boards.len(), boards, "{path}");
    }
}

#[test]
fn deal_only_fixture_has_no_number_or_contract() {
    let board = only_board("tests/deal_only.pbn");

    assert_eq!(board.number(), None);
    assert_eq!(board.dealer(), Seat::North);
    assert_eq!(board.vulnerability(), Vulnerability::None);
    assert_eq!(board.contract(), None);
    assert_eq!(board.auction(), None);
    assert_eq!(board.play(), None);
    assert_one_suit_per_seat(&board);
}

#[test]
fn numbered_vulnerable_fixture_carries_its_identification() {
    let board = only_board("tests/numbered_vulnerable.pbn");

    assert_eq!(board.number(), Some(12));
    assert_eq!(board.dealer(), Seat::West);
    assert_eq!(board.vulnerability(), Vulnerability::All);
    assert_one_suit_per_seat(&board);
}

#[test]
fn contract_fixture_builds_a_doubled_contract() {
    let board = only_board("tests/contract.pbn");

    assert_eq!(board.number(), Some(3));
    assert_eq!(board.dealer(), Seat::North);
    assert_eq!(board.vulnerability(), Vulnerability::NS);
    assert_eq!(
        board.contract(),
        Some(Contract::new(
            4,
            Strain::Spades,
            Seat::North,
            Vulnerable::Yes,
            Doubled::Doubled,
        ))
    );
    assert_one_suit_per_seat(&board);
}

#[test]
fn unknown_hand_fixture_is_filled_from_the_deck() {
    let board = only_board("tests/unknown_hand.pbn");

    assert_eq!(board.number(), Some(4));
    assert_eq!(board.vulnerability(), Vulnerability::EW);
    assert_one_suit_per_seat(&board);
}

#[test]
fn lowercase_fixture_accepts_lowercase_and_out_of_order_ranks() {
    let board = only_board("tests/lowercase.pbn");

    assert_eq!(board.number(), Some(5));
    assert_eq!(board.dealer(), Seat::East);
    assert_eq!(board.vulnerability(), Vulnerability::All);
    assert_one_suit_per_seat(&board);
}

#[test]
fn two_game_fixture_parses_in_file_order() {
    let file = parse_fixture("tests/two_games.pbn");

    assert_eq!(file.boards.len(), 2);
    assert_eq!(file.boards[0].number(), Some(1));
    assert_eq!(file.boards[0].dealer(), Seat::North);
    assert_eq!(file.boards[0].vulnerability(), Vulnerability::None);
    assert_eq!(file.boards[1].number(), Some(2));
    assert_eq!(file.boards[1].dealer(), Seat::East);
    assert_eq!(file.boards[1].vulnerability(), Vulnerability::NS);
}

#[test]
fn wire_example_fixture_is_accepted() {
    let board = only_board("tests/wire_example.pbn");

    assert_eq!(board.number(), Some(1));
    assert_eq!(board.dealer(), Seat::North);
    assert_eq!(board.vulnerability(), Vulnerability::None);
    assert_one_suit_per_seat(&board);
    assert_eq!(
        board.contract(),
        Some(Contract::new(
            4,
            Strain::Spades,
            Seat::North,
            Vulnerable::No,
            Doubled::NotDoubled,
        ))
    );
    // The auction and play sections are ticket 04's; this ticket accepts them
    // and leaves both out of the board after collecting their bodies.
    assert_eq!(board.auction(), None);
    assert_eq!(board.play(), None);
}

#[test]
fn malformed_fixture_is_a_hands_error() {
    let data = std::fs::read_to_string("tests/malformed.pbn").expect("fixture is readable");
    let error = parse_pbn_file(&data).expect_err("a short hand cannot be a deal");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Hands);
}

// ============================================================================
// Writer and malformed input
// ============================================================================

#[test]
fn pbn_file_writer_separates_games_with_one_empty_line() {
    let file = parse_fixture("tests/two_games.pbn");
    let written = file.as_pbn();

    assert_eq!(
        written.matches("\n\n").count(),
        1,
        "exactly one empty line between the two games:\n{written}"
    );
    assert!(!written.ends_with("\n\n"), "no trailing empty line");

    let reparsed = PbnFile::from_str(&written).expect("the writer's output re-imports");
    assert_eq!(reparsed.boards, file.boards);
}

#[test]
fn trailing_garbage_is_an_error() {
    let pbn = [
        "[Dealer \"N\"]",
        "[Vulnerable \"None\"]",
        "[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]",
        "not-a-tag",
    ]
    .join("\n");
    let error = parse_pbn_file(&pbn).expect_err("trailing input must fail");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Trailing);
}
