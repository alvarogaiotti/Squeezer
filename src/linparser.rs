// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

#![allow(dead_code)]
use crate::prelude::*;
use fmt::Display;
use log::error;
use regex::Regex;
use std::{
    num::{NonZeroU8, ParseIntError},
    sync::OnceLock,
};
/* Lin reference:
 pn|simodra,fra97,matmont,thevava|st||md|3S34JH258TQKD2JQC7,S27TH69D679TKAC23,S6QH47JD458C468JA,|rh||ah|Board 1|sv|o|mb|p|mb|1S|mb|2H|mb|2S|mb|3H|mb|4S|mb|p|mb|p|mb|p|pg||pc|C7|pc|C3|pc|CA|pc|C5|pg||pc|H4|pc|HA|pc|H5|pc|H6|pg||pc|SA|pc|S3|pc|S2|pc|S6|pg||pc|SK|pc|S4|pc|S7|pc|SQ|pg||pc|D3|pc|D2|pc|DA|pc|D5|pg||pc|DK|pc|D4|pc|H3|pc|DJ|pg||pc|C2|pc|C4|pc|C9|pc|SJ|pg||pc|HK|mc|11|
*/

/// A struct for parsing lin files.
/// You can feed a lin file and parse it,
/// expecting to obtai a [`ParsedLin`] struct.
/// The struct expects a something that can be turn into a
/// Iterator<Item=char> for parsing it.
/// You'll interact very rarely with this struct directly.
/// Instead, you'll use the [`LinDeal`] struct and its `from_str` method.

struct LinParser<T: IntoIterator<Item = char>> {
    stream: T,
}

/// This structure represents a full lin file parsed.
/// The result if a struct containing:
/// - the players
/// - the hands of the players
/// - the board number
/// - the bidding (if present)
/// - the play sequence (if present)
///
/// This is a 'low level' struct, and you'll interact with it very rarely.
/// Instead, you'll use the `LinDeal` struct and its methods.

pub struct ParsedLin {
    players: String,
    hands: String,
    number: u8,
    bidding: String,
    play_sequence: String,
}

/// This struct represents a deal parsed from a lin file.
/// This struct has everything needed to work with it.
/// It contains:
/// - the players
/// - the hands of the players
/// - the board number
/// - the bidding (if present)
/// - the play sequence (if present)
/// - the dealer
///
/// You can create a lin deal starting from a str with
#[derive(Debug)]
pub struct LinDeal {
    players: [String; 4],
    hands: Hands,
    number: u8,
    bidding: Option<Bidding>,
    play_sequence: Option<PlaySequence>,
    dealer: Seat,
}

impl LinDeal {
    #[must_use]
    pub fn players(&self) -> &[String; 4] {
        &self.players
    }

    #[must_use]
    pub fn hands(&self) -> Hands {
        self.hands
    }

    #[must_use]
    pub fn number(&self) -> u8 {
        self.number
    }

    #[must_use]
    pub fn bidding(&self) -> Option<&Bidding> {
        self.bidding.as_ref()
    }

    #[must_use]
    pub fn play_sequence(&self) -> Option<&PlaySequence> {
        self.play_sequence.as_ref()
    }

    #[must_use]
    pub fn dealer(&self) -> Seat {
        self.dealer
    }

    #[must_use]
    pub fn contract(&self) -> Option<Contract> {
        let mut contract = None;
        if let Some(ref bidding) = self.bidding {
            let mut declarer = None;
            let mut last_bidder = self.dealer as usize + bidding.len();
            let mut doubled = Doubled::NotDoubled;
            let mut strain_found = (false, Strain::NoTrumps, 7u8);
            for bid in bidding.iter().rev() {
                if strain_found.0 {
                    match *bid {
                        Bid::Contract(_, strain) if strain == strain_found.1 => {
                            last_bidder -= 1;
                            declarer = Some(last_bidder);
                        }
                        _ => last_bidder -= 1,
                    }
                } else {
                    match *bid {
                        Bid::Double => {
                            doubled = Doubled::Doubled;
                            last_bidder -= 1;
                        }
                        Bid::Pass => last_bidder -= 1,
                        Bid::Redouble => {
                            doubled = Doubled::Redoubled;
                            last_bidder -= 1;
                        }
                        Bid::Contract(level, strain) => {
                            strain_found = (true, strain, level.into());
                            last_bidder -= 1;
                            declarer = Some(last_bidder);
                        }
                    }
                }
            }
            if let Some(declarer) = declarer {
                let declarer = declarer.into();
                let vuln = Vulnerable::from_number_and_seat(self.number, declarer);
                contract = Some(Contract::new(
                    strain_found.2,
                    strain_found.1,
                    declarer,
                    vuln,
                    doubled,
                ));
            }
        }
        contract
    }
}

impl std::fmt::Display for LinDeal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Board n.{}\nDealer: {}\n{}",
            self.number, self.dealer, self.hands
        )
    }
}

/// Error kind that models possible errors
/// that could occur while parsing a `.lin` file
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum LinParsingErrorKind {
    Player,
    Hands,
    Number,
    Bidding(BiddingError),
}

#[derive(Debug, Clone)]
pub struct LinParsingError {
    lin: String,
    kind: LinParsingErrorKind,
}

impl LinParsingError {
    fn new<T: ToString + ?Sized>(kind: LinParsingErrorKind, lin: &T) -> Self {
        Self {
            lin: lin.to_string(),
            kind,
        }
    }
}

impl std::fmt::Display for LinParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to parse the following lin file:\n{}", self.lin)
    }
}

impl std::error::Error for LinParsingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.kind {
            LinParsingErrorKind::Bidding(ref e) => Some(e),
            LinParsingErrorKind::Player
            | LinParsingErrorKind::Hands
            | LinParsingErrorKind::Number => None,
        }
    }
}
impl From<BiddingError> for LinParsingErrorKind {
    fn from(value: BiddingError) -> Self {
        Self::Bidding(value)
    }
}

impl FromStr for LinDeal {
    type Err = LinParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let players = players(s);
        // Safety: Players always return a Vec of len 4;
        let players: [String; 4] = players.try_into().unwrap();
        let (dealer, hands) = dealer_and_hands(s)?;
        let number = number(s)?;
        let bidding = Some(match bidding(s) {
            Ok(value) => value,
            Err(e) => {
                return Err(LinParsingError {
                    lin: s.to_owned(),
                    kind: e.into(),
                })
            }
        });
        let play_sequence = Some(play_sequence(s));

        Ok(Self {
            players,
            hands,
            number,
            bidding,
            play_sequence,
            dealer,
        })
    }
}

#[cfg(feature = "dds")]
impl dds::AsDDSDeal for LinDeal {
    fn as_dds_deal(&self) -> dds::DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, hand) in self.hands.into_iter().enumerate() {
            for (index, suit) in hand.into_iter().enumerate() {
                remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
            }
        }
        remain_cards.into()
    }
}
/// Structure that represents a bidding sequence
/// made of [`Bid`]s
#[derive(Debug, Default)]
pub struct Bidding {
    bidding: Vec<Bid>,
}

impl IntoIterator for Bidding {
    type Item = Bid;
    type IntoIter = <Vec<Bid> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.bidding.into_iter()
    }
}

impl Display for Bidding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bid in &self.iter().chunks(4) {
            writeln!(f, "{}", bid.into_iter().format("-"))?;
        }
        Ok(())
    }
}

/// Enum modelling the possible kinds of errors we
/// can encounter while parsing a bidding sequence:
/// either the bid is not a starting bid (e.g. bidding starts with a double),
/// insufficient, or simply we were unable to parse the last [`Bid`]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum BiddingErrorKind {
    Insufficient,
    NonStarter,
    NonExistent(BidError),
}

impl std::fmt::Display for BiddingErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BiddingErrorKind::Insufficient => write!(f, "bidding is insufficient"),
            BiddingErrorKind::NonStarter => {
                write!(f, "bidding cannot be started with a Double or a Redouble")
            }
            BiddingErrorKind::NonExistent(ref bid) => write!(f, "{bid}"),
        }
    }
}
impl std::error::Error for BiddingErrorKind {}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BiddingError {
    bid: Bid,
    kind: BiddingErrorKind,
}

impl std::fmt::Display for BiddingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to bid `{}`", self.bid)
    }
}

impl std::error::Error for BiddingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.kind {
            BiddingErrorKind::Insufficient | BiddingErrorKind::NonStarter => None,
            BiddingErrorKind::NonExistent(ref e) => e.source(),
        }
    }
}

impl Bidding {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            bidding: Vec::new(),
        }
    }
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.bidding.len()
    }
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0usize
    }
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Bid> {
        self.bidding.get(index)
    }
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<Bid> {
        self.bidding.iter()
    }
    #[inline]
    /// # Errors
    /// - If the `Bid` is insufficient
    /// - If we are at the first `Bid` and find a Double or Redouble
    pub fn push(&mut self, bid: Bid) -> Result<(), BiddingError> {
        // If we have already some bids pushed
        if let Some(last) = self.bidding.last() {
            if bid.can_bid_over(last) {
                self.bidding.push(bid);
                Ok(())
            } else {
                Err(BiddingError {
                    bid,
                    kind: BiddingErrorKind::Insufficient,
                })
            }
            // If this is the first bid that we push and is a contract or pass go ahead
        } else if bid.is_contract() || bid == Bid::Pass {
            self.bidding.push(bid);
            Ok(())
            // Else terminate error
        } else {
            Err(BiddingError {
                bid,
                kind: BiddingErrorKind::NonStarter,
            })
        }
    }
}

impl<'a> IntoIterator for &'a Bidding {
    type Item = &'a Bid;
    type IntoIter = std::slice::Iter<'a, Bid>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// We model bids as an Enum, with possible contracts as tuple
/// variants containing [`NonZeroU8`] and a [`Strain`]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Bid {
    Pass,
    Double,
    Redouble,
    Contract(NonZeroU8, Strain),
}

impl std::fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bid::Contract(level, strain) => write!(f, "{level}{strain}"),
            Bid::Pass => write!(f, "Pass"),
            Bid::Redouble => write!(f, "Redouble"),
            Bid::Double => write!(f, "Double"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BidError {
    pub bid: String,
    pub kind: BidErrorKind,
}

/// Enum representing the various kind of errors
/// we could encounter while parsing a single bid:
/// either we are unable to parse the integer part of the bid
/// or we are unable to parse the strain of the bid.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BidErrorKind {
    Level(ParseIntError),
    Strain,
}
impl std::fmt::Display for BidErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BidErrorKind::Level(_e) => write!(f, "unable to parse level"),
            BidErrorKind::Strain => write!(f, "unable to parse strain"),
        }
    }
}

impl std::error::Error for BidError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.kind {
            BidErrorKind::Level(ref e) => Some(e),
            BidErrorKind::Strain => None,
        }
    }
}

impl std::fmt::Display for BidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bid: {} cannot be parsed", self.bid)
    }
}

impl Bid {
    #[must_use]
    pub fn can_bid_over(&self, before: &Self) -> bool {
        // We are not strictly checking if we keep bidding higher
        // e.g. 1S-p-1H is validated with this implementation since we do not keep track of the
        // level.
        // FIX : Implement a builder to enforce state
        match before {
            Bid::Contract(level, strain) => match self {
                Bid::Contract(self_level, self_strain) => {
                    level < self_level || (level == self_level && strain < self_strain)
                }
                Bid::Pass | Bid::Double => true,
                Bid::Redouble => false,
            },
            Bid::Pass => true,
            Bid::Double => self != &Bid::Double,
            Bid::Redouble => self != &Bid::Double || self != &Bid::Redouble,
        }
    }

    #[must_use]
    pub fn is_contract(&self) -> bool {
        matches!(self, Bid::Contract(_, _))
    }
}

fn players(lin: &str) -> Vec<String> {
    static PLAYERS: OnceLock<Regex> = OnceLock::new();
    let players = PLAYERS.get_or_init(|| {
        Regex::new(r"pn\|(?P<south>[\w ]+),(?P<west>[\w ]+),(?P<north>[\w ]+),(?P<east>[\w ]+)\|st")
            .unwrap()
    });
    players.captures(lin).map_or_else(
        || {
            let mut data = Vec::new();
            for _ in 0..4 {
                data.push(String::from("NN"));
            }
            data
        },
        |captures| {
            let mut vec: Vec<String> = captures
                .iter()
                .skip(1)
                .map(|x| x.map_or(String::from("NN"), |m| m.as_str().to_string()))
                .collect();
            vec.rotate_left(2);
            vec
        },
    )
}

// TODO: Correct error handling of this function and make it return a Result
// since this operation is clearly fallible and we should not fail silently
fn dealer_and_hands(lin: &str) -> Result<(Seat, Hands), LinParsingError> {
    static HANDS: OnceLock<Regex> = OnceLock::new();
    let hands =
        HANDS.get_or_init(|| Regex::new(r"md\|(?P<dealer>\d)(?P<hands>[\w,]+?)\|").unwrap());
    if let Some(captures) = hands.captures(lin) {
        let capture = captures.name("dealer").expect("No dealer").as_str();
        let dealer = match capture.parse::<u8>() {
            Ok(dealer) => Seat::from(dealer + 1),
            Err(_e) => {
                error!("unable to parse dealer");
                return Err(LinParsingError::new(LinParsingErrorKind::Hands, capture));
            }
        };
        let mut deck = Cards::ALL;
        let hand_capture = captures.name("hands").expect("No hands").as_str();

        let mut vec = hand_capture
            .split(',')
            .map(|hand_str| {
                let mut hand = Cards::from_str(hand_str)
                    .map_err(|_| LinParsingError::new(LinParsingErrorKind::Hands, hand_str))?;
                deck -= hand;
                // We should get a empty [`Hand`] only at the end of the stream.
                // Looks really error prone and should be corrected, so:
                // TODO: FIX ME
                if hand.is_empty() {
                    if deck.len() == 13 {
                        hand = deck;
                    } else {
                        return Err(LinParsingError::new(LinParsingErrorKind::Hands, hand_str));
                    }
                }
                Hand::try_from(hand)
                    .map_err(|_| LinParsingError::new(LinParsingErrorKind::Hands, hand_str))
            })
            .collect::<Result<Vec<Hand>, LinParsingError>>()?;
        // Lin format has player start from south.
        vec.rotate_right(2);
        let hands: [Hand; 4] = match vec.try_into() {
            Ok(array) => array,
            Err(_) => {
                return Err(LinParsingError::new(
                    LinParsingErrorKind::Hands,
                    hand_capture,
                ))
            }
        };
        Ok((dealer, Hands::new_from(hands)))
    } else {
        Err(LinParsingError::new(LinParsingErrorKind::Hands, lin))
    }
}

fn number(lin: &str) -> Result<u8, LinParsingError> {
    static NUMBER: OnceLock<Regex> = OnceLock::new();
    let number = NUMBER.get_or_init(|| Regex::new(r"ah\|\s?Board\s?(?P<number>[\d]+?)\|").unwrap());
    let number = number
        .captures(lin)
        .expect("No number capture")
        .name("number")
        .expect("No number named match")
        .as_str();
    number
        .parse::<u8>()
        .map_err(|_| LinParsingError::new(LinParsingErrorKind::Number, number))
}

fn bidding(lin: &str) -> Result<Bidding, BiddingError> {
    static BIDDING: OnceLock<Regex> = OnceLock::new();
    let bids = BIDDING.get_or_init(|| Regex::new(r"(?P<waste>mb\|(?P<bid>\w+?)\|)+?").unwrap());
    let captures = bids.captures_iter(lin);
    let mut bidding = Bidding::new();
    for cap in captures {
        let bid = match &cap["bid"] {
            "d" => Bid::Double,
            "r" => Bid::Redouble,
            "p" => Bid::Pass,
            contract_bid => Bid::Contract(
                match contract_bid[0..1].parse::<std::num::NonZeroU8>() {
                    Ok(num) => num,
                    Err(e) => {
                        return Err(BiddingError {
                            bid: bidding.into_iter().last().unwrap_or(Bid::Pass),
                            kind: BiddingErrorKind::NonExistent(BidError {
                                bid: String::from(contract_bid),
                                kind: BidErrorKind::Level(e),
                            }),
                        });
                    }
                },
                match &contract_bid[1..] {
                    "c" | "C" => Strain::Clubs,
                    "d" | "D" => Strain::Diamonds,
                    "h" | "H" => Strain::Hearts,
                    "s" | "S" => Strain::Spades,
                    "n" | "N" => Strain::NoTrumps,
                    _ => {
                        return Err(BiddingError {
                            bid: bidding.into_iter().last().unwrap_or(Bid::Pass),
                            kind: BiddingErrorKind::NonExistent(BidError {
                                bid: String::from(contract_bid),
                                kind: BidErrorKind::Strain,
                            }),
                        })
                    }
                },
            ),
        };
        bidding.push(bid)?;
    }
    Ok(bidding)
}

#[derive(Debug, Clone)]
pub struct PlaySequence {
    sequence: Vec<Card>,
}

impl PlaySequence {
    #[must_use]
    pub fn new(sequence: Vec<Card>) -> Self {
        Self { sequence }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn iter(&self) -> <&Self as std::iter::IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl IntoIterator for PlaySequence {
    type Item = Card;
    type IntoIter = <Vec<Card> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.sequence.into_iter()
    }
}

impl Display for PlaySequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for trick in &self.iter().chunks(4) {
            writeln!(f, "{}", trick.into_iter().format("-"))?;
        }
        Ok(())
    }
}

#[cfg(feature = "dds")]
impl TryFrom<&PlaySequence> for (dds::SuitSeq, dds::RankSeq) {
    type Error = dds::SeqError;

    fn try_from(value: &PlaySequence) -> Result<Self, Self::Error> {
        use dds::{RankSeq, SeqError, SuitSeq, SEQUENCE_LENGTH};
        let len = value.len();
        if len == 0 {
            return Err(SeqError::SequenceTooShort);
        } else if len > SEQUENCE_LENGTH {
            return Err(SeqError::SequenceTooLong);
        }

        let (suitseq, rankseq): (Vec<_>, Vec<_>) = value
            .into_iter()
            .map(|card| (i32::from(card.suit() as u8), i32::from(card.rank())))
            .unzip();
        Ok((
            SuitSeq::try_from(suitseq.as_slice())?,
            RankSeq::try_from(rankseq.as_slice())?,
        ))
    }
}

#[cfg(feature = "dds")]
impl TryFrom<&PlaySequence> for dds::PlayTraceBin {
    type Error = dds::SeqError;

    fn try_from(value: &PlaySequence) -> Result<Self, Self::Error> {
        let sequences = <(dds::SuitSeq, dds::RankSeq)>::try_from(value)?;
        Ok(Self::from_sequences(sequences.0, sequences.1))
    }
}

impl<'a> IntoIterator for &'a PlaySequence {
    type Item = &'a Card;
    type IntoIter = <&'a Vec<Card> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.sequence.iter()
    }
}

// This function does not return an error if it is unable to parse a card, but returns a JOKER, so
// remember to check for it and maybe try to reconstruct the right card based on the hand.
fn play_sequence(lin: &str) -> PlaySequence {
    static PLAY_SEQUENCE: OnceLock<Regex> = OnceLock::new();
    let play_sequence =
        PLAY_SEQUENCE.get_or_init(|| Regex::new(r"(?P<waste>pc\|(?P<card>\w+?)\|)+?").unwrap());
    let captures = play_sequence.captures_iter(lin);
    let mut sequence: Vec<Card> = Vec::new();
    for card in captures {
        sequence.push(Card::from_str(&card["card"]).unwrap_or_else(|_| {
            error!("cannot parse a card, using joker");
            Card::JOKER
        }));
    }
    PlaySequence { sequence }
}

fn claim(lin: &str) -> Option<u8> {
    static CLAIM: OnceLock<Regex> = OnceLock::new();
    let claim = CLAIM.get_or_init(|| Regex::new(r"(?P<claim>mc\|(?P<tricks>\d+))").unwrap());
    claim
        .captures(lin)
        .map(|capture| match capture.name("tricks") {
            Some(tricks) => tricks.as_str().parse::<u8>().unwrap_or(0),
            None => 0,
        })
}

mod test {
    use crate::{Contract, Doubled, Seat, Strain, Vulnerable};

    use super::LinDeal;
    use std::str::FromStr;

    #[test]
    fn parses_lin_test0() {
        let lin = String::from("pn|simodra,fra97,matmont,thevava|st||md|3S34JH258TQKD2JQC7,S27TH69D679TKAC23,S6QH47JD458C468JA,|rh||ah|Board 1|sv|o|mb|p|mb|1S|mb|2H|mb|2S|mb|3H|mb|4S|mb|p|mb|p|mb|p|pg||pc|C7|pc|C3|pc|CA|pc|C5|pg||pc|H4|pc|HA|pc|H5|pc|H6|pg||pc|SA|pc|S3|pc|S2|pc|S6|pg||pc|SK|pc|S4|pc|S7|pc|SQ|pg||pc|D3|pc|D2|pc|DA|pc|D5|pg||pc|DK|pc|D4|pc|H3|pc|DJ|pg||pc|C2|pc|C4|pc|C9|pc|SJ|pg||pc|HK|mc|11|");
        let parsed_lin = LinDeal::from_str(&lin).unwrap();
        assert_eq!(
            parsed_lin.contract().unwrap(),
            Contract::new(
                4,
                Strain::Spades,
                Seat::East,
                Vulnerable::No,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin_test1() {
        let lin = String::from("pn|gattochef,sebyx,Inter2018,fede00|st||md|3SAQ432HQJT72DT3CQ,SKJH983D974CT9876,S965HK654DKJ6CAJ5,ST87HADAQ852CK432|rh||ah|Board 1|sv|o|mb|1C|an|2+|mb|1D|mb|1H|an|picche|mb|2D|mb|p|mb|p|mb|3H|mb|p|mb|3S|mb|p|mb|4S|mb|p|mb|p|mb|p|pg||pc|DA|pc|D3|pc|D9|pc|D6|pg||pc|HA|pc|H2|pc|H9|pc|H4|pg||pc|D8|pc|DT|pc|D7|pc|DJ|pg||pc|S5|pc|S7|pc|SA|pc|SJ|pg||pc|CQ|pc|CT|pc|CA|pc|C2|pg||pc|S6|pc|S8|pc|SQ|pc|SK|pg||pc|H8|mc|9|");
        let parsed_lin = LinDeal::from_str(&lin).unwrap();
        println!("{parsed_lin}");
        assert_eq!(
            parsed_lin.contract().unwrap(),
            Contract::new(
                4,
                Strain::Spades,
                Seat::North,
                Vulnerable::No,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin_test2() {
        let lin = String::from(include_str!("../tests/4207070707.lin"));
        let parsed_lin = LinDeal::from_str(&lin).unwrap();
        println!("{parsed_lin}");
        assert_eq!(
            parsed_lin.contract().unwrap(),
            Contract::new(
                4,
                Strain::Spades,
                Seat::North,
                Vulnerable::No,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin_test3() {
        let lin = String::from(include_str!("../tests/4207076395.lin"));
        let parsed_lin = LinDeal::from_str(&lin).unwrap();
        println!("{parsed_lin}");
        assert_eq!(
            parsed_lin.contract().unwrap(),
            Contract::new(
                6,
                Strain::Diamonds,
                Seat::North,
                Vulnerable::Yes,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin_test4() {
        let lin = String::from(include_str!("../tests/4207079732.lin"));
        let parsed_lin = LinDeal::from_str(&lin).unwrap();
        println!("{parsed_lin}");
        assert_eq!(
            parsed_lin.contract().unwrap(),
            Contract::new(
                3,
                Strain::NoTrumps,
                Seat::West,
                Vulnerable::Yes,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin_test6() {
        let lin = String::from(include_str!("../tests/4207083254.lin"));
        let parsed_lin = LinDeal::from_str(&lin).unwrap();
        println!("{parsed_lin}");
        assert_eq!(
            parsed_lin.contract().unwrap(),
            Contract::new(
                3,
                Strain::Clubs,
                Seat::South,
                Vulnerable::Yes,
                Doubled::NotDoubled
            )
        );
    }
    #[test]
    fn parses_lin_test7() {
        let lin = String::from(include_str!("../tests/4207093356.lin"));
        let parsed_lin = LinDeal::from_str(&lin).unwrap();
        println!("{parsed_lin}");
    }
    #[test]
    fn parses_lin_test8() {
        let lin = String::from(include_str!("../tests/4207094241.lin"));
        let parsed_lin = LinDeal::from_str(&lin).unwrap();
        println!("{parsed_lin}");
        assert_eq!(parsed_lin.contract(), None);
    }
}
