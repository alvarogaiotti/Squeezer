// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use std::{cell::Cell, num::NonZeroU8};

use crate::prelude::*;

/// Type of the function that checks if a Deal is to be accepted or not
pub type AcceptFunction = Box<(dyn Fn(&Hands) -> bool + Send)>;

/// Structure that holds 4 `Hand`s of 13 cards
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hands {
    hands: [Hand; 4],
}

impl std::fmt::Display for Hands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let west = self.hands[Seat::West as usize].to_string();
        let east = self.hands[Seat::East as usize].to_string();
        let north = self.hands[Seat::North as usize].to_string();
        let south = self.hands[Seat::South as usize].to_string();
        let width = west.chars().count() + east.chars().count() + north.chars().count() / 2;
        write!(
            f,
            "{north:^0$}\n{west:<1$}{east:>1$}\n{south:^0$}",
            width + 1,
            width / 2,
        )
    }
}

impl IntoIterator for Hands {
    type IntoIter = IntoIter<Hand, 4>;
    type Item = Hand;

    fn into_iter(self) -> Self::IntoIter {
        self.hands.into_iter()
    }
}

impl std::ops::Index<Suit> for Hands {
    type Output = Hand;
    fn index(&self, index: Suit) -> &Self::Output {
        &self.hands[index as usize]
    }
}

impl std::ops::Index<usize> for Hands {
    type Output = Hand;
    fn index(&self, index: usize) -> &Self::Output {
        &self.hands[index]
    }
}

impl Hands {
    #[inline]
    pub(crate) fn new_from(hands: [Hand; 4]) -> Self {
        Self { hands }
    }

    /// Returns a reference to the array of [`Hand`]'s
    #[must_use]
    #[inline]
    pub fn hands(&self) -> &[Hand; 4] {
        &self.hands
    }

    /// Returns North [`Hand`]
    #[must_use]
    #[inline]
    pub fn north(&self) -> &Hand {
        &self.hands[Seat::North as usize]
    }
    /// Returns South [`Hand`]
    #[must_use]
    #[inline]
    pub fn south(&self) -> &Hand {
        &self.hands[Seat::South as usize]
    }
    /// Returns East [`Hand`]
    #[must_use]
    #[inline]
    pub fn east(&self) -> &Hand {
        &self.hands[Seat::East as usize]
    }
    /// Returns West [`Hand`]
    #[must_use]
    #[inline]
    pub fn west(&self) -> &Hand {
        &self.hands[Seat::West as usize]
    }
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<Hand> {
        self.hands.iter()
    }
}

impl<'a> IntoIterator for &'a Hands {
    type IntoIter = std::slice::Iter<'a, Hand>;
    type Item = &'a Hand;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Represents a seat in a Bridge game: North, South, East or West
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Seat {
    #[default]
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::North => write!(f, "N"),
            Self::East => write!(f, "E"),
            Self::West => write!(f, "W"),
            Self::South => write!(f, "S"),
        }
    }
}

impl TryFrom<char> for Seat {
    type Error = DealerError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'N' => Ok(Self::North),
            'S' => Ok(Self::South),
            'W' => Ok(Self::West),
            'E' => Ok(Self::East),
            _ => Err(DealerError::new("Is not a seat!")),
        }
    }
}

#[cfg(feature = "dds")]
impl From<Seat> for dds::deal::DdsHandEncoding {
    fn from(value: Seat) -> Self {
        match value {
            Seat::North => Self::North,
            Seat::East => Self::East,
            Seat::South => Self::South,
            Seat::West => Self::West,
        }
    }
}
impl Seat {
    ///Returns the next seat in a cyclic manner in this order: North, East, South, West
    #[must_use]
    #[inline]
    pub fn next(self) -> Seat {
        self + 1
    }

    /// Whether a player is on the same line as another (which might be himself!)
    #[must_use]
    #[inline]
    pub fn is_same_line(&self, other: &Seat) -> bool {
        *self as u8 % 2 == *other as u8 % 2
    }
    ///Iteration over seats starting from North
    #[must_use]
    #[inline]
    pub fn iter() -> IntoIter<Seat, 4> {
        [Seat::North, Seat::East, Seat::South, Seat::West].into_iter()
    }

    #[must_use]
    #[inline]
    pub fn long_str(&self) -> &str {
        match self {
            Self::North => "North",
            Self::East => "East",
            Self::West => "West",
            Self::South => "South",
        }
    }
    #[inline]
    #[must_use]
    /// Beware, this iterator never stops
    pub fn iter_from(self) -> RotatingSuitIterator {
        RotatingSuitIterator::new(self)
    }
}

/// Iterator that cycles over the seats in a Bridge game.
/// Can set state and restart.
/// Start iteration from the seat next to the one with which is initialised.
pub struct RotatingSuitIterator {
    state: Seat,
}

impl Iterator for RotatingSuitIterator {
    type Item = Seat;

    fn next(&mut self) -> Option<Self::Item> {
        self.state = self.state.next();
        Some(self.state)
    }
}

impl RotatingSuitIterator {
    /// Beware, this iterator never stops
    #[inline]
    #[must_use]
    pub fn new(state: Seat) -> Self {
        Self { state }
    }
    pub fn set_state(&mut self, state: Seat) {
        self.state = state;
    }
    #[must_use]
    pub fn state(&self) -> Seat {
        self.state
    }
}

macro_rules! impl_add_and_from_ints_for_seat {
    ($($t:ty),*) => {
        $(
        impl std::ops::Add<$t> for Seat {
            type Output = Seat;

            fn add(self, rhs: $t) -> Self::Output {
                (self as $t + rhs).into()
            }
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        impl From<$t> for Seat {
            fn from(n: $t) -> Self {
                match n % NUMBER_OF_HANDS as $t {
                    x if x == Seat::North as $t => Seat::North,
                    x if x == Seat::East as $t => Seat::East,
                    x if x == Seat::South as $t => Seat::South,
                    x if x == Seat::West as $t => Seat::West,
                    _ => unreachable!(),
                }
            }
        }
        impl std::ops::Add<&$t> for Seat {
            type Output = Seat;

            fn add(self, rhs: &$t) -> Self::Output {
                (self as $t + *rhs).into()
            }
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        impl From<&$t> for Seat {
            fn from(n: &$t) -> Self {
                match *n % NUMBER_OF_HANDS as $t {
                    x if x == Seat::North as $t => Seat::North,
                    x if x == Seat::East as $t => Seat::East,
                    x if x == Seat::South as $t => Seat::South,
                    x if x == Seat::West as $t => Seat::West,
                    _ => unreachable!(),
                }
            }
        }
        )*
    };
}
impl_add_and_from_ints_for_seat!(usize, u64, u32, u16, u8, isize, i64, i32, i16, i8);

///Models vulnerability as an enum.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Vulnerability {
    #[default]
    None = 0,
    NS = 1,
    EW = 2,
    All = 3,
}

impl Vulnerability {
    /// Vulnerability of the first 16 boards, then it repeats.
    pub const VULNERABILITY_TABLE: [Vulnerability; 16] = [
        Vulnerability::None,
        Vulnerability::NS,
        Vulnerability::EW,
        Vulnerability::All,
        Vulnerability::NS,
        Vulnerability::EW,
        Vulnerability::All,
        Vulnerability::None,
        Vulnerability::EW,
        Vulnerability::All,
        Vulnerability::None,
        Vulnerability::NS,
        Vulnerability::All,
        Vulnerability::None,
        Vulnerability::NS,
        Vulnerability::EW,
    ];

    #[inline]
    #[must_use]
    /// Whether or not a seat is vulnerable given a vulnerability.
    pub const fn is_vulnerable(&self, seat: Seat) -> Vulnerable {
        let seat_position = (seat as usize) % 2;
        if ((*self as usize) & (1 << seat_position)) != 0 {
            Vulnerable::Yes
        } else {
            Vulnerable::No
        }
    }
    #[inline]
    #[must_use]
    /// Get the vulnerability state for a given number of board.
    pub const fn from_number(board_number: u8) -> Self {
        Self::VULNERABILITY_TABLE[((board_number - 1) % 16) as usize]
    }
}

/// Iterator over the Vulnerability state.
pub struct VulnerabilityIterator {
    board_number: u8,
}

impl VulnerabilityIterator {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { board_number: 0 }
    }

    #[allow(clippy::missing_panics_doc, clippy::cast_possible_truncation)]
    #[inline]
    #[must_use]
    /// Create a [`VulnerabilityIterator`] starting from a given state.
    /// Always starts from the first 4 boards.
    pub fn from_state(state: Vulnerability) -> Self {
        // SAFETY: We'll find the state
        let board_number = match state {
            Vulnerability::None => 1,
            Vulnerability::NS => 2,
            Vulnerability::EW => 3,
            Vulnerability::All => 4,
        };
        Self { board_number }
    }

    #[inline]
    #[must_use]
    /// Create a [`VulnerabilityIterator`] starting from a given board number.
    pub const fn from_board_number(board_number: u8) -> Self {
        Self { board_number }
    }
}

impl Default for VulnerabilityIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for VulnerabilityIterator {
    type Item = Vulnerability;
    fn next(&mut self) -> Option<Self::Item> {
        let state = Vulnerability::from_number(self.board_number);
        self.board_number += 1;
        Some(state)
    }
}

impl IntoIterator for Vulnerability {
    type IntoIter = VulnerabilityIterator;
    type Item = Vulnerability;
    fn into_iter(self) -> Self::IntoIter {
        VulnerabilityIterator::from_state(self)
    }
}

/// A builder for a dealer object. It's the standard way to create
/// a [`Dealer`] that deals a specific type of deal.
///
/// # Usage
/// ```
/// # use squeezer::*;
/// # use std::error::Error;
/// # fn main()->Result<(), Box<dyn Error>>{
/// let mut builder = DealerBuilder::new();
/// builder.predeal(Seat::North, Cards::from_str("SAKQHAKQDAKQCAKQJ")?.try_into()?);
/// let dealer = builder.build()?;
/// //North will have AKQ AKQ AKQ AKQJ.
/// println!("{}",dealer.deal()?);
/// # Ok(())
/// # }
/// ```
pub struct DealerBuilder {
    // Function that decides if the deal is to be accepted
    // normally used to set things like at least a 9 card
    // fit in a major, but can still be used to do things like this:
    //
    // if hands.north.spades.len() < 6 && hands.north.hcp() > 13 {
    // do something...
    // }
    //
    // even with a HandDescriptor:
    //
    // if some_hand_descriptor.match(hands.north) {
    // do stuff ...
    // }
    accept: AcceptFunction,
    deck: Cards,

    /// Descriptor of the hands we would like, e.g.
    hand_descriptors: [Option<HandDescriptor>; NUMBER_OF_HANDS],
    // FIX: Use an array for that, and don't use hands but
    // Cards, so we can predeal less than 13 cards if we want to.
    /// Hands to predeal.
    predealt_hands: [Option<Cards>; NUMBER_OF_HANDS],
    vulnerability: Vulnerability,
}

impl std::fmt::Debug for DealerBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DealerBuilder")
            .field("hand_descriptors", &self.hand_descriptors)
            .field("predealt_hands", &self.predealt_hands)
            .field("vulnerability", &self.vulnerability)
            .finish_non_exhaustive()
    }
}

impl Default for DealerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DealerBuilder {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            accept: Box::new(|_: &Hands| true),
            hand_descriptors: [None, None, None, None],
            predealt_hands: [None; 4],
            vulnerability: Vulnerability::default(),
            deck: Cards::ALL,
        }
    }

    /// Set the cards that a particular [`Seat`] will be dealt.
    /// # Errors
    /// Will return an error if same card is dealt twice.
    #[inline]
    pub fn predeal(&mut self, seat: Seat, hand: Cards) -> Result<&mut Self, DealerError> {
        if !hand.difference(self.deck).is_empty() {
            return Err(DealerError::new(
                format!("card dealt twice: {}", hand.difference(self.deck)).as_str(),
            ));
        }

        self.deck -= hand;
        self.predealt_hands[seat as usize] = Some(hand);
        Ok(self)
    }

    /// Sets a functions that will be used by the [`Dealer`] to check if the [`Deal`] is to be accepted.
    /// Do not set your hand types with this method (use the
    /// [`DealerBuilder::with_hand_descriptor`] method istead); but use it to set cross hand
    /// constraints.
    ///
    /// # Example
    /// ```
    /// # use squeezer::*;
    /// let mut builder = DealerBuilder::new();
    /// builder.with_function(Box::new(|hands: &Hands| {
    ///          (hands.north().hearts() + hands.south().hearts()).len() >= 8
    ///          }
    ///      )
    /// );
    /// //This Dealer will only deal Deals in which North and South have a heart fit.
    /// let dealer = builder.build().unwrap();
    /// ```
    #[inline]
    pub fn with_function<T: Fn(&Hands) -> bool + Send + 'static>(
        &mut self,
        accept_function: T,
    ) -> &mut Self {
        self.accept = Box::new(accept_function);
        self
    }

    /// Method used to set hand specification for a [`Seat`]. See [`HandDescriptor`] for
    /// details.
    #[inline]
    pub fn with_hand_descriptor(
        &mut self,
        seat: Seat,
        hand_description: HandDescriptor,
    ) -> &mut Self {
        self.hand_descriptors[seat as usize] = Some(hand_description);
        self
    }

    #[inline]
    /// Output just deals with this vulnerability.
    pub fn with_vulnerability(&mut self, vulnerability: Vulnerability) -> &mut Self {
        self.vulnerability = vulnerability;
        self
    }

    /// Builds the Dealer.
    /// # Errors
    /// This will method will return an error if you try to predeal the same card twice.
    #[inline]
    pub fn build(self) -> Result<impl Dealer, DealerError> {
        Ok(StandardDealer {
            predeal: self.predealt_hands,
            vulnerability: self.vulnerability,
            deck_starting_state: self.deck,
            accept_function: self.accept,
            hand_constraints: self.hand_descriptors,
            ..Default::default()
        })
    }
}

pub trait Dealer {
    /// # Errors
    /// Errors if is unable to deal a [`Deal`]
    fn deal(&self) -> Result<Deal, DealerError>;
}

#[derive(Debug)]
pub enum BoardNumbering {
    Sequential(Cell<NonZeroU8>),
    OutputAlwaysOne,
}

/// Struct that takes care of the dealing.
/// You won't interact much with this struct other that call the [`StandardDealer::deal`] method. Use the [`DealerBuilder`] instead to create a [`Dealer`] that
/// fits your needs.
pub struct StandardDealer {
    predeal: [Option<Cards>; NUMBER_OF_HANDS],
    vulnerability: Vulnerability,
    deck_starting_state: Cards,
    hand_constraints: [Option<HandDescriptor>; NUMBER_OF_HANDS],
    accept_function: AcceptFunction,
    // needed to print sequentially
    output_as_subsequent: BoardNumbering,
}

impl std::fmt::Debug for StandardDealer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dealer")
            .field("Predeal", &self.predeal)
            .field("Vulnerability", &self.vulnerability)
            .field("Hand Constraints", &self.hand_constraints)
            .finish_non_exhaustive()
    }
}

impl StandardDealer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            predeal: [None; 4],
            vulnerability: Vulnerability::default(),
            deck_starting_state: Cards::ALL,
            hand_constraints: [None, None, None, None],
            accept_function: Box::new(|_: &Hands| true),
            output_as_subsequent: BoardNumbering::OutputAlwaysOne,
        }
    }
}

impl Default for StandardDealer {
    fn default() -> Self {
        Self::new()
    }
}

impl Dealer for StandardDealer {
    /// Deals a deal based on the parameters set via the constructor.
    #[inline]
    fn deal(&self) -> Result<Deal, DealerError> {
        let mut hands: [Hand; 4] = [Hand::default(); 4];
        // This way to write the while loop ensures that we deal at least once
        // before evaluating the accept_function and the constraints.
        while {
            let mut deck = self.deck_starting_state;
            for seat in Seat::iter() {
                if let Some(&Some(cards)) = self.predeal.get(seat as usize) {
                    let predeal_len = cards.len();
                    if predeal_len < 13 {
                        let Some(cards_to_add) = deck.pick(13 - predeal_len as usize) else {
                            return Err(DealerError::new("The deck doesn't contain enough cards to deal all the hands. Check all the parameters and try to run again."));
                        };
                        hands[seat as usize].set_cards(cards + cards_to_add);
                    } else {
                        hands[seat as usize].set_cards(cards);
                    }
                } else {
                    hands[seat as usize] = if let Some(cards) = deck.pick(13) {
                        Hand { cards }
                    } else {
                        return Err(DealerError::new("The deck doesn't contain enough cards to deal all the hands. Check all the parameters and try to run again."));
                    };
                }
            }
            let hands = Hands { hands };
            !((self.accept_function)(&hands) && self.constraints_respected(hands.hands()))
        } {}
        Ok(Deal {
            hands,
            number: match self.output_as_subsequent {
                BoardNumbering::Sequential(ref num) => {
                    let actual = num.get().get();
                    num.set(match actual {
                        // SAFETY: Just checked
                        1..=127 => unsafe { NonZeroU8::new_unchecked(actual + 1) },
                        // SAFETY: Literal 1.
                        _ => unsafe { NonZeroU8::new_unchecked(1) },
                    });
                    actual
                }
                BoardNumbering::OutputAlwaysOne => 1,
            },
            ..Default::default()
        })
    }
}

impl StandardDealer {
    /// Checks if the [`Deal`] to be outputted matches the constraints we set.
    fn constraints_respected(&self, hands: &[Hand; NUMBER_OF_HANDS]) -> bool {
        self.hand_constraints
            .iter()
            .enumerate()
            .all(|(seat, hand_constraint)| {
                if let Some(hand_constraint) = hand_constraint {
                    hand_constraint.check(hands[seat])
                } else {
                    true
                }
            })
    }
}

/// State tracker for the deal print output.
#[derive(Default, Debug, Copy, Clone)]
#[non_exhaustive]
pub enum Printer {
    Pbn,
    Lin,
    #[default]
    Short,
    Long,
}

///The central struct of the module: represents a bridge deal, with
///cards, vulnerability, ecc.
/// TODO: Should have a number, a dealer, a contract, ecc.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Deal {
    vulnerability: Vulnerability,
    hands: [Hand; NUMBER_OF_HANDS],
    #[cfg_attr(feature = "serde", serde(skip))]
    printer: Printer,
    number: u8,
}

#[cfg(feature = "dds")]
impl dds::deal::AsDDSDeal for Deal {
    fn as_dds_deal(&self) -> dds::deal::DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, hand) in self.into_iter().enumerate() {
            for (index, suit) in hand.into_iter().enumerate() {
                remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
            }
        }
        remain_cards.into()
    }
}

impl Default for Deal {
    fn default() -> Self {
        Deal::new()
    }
}

impl Deal {
    /// A new `Deal` with cards dealt randomly
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            vulnerability: Vulnerability::None,
            hands: Self::deal(),
            printer: Printer::Short,
            number: 1,
        }
    }

    /// Creates a new deal with conditions
    ///
    /// # Panics
    ///
    /// Can panic when dealing fails in some way
    #[must_use]
    #[inline]
    pub fn deal() -> [Hand; NUMBER_OF_HANDS] {
        let mut deck = Cards::ALL;
        let north = Hand {
            cards: deck.pick(13).unwrap(),
        };
        let east = Hand {
            cards: deck.pick(13).unwrap(),
        };
        let south = Hand {
            cards: deck.pick(13).unwrap(),
        };
        let west = Hand { cards: deck };
        [north, east, south, west]
    }

    #[must_use]
    #[inline]
    pub fn check<T: Fn(&Deal) -> bool>(&self, f: T) -> bool {
        f(self)
    }

    #[inline]
    pub fn set_vuln(&mut self, vuln: Vulnerability) {
        self.vulnerability = vuln;
    }

    #[must_use]
    #[inline]
    pub fn west(&self) -> Hand {
        self.hands[3]
    }

    #[must_use]
    #[inline]
    pub fn north(&self) -> Hand {
        self.hands[0]
    }

    #[must_use]
    #[inline]
    pub fn east(&self) -> Hand {
        self.hands[1]
    }

    #[must_use]
    #[inline]
    pub fn south(&self) -> Hand {
        self.hands[2]
    }

    fn set_print_style(&mut self, style: Printer) {
        self.printer = style;
    }

    #[inline]
    pub fn long(&mut self) {
        self.set_print_style(Printer::Long);
    }

    #[inline]
    pub fn pbn(&mut self) {
        self.set_print_style(Printer::Pbn);
    }

    #[inline]
    pub fn short(&mut self) {
        self.set_print_style(Printer::Short);
    }

    #[inline]
    pub fn lin(&mut self) {
        self.set_print_style(Printer::Lin);
    }

    #[must_use]
    #[inline]
    pub fn as_string(&self) -> String {
        match self.printer {
            Printer::Pbn => self.as_pbn(),
            Printer::Lin => self.as_lin(self.number),
            Printer::Short => self.as_short(),
            Printer::Long => self.as_long(),
        }
    }

    #[must_use]
    #[inline]
    pub fn as_pbn(&self) -> String {
        let mut pbn = format!("[Board \"{}\"]\n[Deal \"N:", self.number);
        pbn = format!(
            "{}",
            format_args!(
                "{}{}\"]",
                pbn,
                self.into_iter()
                    .map(|hand| {
                        hand.into_iter()
                            .map(|holding| holding.into_iter().map(Card::rankchar).rev().format(""))
                            .format(".")
                            .to_string()
                    })
                    .format(" ")
            )
        );
        pbn
    }

    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn as_lin(&self, board_n: u8) -> String {
        let board_n = if board_n % (MAX_N_OF_BOARDS + 1) == 0 {
            1
        } else {
            board_n % (MAX_N_OF_BOARDS + 1)
        };
        let mut stringa = format!(
            "st||md|{}",
            (((board_n % NUMBER_OF_HANDS as u8) + 1) % NUMBER_OF_HANDS as u8) + 1
        ); // Dealer for the deal. LIN is a weird format.
        for (position, hand) in self.into_iter().enumerate() {
            if position != 0 {
                stringa.push(','); // TODO: Write this and next block with iterators
            }
            for (index, holding) in hand.into_iter().enumerate() {
                stringa.push(match index {
                    0 => 'S',
                    1 => 'H',
                    2 => 'D',
                    3 => 'C',
                    _ => unreachable!(),
                });
                stringa = format!(
                    "{}",
                    format_args!(
                        "{}{}",
                        stringa,
                        holding.into_iter().map(Card::rankchar).rev().format("")
                    )
                );
            }
        }
        let data1 = (board_n - 1) / NUMBER_OF_HANDS as u8; // Round of board
        let data2 = (board_n - 1) % NUMBER_OF_HANDS as u8; // Dealer
        let data3 = match (data1 + data2) % NUMBER_OF_HANDS as u8 {
            0 => "o",
            1 => "n",
            2 => "e",
            3 => "b",
            _ => unreachable!(),
        };
        format!("{stringa}|sv|{data3}|rh||ah|Board {board_n}|")
    }

    fn as_short(&self) -> String {
        let west = self.hands[Seat::West as usize].to_string();
        let east = self.hands[Seat::East as usize].to_string();
        let north = self.hands[Seat::North as usize].to_string();
        let south = self.hands[Seat::South as usize].to_string();
        let width = west.chars().count() + east.chars().count() + north.chars().count() / 2;
        format!(
            "{north:^0$}\n{west:<1$}{east:>1$}\n{south:^0$}",
            width + 1,
            width / 2,
        )
    }

    fn as_long(&self) -> String {
        let west_len = self.extract_long_strlen(Seat::West);
        let east_len = self.extract_long_strlen(Seat::East);
        let north_len = self.extract_long_strlen(Seat::North);
        let south_len = self.extract_long_strlen(Seat::South);
        let ns_len = if south_len < north_len {
            north_len
        } else {
            south_len
        };
        let width = west_len + east_len + ns_len;
        let mut stringa = String::with_capacity(204);
        for line in self.hands[Seat::North as usize].long_str().split('\n') {
            stringa = format!("{stringa}{line:^0$}\n", width - ns_len + line.len());
        }
        for (line_w, line_e) in self.hands[Seat::West as usize]
            .long_str()
            .split('\n')
            .zip(self.hands[Seat::East as usize].long_str().split('\n'))
        {
            stringa = format!(
                "{stringa}{line_w:<0$}{line_e:<1$}\n",
                if line_w.is_empty() {
                    width - east_len - 1
                } else {
                    width - east_len
                },
                east_len
            );
        }
        stringa = format!(
            "{stringa}{}",
            self.hands[Seat::South as usize]
                .long_str()
                .split('\n')
                .map(|string| format!("{string:^0$}", width - ns_len + string.len()))
                .format("\n"),
        );
        stringa
    }

    fn extract_long_strlen(&self, seat: Seat) -> usize {
        self.hands[seat as usize]
            .long_str()
            .split('\n')
            .map(str::len)
            .max()
            .unwrap()
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<Hand> {
        self.hands.iter()
    }
}

impl<'a> IntoIterator for &'a Deal {
    type IntoIter = std::slice::Iter<'a, Hand>;
    type Item = &'a Hand;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::ops::Index<Suit> for Deal {
    type Output = Hand;
    fn index(&self, index: Suit) -> &Self::Output {
        &self.hands[index as usize]
    }
}
impl std::ops::Index<usize> for Deal {
    type Output = Hand;
    fn index(&self, index: usize) -> &Self::Output {
        &self.hands[index]
    }
}

impl fmt::Display for Deal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl IntoIterator for Deal {
    type Item = Hand;
    type IntoIter = std::array::IntoIter<Hand, NUMBER_OF_HANDS>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.hands)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn can_deal_test() {
        _ = Deal::new();
    }
    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn predealing_twice_should_panic() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let mut builder = DealerBuilder::new();
        builder.predeal(Seat::North, hand).unwrap();
        builder.predeal(Seat::West, hand).unwrap();
        let dealer = builder.build().unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north().as_cards(), hand);
    }

    #[test]
    fn dealer_builder_test() {
        let dealer_builder = DealerBuilder::new();
        _ = dealer_builder.build();
    }

    #[test]
    fn dealer_deals_test() {
        let db = DealerBuilder::new();
        let dealer = db.build().unwrap();
        _ = dealer.deal().unwrap();
    }

    #[test]
    fn dealer_deals_with_predeal_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let mut builder = DealerBuilder::new();
        builder.predeal(Seat::North, hand).unwrap();
        let dealer = builder.build().unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north().as_cards(), hand);
    }

    #[test]
    fn dealer_deals_with_predeal_and_accept_function_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let mut builder = DealerBuilder::new();
        builder
            .predeal(Seat::North, hand)
            .unwrap()
            .with_function(Box::new(|hands: &Hands| {
                hands.north().slen() + hands.south().slen() > 8
            }));
        let dealer = builder.build().unwrap();
        let deal = dealer.deal().unwrap();
        assert!(deal.north().slen() + deal.south().slen() > 8);
    }
}
