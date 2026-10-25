// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! PBN importer tests: the tracked `tests/*.pbn` fixtures become `Board`s, and
//! malformed input fails at import with a `ParsePbnError` instead of yielding a
//! corrupt board or a silent default.

use std::num::NonZeroU8;
use std::str::FromStr;

use squeezer::pbnparser::{ParsePbnErrorKind, PbnFile, parse_pbn_file};
use squeezer::{
    Board, Call, Card, Contract, Doubled, Hand, Play, Seat, Strain, Vulnerability, Vulnerable,
};

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
        ("tests/auction.pbn", 1),
        ("tests/all_pass.pbn", 1),
        ("tests/truncated_auction.pbn", 1),
        ("tests/full_play.pbn", 1),
        ("tests/short_play.pbn", 1),
        ("tests/annotated.pbn", 1),
        ("tests/multi_board.pbn", 2),
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
    let auction = board.auction().expect("the auction section was parsed");
    assert_eq!(auction.len(), 8);
    assert_eq!(
        auction.calls()[0].call,
        Call::Bid {
            level: NonZeroU8::new(1).expect("1 is non-zero"),
            strain: Strain::Spades,
        }
    );
    assert_eq!(
        board.play().expect("the play section was parsed").plays(),
        &[
            Play::new(Seat::East, Card::HK),
            Play::new(Seat::South, Card::DA),
            Play::new(Seat::West, Card::C2),
            Play::new(Seat::North, Card::SA),
        ]
    );
}

#[test]
fn auction_fixture_derives_the_contract() {
    let board = only_board("tests/auction.pbn");

    assert_eq!(board.number(), Some(6));
    let auction = board.auction().expect("the auction section was parsed");
    assert_eq!(auction.len(), 8);
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
    assert_eq!(board.play(), None);
}

#[test]
fn all_pass_fixture_expands_ap_and_has_no_contract() {
    let board = only_board("tests/all_pass.pbn");

    assert_eq!(board.dealer(), Seat::East);
    let auction = board.auction().expect("the auction section was parsed");
    assert_eq!(auction.len(), 4);
    assert!(
        auction
            .calls()
            .iter()
            .all(|recorded| recorded.call == Call::Pass),
        "AP expands to four passes"
    );
    assert_eq!(board.contract(), None);
}

#[test]
fn truncated_auction_fixture_stops_at_the_star() {
    let board = only_board("tests/truncated_auction.pbn");

    let auction = board.auction().expect("the auction section was parsed");
    assert_eq!(auction.len(), 3, "the tokens after `*` are ignored");
    let contract = board.contract().expect("the auction derives a contract");
    assert_eq!(contract.level(), 2);
    assert_eq!(contract.strain(), Strain::Spades);
    assert_eq!(contract.declarer(), Seat::North);
    assert_eq!(contract.vulnerable(), Vulnerable::No);
}

#[test]
fn full_play_fixture_attributes_every_trick_to_its_winner() {
    let board = only_board("tests/full_play.pbn");

    let plays = board.play().expect("the play section was parsed").plays();
    assert_eq!(plays.len(), 52);
    assert_eq!(plays[0], Play::new(Seat::East, Card::HA));
    assert_eq!(plays[4], Play::new(Seat::North, Card::SK));
    assert_eq!(plays[8], Play::new(Seat::North, Card::SQ));
    assert_eq!(plays[51], Play::new(Seat::West, Card::C2));
    assert_eq!(
        board.contract().expect("contract"),
        Contract::new(
            4,
            Strain::Spades,
            Seat::North,
            Vulnerable::No,
            Doubled::NotDoubled,
        )
    );
}

#[test]
fn short_play_fixture_allows_a_short_final_trick() {
    let board = only_board("tests/short_play.pbn");

    let plays = board.play().expect("the play section was parsed").plays();
    assert_eq!(plays.len(), 5);
    assert_eq!(plays[4], Play::new(Seat::North, Card::SK));
}

#[test]
fn annotated_fixture_ignores_annotations_and_placeholders() {
    let board = only_board("tests/annotated.pbn");

    let auction = board.auction().expect("the auction section was parsed");
    assert_eq!(
        auction
            .calls()
            .iter()
            .map(|recorded| recorded.call)
            .collect::<Vec<_>>(),
        vec![
            Call::Bid {
                level: NonZeroU8::new(1).expect("1 is non-zero"),
                strain: Strain::Spades,
            },
            Call::Pass,
            Call::Bid {
                level: NonZeroU8::new(2).expect("2 is non-zero"),
                strain: Strain::Spades,
            },
            Call::Pass,
            Call::Bid {
                level: NonZeroU8::new(4).expect("4 is non-zero"),
                strain: Strain::Spades,
            },
            Call::Pass,
            Call::Pass,
            Call::Pass,
        ]
    );
    assert_eq!(
        board.play().expect("the play section was parsed").plays(),
        &[
            Play::new(Seat::East, Card::HK),
            Play::new(Seat::South, Card::DA),
            Play::new(Seat::West, Card::C2),
            Play::new(Seat::North, Card::SA),
        ]
    );
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
fn multi_board_fixture_round_trips_board_for_board() {
    let file = parse_fixture("tests/multi_board.pbn");
    assert_eq!(file.boards.len(), 2, "the fixture holds two games");

    // The whole file is round-tripped (not just one `Board::as_pbn`) because
    // the point of a multi-board fixture is the game separator and the order.
    let written = file.as_pbn();
    let reparsed = PbnFile::from_str(&written).expect("the writer's output re-imports");

    assert_eq!(
        reparsed.boards, file.boards,
        "every modeled field survives the round trip:\n{written}"
    );
    assert_eq!(
        reparsed
            .boards
            .iter()
            .map(Board::number)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)],
        "the boards keep their file order:\n{written}"
    );
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
