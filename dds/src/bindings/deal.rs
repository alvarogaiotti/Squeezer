use super::{
    ddsffi::{deal, dealPBN},
    RawDDS,
};
use std::{ffi::c_int, fmt::Display};

// See https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer

#[derive(Debug, RawDDS)]
pub(crate) struct DDSCurrTrickSuit([c_int; 3]);

impl Default for DDSCurrTrickRank {
    fn default() -> Self {
        Self([0; 3])
    }
}

#[derive(Debug, RawDDS)]
pub(crate) struct DDSCurrTrickRank([c_int; 3]);

impl Default for DDSCurrTrickSuit {
    fn default() -> Self {
        Self([0; 3])
    }
}
pub(crate) enum DDSSuitEncoding {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
    NoTrump = 4,
}

impl std::convert::TryFrom<u8> for DDSSuitEncoding {
    type Error = DDSDealConstructionError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(DDSSuitEncoding::Spades),
            1 => Ok(DDSSuitEncoding::Hearts),
            2 => Ok(DDSSuitEncoding::Diamonds),
            3 => Ok(DDSSuitEncoding::Clubs),
            4 => Ok(DDSSuitEncoding::NoTrump),
            _ => Err(DDSDealConstructionError::TrumpUnconvertable),
        }
    }
}

pub(crate) enum DDSHandEncoding {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl std::convert::TryFrom<u8> for DDSHandEncoding {
    type Error = DDSDealConstructionError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(DDSHandEncoding::North),
            1 => Ok(DDSHandEncoding::East),
            2 => Ok(DDSHandEncoding::South),
            3 => Ok(DDSHandEncoding::West),
            _ => Err(DDSDealConstructionError::FirstUnconvertable),
        }
    }
}

#[derive(Debug, RawDDS)]
pub struct DDSDealRepr([[u32; 4]; 4]);

#[derive(Debug, RawDDS)]
pub struct DDSDealPBNRepr([std::ffi::c_char; 80]);

pub trait AsDDSDeal {
    fn as_dds_deal(&self) -> DDSDealRepr;
}

pub struct DDSDealBuilder {
    trump: Option<DDSSuitEncoding>,
    first: Option<DDSHandEncoding>,
    current_trick_suit: Option<DDSCurrTrickSuit>,
    current_trick_rank: Option<DDSCurrTrickRank>,
    remain_cards: Option<DDSDealRepr>,
}
#[derive(Debug)]
pub enum DDSDealConstructionError {
    DuplicatedCard(c_int, c_int),
    CurrentTrickRankNotSet,
    CurrentTrickSuitNotSet,
    DealNotLoaded,
    TrumpNotDeclared,
    FirstNotDeclared,
    FirstUnconvertable,
    TrumpUnconvertable,
}

impl Display for DDSDealConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::CurrentTrickRankNotSet => write!(
                f,
                "current trick rank is not set while current trick suit is"
            ),
            Self::CurrentTrickSuitNotSet => write!(
                f,
                "current trick suit is not set while current trick rank is"
            ),
            Self::DuplicatedCard(suit, rank) => {
                let card = dds_card_tuple_to_string(suit, rank);
                write!(f, "duplicated card: {card}")
            }
            Self::DealNotLoaded => write!(f, "deal not loaded"),
            Self::FirstNotDeclared => write!(f, "leader not declared"),
            Self::TrumpNotDeclared => write!(f, "trump not declared"),
            Self::FirstUnconvertable => {
                write!(f, "first cannot be converted from the value you provided")
            }
            Self::TrumpUnconvertable => {
                write!(f, "first cannot be converted from the value you provided")
            }
        }
    }
}
impl std::error::Error for DDSDealConstructionError {}

impl DDSDealBuilder {
    pub fn new() -> Self {
        DDSDealBuilder {
            trump: None,
            first: None,
            current_trick_suit: None,
            current_trick_rank: None,
            remain_cards: None,
        }
    }
    pub fn trump(mut self, trump: DDSSuitEncoding) -> Self {
        self.trump = Some(trump);
        self
    }
    pub fn first(mut self, first: DDSHandEncoding) -> Self {
        self.first = Some(first);
        self
    }
    pub fn remain_cards(mut self, remain_cards: DDSDealRepr) -> Self {
        self.remain_cards = Some(remain_cards);
        self
    }
    pub fn current_trick_suit(mut self, current_trick_suit: DDSCurrTrickSuit) -> Self {
        self.current_trick_suit = Some(current_trick_suit);
        self
    }
    pub fn current_trick_rank(mut self, current_trick_rank: DDSCurrTrickRank) -> Self {
        self.current_trick_rank = Some(current_trick_rank);
        self
    }
    pub fn build(self) -> Result<DDSDeal, DDSDealConstructionError> {
        let remain_cards = self
            .remain_cards
            .ok_or(DDSDealConstructionError::DealNotLoaded)?;
        let trump = self
            .trump
            .ok_or(DDSDealConstructionError::TrumpNotDeclared)?;
        let first = self
            .first
            .ok_or(DDSDealConstructionError::FirstNotDeclared)?;
        let (current_trick_suit, current_trick_rank) =
            match (self.current_trick_suit, self.current_trick_rank) {
                (Some(suits), Some(ranks)) => Ok((suits, ranks)),
                (None, None) => Ok(Default::default()),
                (None, _) => Err(DDSDealConstructionError::CurrentTrickSuitNotSet),
                (_, None) => Err(DDSDealConstructionError::CurrentTrickRankNotSet),
            }?;
        Ok(DDSDeal::new(
            trump,
            first,
            current_trick_rank,
            current_trick_suit,
            remain_cards,
        ))
    }
}

#[derive(RawDDS, Debug)]
pub(super) struct DDSDeal {
    raw: deal,
}

impl DDSDeal {
    pub fn new(
        trump: DDSSuitEncoding,
        first: DDSHandEncoding,
        current_trick_rank: DDSCurrTrickRank,
        current_trick_suit: DDSCurrTrickSuit,
        remain_cards: DDSDealRepr,
    ) -> Self {
        Self {
            raw: deal {
                trump: trump as c_int,
                first: first as c_int,
                currentTrickSuit: current_trick_suit.get_raw(),
                currentTrickRank: current_trick_rank.get_raw(),
                remainCards: remain_cards.get_raw(),
            },
        }
    }
}

#[derive(RawDDS, Debug)]
pub(super) struct DDSDealPBN {
    raw: dealPBN,
}

impl DDSDealPBN {
    pub fn new(
        trump: c_int,
        first: c_int,
        current_trick_rank: DDSCurrTrickRank,
        current_trick_suit: DDSCurrTrickSuit,
        remain_cards: DDSDealPBNRepr,
    ) -> Self {
        Self {
            raw: dealPBN {
                trump,
                first,
                currentTrickSuit: current_trick_suit.get_raw(),
                currentTrickRank: current_trick_rank.get_raw(),
                remainCards: remain_cards.get_raw(),
            },
        }
    }
}

fn dds_card_tuple_to_string(suit: c_int, rank: c_int) -> String {
    let rankstr = match rank {
        0b_100 => "2",
        0b_1000 => "3",
        0b_10000 => "4",
        0b_100000 => "5",
        0b_1000000 => "6",
        0b_10000000 => "7",
        0b_100000000 => "8",
        0b_1000000000 => "9",
        0b_10000000000 => "10",
        0b_100000000000 => "J",
        0b_1000000000000 => "Q",
        0b_10000000000000 => "K",
        0b_100000000000000 => "A",
    };
    let suitstr = match suit {
        0 => "♠",
        1 => "♥",
        2 => "◆",
        3 => "♣",
    };
    let mut res = String::with_capacity(2);
    res.push_str(suitstr);
    res.push_str(rankstr);
    res
}
