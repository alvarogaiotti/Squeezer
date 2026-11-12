// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Cross-crate serialization tests: the model writers against the importers.
//!
//! The model crate covers the writer output itself; what can only be tested
//! here is the round trip through this crate's LIN and PBN importers, which
//! read the same records back.

use std::num::NonZeroU8;
use std::str::FromStr;

use quickcheck::{Arbitrary, Gen, QuickCheck};

use squeezer_cards::*;
use squeezer_model::record::position::trick_winner;
use squeezer_model::*;

mod common;

/// What a file format can carry of a board.
///
/// The LIN and PBN generators differ only in these two facts, so they are data
/// consumed by the single [`representable_board`] builder and by each format's
/// `Arbitrary` wrapper. Keeping the difference in one value is what stops the
/// two generators from drifting into different sampled domains.
#[derive(Clone, Copy)]
struct Representability {
    /// Whether the format records a claimed trick count: LIN's `mc|` tag.
    records_claim: bool,
    /// Whether a call keeps its alert marker and annotation: LIN's `!` and
    /// `an|`, which PBN does not write and its importer discards.
    records_call_metadata: bool,
}

/// LIN keeps every modeled field, including the claim and the call metadata.
const LIN_REPRESENTABILITY: Representability = Representability {
    records_claim: true,
    records_call_metadata: true,
};

/// PBN has no claim tag, and its importer discards alert markers and note
/// references, so the generator leaves those fields absent.
const PBN_REPRESENTABILITY: Representability = Representability {
    records_claim: false,
    records_call_metadata: false,
};

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
    let mut hands = [Cards::EMPTY; 4];
    for slot in &mut hands {
        let mut cards = Cards::EMPTY;
        for _ in 0..13 {
            let index = usize::arbitrary(g) % deck.len();
            cards = cards.insert(deck.swap_remove(index));
        }
        *slot = cards;
    }
    Deal::new(hands).expect("the four hands partition the deck")
}

/// A legal, complete auction, built one legal call at a time and stopped as
/// soon as the call list completes.
///
/// The completion rule is [`Auction::is_complete`], the same predicate
/// validation and the writers use: the loop stops on the call that ends the
/// auction, so the generator can never append a call after the fourth opening
/// pass or the third closing pass, and it never emits a cut auction either.
/// Bids strictly increase and there are only thirty-five, so the loop always
/// reaches its closing passes.
fn arbitrary_auction(g: &mut Gen, dealer: Seat) -> Auction {
    let mut calls = Vec::new();
    let mut highest: Option<(u8, Strain)> = None;
    let mut last_non_pass: Option<(Call, Seat)> = None;
    let mut index = 0usize;
    loop {
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
        // Cloning the short call list is cheaper than duplicating the rule
        // here; keeping one predicate is what stops the generator drifting
        // from validation.
        if Auction::new(calls.clone()).is_complete() {
            break;
        }
        index += 1;
    }
    Auction::new(calls)
}

/// The four opening passes: a complete auction with no contract.
fn pass_out() -> Auction {
    Auction::new(vec![RecordedCall::new(Call::Pass); 4])
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
        remaining[seat as usize] = deal[seat].into_iter().collect();
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

/// A board the given format can represent, built once for both formats.
///
/// The shared domain is derived, never stored: the contract comes from the
/// auction and the play is built only when a contract exists, because `[Play]`
/// names a leader only the contract fixes and both importers re-derive the
/// contract from the auction. Only the [`Representability`] decides which
/// optional parts survive: the claim and the call metadata.
///
/// A random legal auction almost always contains a bid — four opening passes
/// need four consecutive picks from up to thirty-six options — so two auctions
/// in eight are fixed pass-outs, to exercise the contractless writer and
/// importer paths. A further one in eight records no auction at all, which both
/// formats distinguish from a passed-out one.
fn representable_board(g: &mut Gen, format: Representability) -> Board {
    let deal = arbitrary_deal(g);
    let dealer = arbitrary_seat(g);
    let vulnerability = arbitrary_vulnerability(g);
    let number = bool::arbitrary(g).then(|| u8::arbitrary(g));
    let auction = match u8::arbitrary(g) % 8 {
        0 => None,
        1 | 2 => Some(pass_out()),
        _ => Some(arbitrary_auction(g, dealer)),
    };
    let auction = auction.map(|auction| {
        if format.records_call_metadata {
            auction
        } else {
            // PBN writes no alert marker and no annotation, and its importer
            // reads note references and suffixes only to discard them, so a
            // call carrying either cannot survive the round trip.
            without_call_metadata(&auction)
        }
    });
    let contract = auction
        .as_ref()
        .and_then(|auction| auction.contract(dealer, vulnerability));
    let play = contract
        .as_ref()
        .map(|contract| arbitrary_play(g, &deal, *contract));
    // The claim is independent of the play: LIN's `mc|` needs no preceding
    // `pc|` cards, so a claimed board with no recorded play is representable
    // and sampled on its own.
    let claim = (format.records_claim && bool::arbitrary(g)).then(|| u8::arbitrary(g) % 14);
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
        Self(representable_board(g, LIN_REPRESENTABILITY))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(shrink_board(&self.0).into_iter().map(Self))
    }
}

/// An auction with every call's LIN-only metadata stripped.
fn without_call_metadata(auction: &Auction) -> Auction {
    Auction::new(
        auction
            .calls()
            .iter()
            .map(|recorded| RecordedCall::new(recorded.call))
            .collect(),
    )
}

/// A board that a PBN round trip can represent: the same domain as
/// [`AnyBoard`], with the fields PBN cannot write left absent by
/// [`PBN_REPRESENTABILITY`].
#[derive(Clone, Debug)]
struct AnyPbnBoard(Board);

impl Arbitrary for AnyPbnBoard {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(representable_board(g, PBN_REPRESENTABILITY))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(shrink_board(&self.0).into_iter().map(Self))
    }
}

/// The board fields a shrink candidate keeps or replaces.
///
/// Bundling them turns every candidate into a one-field edit of the board under
/// shrink, instead of a repeated seven-argument call whose two `Option<u8>`
/// fields are easy to transpose.
#[derive(Clone)]
struct Parts<'a> {
    deal: &'a Deal,
    number: Option<u8>,
    dealer: Seat,
    vulnerability: Vulnerability,
    auction: Option<Auction>,
    play: Option<PlayRecord>,
    claim: Option<u8>,
}

impl<'a> Parts<'a> {
    fn of(board: &'a Board) -> Self {
        Self {
            deal: board.deal(),
            number: board.number(),
            dealer: board.dealer(),
            vulnerability: board.vulnerability(),
            auction: board.auction().cloned(),
            play: board.play().cloned(),
            claim: board.claim(),
        }
    }

    /// Builds the candidate, deriving the contract its auction implies.
    ///
    /// The contract is never a candidate on its own: [`Board::new`] rejects a
    /// stored contract that disagrees with its auction, so every auction edit
    /// re-derives it. A candidate whose parts no longer fit — a shortened
    /// auction that stopped being legal, say — builds to `None` and is dropped.
    fn build(&self) -> Option<Board> {
        let contract = self
            .auction
            .as_ref()
            .and_then(|auction| auction.contract(self.dealer, self.vulnerability));
        Board::new(
            self.deal.clone(),
            self.number,
            self.dealer,
            self.vulnerability,
            self.auction.clone(),
            contract,
            self.play.clone(),
            self.claim,
        )
        .ok()
    }
}

/// The boards a shrinker tries next, biggest reduction first.
///
/// quickcheck keeps the first candidate that still fails, so the order decides
/// how fast a failure reaches a small case: dropping the auction and the play
/// comes before trimming a single call, and the purely cosmetic fields (board
/// number, claim) come last. Both formats share this one shrinker.
///
/// A trimmed auction is cut: both formats represent one (LIN writes the calls
/// it has, PBN marks the cut with `*`), and the shrink-candidate properties pin
/// that every candidate survives a round trip.
///
/// The deal is not shrunk: a [`Deal`] is exactly thirteen cards per seat, so
/// there is no smaller deal to shrink to — the deal *is* part of the offending
/// board a report should show.
fn shrink_board(board: &Board) -> Vec<Board> {
    let base = Parts::of(board);
    let mut candidates = Vec::new();
    let mut push = |candidate: Parts<'_>| {
        let candidate = candidate.build();
        // An equal candidate would make quickcheck shrink the same value
        // forever, so it is never emitted.
        if let Some(candidate) = candidate
            && candidate != *board
            && !candidates.contains(&candidate)
        {
            candidates.push(candidate);
        }
    };

    // A board with no auction has no contract and no play to hang off it.
    if base.auction.is_some() {
        push(Parts {
            auction: None,
            play: None,
            ..base.clone()
        });
    }

    if base.play.is_some() {
        push(Parts {
            play: None,
            ..base.clone()
        });
    }

    // Removing the play's last card. A one-card play has the candidate above as
    // its smaller case, and `Board::new` normalises an empty play anyway.
    if let Some(play) = &base.play
        && play.len() > 1
    {
        push(Parts {
            play: Some(PlayRecord::new(play.plays()[..play.len() - 1].to_vec())),
            ..base.clone()
        });
    }

    // Removing the auction's last call. The shortest auction is one call: an
    // empty auction writes no tag at all and re-imports as no auction, which is
    // the candidate above, not a board of its own.
    if let Some(auction) = &base.auction
        && auction.len() > 1
    {
        // A shorter call list can re-derive a different contract and leader, so
        // the play is dropped rather than re-attributed.
        push(Parts {
            auction: Some(Auction::new(auction.calls()[..auction.len() - 1].to_vec())),
            play: None,
            ..base.clone()
        });
    }

    // LIN-only call metadata: dropping it keeps the calls and the play.
    if let Some(auction) = &base.auction
        && auction
            .calls()
            .iter()
            .any(|recorded| recorded.alert || recorded.annotation.is_some())
    {
        push(Parts {
            auction: Some(without_call_metadata(auction)),
            ..base.clone()
        });
    }

    if base.number.is_some() {
        push(Parts {
            number: None,
            ..base.clone()
        });
    }

    if base.claim.is_some() {
        push(Parts {
            claim: None,
            ..base.clone()
        });
    }

    // Rotating the dealer changes the declarer and the opening leader, so the
    // play is dropped rather than re-attributed. Legality is unaffected: every
    // seat rotates together, so who is entitled to double whom is preserved.
    if base.dealer != Seat::North {
        push(Parts {
            dealer: Seat::North,
            play: None,
            ..base.clone()
        });
    }

    if base.vulnerability != Vulnerability::None {
        push(Parts {
            vulnerability: Vulnerability::None,
            ..base.clone()
        });
    }

    candidates
}

/// An auction straight from the round-trip generator, with the dealer its
/// seats rotate from.
#[derive(Clone, Debug)]
struct AnyGeneratedAuction {
    dealer: Seat,
    auction: Auction,
}

impl Arbitrary for AnyGeneratedAuction {
    fn arbitrary(g: &mut Gen) -> Self {
        let dealer = arbitrary_seat(g);
        Self {
            dealer,
            auction: arbitrary_auction(g, dealer),
        }
    }
}

/// The generator must emit auctions that pass the completion rule: the loop
/// stops exactly at the call that ends the auction, so [`Auction::is_complete`]
/// holds, and nothing can follow the end, so [`Auction::validate`] holds too.
/// A cut or over-long auction would make `Board::new` reject the generated
/// board, and the round-trip properties would panic instead of testing the
/// formats.
#[quickcheck_macros::quickcheck]
fn generated_auctions_are_complete_and_legal(any: AnyGeneratedAuction) -> bool {
    any.auction.is_complete() && any.auction.validate(any.dealer).is_ok()
}

/// `parse(write(board)) == board` through the LIN importer.
///
/// The importer requires a `pn|` line, so the board is wrapped in a one-board
/// file; the room is not part of the board and stays `None`.
fn lin_round_trips(board: &Board) -> bool {
    let lin = format!("pn|N,E,S,W|{}", board.as_lin());
    let Ok(file) = <squeezer_io::linparser::LinFile as FromStr>::from_str(&lin) else {
        return false;
    };
    file.boards.len() == 1 && file.boards[0] == *board
}

/// `parse(write(board)) == board` through the PBN importer.
///
/// A board's own writer is a complete one-game file: it always emits
/// `[Dealer]`, `[Vulnerable]` and `[Deal]`, the three tags an importer
/// needs to build a board, so no wrapper is required (unlike LIN's `pn|`).
///
/// Unlike [`lin_round_trips`], the assertions panic rather than return `false`:
/// that puts the written PBN in the failure message, so a report says which
/// part of the format broke. quickcheck treats either outcome as a failure.
fn pbn_round_trips(board: &Board) -> bool {
    let pbn = board.as_pbn();
    let file = <squeezer_io::pbnparser::PbnFile as FromStr>::from_str(&pbn)
        .unwrap_or_else(|error| panic!("the writer's output must re-import: {error}"));
    assert_eq!(
        file.boards.len(),
        1,
        "a board writes exactly one game:\n{pbn}"
    );
    assert_eq!(
        file.boards[0], *board,
        "the board must survive a PBN round trip:\n{pbn}"
    );
    true
}

#[quickcheck_macros::quickcheck]
fn lin_round_trips_generated_boards(any: AnyBoard) -> bool {
    lin_round_trips(&any.0)
}

#[quickcheck_macros::quickcheck]
fn pbn_round_trips_generated_boards(any: AnyPbnBoard) -> bool {
    pbn_round_trips(&any.0)
}

/// Every board a shrinker proposes must stay inside the format's domain: a
/// reported case has to be a board the format can represent, including the
/// cut auction and the shortened play a reduction produces.
#[quickcheck_macros::quickcheck]
fn lin_shrink_candidates_stay_in_the_format(any: AnyBoard) -> bool {
    any.shrink().all(|candidate| lin_round_trips(&candidate.0))
}

#[quickcheck_macros::quickcheck]
fn pbn_shrink_candidates_stay_in_the_format(any: AnyPbnBoard) -> bool {
    any.shrink().all(|candidate| pbn_round_trips(&candidate.0))
}

/// A property that rejects every LIN board, so quickcheck has to shrink
/// whatever it drew.
fn rejects_every_lin_board(_: AnyBoard) -> bool {
    false
}

/// A property that rejects every PBN board.
fn rejects_every_pbn_board(_: AnyPbnBoard) -> bool {
    false
}

/// A failing LIN property reports the offending board.
///
/// quickcheck only names the arguments once a shrunk candidate failed too, so
/// an empty shrinker reports `No Arguments Provided` instead of the board. The
/// small case itself is pinned by
/// [`a_deliberately_failing_board_shrinks_to_a_small_case`].
#[test]
fn a_failing_lin_property_reports_the_board() {
    let mut quickcheck = QuickCheck::new().tests(1);
    let failure = quickcheck
        .quicktest(rejects_every_lin_board as fn(AnyBoard) -> bool)
        .expect_err("the property always fails");

    let report = format!("{failure:?}");
    assert!(
        report.contains("AnyBoard("),
        "the report must name the offending board: {report}"
    );
    assert!(
        !report.contains("No Arguments Provided"),
        "the shrinker must report a witness, not the empty-shrinker message: {report}"
    );
}

/// The same guarantee for a PBN failure.
#[test]
fn a_failing_pbn_property_reports_the_board() {
    let mut quickcheck = QuickCheck::new().tests(1);
    let failure = quickcheck
        .quicktest(rejects_every_pbn_board as fn(AnyPbnBoard) -> bool)
        .expect_err("the property always fails");

    let report = format!("{failure:?}");
    assert!(
        report.contains("AnyPbnBoard("),
        "the report must name the offending board: {report}"
    );
    assert!(
        !report.contains("No Arguments Provided"),
        "the shrinker must report a witness, not the empty-shrinker message: {report}"
    );
}

/// A board with every field LIN carries filled in: a numbered board with a
/// doubled auction, a whole trick of play and a claim. It is the largest case
/// [`representable_board`] can build for LIN.
fn largest_lin_board() -> Board {
    let deal = one_suit_per_seat();
    let auction = Auction::new(vec![
        RecordedCall::new(Call::Bid {
            level: NonZeroU8::new(1).expect("1 is non-zero"),
            strain: Strain::Spades,
        }),
        RecordedCall::new(Call::Double),
        RecordedCall::new(Call::Pass),
        RecordedCall::new(Call::Pass),
        RecordedCall::new(Call::Pass),
    ]);
    let contract = auction
        .contract(Seat::North, Vulnerability::All)
        .expect("the auction names a contract");
    // The play starts with the declarer's left-hand opponent and rotates, as
    // the importers attribute it.
    let play = PlayRecord::new(
        [Seat::East, Seat::South, Seat::West, Seat::North]
            .into_iter()
            .map(|seat| {
                Play::new(
                    seat,
                    deal[seat]
                        .into_iter()
                        .next()
                        .expect("every hand holds thirteen cards"),
                )
            })
            .collect(),
    );
    Board::builder(deal, Seat::North, Vulnerability::All)
        .number(7)
        .auction(auction)
        .contract(contract)
        .play(play)
        .claim(9)
        .build()
        .expect("the largest LIN board is valid")
}

/// Replays quickcheck's shrinking loop: take the first candidate whose
/// property still returns `false`, and shrink that, until none does.
fn shrink_to_fixpoint(board: AnyBoard) -> AnyBoard {
    let mut current = board;
    'outer: loop {
        for candidate in current.clone().shrink() {
            // A property returns `false` on failure, so this candidate is still
            // a witness and becomes the next value to shrink.
            if !rejects_every_lin_board(candidate.clone()) {
                current = candidate;
                continue 'outer;
            }
        }
        return current;
    }
}

/// The deliberately failing board shrinks to a small case.
///
/// The property rejects every board, so the whole chain of reductions is
/// accepted and the fixpoint is the smallest board the shrinker can build: the
/// deal alone, dealt by North and non-vulnerable.
#[test]
fn a_deliberately_failing_board_shrinks_to_a_small_case() {
    let largest = largest_lin_board();
    let shrunk = shrink_to_fixpoint(AnyBoard(largest.clone()));
    let smallest = Board::builder(largest.deal().clone(), Seat::North, Vulnerability::None)
        .build()
        .expect("a bare deal is a valid board");
    assert_eq!(
        shrunk.0, smallest,
        "the shrinker must strip every optional part"
    );
}

/// A deal with one whole suit per seat, so a golden hand listing is easy to
/// read and each hand exercises three void suits.
///
/// The model crate's own writer tests use the same fixture internally; this
/// copy serves the container and round-trip tests in this crate.
fn one_suit_per_seat() -> Deal {
    Deal::new([
        Cards::from_str("SAKQJT98765432").expect("spades hand parses"),
        Cards::from_str("HAKQJT98765432").expect("hearts hand parses"),
        Cards::from_str("DAKQJT98765432").expect("diamonds hand parses"),
        Cards::from_str("CAKQJT98765432").expect("clubs hand parses"),
    ])
    .expect("one suit per seat is a valid deal")
}

#[test]
fn md_digit_reparses_as_the_stored_dealer() {
    use squeezer_io::linparser::LinFile;

    // Board 1 rotates to North, but this board stores West as dealer. The
    // writer's model-side assertion lives in `squeezer-model`; what this test
    // adds is that the importer reads the same `md|` digit back as West, not as
    // the North that board 1 would otherwise rotate to.
    let board = Board::builder(one_suit_per_seat(), Seat::West, Vulnerability::None)
        .number(1)
        .build()
        .expect("a bare deal is a valid board");
    let lin = board.as_lin();

    let parsed = <LinFile as FromStr>::from_str(&format!("pn|N,E,S,W|{lin}")).expect("reparses");
    assert_eq!(parsed.boards[0].dealer(), Seat::West);
    assert_eq!(parsed.boards[0].number(), Some(1));
}

#[test]
fn fixture_board_round_trips_through_lin() {
    use squeezer_io::linparser::LinFile;
    let data = common::read_fixture("4207070707.lin");
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
#[test]
fn lin_file_round_trips_the_comma_free_fixture() {
    use squeezer_io::linparser::LinFile;
    let data = common::read_fixture("83469.lin");
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
#[test]
fn comma_in_metadata_is_documented_as_lossy() {
    use squeezer_io::linparser::{LinFile, LinMetadata, Players};
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
