#![allow(dead_code)]
use crate::prelude::*;
use log::{error, warn};
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
/// expecting to obtai a `ParsedLin` struct.
/// The struct expects a something that can be turn into a
/// Iterator<Item=char> for parsing it.
/// You'll interact very rarely with this struct directly.
/// Instead, you'll use the `LinDeal` struct and its `from_str` method.

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
            let mut declarer = self.dealer as usize + bidding.len();
            let mut doubled = Doubled::NotDoubled;
            for bid in bidding.iter().rev() {
                match *bid {
                    Bid::Double => {
                        doubled = Doubled::Doubled;
                        declarer -= 1;
                    }
                    Bid::Pass => declarer -= 1,
                    Bid::Redouble => {
                        doubled = Doubled::Redoubled;
                        declarer -= 1;
                    }
                    Bid::Contract(level, strain) => {
                        let declarer = declarer.into();
                        let vuln = Vulnerable::from_number_and_seat(self.number, declarer);
                        contract =
                            Some(Contract::new(level.into(), strain, declarer, vuln, doubled));
                        break;
                    }
                }
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
#[derive(Debug)]
#[non_exhaustive]
pub enum LinParsingErrorKind {
    Player,
    Hands,
    Number,
    Bidding(BiddingError),
}

#[derive(Debug)]
pub struct LinParsingError {
    lin: String,
    kind: LinParsingErrorKind,
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
        let (dealer, hands) = hands_and_dealer(s);
        let number = number(s);
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

/// Enum modelling the possible kinds of errors we
/// can encounter while parsing a bidding sequence:
/// either the bid is not a starting bid (e.g. bidding starts with a double),
/// insufficient, or simply we were unable to parse the last [`Bid`]
#[derive(Debug)]
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

#[derive(Debug)]
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
        } else if bid.is_contract() || bid == Bid::Pass {
            self.bidding.push(bid);
            Ok(())
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
#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct BidError {
    pub bid: String,
    pub kind: BidErrorKind,
}

/// Enum representing the various kind of errors
/// we could encounter while parsing a single bid:
/// either we are unable to parse the integer part of the bid
/// or we are unable to parse the strain of the bid.
#[non_exhaustive]
#[derive(Debug)]
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
        match before {
            Bid::Contract(level, strain) => match self {
                Bid::Contract(self_level, self_strain) => {
                    level < self_level || (level == self_level && strain < self_strain)
                }
                Bid::Pass | Bid::Double => true,
                Bid::Redouble => false,
            },
            Bid::Pass => self != &Bid::Redouble,
            Bid::Double => self == &Bid::Redouble,
            Bid::Redouble => self.is_contract(),
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
            let vec = captures
                .iter()
                .skip(1)
                .map(|x| x.map_or(String::from("NN"), |m| m.as_str().to_string()))
                .collect();
            vec
        },
    )
}

fn hands_and_dealer(lin: &str) -> (Seat, Hands) {
    static HANDS: OnceLock<Regex> = OnceLock::new();
    let hands =
        HANDS.get_or_init(|| Regex::new(r"md\|(?P<dealer>\d)(?P<hands>[\w,]+?)\|").unwrap());
    if let Some(captures) = hands.captures(lin) {
        let dealer = match captures
            .name("dealer")
            .expect("No dealer")
            .as_str()
            .parse::<u8>()
        {
            Ok(dealer) => Seat::from(dealer),
            Err(_e) => {
                error!("unable to parse dealer");
                Seat::from(0)
            }
        };
        let vec: Vec<_> = captures
            .name("hands")
            .expect("No hands")
            .as_str()
            .split(',')
            .map(|hand| Hand::from_str(hand).unwrap_or_else(|_| Hand::new_empty()))
            .collect();
        let hands: [Hand; 4] = vec.try_into().unwrap_or_else(|_| [Hand::new_empty(); 4]);
        (dealer, Hands::new_from(hands))
    } else {
        error!("unable to extract hands and dealer, returning empty!");
        (Seat::from(0), Hands::new_from([Hand::new_empty(); 4]))
    }
}

fn number(lin: &str) -> u8 {
    static NUMBER: OnceLock<Regex> = OnceLock::new();
    let number = NUMBER.get_or_init(|| Regex::new(r"ah\|(?P<number>[\w, ]+?)\|").unwrap());
    number
        .captures(lin)
        .expect("No number capture")
        .name("number")
        .expect("No number named match")
        .as_str()
        .parse::<u8>()
        .unwrap_or_else(|_| {
            warn!("unable to parse board number, using default");
            1
        })
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

#[derive(Debug)]
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

#[test]
fn parses_lin_test() {
    let lin = String::from("pn|simodra,fra97,matmont,thevava|st||md|3S34JH258TQKD2JQC7,S27TH69D679TKAC23,S6QH47JD458C468JA,|rh||ah|Board 1|sv|o|mb|p|mb|1S|mb|2H|mb|2S|mb|3H|mb|4S|mb|p|mb|p|mb|p|pg||pc|C7|pc|C3|pc|CA|pc|C5|pg||pc|H4|pc|HA|pc|H5|pc|H6|pg||pc|SA|pc|S3|pc|S2|pc|S6|pg||pc|SK|pc|S4|pc|S7|pc|SQ|pg||pc|D3|pc|D2|pc|DA|pc|D5|pg||pc|DK|pc|D4|pc|H3|pc|DJ|pg||pc|C2|pc|C4|pc|C9|pc|SJ|pg||pc|HK|mc|11|");
    let parsed_lin = LinDeal::from_str(&lin).unwrap();
    println!("{parsed_lin}");
}
