#![allow(dead_code)]
use crate::prelude::*;
use lazy_static::lazy_static;
use log::{error, warn};
use regex::Regex;

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
#[derive(Debug, Default)]
pub struct Bidding {
    bidding: Vec<Bid>,
}
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
            BiddingErrorKind::NonExistent(bid) => write!(f, "{}", bid),
        }
    }
}
impl std::error::Error for BiddingErrorKind {}

#[derive(Debug)]
#[non_exhaustive]
pub struct BiddingError {
    pub bid: Bid,
    pub kind: BiddingErrorKind,
}

impl std::fmt::Display for BiddingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to bid `{}`", self.bid)
    }
}

impl std::error::Error for BiddingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}
impl From<BidError> for BiddingError {}

impl Bidding {
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

#[derive(Debug, PartialEq)]
pub enum Bid {
    Pass,
    Double,
    Redouble,
    Contract(u8, Strain),
}

impl std::fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bid::Contract(level, strain) => write!(f, "{}{}", level, strain),
            Bid::Pass => write!(f, "Pass"),
            Bid::Redouble => write!(f, "Redouble"),
            Bid::Double => write!(f, "Double"),
        }
    }
}
#[derive(Debug)]
pub struct BidError {
    pub bid: String,
}

impl std::fmt::Display for BidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bid: {} cannot be parsed", self.bid)
    }
}

impl Bid {
    pub fn can_bid_over(&self, before: &Self) -> bool {
        match before {
            Bid::Contract(level, strain) => match self {
                Bid::Contract(self_level, self_strain) => {
                    level < self_level || (level == self_level && strain < self_strain)
                }
                Bid::Pass => true,
                Bid::Double => true,
                Bid::Redouble => false,
            },
            Bid::Pass => self != &Bid::Redouble,
            Bid::Double => self == &Bid::Redouble,
            Bid::Redouble => self.is_contract(),
        }
    }

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
                .split(",")
                .map(|string| string.to_owned())
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
            Ok(dealer) => Seat::from_u8(dealer),
            Err(_e) => {
                error!("unable to parse dealer");
                Seat::from_u8(0)
            }
        };
        let vec: Vec<_> = captures
            .name("hands")
            .unwrap()
            .as_str()
            .split(",")
            .map(|hand| Hand::from_str(hand).unwrap_or_else(|_| Hand::new_empty()))
            .collect();
        let hands: [Hand; 4] = vec.try_into().unwrap_or_else(|_| [Hand::new_empty(); 4]);
        (dealer, Hands::new_from(hands))
    } else {
        error!("unable to extract hands and dealer, returning empty!");
        (Seat::from_u8(0), Hands::new_from([Hand::new_empty(); 4]))
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
            data => Bid::Contract(
                data[0..1].parse::<u8>().unwrap_or_else(|_| {
                    match bidding.iter().filter(|&bid| bid.is_contract()).last() {
                        Some(&Bid::Contract(num, _)) => num,
                        None => 1u8,
                        Some(_) => {
                            return BidError {
                                bid: String::from(data),
                            }
                        }
                    }
                }),
                match &data[1..] {
                    "c" => Strain::Clubs,
                    "d" => Strain::Diamonds,
                    "h" => Strain::Hearts,
                    "s" => Strain::Spades,
                    "n" => Strain::NoTrumps,
                    _ => {
                        return BidError {
                            bid: String::from(data),
                        }
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
// remember to check for it.
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
