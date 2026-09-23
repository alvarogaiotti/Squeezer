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

/// Writes one hand as `<suit-letter><ranks>...`, ranks high to low.
///
/// A void suit is omitted entirely: LIN's hand grammar is position-independent
/// (each suit letter introduces its own ranks), so writing an empty suit's
/// letter would make the importer read the next suit's cards into the void.
/// `Suit::iter()` and `Hand`'s iterator share the S,H,D,C order.
fn write_hand(writer: &mut impl fmt::Write, hand: Hand) -> fmt::Result {
    for (suit, holding) in Suit::iter().zip(hand) {
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

#[cfg(feature = "lin")]
impl crate::linparser::LinFile {
    /// Writes the file metadata and its boards as LIN.
    ///
    /// The importer requires a `pn|` line, so one is always emitted; `vg|` and
    /// `rs|` are emitted only when present. Each board's `qx|` room (kept in
    /// [`crate::linparser::LinMetadata`], not on the board) is written before
    /// the board itself.
    ///
    /// # Lossy metadata
    ///
    /// `vg|`, `rs|` and `pn|` are comma-separated lists and LIN has no escaping,
    /// so a player name, team or result string that itself contains a comma
    /// re-parses as extra fields. Such values are not round-trippable; the
    /// board data is unaffected.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_lin(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if let Some(tournament) = &self.metadata.tournament_info {
            write!(
                writer,
                "vg|{},{},{},{},{},{},{},{},{}|",
                tournament.tournament,
                tournament.round,
                tournament.board_range,
                tournament.board_start,
                tournament.board_end,
                tournament.team1,
                tournament.team1_score,
                tournament.team2,
                tournament.team2_score,
            )?;
        }

        if let Some(results) = &self.metadata.results {
            write!(writer, "rs|{}|", results.results.join(","))?;
        }

        write!(writer, "pn|{}|", self.metadata.players.players.join(","))?;
        writer.write_str("pg||")?;

        for (index, board) in self.boards.iter().enumerate() {
            if let Some(room) = self.metadata.rooms.get(index).copied().flatten() {
                let code = match room {
                    crate::linparser::Room::Open => 'o',
                    crate::linparser::Room::Closed => 'c',
                };
                match board.number() {
                    Some(number) => write!(writer, "qx|{code}{number}|")?,
                    None => write!(writer, "qx|{code}|")?,
                }
            }
            board.write_lin(writer)?;
        }

        Ok(())
    }

    /// Formats the file metadata and its boards as LIN.
    #[must_use]
    #[inline]
    pub fn as_lin(&self) -> String {
        super::written_string(0, |lin| self.write_lin(lin))
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

    #[test]
    fn md_encodes_the_stored_dealer_not_the_board_number() {
        // Board 1 rotates to North, but this board stores West as dealer. The
        // writer must emit the stored dealer rather than deriving it from the
        // number (the old `Deal::as_lin` derived it and got this wrong).
        let board = Board::builder(one_suit_per_seat(), Seat::West, Vulnerability::None)
            .number(1)
            .build()
            .expect("a bare deal is a valid board");
        let lin = board.as_lin();
        assert!(
            lin.starts_with("st||md|2"),
            "dealer digit should encode the stored West dealer: {lin}"
        );

        // And the importer agrees: the same `md|` digit comes back as West,
        // not as the North that board 1 would otherwise rotate to.
        #[cfg(feature = "lin")]
        {
            use crate::linparser::LinFile;
            let parsed =
                <LinFile as FromStr>::from_str(&format!("pn|N,E,S,W|{lin}")).expect("reparses");
            assert_eq!(parsed.boards[0].dealer(), Seat::West);
            assert_eq!(parsed.boards[0].number(), Some(1));
        }
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
}
