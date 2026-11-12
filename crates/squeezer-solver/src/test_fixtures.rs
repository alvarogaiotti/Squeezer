// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! Shared fixtures for DDS's `examples/hands.cpp` hand 0.
//!
//! These are test data, not library behaviour: the module is compiled for this
//! crate's unit tests and for builds that enable the `test-support` feature,
//! which the root crate does from its dev-dependencies. `Deal`, `Board` and
//! `Position` are public types, so the helpers can be ordinary public items
//! within that gate rather than reaching for `pub(crate)` visibility that would
//! be useless across a crate boundary.

use crate::prelude::*;
use squeezer_model::record::position::trick_winner;
use std::str::FromStr;

/// The four parsed hands of DDS example hand 0.
///
/// # Panics
///
/// Panics if the hand strings do not parse; they are fixed literals.
#[must_use]
pub fn example_deal() -> Deal {
    Deal::new([
        Cards::from_str("SQJ6 HK652 DJ85 CT98").expect("north parses"),
        Cards::from_str("S873 HJ97 DAT764 CQ4").expect("east parses"),
        Cards::from_str("SK5 HT83 DKQ9 CA7652").expect("south parses"),
        Cards::from_str("SAT942 HAQ4 D32 CKJ3").expect("west parses"),
    ])
    .expect("example hand 0 is a partition")
}

/// A spade contract declared by West, so North is on opening lead.
#[must_use]
pub fn example_contract() -> Contract {
    Contract::new(
        1,
        Strain::Spades,
        Seat::West,
        Vulnerable::No,
        Doubled::NotDoubled,
    )
}

/// The example deal with `plays` already recorded.
///
/// # Panics
///
/// Panics if the fixed example deal or play leaves the board inconsistent.
#[must_use]
pub fn example_board_with(plays: Vec<Play>) -> Board {
    Board::builder(example_deal(), Seat::North, Vulnerability::None)
        .contract(example_contract())
        .play(PlayRecord::new(plays))
        .build()
        .expect("example board is consistent")
}

/// The example deal at the opening lead.
///
/// # Panics
///
/// Panics if the fixed example deal leaves the board inconsistent.
#[must_use]
pub fn example_board() -> Board {
    Board::builder(example_deal(), Seat::North, Vulnerability::None)
        .contract(example_contract())
        .build()
        .expect("example board is consistent")
}

/// The opening position of the example board.
///
/// # Panics
///
/// Panics if the example board carries no contract.
#[must_use]
pub fn opening_position() -> Position {
    Position::after_play(&example_board(), 0).expect("the board carries a contract")
}

/// DDS's `examples/hands.cpp` hand-0 play: 45 cards in play order.
///
/// Its `AnalysePlayBin` oracle is 46 values, all `8` (the declarer's
/// double-dummy tricks), which is why this play is the independent reference
/// for the seam's play analysis.
pub const EXAMPLE_HAND0_PLAY: &str =
    "CTC4CACJH8H4HKH9D5DAD9D2S7S5S2SQD8D4DQD3H3HAH6H7C3C8CQC2S3SKSAS6HQH5HJHTCKC9D6C5S4SJS8C6DJ";

/// Parses a flat DDS play string (`"CTC4CA…"`, suit then rank) into cards.
///
/// # Panics
///
/// Panics if the string has an odd length or a token is not a card.
#[must_use]
pub fn parse_play(sequence: &str) -> Vec<Card> {
    sequence
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let token = std::str::from_utf8(pair).expect("play strings are ASCII");
            Card::from_str(token).expect("valid card token")
        })
        .collect()
}

/// Attributes an ordered card list to seats trick by trick.
///
/// The contract supplies the opening leader and the winner rule; this is what a
/// [`Board`]'s play validation needs, since a DDS `PlayTraceBin` is seat-free.
#[must_use]
pub fn attribute_seats(cards: &[Card], contract: Contract) -> Vec<Play> {
    let mut plays = Vec::with_capacity(cards.len());
    let mut leader = contract.leader();
    for trick in cards.chunks(4) {
        let group: Vec<Play> = trick
            .iter()
            .enumerate()
            .map(|(offset, &card)| Play::new(leader + offset, card))
            .collect();
        if group.len() == 4 {
            leader = trick_winner(&group, contract.strain());
        }
        plays.extend(group);
    }
    plays
}

/// The example board with DDS hand 0's 45-card play recorded.
///
/// # Panics
///
/// Panics if the fixed play does not produce a consistent board.
#[must_use]
pub fn example_board_hand0_play() -> Board {
    let cards = parse_play(EXAMPLE_HAND0_PLAY);
    example_board_with(attribute_seats(&cards, example_contract()))
}
