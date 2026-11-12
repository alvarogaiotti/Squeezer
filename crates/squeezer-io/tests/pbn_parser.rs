// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! PBN importer tests: the tracked `tests/fixtures/*.pbn` fixtures become
//! `Board`s, and malformed input fails at import with a `ParsePbnError` instead
//! of yielding a corrupt board or a silent default.
//!
//! Fixtures are resolved from the crate manifest directory, so the tests run
//! whatever the process working directory is.

use std::num::NonZeroU8;
use std::str::FromStr;

use squeezer_cards::{Card, Cards};
use squeezer_io::pbnparser::{ParsePbnErrorKind, PbnFile, parse_pbn_file};
use squeezer_model::{
    Board, Call, Contract, Doubled, Play, Seat, Strain, Vulnerability, Vulnerable,
};

mod common;

fn parse_fixture(path: &str) -> PbnFile {
    let data = common::read_fixture(path);
    PbnFile::from_str(&data).unwrap_or_else(|error| panic!("{path} should parse: {error}"))
}

fn parse(input: &str) -> PbnFile {
    PbnFile::from_str(input).unwrap_or_else(|error| panic!("valid PBN should parse: {error}"))
}

fn only_board(path: &str) -> Board {
    let mut boards = parse_fixture(path).boards;
    assert_eq!(boards.len(), 1, "{path} holds one board");
    boards.remove(0)
}

fn assert_hand(board: &Board, seat: Seat, hand: &str) {
    assert_eq!(
        board.deal()[seat],
        Cards::from_str(hand).expect("expected hand parses"),
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
        ("deal_only.pbn", 1),
        ("numbered_vulnerable.pbn", 1),
        ("contract.pbn", 1),
        ("unknown_hand.pbn", 1),
        ("lowercase.pbn", 1),
        ("two_games.pbn", 2),
        ("wire_example.pbn", 1),
        ("auction.pbn", 1),
        ("all_pass.pbn", 1),
        ("truncated_auction.pbn", 1),
        ("full_play.pbn", 1),
        ("short_play.pbn", 1),
        ("annotated.pbn", 1),
        ("multi_board.pbn", 2),
    ] {
        assert_eq!(parse_fixture(path).boards.len(), boards, "{path}");
    }
}

#[test]
fn deal_only_fixture_has_no_number_or_contract() {
    let board = only_board("deal_only.pbn");

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
    let board = only_board("numbered_vulnerable.pbn");

    assert_eq!(board.number(), Some(12));
    assert_eq!(board.dealer(), Seat::West);
    assert_eq!(board.vulnerability(), Vulnerability::All);
    assert_one_suit_per_seat(&board);
}

#[test]
fn contract_fixture_builds_a_doubled_contract() {
    let board = only_board("contract.pbn");

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
    let board = only_board("unknown_hand.pbn");

    assert_eq!(board.number(), Some(4));
    assert_eq!(board.vulnerability(), Vulnerability::EW);
    assert_one_suit_per_seat(&board);
}

#[test]
fn lowercase_fixture_accepts_lowercase_and_out_of_order_ranks() {
    let board = only_board("lowercase.pbn");

    assert_eq!(board.number(), Some(5));
    assert_eq!(board.dealer(), Seat::East);
    assert_eq!(board.vulnerability(), Vulnerability::All);
    assert_one_suit_per_seat(&board);
}

#[test]
fn two_game_fixture_parses_in_file_order() {
    let file = parse_fixture("two_games.pbn");

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
    let board = only_board("wire_example.pbn");

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
    let board = only_board("auction.pbn");

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
    let board = only_board("all_pass.pbn");

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
    let board = only_board("truncated_auction.pbn");

    let auction = board.auction().expect("the auction section was parsed");
    assert_eq!(auction.len(), 3, "the tokens after `*` are ignored");
    let contract = board.contract().expect("the auction derives a contract");
    assert_eq!(contract.level(), 2);
    assert_eq!(contract.strain(), Strain::Spades);
    assert_eq!(contract.declarer(), Seat::North);
    assert_eq!(contract.vulnerable(), Vulnerable::No);
}

#[test]
fn a_completed_pbn_auction_needs_no_star_and_keeps_its_contract() {
    let pbn = format!("{}[Auction \"N\"]\n1S Pass Pass Pass\n", identification());
    let board = &parse(&pbn).boards[0];
    let auction = board.auction().expect("the auction section was parsed");
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
fn four_opening_passes_without_a_star_are_a_pass_out() {
    let pbn = format!("{}[Auction \"N\"]\nPass Pass Pass Pass\n", identification());
    let board = &parse(&pbn).boards[0];
    let auction = board.auction().expect("the auction section was parsed");
    assert_eq!(auction.len(), 4);
    assert!(auction.is_complete());
    assert_eq!(board.contract(), None);
}

#[test]
fn a_doubled_pbn_auction_completes_after_three_passes() {
    let pbn = format!("{}[Auction \"N\"]\n1S X Pass Pass Pass\n", identification());
    let board = &parse(&pbn).boards[0];
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
fn a_cut_pbn_auction_without_a_star_is_recorded_as_incomplete() {
    let pbn = format!("{}[Auction \"N\"]\n1S Pass 2S\n", identification());
    let board = &parse(&pbn).boards[0];
    let auction = board.auction().expect("the auction section was parsed");
    assert_eq!(auction.len(), 3);
    assert!(!auction.is_complete());
}

#[test]
fn full_play_fixture_attributes_every_trick_to_its_winner() {
    let board = only_board("full_play.pbn");

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
    let board = only_board("short_play.pbn");

    let plays = board.play().expect("the play section was parsed").plays();
    assert_eq!(plays.len(), 5);
    assert_eq!(plays[4], Play::new(Seat::North, Card::SK));
}

#[test]
fn annotated_fixture_ignores_annotations_and_placeholders() {
    let board = only_board("annotated.pbn");

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

// ============================================================================
// Section-body tokenisation
// ============================================================================

/// The one-suit deal the unit tests build games around. The integration tests
/// exercise the public entry points, so they spell their own identification.
const ONE_SUIT_DEAL: &str =
    "[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]";

fn identification() -> String {
    format!("[Dealer \"N\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n")
}

fn bid(level: u8, strain: Strain) -> Call {
    Call::Bid {
        level: NonZeroU8::new(level).expect("test level is non-zero"),
        strain,
    }
}

fn calls(board: &Board) -> Vec<Call> {
    board
        .auction()
        .expect("the board records an auction")
        .calls()
        .iter()
        .map(|recorded| recorded.call)
        .collect()
}

#[test]
fn attached_auction_annotations_are_read_as_tokens() {
    // PBN's call token stops at the suffix (`!`), the note reference (`=1=`)
    // and the NAG (`$3`), so these annotations are tokens even without the
    // spaces an export-format writer would insert.
    let pbn = format!(
        "{}[Auction \"N\"]\n1S!=1=$3 Pass 2S!! Pass\n4S?? Pass Pass Pass\n",
        identification()
    );
    let board = &parse_pbn_file(&pbn)
        .expect("attached auction annotations must not change the calls")
        .boards[0];
    assert_eq!(
        calls(board),
        vec![
            bid(1, Strain::Spades),
            Call::Pass,
            bid(2, Strain::Spades),
            Call::Pass,
            bid(4, Strain::Spades),
            Call::Pass,
            Call::Pass,
            Call::Pass,
        ]
    );
}

#[test]
fn attached_play_annotations_are_read_as_tokens() {
    let pbn = format!(
        "{}[Declarer \"N\"]\n[Contract \"4S\"]\n[Play \"E\"]\nHK=1= DA! C2$7 SA??\n*\n",
        identification()
    );
    let board = &parse_pbn_file(&pbn)
        .expect("attached play annotations must not change the cards")
        .boards[0];
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
fn revoke_marker_is_accepted_as_card_data() {
    // PBN 2.1 §3.9 (C): `^R` precedes the card that revoked. The card itself
    // is still a real play, so the marker is read and discarded rather than
    // treated as an unparseable card.
    let pbn = format!(
        "{}[Declarer \"N\"]\n[Contract \"4S\"]\n[Play \"E\"]\nHA ^R DA C2 SA\n*\n",
        identification()
    );
    let board = &parse_pbn_file(&pbn)
        .expect("a revoke marker must not change the cards")
        .boards[0];
    assert_eq!(
        board.play().expect("the play section was parsed").plays(),
        &[
            Play::new(Seat::East, Card::HA),
            Play::new(Seat::South, Card::DA),
            Play::new(Seat::West, Card::C2),
            Play::new(Seat::North, Card::SA),
        ]
    );
}

#[test]
fn irregularity_markers_are_rejected() {
    // PBN 2.1 §3.9: `^I` marks an insufficient bid, `^S` a call out of
    // rotation, and `^L` a lead out of turn. The model stores neither the
    // auction nor the play in a shape that can carry them, so each one is a
    // named error rather than an unparseable call or card.
    let auction = |body: &str| format!("{}[Auction \"N\"]\n{body}\n", identification());
    let play = |body: &str| {
        format!(
            "{}[Declarer \"N\"]\n[Contract \"4S\"]\n[Play \"E\"]\n{body}\n",
            identification()
        )
    };
    for (pbn, marker) in [
        (auction("^I 1S Pass Pass Pass"), "^I"),
        (auction("^S 1S Pass Pass Pass"), "^S"),
        (play("HA C2 ^L C3 C4"), "^L"),
    ] {
        let error = parse_pbn_file(&pbn).expect_err("an irregularity marker must be rejected");
        assert_eq!(error.kind(), &ParsePbnErrorKind::Irregularity, "{marker}");
    }
}

#[test]
fn revoke_marker_must_precede_a_card() {
    // The marker annotates the card after it; a dangling one is not a play,
    // so accepting it silently would invent a revoke that never happened.
    let pbn = format!(
        "{}[Declarer \"N\"]\n[Contract \"4S\"]\n[Play \"E\"]\nHA DA C2 SA ^R\n*\n",
        identification()
    );
    let error = parse_pbn_file(&pbn).expect_err("a dangling ^R is not a card");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Irregularity);
    assert_eq!(
        error.detail(),
        Some("irregularity ^R: a revoke marker must be followed by a card")
    );
}

#[test]
fn irregularity_errors_name_the_marker() {
    let pbn = format!(
        "{}[Declarer \"N\"]\n[Contract \"4S\"]\n[Play \"E\"]\nHA C2 ^L C3 C4\n",
        identification()
    );
    let error = parse_pbn_file(&pbn).expect_err("a lead out of turn must be rejected");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Irregularity);
    assert_eq!(
        error.detail(),
        Some("irregularity ^L: a lead out of turn is not representable")
    );
}

// ============================================================================
// Error causes
// ============================================================================

#[test]
fn seat_errors_keep_the_exact_cause() {
    let pbn = format!("[Dealer \"X\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n");
    let error = parse_pbn_file(&pbn).expect_err("X is not a seat");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Dealer);
    assert_eq!(error.detail(), Some("'X' is not a valid seat"));
}

#[test]
fn deal_seat_errors_keep_the_exact_cause() {
    // The `[Deal]` first letter is parsed as a seat too, so it carries the
    // same typed cause as a bad `[Dealer]`.
    let pbn = "[Deal \"X:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]\n";
    let error = parse_pbn_file(pbn).expect_err("X is not a seat");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Hands);
    assert_eq!(error.detail(), Some("'X' is not a valid seat"));
}

#[test]
fn malformed_card_errors_keep_the_exact_cause() {
    let pbn = format!(
        "{}[Declarer \"N\"]\n[Contract \"4S\"]\n[Play \"E\"]\nHK ZZ C2 SA\n*\n",
        identification()
    );
    let error = parse_pbn_file(&pbn).expect_err("ZZ is not a card");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Card);
    assert_eq!(error.detail(), Some("invalid card \"ZZ\""));
}

#[test]
fn board_errors_keep_the_exact_cause() {
    let pbn = format!(
        "{}[Declarer \"N\"]\n[Contract \"4S\"]\n[Play \"E\"]\nSA DA C2 HK\n*\n",
        identification()
    );
    let error = parse_pbn_file(&pbn).expect_err("East does not hold the spade ace");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Board);
    assert_eq!(
        error.detail(),
        Some("E played A♠, which is not in their hand")
    );
}

#[test]
fn a_bid_after_four_opening_passes_is_an_error() {
    // The auction passed out after four passes; the following 1S cannot turn
    // the board into a 1S contract.
    let pbn = format!(
        "{}[Auction \"N\"]\nPass Pass Pass Pass 1S\n",
        identification()
    );
    let error = parse_pbn_file(&pbn).expect_err("the auction had already ended");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Board);
    assert!(
        error
            .detail()
            .is_some_and(|detail| detail.contains("after the auction")),
        "the error should say the auction ended: {error}"
    );
}

#[test]
fn a_pass_contract_cannot_rescue_an_auction_that_ended_early() {
    // The `[Contract "Pass"]` tag would make this board a pass-out, but the
    // auction itself is illegal: the 1S follows the fourth pass. The importer
    // must reject it rather than let the tag silence the auction error.
    let pbn = format!(
        "{}[Contract \"Pass\"]\n[Auction \"N\"]\nPass Pass Pass Pass 1S\n",
        identification()
    );
    let error = parse_pbn_file(&pbn).expect_err("the auction had already ended");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Board);
    assert!(
        error
            .detail()
            .is_some_and(|detail| detail.contains("after the auction")),
        "the error should say the auction ended: {error}"
    );
}

#[test]
fn all_pass_after_a_bid_is_rejected() {
    // `AP` is four passes, not "three more passes", so `1S AP` puts the fourth
    // pass after the auction's end. Rejecting it is clearer than silently
    // treating `AP` as a closing shorthand.
    let pbn = format!("{}[Auction \"N\"]\n1S AP\n", identification());
    let error = parse_pbn_file(&pbn).expect_err("the fourth pass follows the auction end");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Board);
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
    let pbn = format!(
        "{}[Auction \"N\"]\n1S Pass Pass Pass 2S\n",
        identification()
    );
    let error = parse_pbn_file(&pbn).expect_err("the auction had already ended");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Board);
    assert!(
        error
            .detail()
            .is_some_and(|detail| detail.contains("after the auction")),
        "the error should say the auction ended: {error}"
    );
}

#[test]
fn display_includes_the_detail() {
    let pbn = format!("[Dealer \"X\"]\n[Vulnerable \"None\"]\n{ONE_SUIT_DEAL}\n");
    let error = parse_pbn_file(&pbn).expect_err("X is not a seat");
    let rendered = error.to_string();
    assert!(
        rendered.contains("unable to parse PBN input (Dealer): 'X' is not a valid seat:"),
        "{rendered}"
    );
}

#[test]
fn malformed_fixture_is_a_hands_error() {
    let data = common::read_fixture("malformed.pbn");
    let error = parse_pbn_file(&data).expect_err("a short hand cannot be a deal");
    assert_eq!(error.kind(), &ParsePbnErrorKind::Hands);
}

// ============================================================================
// Writer and malformed input
// ============================================================================

#[test]
fn pbn_file_writer_separates_games_with_one_empty_line() {
    let file = parse_fixture("two_games.pbn");
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
    let file = parse_fixture("multi_board.pbn");
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
