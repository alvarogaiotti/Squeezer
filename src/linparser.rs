#![allow(dead_code)]
use crate::prelude::*;
use lazy_static::lazy_static;
use log::{error, warn};
use regex::Regex;
use std::num::{NonZeroU8, ParseIntError};
/* Lin reference:
 pn|simodra,fra97,matmont,thevava|st||md|3S34JH258TQKD2JQC7,S27TH69D679TKAC23,S6QH47JD458C468JA,|rh||ah|Board 1|sv|o|mb|p|mb|1S|mb|2H|mb|2S|mb|3H|mb|4S|mb|p|mb|p|mb|p|pg||pc|C7|pc|C3|pc|CA|pc|C5|pg||pc|H4|pc|HA|pc|H5|pc|H6|pg||pc|SA|pc|S3|pc|S2|pc|S6|pg||pc|SK|pc|S4|pc|S7|pc|SQ|pg||pc|D3|pc|D2|pc|DA|pc|D5|pg||pc|DK|pc|D4|pc|H3|pc|DJ|pg||pc|C2|pc|C4|pc|C9|pc|SJ|pg||pc|HK|mc|11|
*/

pub struct LinParser<T: IntoIterator<Item = char>> {
    stream: T,
}

pub struct ParsedLin {
    players: String,
    hands: String,
    number: u8,
    bidding: String,
    play_sequence: String,
}

pub struct LinDeal {
    players: [String; 4],
    hands: Hands,
    number: u8,
    bidding: Option<Bidding>,
    play_sequence: Option<PlaySequence>,
    dealer: Seat,
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
        match &self.kind {
            LinParsingErrorKind::Bidding(e) => Some(e),
            _ => None,
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
        let mut players = players(s).into_iter();
        let players = {
            let mut data = [String::new(), String::new(), String::new(), String::new()];
            let mut counter = 0;
            while counter < 4 {
                data[counter] = players.next().unwrap();
                counter += 1;
            }
            data
        };
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
        match self {
            BiddingErrorKind::Insufficient => write!(f, "bidding is insufficient"),
            BiddingErrorKind::NonStarter => {
                write!(f, "bidding cannot be started with a Double or a Redouble")
            }
            BiddingErrorKind::NonExistent(bid) => write!(f, "{bid}"),
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
            BiddingErrorKind::Insufficient => None,
            BiddingErrorKind::NonStarter => None,
            BiddingErrorKind::NonExistent(ref e) => e.source(),
        }
    }
}

impl Bidding {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bidding: Vec::new(),
        }
    }
    pub fn iter(&self) -> std::slice::Iter<Bid> {
        self.bidding.iter()
    }
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
    lazy_static! {
        static ref PLAYERS: Regex = Regex::new(
            r"pn|(?P<south>[\w ]+,)(?P<west>[\w ]+,)(?P<north>[\w ]+,)(?P<east>[\w ]+)|st"
        )
        .unwrap();
    }
    PLAYERS.captures(lin).map_or_else(
        || {
            let mut data = Vec::new();
            for _ in 0..4 {
                data.push(String::from("NN"));
            }
            data
        },
        |captures| {
            captures
                .get(0)
                .unwrap() // Cannot fail
                .as_str()
                .split(',')
                .map(std::borrow::ToOwned::to_owned)
                .collect()
        },
    )
}

fn hands_and_dealer(lin: &str) -> (Seat, Hands) {
    lazy_static! {
        static ref HANDS: Regex = Regex::new(r"md\|(?P<dealer>\d)(?P<hands>[\w,]+?)\|").unwrap();
    }
    if let Some(captures) = HANDS.captures(lin) {
        let dealer = match captures.name("dealer").unwrap().as_str().parse::<u8>() {
            Ok(dealer) => Seat::from(dealer),
            Err(_e) => {
                error!("unable to parse dealer");
                Seat::from(0)
            }
        };
        let vec: Vec<_> = captures
            .name("hands")
            .unwrap()
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
    lazy_static! {
        static ref NUMBER: Regex = Regex::new(r"ah\|(?P<number>[\w, ]+?)\|").unwrap();
    }
    NUMBER
        .captures(lin)
        .unwrap()
        .name("number")
        .unwrap()
        .as_str()
        .parse::<u8>()
        .unwrap_or_else(|_| {
            warn!("unable to parse board number, using default");
            1
        })
}

fn bidding(lin: &str) -> Result<Bidding, BiddingError> {
    lazy_static! {
        static ref BIDDING: Regex = Regex::new(r"(?P<waste>mb\|(?P<bid>\w+?)\|)+?").unwrap();
    }
    let mut bidding = Bidding::new();
    let captures = BIDDING.captures_iter(lin);
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
                    "c" => Strain::Clubs,
                    "d" => Strain::Diamonds,
                    "h" => Strain::Hearts,
                    "s" => Strain::Spades,
                    "n" => Strain::NoTrumps,
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

pub struct PlaySequence {
    sequence: Vec<Card>,
}

impl IntoIterator for PlaySequence {
    type Item = Card;
    type IntoIter = <Vec<Card> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.sequence.into_iter()
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
    lazy_static! {
        static ref PLAY_SEQUENCE: Regex = Regex::new(r"(?P<waste>pc\|(?P<card>\w+?)\|)+?").unwrap();
    }
    let captures = PLAY_SEQUENCE.captures_iter(lin);
    let mut sequence: Vec<Card> = Vec::new();
    for card in captures {
        sequence.push(Card::from_str(&card["card"]).unwrap_or_else(|_| {
            error!("cannot parse a card, using joker");
            Card::JOKER
        }))
    }
    PlaySequence { sequence }
}

fn claim(lin: &str) -> Option<u8> {
    lazy_static! {
        static ref CLAIM: Regex = Regex::new(r"(?P<claim>mc\|(?P<tricks>\d+))").unwrap();
    };
    CLAIM
        .captures(lin)
        .map(|capture| match capture.name("tricks") {
            Some(tricks) => tricks.as_str().parse::<u8>().unwrap_or(0),
            None => 0,
        })
}
pub struct Scanner {
    cursor: usize,
    characters: Vec<char>,
}

impl Scanner {
    #[must_use]
    pub fn new(string: &str) -> Self {
        Self {
            cursor: 0,
            characters: string.chars().collect(),
        }
    }

    /// Returns the cursor position
    #[must_use]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns next char without advancing the cursor
    #[must_use]
    pub fn peek(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    /// Returns whether the string is exhausted or not
    #[must_use]
    pub fn exhausted(&self) -> bool {
        self.cursor == self.characters.len()
    }

    /// Returns next character, if available, advancing the cursor
    pub fn pop(&mut self) -> Option<&char> {
        match self.characters.get(self.cursor) {
            Some(character) => {
                self.cursor += 1;

                Some(character)
            }
            None => None,
        }
    }
}
