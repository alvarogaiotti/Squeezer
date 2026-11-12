// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Portable Bridge Notation (PBN) writers for the record types.
//!
//! The output is importer-format PBN (PBN 2.1, the MTS tag set the DDS fixtures
//! use), not a scored hand record: it is exactly the tag set this crate models,
//! so a PBN importer is designed to read it back tag for tag. A board writes its
//! identification tags first, one per line, in MTS order for the fields
//! present; the `Auction` and `Play` bodies then list four tokens per line.
//!
//! ```text
//! [Board "N"] [Dealer "N|E|S|W"] [Vulnerable "None|NS|EW|All"]
//! [Deal "<dealer>:<4 hands>"] [Declarer "..."] [Contract "<k><denom><risk>"]
//! [Auction "<dealer>"] <calls, 4 per line> [*]
//! [Play "<opening leader>"] <cards, 4 per line> [*]
//! ```
//!
//! # The `[Deal]` first letter and `[Dealer]`
//!
//! PBN's export convention lists the hands clockwise from the *dealer*, so the
//! writer emits `first = the stored dealer`, never a value derived from the
//! board number. The importer reads `[Dealer]` and the `[Deal]` first letter
//! independently and trusts each for its own purpose; the DDS fixtures under
//! `references/dds` write `N:` regardless of the dealer, which is why the two
//! must not be conflated.
//!
//! # Documented losses
//!
//! The output re-emits only the modeled tags, so it is lossy in the same way
//! LIN is:
//!
//! - `[Result]` (tricks won) and [`Board::claim`] (LIN's `mc|`) have no
//!   counterpart. A play shorter than 52 cards — and an auction that does not
//!   reach a pass-out — ends with `*` to mark the cut (there is no place to
//!   record *why* it was cut, e.g. a claim).
//! - A call's `alert` marker and `annotation` (PBN note references, `[Note]`
//!   text, suffixes and NAGs) are read and discarded by the importer, so they
//!   do not survive a round trip.
//! - Every unmodeled mandatory and supplemental tag (`Event`, `Site`, `Date`,
//!   players, `Scoring`, `Room`, …) is dropped.
//! - Irregular declarer (`^X`), end-position (denomination-only) contracts, the
//!   `-` unknown card and an out-of-turn lead (`^L`) are not modeled, so the
//!   importer rejects them rather than dropping data. The model stores every
//!   played card and derives each play's seat by rotation from the contract's
//!   leader.
//!
//! A board with no contract has no known opening leader, so its play is omitted
//! entirely (there is no `[Play]` section without a `[Contract]`).

use crate::prelude::*;

/// The PBN wire letter for a seat: `N`, `E`, `S` or `W`.
///
/// One home for the spelling shared by `[Dealer]`, `[Declarer]`, `[Deal]`,
/// `[Auction]` and `[Play]`, so the importer in later tickets reuses it rather
/// than re-deriving the mapping.
pub(crate) const fn seat_letter(seat: Seat) -> char {
    match seat {
        Seat::North => 'N',
        Seat::East => 'E',
        Seat::South => 'S',
        Seat::West => 'W',
    }
}

/// The PBN `[Vulnerable]` word: `None`, `NS`, `EW` or `All`.
pub(crate) const fn vulnerability_word(vulnerability: Vulnerability) -> &'static str {
    match vulnerability {
        Vulnerability::None => "None",
        Vulnerability::NS => "NS",
        Vulnerability::EW => "EW",
        Vulnerability::All => "All",
    }
}

/// The PBN strain token used in bids and contracts: `S`, `H`, `D`, `C` or
/// `NT`.
///
/// This is the wire spelling, not a display form: `Strain`'s `Display` emits
/// suit symbols and `Strain::not_unicode_str` is private to the contract
/// module.
pub(crate) const fn strain_token(strain: Strain) -> &'static str {
    match strain {
        Strain::Spades => "S",
        Strain::Hearts => "H",
        Strain::Diamonds => "D",
        Strain::Clubs => "C",
        Strain::NoTrumps => "NT",
    }
}

/// The PBN contract token `<level><denom><risk>`, e.g. `4S`, `3NT`, `4SX` or
/// `3NTXX`.
///
/// [`Contract`] exposes no accessor for its `Doubled` state, and this module
/// may not add one, so the risk suffix is recovered from the contract's own
/// non-unicode form (`<level><strain><declarer><X*>`): no other part of that
/// string can contain an `X`.
pub(crate) fn contract_token(contract: Contract) -> String {
    let risk = match contract
        .not_unicode_str()
        .chars()
        .rev()
        .take_while(|character| *character == 'X')
        .count()
    {
        0 => "",
        1 => "X",
        _ => "XX",
    };
    format!(
        "{}{}{}",
        contract.level(),
        strain_token(contract.strain()),
        risk
    )
}

/// The PBN token for one call: `Pass`, `X`, `XX` or `<level><denom>`.
pub(crate) fn call_token(call: Call) -> String {
    match call {
        Call::Pass => "Pass".to_string(),
        Call::Double => "X".to_string(),
        Call::Redouble => "XX".to_string(),
        Call::Bid { level, strain } => format!("{level}{}", strain_token(strain)),
    }
}

/// True when `auction` reached its natural end: either four opening passes (a
/// passed-out board) or a bid followed by three closing passes.
///
/// The model stores a raw call list with no completion flag, so this is how the
/// writer decides whether to terminate the section with `*`, PBN's marker for a
/// cut-off auction (PBN 2.1; see the spec's format contract).
fn auction_is_complete(auction: &Auction) -> bool {
    let calls = auction.calls();
    if calls.len() < 4 {
        return false;
    }
    if calls.iter().all(|recorded| recorded.call == Call::Pass) {
        // Only four opening passes end an auction that never had a bid; more
        // passes than that cannot be a completed auction.
        return calls.len() == 4;
    }
    calls
        .iter()
        .rev()
        .take_while(|recorded| recorded.call == Call::Pass)
        .count()
        >= 3
}

/// Writes an `Auction` or `Play` body: tokens whitespace-separated, four per
/// line, always terminated by a newline.
fn write_token_grid(
    writer: &mut impl fmt::Write,
    tokens: impl IntoIterator<Item = String>,
) -> fmt::Result {
    for (index, token) in tokens.into_iter().enumerate() {
        if index != 0 {
            writer.write_char(if index % 4 == 0 { '\n' } else { ' ' })?;
        }
        writer.write_str(&token)?;
    }
    writer.write_char('\n')
}

/// Writes one seat's cards as `<suit ranks>.<suit ranks>...`, spades to clubs,
/// ranks high to low.
///
/// A void suit is an empty field between the dots, so `...` is a holding whose
/// only suit is clubs. This is fixed-width-ish PBN hand grammar, unlike LIN's
/// position-independent `S...H...D...C...` form.
fn write_hand(writer: &mut impl fmt::Write, hand: Cards) -> fmt::Result {
    // `Cards: IntoIterator` yields individual cards, so iterate the suits
    // explicitly in the S,H,D,C order the PBN grammar expects.
    for (suit_index, holding) in [hand.spades(), hand.hearts(), hand.diamonds(), hand.clubs()]
        .into_iter()
        .enumerate()
    {
        if suit_index != 0 {
            writer.write_char('.')?;
        }
        for card in holding {
            writer.write_char(card.rankchar())?;
        }
    }
    Ok(())
}

impl Deal {
    /// Writes the four hands as a PBN `[Deal "<first>:..."]` hand record,
    /// listing the hands clockwise from `first`.
    ///
    /// Each hand lists its suits spades, hearts, diamonds, clubs, separated by
    /// `.` and with ranks high to low. An empty suit is an empty field, so a
    /// void shows as the conventional leading/interior dot.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_pbn_from(&self, first: Seat, writer: &mut impl fmt::Write) -> fmt::Result {
        writer.write_str("[Deal \"")?;
        writer.write_char(seat_letter(first))?;
        writer.write_char(':')?;
        for (index, seat) in first.iter_from().take(4).enumerate() {
            if index != 0 {
                writer.write_char(' ')?;
            }
            write_hand(writer, self[seat])?;
        }
        writer.write_str("\"]")
    }

    /// Formats the four hands as a PBN `[Deal "<first>:..."]` hand record.
    #[must_use]
    #[inline]
    pub fn as_pbn_from(&self, first: Seat) -> String {
        super::written_string(128, |pbn| self.write_pbn_from(first, pbn))
    }

    /// Writes the four hands as a PBN `[Deal "N:..."]` hand record.
    ///
    /// Delegates to [`Deal::write_pbn_from`] with [`Seat::North`], the
    /// historical default.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_pbn(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        self.write_pbn_from(Seat::North, writer)
    }

    /// Formats the four hands as a PBN `[Deal "N:..."]` hand record.
    #[must_use]
    #[inline]
    pub fn as_pbn(&self) -> String {
        self.as_pbn_from(Seat::North)
    }
}

impl Board {
    /// Writes the recorded board as PBN.
    ///
    /// Emits the identification tags one per line in MTS order for the fields
    /// present — `[Board]` (only when numbered), `[Dealer]`, `[Vulnerable]`,
    /// `[Deal]` (clockwise from the stored dealer) — then `[Declarer]` and
    /// `[Contract]` when a contract exists, the `Auction` section, and finally
    /// the `Play` section when a contract fixes the opening leader. See the
    /// module docs for what is lost.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_pbn(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        if let Some(number) = self.number() {
            writeln!(writer, "[Board \"{number}\"]")?;
        }
        writeln!(writer, "[Dealer \"{}\"]", seat_letter(self.dealer()))?;
        writeln!(
            writer,
            "[Vulnerable \"{}\"]",
            vulnerability_word(self.vulnerability())
        )?;
        self.deal().write_pbn_from(self.dealer(), writer)?;
        writer.write_char('\n')?;

        if let Some(contract) = self.contract() {
            writeln!(
                writer,
                "[Declarer \"{}\"]",
                seat_letter(contract.declarer())
            )?;
            writeln!(writer, "[Contract \"{}\"]", contract_token(contract))?;
        }

        if let Some(auction) = self.auction().filter(|auction| !auction.is_empty()) {
            writeln!(writer, "[Auction \"{}\"]", seat_letter(self.dealer()))?;
            write_token_grid(
                writer,
                auction.iter().map(|recorded| call_token(recorded.call)),
            )?;
            // A cut-off auction ends with `*`, mirroring a cut-off play; a
            // completed auction ends on its closing (or opening) passes.
            if !auction_is_complete(auction) {
                writer.write_str("*\n")?;
            }
        }

        // The play needs a contract: only it fixes the opening leader, so a
        // board with cards but no contract cannot place its `[Play]` tag. The
        // empty-play normalisation in `Board::new` means this is the only
        // omission case in practice.
        if let Some(contract) = self.contract()
            && let Some(play) = self.play()
        {
            writeln!(writer, "[Play \"{}\"]", seat_letter(contract.leader()))?;
            write_token_grid(
                writer,
                play.iter().map(|played| {
                    format!("{}{}", played.card.suit().latin(), played.card.rankchar())
                }),
            )?;
            // A cut-off play ends with `*`; 52 cards is a complete hand.
            if play.len() < 52 {
                writer.write_str("*\n")?;
            }
        }

        Ok(())
    }

    /// Formats the recorded board as PBN.
    #[must_use]
    #[inline]
    pub fn as_pbn(&self) -> String {
        super::written_string(512, |pbn| self.write_pbn(pbn))
    }
}

#[cfg(feature = "pbn")]
impl crate::pbnparser::PbnFile {
    /// Writes every board as PBN, with one empty line between games.
    ///
    /// PBN separates games with a semi-empty line. Each board's own writer
    /// already ends with a newline, so writing one more before every board
    /// after the first produces exactly one blank line and no trailing one.
    ///
    /// # Errors
    ///
    /// Propagates the writer's error; writing to a [`String`] never fails.
    pub fn write_pbn(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        for (index, board) in self.boards.iter().enumerate() {
            if index != 0 {
                writer.write_char('\n')?;
            }
            board.write_pbn(writer)?;
        }
        Ok(())
    }

    /// Formats every board as PBN, with one empty line between games.
    #[must_use]
    pub fn as_pbn(&self) -> String {
        let capacity = 512 * self.boards.len().max(1);
        super::written_string(capacity, |pbn| self.write_pbn(pbn))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use super::super::fixtures::one_suit_per_seat;
    use super::*;

    /// The `[Deal]` value for `one_suit_per_seat` with North listed first.
    const DEAL_FROM_NORTH: &str =
        "[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]\n";

    /// The `[Deal]` value for `one_suit_per_seat` with East listed first.
    const DEAL_FROM_EAST: &str =
        "[Deal \"E:.AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432 AKQJT98765432...\"]\n";

    /// The `[Deal]` value for `one_suit_per_seat` with South listed first.
    const DEAL_FROM_SOUTH: &str =
        "[Deal \"S:..AKQJT98765432. ...AKQJT98765432 AKQJT98765432... .AKQJT98765432..\"]\n";

    /// The `[Deal]` value for `one_suit_per_seat` with West listed first.
    const DEAL_FROM_WEST: &str =
        "[Deal \"W:...AKQJT98765432 AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432.\"]\n";

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

    /// North deals and declares 4S; East leads HK and one trick is played.
    fn known_board() -> Board {
        let auction = spade_auction();
        let contract = auction
            .contract(Seat::North, Vulnerability::None)
            .expect("the auction produced a contract");
        let play = PlayRecord::new(vec![
            Play::new(Seat::East, Card::HK),
            Play::new(Seat::South, Card::DA),
            Play::new(Seat::West, Card::C2),
            Play::new(Seat::North, Card::SA),
        ]);
        Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .number(1)
            .auction(auction)
            .contract(contract)
            .play(play)
            .build()
            .expect("the known board is valid")
    }

    /// The same auction and contract, but all 52 cards are played: each seat
    /// sheds one card of its single suit per trick, E,S,W,N from East's lead.
    fn full_play_board() -> Board {
        let deal = one_suit_per_seat();
        let auction = spade_auction();
        let contract = auction
            .contract(Seat::North, Vulnerability::None)
            .expect("the auction produced a contract");
        let mut remaining: [Vec<Card>; 4] =
            std::array::from_fn(|seat| deal[Seat::from(seat)].into_iter().collect());
        let mut plays = Vec::with_capacity(52);
        for _ in 0..13 {
            for seat in Seat::East.iter_from().take(4) {
                let card = remaining[seat as usize].remove(0);
                plays.push(Play::new(seat, card));
            }
        }
        Board::builder(deal, Seat::North, Vulnerability::None)
            .number(1)
            .auction(auction)
            .contract(contract)
            .play(PlayRecord::new(plays))
            .build()
            .expect("the full-play board is valid")
    }

    #[test]
    fn pbn_hand_record_golden_string() {
        // One whole suit per seat: each hand is a single suit, so the other
        // three suits show as the conventional empty `.` fields.
        assert_eq!(
            one_suit_per_seat().as_pbn(),
            "[Deal \"N:AKQJT98765432... .AKQJT98765432.. ..AKQJT98765432. ...AKQJT98765432\"]"
        );
    }

    #[test]
    fn as_pbn_from_lists_hands_clockwise_from_first() {
        assert_eq!(
            one_suit_per_seat().as_pbn_from(Seat::West),
            DEAL_FROM_WEST.trim_end()
        );
        assert_eq!(
            one_suit_per_seat().as_pbn_from(Seat::North),
            one_suit_per_seat().as_pbn()
        );
    }

    #[test]
    fn deal_only_board_emits_dealer_vulnerability_and_deal() {
        let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .build()
            .expect("a bare deal is a valid board");
        let expected = [
            "[Dealer \"N\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_NORTH,
        ]
        .concat();
        assert_eq!(board.as_pbn(), expected);
    }

    #[test]
    fn numbered_board_leads_with_the_board_tag() {
        let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .number(12)
            .build()
            .expect("a bare deal is a valid board");
        let expected = [
            "[Board \"12\"]\n",
            "[Dealer \"N\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_NORTH,
        ]
        .concat();
        assert_eq!(board.as_pbn(), expected);
    }

    #[test]
    fn no_trump_contract_uses_the_nt_strain_token() {
        let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .contract(Contract::new(
                3,
                Strain::NoTrumps,
                Seat::North,
                Vulnerable::No,
                Doubled::NotDoubled,
            ))
            .build()
            .expect("a contract-only board is valid");
        let expected = [
            "[Dealer \"N\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_NORTH,
            "[Declarer \"N\"]\n",
            "[Contract \"3NT\"]\n",
        ]
        .concat();
        assert_eq!(board.as_pbn(), expected);
    }

    #[test]
    fn doubled_and_redoubled_contracts_carry_the_risk_suffix() {
        let doubled = Board::builder(one_suit_per_seat(), Seat::East, Vulnerability::None)
            .contract(Contract::new(
                4,
                Strain::Spades,
                Seat::East,
                Vulnerable::No,
                Doubled::Doubled,
            ))
            .build()
            .expect("a doubled contract-only board is valid");
        let expected = [
            "[Dealer \"E\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_EAST,
            "[Declarer \"E\"]\n",
            "[Contract \"4SX\"]\n",
        ]
        .concat();
        assert_eq!(doubled.as_pbn(), expected);

        let redoubled = Board::builder(one_suit_per_seat(), Seat::South, Vulnerability::None)
            .contract(Contract::new(
                3,
                Strain::NoTrumps,
                Seat::South,
                Vulnerable::No,
                Doubled::Redoubled,
            ))
            .build()
            .expect("a redoubled contract-only board is valid");
        let expected = [
            "[Dealer \"S\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_SOUTH,
            "[Declarer \"S\"]\n",
            "[Contract \"3NTXX\"]\n",
        ]
        .concat();
        assert_eq!(redoubled.as_pbn(), expected);
    }

    #[test]
    fn contract_token_encodes_level_strain_and_risk() {
        let token = |level, strain, doubled| {
            contract_token(Contract::new(
                level,
                strain,
                Seat::North,
                Vulnerable::No,
                doubled,
            ))
        };
        assert_eq!(token(4, Strain::Spades, Doubled::NotDoubled), "4S");
        assert_eq!(token(3, Strain::NoTrumps, Doubled::Doubled), "3NTX");
        assert_eq!(token(2, Strain::Hearts, Doubled::Redoubled), "2HXX");
        assert_eq!(token(7, Strain::Clubs, Doubled::NotDoubled), "7C");
    }

    #[test]
    fn auction_writes_double_and_redouble_tokens() {
        // North 1S, East double, South redouble, then three passes close it.
        let auction = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Double),
            RecordedCall::new(Call::Redouble),
            pass(),
            pass(),
            pass(),
        ]);
        let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .auction(auction)
            .build()
            .expect("a legal doubled auction is a valid board");
        let expected = [
            "[Dealer \"N\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_NORTH,
            "[Auction \"N\"]\n",
            "1S X XX Pass\n",
            "Pass Pass\n",
        ]
        .concat();
        assert_eq!(board.as_pbn(), expected);
    }

    #[test]
    fn truncated_auction_ends_with_a_star() {
        // North 1S, pass, 2S and then cut off: the section is incomplete, so it
        // closes with `*` rather than a pass-out.
        let auction = Auction::new(vec![bid(1, Strain::Spades), pass(), bid(2, Strain::Spades)]);
        let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .auction(auction)
            .build()
            .expect("a legal truncated auction is a valid board");
        let expected = [
            "[Dealer \"N\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_NORTH,
            "[Auction \"N\"]\n",
            "1S Pass 2S\n",
            "*\n",
        ]
        .concat();
        assert_eq!(board.as_pbn(), expected);
    }

    #[test]
    fn passed_out_auction_writes_no_contract_or_play() {
        let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .number(3)
            .auction(Auction::new(vec![pass(), pass(), pass(), pass()]))
            .build()
            .expect("a passed-out board is valid");
        let expected = [
            "[Board \"3\"]\n",
            "[Dealer \"N\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_NORTH,
            "[Auction \"N\"]\n",
            "Pass Pass Pass Pass\n",
        ]
        .concat();
        assert_eq!(board.as_pbn(), expected);
    }

    #[test]
    fn wire_example_board_matches_the_spec_byte_for_byte() {
        let expected = [
            "[Board \"1\"]\n",
            "[Dealer \"N\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_NORTH,
            "[Declarer \"N\"]\n",
            "[Contract \"4S\"]\n",
            "[Auction \"N\"]\n",
            "1S Pass 2S Pass\n",
            "4S Pass Pass Pass\n",
            "[Play \"E\"]\n",
            "HK DA C2 SA\n",
            "*\n",
        ]
        .concat();
        assert_eq!(known_board().as_pbn(), expected);
    }

    #[test]
    fn full_play_is_thirteen_lines_without_a_star() {
        let expected = [
            "[Board \"1\"]\n",
            "[Dealer \"N\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_NORTH,
            "[Declarer \"N\"]\n",
            "[Contract \"4S\"]\n",
            "[Auction \"N\"]\n",
            "1S Pass 2S Pass\n",
            "4S Pass Pass Pass\n",
            "[Play \"E\"]\n",
            "HA DA CA SA\n",
            "HK DK CK SK\n",
            "HQ DQ CQ SQ\n",
            "HJ DJ CJ SJ\n",
            "HT DT CT ST\n",
            "H9 D9 C9 S9\n",
            "H8 D8 C8 S8\n",
            "H7 D7 C7 S7\n",
            "H6 D6 C6 S6\n",
            "H5 D5 C5 S5\n",
            "H4 D4 C4 S4\n",
            "H3 D3 C3 S3\n",
            "H2 D2 C2 S2\n",
        ]
        .concat();
        assert_eq!(full_play_board().as_pbn(), expected);
    }

    #[test]
    fn short_play_ends_with_a_star() {
        let auction = spade_auction();
        let contract = auction
            .contract(Seat::North, Vulnerability::None)
            .expect("the auction produced a contract");
        let play = PlayRecord::new(vec![
            Play::new(Seat::East, Card::HK),
            Play::new(Seat::South, Card::DA),
            Play::new(Seat::West, Card::C2),
            Play::new(Seat::North, Card::SA),
            Play::new(Seat::East, Card::HQ),
        ]);
        let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .auction(auction)
            .contract(contract)
            .play(play)
            .build()
            .expect("a short claimed play is a valid board");
        assert!(
            board
                .as_pbn()
                .ends_with("[Play \"E\"]\nHK DA C2 SA\nHQ\n*\n"),
            "short play should end with a starred trailing line: {}",
            board.as_pbn()
        );
    }

    #[test]
    fn play_without_a_contract_is_omitted() {
        // Without a contract there is no opening leader, so the `[Play]` tag
        // has no seat to name and the section is dropped entirely.
        let play = PlayRecord::new(vec![Play::new(Seat::North, Card::SA)]);
        let board = Board::builder(one_suit_per_seat(), Seat::North, Vulnerability::None)
            .play(play)
            .build()
            .expect("a play without a contract is a valid board");
        assert!(
            !board.as_pbn().contains("[Play"),
            "a contractless play cannot name a leader: {}",
            board.as_pbn()
        );
    }

    #[test]
    fn pbn_deal_first_letter_is_the_stored_dealer_not_the_board_number() {
        // Board 1 rotates to North, but this board stores West as dealer. Both
        // the `[Dealer]` tag and the `[Deal]` first letter must be West,
        // independent of the number (mirrors the LIN `md` test).
        let board = Board::builder(one_suit_per_seat(), Seat::West, Vulnerability::None)
            .number(1)
            .build()
            .expect("a bare deal is a valid board");
        let expected = [
            "[Board \"1\"]\n",
            "[Dealer \"W\"]\n",
            "[Vulnerable \"None\"]\n",
            DEAL_FROM_WEST,
        ]
        .concat();
        assert_eq!(board.as_pbn(), expected);
    }
}
