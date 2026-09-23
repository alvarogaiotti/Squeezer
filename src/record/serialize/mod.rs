// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Outbound serialization for the record types.
//!
//! Writing needs no parser and no native solver, so this module is
//! unconditional: a consumer can export a [`crate::record::Board`] or
//! [`crate::record::Deal`] without the `lin` feature (and therefore without
//! `nom`). The LIN **importer** stays behind `lin`; [`crate::linparser::LinFile`]
//! itself is parser-side, so its writer is gated accordingly.
//!
//! # Formats
//!
//! A board has two interchange formats, so there is deliberately no
//! `impl Display for Board`: an implicit format choice is a footgun. Call
//! [`crate::record::Board::as_lin`] explicitly.
//!
//! The writers are **lossy** for anything the model does not represent. A LIN
//! board can carry arbitrary tags, comments and formatting that
//! [`crate::record::Board`] does not keep; the output re-emits only the
//! modeled tags. LIN's file metadata (`vg|`/`rs|`/`pn|`) is comma-separated
//! with no escaping, so a name, team or result containing a comma does not
//! round-trip. The PBN board writer is deferred (design ticket 07); only the
//! [`crate::record::Deal`] PBN hand record is provided, which covers current
//! interchange needs.

use crate::prelude::*;

pub mod lin;
pub mod pbn;

/// Renders a writer into a fresh `String`.
///
/// The `as_*` methods all share this shape; the `expect` is sound because
/// [`fmt::Write`] for [`String`] never fails.
fn written_string(capacity: usize, write: impl FnOnce(&mut String) -> fmt::Result) -> String {
    let mut out = String::with_capacity(capacity);
    write(&mut out).expect("writing to a String is infallible");
    out
}

#[cfg(test)]
pub(crate) mod fixtures {
    use crate::prelude::*;

    /// A deal with one whole suit per seat, so a golden hand listing is easy to
    /// read and each hand exercises three void suits.
    pub(crate) fn one_suit_per_seat() -> Deal {
        Deal::new([
            Hand::from_str("SAKQJT98765432").expect("spades hand parses"),
            Hand::from_str("HAKQJT98765432").expect("hearts hand parses"),
            Hand::from_str("DAKQJT98765432").expect("diamonds hand parses"),
            Hand::from_str("CAKQJT98765432").expect("clubs hand parses"),
        ])
        .expect("one suit per seat is a valid deal")
    }
}

#[cfg(test)]
mod tests {
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
        let mut hands = [Hand::new_empty(); 4];
        for slot in &mut hands {
            let mut cards = Cards::EMPTY;
            for _ in 0..13 {
                let index = usize::arbitrary(g) % deck.len();
                cards = cards.insert(deck.swap_remove(index));
            }
            *slot = Hand::try_from(cards).expect("thirteen distinct cards form a hand");
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
            remaining[seat as usize] = deal[seat].cards().into_iter().collect();
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
}
