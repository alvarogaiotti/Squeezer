use crate::prelude::*;
use anyhow::{bail, Result};
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
    vulnerability: Vulnerability,
    bidding: Bidding,
}

pub struct Bidding {
    bidding: Vec<Bid>,
}

impl Bidding {
    pub fn push(&mut self, bid: Bid) -> Result<()> {
        if let Some(last) = self.bidding.last() {
            if bid.can_bid_over(last) {
                self.bidding.push(bid);
                Ok(())
            } else {
                bail!("bid passed cannot be subsequent to {}", last)
            }
        } else if bid.is_contract() || bid == Bid::Pass {
            self.bidding.push(bid);
            Ok(())
        } else {
            bail!("bidding can be started only with a contract or a pass")
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

fn players() {
    let players: Regex =
        Regex::new(r"pn|(?P<south>[\w ]+,)(?P<west>[\w ]+,)(?P<north>[\w ]+,)(?P<east>[\w ]+)|st")
            .unwrap();
}

fn hands() {
    let hands: Regex = Regex::new(r"md\|(?P<dealer>\d)(?P<hand>[\w,]+?)\|").unwrap();
}

fn number() {
    let number: Regex = Regex::new(r"ah\|(?P<number>[\w, ]+?)\|").unwrap();
}

fn bidding() {
    let bidding: Regex = Regex::new(r"(?P<waste>mb\|(?P<bid>\w+?)\|)+?").unwrap();
}

fn play_sequence() {
    let play_sequence: Regex = Regex::new(r"(?P<waste>pc\|(?P<card>\w+?)\|)+?").unwrap();
}

fn claim() {
    let claim: Regex = Regex::new(r"(?P<claim>mc\|(?P<tricks>\d+))").unwrap();
}
