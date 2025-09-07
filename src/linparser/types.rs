#[cfg(feature = "dds")]
use dds::{
    analyseplay::PlayTraceBin,
    utils::{BuildSequenceError, RankSeq, SEQUENCE_LENGTH, SuitSeq},
};
use std::num::NonZeroU8;
use std::{error::Error, num::ParseIntError};

use itertools::Itertools;

use crate::{Card, Cards, Contract, Hands, Seat, Strain, Vulnerability};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl From<PlaySequence> for Cards {
    fn from(value: PlaySequence) -> Self {
        let mut deck = Cards::EMPTY;
        for card in value {
            deck = deck.insert(card);
        }
        deck
    }
}

impl IntoIterator for PlaySequence {
    type Item = Card;
    type IntoIter = <Vec<Card> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.sequence.into_iter()
    }
}

impl std::fmt::Display for PlaySequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for trick in &self.iter().chunks(4) {
            writeln!(f, "{}", trick.into_iter().format("-"))?;
        }
        Ok(())
    }
}

#[cfg(feature = "dds")]
impl TryFrom<&PlaySequence> for (SuitSeq, RankSeq) {
    type Error = BuildSequenceError;

    fn try_from(value: &PlaySequence) -> Result<Self, Self::Error> {
        let len = value.len();
        if len == 0 {
            return Err(BuildSequenceError::SequenceTooShort);
        } else if len > SEQUENCE_LENGTH {
            return Err(BuildSequenceError::SequenceTooLong);
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
impl TryFrom<&PlaySequence> for PlayTraceBin {
    type Error = BuildSequenceError;

    fn try_from(value: &PlaySequence) -> Result<Self, Self::Error> {
        let sequences = <(SuitSeq, RankSeq)>::try_from(value)?;
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
/// Enum modelling the possible kinds of errors we
/// can encounter while parsing a bidding sequence:
/// either the bid is not a starting bid (e.g. bidding starts with a double),
/// insufficient, or simply we were unable to parse the last [`Bid`]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum BuildBiddingErrorKind {
    Insufficient,
    NonStarter,
    NonExistent(BuildBidError),
}

impl std::fmt::Display for BuildBiddingErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BuildBiddingErrorKind::Insufficient => write!(f, "bidding is insufficient"),
            BuildBiddingErrorKind::NonStarter => {
                write!(f, "bidding cannot be started with a Double or a Redouble")
            }
            BuildBiddingErrorKind::NonExistent(ref bid) => write!(f, "{bid}"),
        }
    }
}
impl std::error::Error for BuildBiddingErrorKind {}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BuildBiddingError {
    pub(crate) bid: Option<Bid>,
    pub(crate) kind: BuildBiddingErrorKind,
}

impl std::fmt::Display for BuildBiddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(bid) = self.bid {
            write!(f, "unable to add to auction `{bid}`")
        } else {
            write!(
                f,
                "unable to parse bid: {}",
                match &self.kind {
                    BuildBiddingErrorKind::NonExistent(err) => &err.bid,
                    _ => unreachable!(
                        "tried to extract a string for failed bid parsing from a non BuildBiddingErrorKind::NonExistent"
                    ),
                }
            )
        }
    }
}

impl std::error::Error for BuildBiddingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.kind {
            BuildBiddingErrorKind::Insufficient | BuildBiddingErrorKind::NonStarter => None,
            BuildBiddingErrorKind::NonExistent(ref e) => e.source(),
        }
    }
}

/// We model bids as an Enum, with possible contracts as tuple
/// variants containing [`NonZeroU8`] and a [`Strain`]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Bid {
    Pass,
    Double,
    Redouble,
    Contract(NonZeroU8, Strain),
}

impl std::str::FromStr for Bid {
    type Err = BuildBiddingError;

    fn from_str(bid: &str) -> Result<Self, Self::Err> {
        match bid {
            "d" => Ok(Bid::Double),
            "r" => Ok(Bid::Redouble),
            "p" => Ok(Bid::Pass),
            contract_bid => Ok(Bid::Contract(
                match contract_bid[0..1].parse::<std::num::NonZeroU8>() {
                    Ok(num) => num,
                    Err(e) => {
                        return Err(BuildBiddingError {
                            bid: None,
                            kind: BuildBiddingErrorKind::NonExistent(BuildBidError {
                                bid: String::from(contract_bid),
                                kind: BuildBidErrorKind::Level(e),
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
                        return Err(BuildBiddingError {
                            bid: None,
                            kind: BuildBiddingErrorKind::NonExistent(BuildBidError {
                                bid: String::from(contract_bid),
                                kind: BuildBidErrorKind::Strain,
                            }),
                        });
                    }
                },
            )),
        }
    }
}

impl std::fmt::Display for Bid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bid::Contract(level, strain) => write!(f, "{level}{strain}"),
            Bid::Pass => write!(f, "Pass"),
            Bid::Redouble => write!(f, "Redouble"),
            Bid::Double => write!(f, "Double"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuildBidError {
    pub bid: String,
    pub kind: BuildBidErrorKind,
}

/// Enum representing the various kind of errors
/// we could encounter while parsing a single bid:
/// either we are unable to parse the integer part of the bid
/// or we are unable to parse the strain of the bid.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BuildBidErrorKind {
    Level(ParseIntError),
    Strain,
}
impl std::fmt::Display for BuildBidErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildBidErrorKind::Level(_e) => write!(f, "unable to parse level"),
            BuildBidErrorKind::Strain => write!(f, "unable to parse strain"),
        }
    }
}

impl std::error::Error for BuildBidError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.kind {
            BuildBidErrorKind::Level(ref e) => Some(e),
            BuildBidErrorKind::Strain => None,
        }
    }
}

impl std::fmt::Display for BuildBidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                    level < self_level
                        || (level == self_level
                            && (self_strain == &Strain::NoTrumps || strain < self_strain))
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
/// Error kind that models possible errors
/// that could occur while parsing a `.lin` file
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ParseLinErrorKind {
    Player,
    Hands,
    Number,
    Bidding(BuildBiddingError),
    Card,
}

#[derive(Debug, Clone)]
pub struct ParseLinError {
    lin: String,
    kind: ParseLinErrorKind,
}

impl ParseLinError {
    fn new<T: Into<String>>(kind: ParseLinErrorKind, lin: T) -> Self {
        Self {
            lin: lin.into(),
            kind,
        }
    }
}

impl std::fmt::Display for ParseLinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to parse the following lin file:\n{}", self.lin)
    }
}

impl std::error::Error for ParseLinError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.kind {
            ParseLinErrorKind::Bidding(ref e) => Some(e),
            ParseLinErrorKind::Player
            | ParseLinErrorKind::Hands
            | ParseLinErrorKind::Number
            | ParseLinErrorKind::Card => None,
        }
    }
}
impl From<BuildBiddingError> for ParseLinErrorKind {
    fn from(value: BuildBiddingError) -> Self {
        Self::Bidding(value)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Room {
    Open,
    Closed,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct RecordedBid {
    pub bid: Bid,
    pub alert: Option<String>,
}
/// Structure that represents a bidding sequence
/// made of [`Bid`]s
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct LinBidding {
    bidding: Vec<RecordedBid>,
}

impl Default for LinBidding {
    fn default() -> Self {
        Self::new()
    }
}

impl LinBidding {
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
    pub fn get(&self, index: usize) -> Option<&RecordedBid> {
        self.bidding.get(index)
    }
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, RecordedBid> {
        self.bidding.iter()
    }
    #[inline]
    /// # Errors
    /// - If the `Bid` is insufficient
    /// - If we are at the first `Bid` and find a Double or Redouble
    pub fn push(&mut self, bid: RecordedBid) -> Result<(), BuildBiddingError> {
        // If we have already some bids pushed
        if let Some(last) = self.bidding.last() {
            if bid.bid.can_bid_over(&last.bid) {
                self.bidding.push(bid);
                Ok(())
            } else {
                Err(BuildBiddingError {
                    bid: Some(bid.bid),
                    kind: BuildBiddingErrorKind::Insufficient,
                })
            }
            // If this is the first bid that we push and is a contract or pass go ahead
        } else if bid.bid.is_contract() || bid.bid == Bid::Pass {
            self.bidding.push(bid);
            Ok(())
            // Else terminate error
        } else {
            Err(BuildBiddingError {
                bid: Some(bid.bid),
                kind: BuildBiddingErrorKind::NonStarter,
            })
        }
    }
}

impl<'a> IntoIterator for &'a LinBidding {
    type Item = &'a RecordedBid;
    type IntoIter = std::slice::Iter<'a, RecordedBid>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl TryFrom<Vec<RecordedBid>> for LinBidding {
    type Error = BuildBiddingError;
    fn try_from(value: Vec<RecordedBid>) -> Result<Self, Self::Error> {
        let mut bidding = LinBidding::new();
        for bid in value {
            bidding.push(bid)?;
        }
        Ok(bidding)
    }
}

impl IntoIterator for LinBidding {
    type Item = RecordedBid;
    type IntoIter = <Vec<RecordedBid> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.bidding.into_iter()
    }
}

impl std::fmt::Display for LinBidding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bid in &self.iter().chunks(4) {
            writeln!(f, "{}", bid.map(|e| e.bid).format("-"))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LinFile {
    pub tournament_info: Option<TournamentInfo>,
    pub results: Option<Results>,
    pub players: Players,
    pub deals: Vec<LinDeal>,
}

impl LinFile {
    #[must_use]
    #[inline]
    pub fn players(&self) -> &Players {
        &self.players
    }
}

impl std::str::FromStr for LinFile {
    type Err = ParseLinError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, deal) = crate::linparser::parse_bridge_file(s).map_err(|e|
            //ParseLinError::new(ParseLinErrorKind::Card, e.)
            match e {
                                nom::Err::Error(err) | nom::Err::Failure(err) => {
                                    let error_input = err.input;
                                    ParseLinError::new(
                                        //TODO: fix
                                        ParseLinErrorKind::Player,
                                        error_input,
                                    )
                                },
                                nom::Err::Incomplete(_) => unreachable!(),
                            })?;
        Ok(deal)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TournamentInfo {
    pub tournament: String,
    pub round: String,
    pub board_range: String,
    pub board_start: u8,
    pub board_end: u8,
    pub team1: String,
    pub team1_score: u32,
    pub team2: String,
    pub team2_score: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Results {
    pub results: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Players {
    pub players: Vec<String>,
}

impl Players {
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.players.iter()
    }
}

#[derive(Debug, Clone)]
pub struct LinDeal {
    pub room: Option<Room>,
    pub number: u8,
    pub dealer: Seat,
    pub hands: Hands,
    pub vulnerability: Vulnerability,
    pub bidding: LinBidding,
    pub play: Option<PlaySequence>,
    pub claim: Option<u8>,
    pub contract: Option<Contract>,
}

impl LinDeal {
    #[must_use]
    pub fn hands(&self) -> Hands {
        self.hands
    }

    #[must_use]
    pub fn number(&self) -> u8 {
        self.number
    }

    #[must_use]
    pub fn bidding(&self) -> &LinBidding {
        &self.bidding
    }

    #[must_use]
    pub fn play_sequence(&self) -> Option<&PlaySequence> {
        self.play.as_ref()
    }

    #[must_use]
    pub fn dealer(&self) -> Seat {
        self.dealer
    }

    #[must_use]
    pub fn contract(&self) -> Option<Contract> {
        self.contract
    }
}

#[cfg(feature = "dds")]
impl dds::deal::AsDDSDeal for LinDeal {
    fn as_dds_deal(&self) -> dds::deal::DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, hand) in self.hands.into_iter().enumerate() {
            for (index, suit) in hand.into_iter().enumerate() {
                remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
            }
        }
        remain_cards.into()
    }
}
