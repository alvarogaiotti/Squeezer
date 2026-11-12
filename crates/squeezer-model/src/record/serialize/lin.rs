// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! LIN writers for the record types.
//!
//! The output mirrors the `linparser` importer tag for tag, so a written board
//! parses back to an equal board for every modeled field:
//!
//! ```text
//! st||md|<dealer><hands>|rh||ah|Board N|sv|<vuln>|mb|<call>|...|pc|<card>|...|mc|<tricks>|
//! ```
//!
//! The output is lossy for tags and formatting the model does not represent
//! (BBO result strings, arbitrary comments, the exact whitespace of the input);
//! only the modeled tags are re-emitted.
//!
//! # The `md|` dealer and hand order
//!
//! LIN's `md|` tag is `<dealer-digit><hands>`, where the digit numbers the
//! dealer within the hand listing and the hands are listed South, West, North,
//! East. The importer reads exactly that and rotates the listing into the core
//! `[N, E, S, W]` order. This writer emits the **stored**
//! [`crate::record::Board::dealer`] and the same S,W,N,E listing, so the dealer
//! is never re-derived from the board number.

use crate::contract::{Seat, Strain, Vulnerability};
use crate::prelude::*;

/// The `md|` dealer digit, which numbers the dealer in the S,W,N,E listing:
/// `1` South, `2` West, `3` North, `4` East.
const fn dealer_digit(dealer: Seat) -> char {
    match dealer {
        Seat::South => '1',
        Seat::West => '2',
        Seat::North => '3',
        Seat::East => '4',
    }
}

/// The `sv|` vulnerability code, as the importer reads it.
const fn vulnerability_code(vulnerability: Vulnerability) -> char {
    match vulnerability {
        Vulnerability::None => 'o',
        Vulnerability::NS => 'n',
        Vulnerability::EW => 'e',
        Vulnerability::All => 'b',
    }
}

/// The single-letter strain used in `mb|` bid tokens.
///
/// This is the LIN wire code, not a display form: `Strain::not_unicode_str()`
/// spells `NoTrumps` as `"NT"`, which the importer would reject after the `N`
/// (it reads exactly one strain character). The dealer digit and vulnerability
/// code below are likewise LIN wire encodings; the core deliberately keeps
/// serialization out of `record`, so there is no `Deal`/`Board` accessor to
/// reuse.
const fn strain_char(strain: Strain) -> char {
    match strain {
        Strain::Spades => 'S',
        Strain::Hearts => 'H',
        Strain::Diamonds => 'D',
        Strain::Clubs => 'C',
        Strain::NoTrumps => 'N',
    }
}

/// Writes one seat's cards as `<suit-letter><ranks>...`, ranks high to low.
///
/// A void suit is omitted entirely: LIN's hand grammar is position-independent
/// (each suit letter introduces its own ranks), so writing an empty suit's
/// letter would make the importer read the next suit's cards into the void.
fn write_hand(writer: &mut impl fmt::Write, hand: Cards) -> fmt::Result {
    // `Cards: IntoIterator` yields individual cards, so iterate the suits
    // explicitly in the S,H,D,C order the LIN grammar expects.
    let suits = [hand.spades(), hand.hearts(), hand.diamonds(), hand.clubs()];
    for (suit, holding) in Suit::iter().zip(suits) {
        if holding.is_empty() {
            continue;
        }
        writer.write_char(suit.latin())?;
        for card in holding {
            writer.write_char(card.rankchar())?;
        }
    }
    Ok(())
}

/// Writes one call as an `mb|` token: `p`, `d`, `r` or `<level><strain>`.
fn write_call(writer: &mut impl fmt::Write, call: Call) -> fmt::Result {
    match call {
        Call::Pass => writer.write_char('p'),
        Call::Double => writer.write_char('d'),
        Call::Redouble => writer.write_char('r'),
        Call::Bid { level, strain } => write!(writer, "{level}{}", strain_char(strain)),
    }
}

impl Deal {
    /// Writes the minimal BBO LIN hand record: `st||md|<dealer><hands>|`.
    ///
    /// This is what a hand-record upload needs, and it is the primitive
    /// [`Board::write_lin`] composes. `dealer` is written verbatim; it is never
    /// derived from a board number.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_lin(&self, dealer: Seat, writer: &mut impl fmt::Write) -> fmt::Result {
        writer.write_str("st||md|")?;
        writer.write_char(dealer_digit(dealer))?;
        // LIN lists the hands South, West, North, East.
        for (index, seat) in [Seat::South, Seat::West, Seat::North, Seat::East]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                writer.write_char(',')?;
            }
            write_hand(writer, self[seat])?;
        }
        writer.write_char('|')
    }

    /// Formats the minimal BBO LIN hand record (`st||md|<dealer><hands>|`).
    #[must_use]
    #[inline]
    pub fn as_lin(&self, dealer: Seat) -> String {
        super::written_string(96, |lin| self.write_lin(dealer, lin))
    }
}

impl Board {
    /// Writes the full modeled board as LIN.
    ///
    /// Emits, in the importer's order, the `st||md|` hand record, an optional
    /// `rh||ah|Board N|`, `sv|`, the auction (`mb|`/`an|`), the play
    /// (`pc|`/`pg||`) and the claim (`mc|`). See the module docs for what is
    /// lost.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_lin(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        self.deal().write_lin(self.dealer(), writer)?;

        // The importer's `rh||ah|Board N|` is optional; it is the only modeled
        // home of the board number.
        if let Some(number) = self.number() {
            write!(writer, "rh||ah|Board {number}|")?;
        }

        // `sv|` is mandatory: the importer requires it to rebuild a contract.
        write!(writer, "sv|{}|", vulnerability_code(self.vulnerability()))?;

        if let Some(auction) = self.auction() {
            for recorded in auction {
                writer.write_str("mb|")?;
                write_call(writer, recorded.call)?;
                if recorded.alert {
                    writer.write_char('!')?;
                }
                writer.write_char('|')?;
                if let Some(annotation) = &recorded.annotation {
                    write!(writer, "an|{annotation}|")?;
                }
            }
        }

        if let Some(play) = self.play() {
            writer.write_str("pg||")?;
            // `pg||` after every trick, including a short claimed tail, is what
            // lets the importer group the flat card list.
            for trick in play.tricks() {
                for played in trick {
                    // The importer reads a card as `<suit><rank>` (suit first),
                    // so the suit letter must precede the rank.
                    write!(
                        writer,
                        "pc|{}{}|",
                        played.card.suit().latin(),
                        played.card.rankchar()
                    )?;
                }
                writer.write_str("pg||")?;
            }
        }

        if let Some(claim) = self.claim() {
            write!(writer, "mc|{claim}|")?;
        }

        Ok(())
    }

    /// Formats the full modeled board as LIN.
    #[must_use]
    #[inline]
    pub fn as_lin(&self) -> String {
        super::written_string(256, |lin| self.write_lin(lin))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use super::super::fixtures::one_suit_per_seat;
    use super::*;

    fn bid(level: u8, strain: Strain) -> RecordedCall {
        RecordedCall::new(Call::Bid {
            level: NonZeroU8::new(level).expect("test level is non-zero"),
            strain,
        })
    }

    fn pass() -> RecordedCall {
        RecordedCall::new(Call::Pass)
    }

    /// North 1S, East pass, South 2S, West pass, North 4S, three passes.
    fn spade_auction() -> Auction {
        Auction::new(vec![
            bid(1, Strain::Spades),
            pass(),
            bid(2, Strain::Spades),
            pass(),
            bid(4, Strain::Spades),
            pass(),
            pass(),
            pass(),
        ])
    }

    fn known_board() -> Board {
        let auction = spade_auction();
        let contract = auction
            .contract(Seat::North, Vulnerability::None)
            .expect("the auction produced a contract");
        Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .number(1)
            .auction(auction)
            .contract(contract)
            .build()
            .expect("the known board is valid")
    }

    #[test]
    fn lin_golden_string_matches_the_modeled_tags() {
        let expected = concat!(
            "st||md|3DAKQJT98765432,CAKQJT98765432,SAKQJT98765432,HAKQJT98765432|",
            "rh||ah|Board 1|",
            "sv|o|",
            "mb|1S|mb|p|mb|2S|mb|p|mb|4S|mb|p|mb|p|mb|p|",
        );
        assert_eq!(known_board().as_lin(), expected);
    }

    #[test]
    fn deal_lin_is_the_minimal_hand_record() {
        assert_eq!(
            one_suit_per_seat().as_lin(Seat::North),
            "st||md|3DAKQJT98765432,CAKQJT98765432,SAKQJT98765432,HAKQJT98765432|"
        );
    }

    /// Board 1 rotates to North, but this board stores West as dealer: the
    /// writer must emit the stored dealer, not derive one from the number.
    #[test]
    fn md_encodes_the_stored_dealer_not_the_board_number() {
        let board = Board::builder(one_suit_per_seat(), Seat::West, Vulnerability::None)
            .number(1)
            .build()
            .expect("a bare deal is a valid board");
        let lin = board.as_lin();
        assert!(
            lin.starts_with("st||md|2"),
            "dealer digit should encode the stored West dealer: {lin}"
        );
    }
}
