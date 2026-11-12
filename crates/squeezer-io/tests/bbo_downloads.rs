// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Converting BBO hand downloads into boards.
//!
//! BBO's `fetchlin.php` answers with one complete LIN document per hand, so a
//! batch of downloads is a list of documents rather than one multi-board file.
//! `parse_lin_documents` is the importer's entry point for that shape; these
//! tests pin it to tracked fixtures so the round trip is testable offline.

use squeezer_io::linparser::{ParseLinErrorKind, parse_lin_documents};
use squeezer_model::{Contract, Doubled, Seat, Strain, Vulnerable};

mod common;

#[test]
fn converts_a_downloaded_lin_document_into_a_board() {
    let document = common::read_fixture("4207070707.lin");

    let boards = parse_lin_documents(&[document]).expect("the fixture is a valid LIN document");

    assert_eq!(boards.len(), 1);
    let board = &boards[0];
    assert_eq!(board.number(), Some(1));
    assert_eq!(board.dealer(), Seat::North);
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
    assert_eq!(board.play().expect("the board has a play").len(), 40);
}

#[test]
fn keeps_download_order_across_documents() {
    let documents = [
        common::read_fixture("4207076395.lin"),
        common::read_fixture("4207070707.lin"),
    ];

    let boards = parse_lin_documents(&documents).expect("both fixtures are valid LIN documents");

    let numbers: Vec<_> = boards.iter().map(|board| board.number()).collect();
    assert_eq!(numbers, vec![Some(2), Some(1)]);
}

#[test]
fn reports_the_first_malformed_document() {
    // South holds only twelve cards, so the deal is not a 52-card partition.
    let documents = [
        common::read_fixture("4207070707.lin"),
        String::from(
            "pn|a,b,c,d|st||md|3SAKQJT9876543,HAKQJT98765432,DAKQJT98765432,CAKQJT98765432|rh||ah|Board 1|sv|o|mb|1S|mb|p|mb|p|mb|p|",
        ),
    ];

    let error = parse_lin_documents(&documents).expect_err("the second document is malformed");

    assert_eq!(error.kind(), &ParseLinErrorKind::Hands);
}
