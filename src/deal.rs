use crate::prelude::*;
use dds;
/// Type of the function that checks if a Deal is to be accepted or not
type AcceptFunction = Box<(dyn Fn(&Hands) -> bool + Send + Sync)>;

/// Structure that holds 4 `Hand`s of 13 cards
pub struct Hands {
    hands: [Hand; 4],
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
    pub(crate) fn new_from(hands: [Hand; 4]) -> Self {
        Self { hands }
    }

    /// Returns the array of `[Hand]`s
    pub fn hands(&self) -> &[Hand; 4] {
        &self.hands
    }

    /// Returns North `[Hand]`
    pub fn north(&self) -> &Hand {
        &self.hands[Seat::North as usize]
    }
    /// Returns South `[Hand]`
    pub fn south(&self) -> &Hand {
        &self.hands[Seat::South as usize]
    }
    /// Returns East `[Hand]`
    pub fn east(&self) -> &Hand {
        &self.hands[Seat::East as usize]
    }
    /// Returns West `[Hand]`
    pub fn west(&self) -> &Hand {
        &self.hands[Seat::West as usize]
    }
    pub fn iter(&self) -> std::slice::Iter<Hand> {
        self.hands.iter()
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

impl Seat {
    ///Returns the next seat in a cyclic manner in this order: North, East, South, West
    pub fn next(self) -> Seat {
        self + 1
    }

    ///Iteration over seats starting from North
    pub fn iter() -> IntoIter<Seat, 4> {
        [Seat::North, Seat::East, Seat::South, Seat::West].into_iter()
    }

    pub fn long_str(&self) -> &str {
        match self {
            Self::North => "North",
            Self::East => "East",
            Self::West => "West",
            Self::South => "South",
        }
    }
}

macro_rules! impl_add_and_from_ints_for_seat {
    ($t:ty) => {
        impl std::ops::Add<$t> for Seat {
            type Output = Seat;

            fn add(self, rhs: $t) -> Self::Output {
                (self as $t + rhs).into()
            }
        }
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
    };
}
impl_add_and_from_ints_for_seat!(usize);
impl_add_and_from_ints_for_seat!(u64);
impl_add_and_from_ints_for_seat!(u32);
impl_add_and_from_ints_for_seat!(u16);
impl_add_and_from_ints_for_seat!(u8);
impl_add_and_from_ints_for_seat!(i64);
impl_add_and_from_ints_for_seat!(i32);
impl_add_and_from_ints_for_seat!(i16);
impl_add_and_from_ints_for_seat!(i8);

///Models vulnerability as an enum.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Vulnerability {
    #[default]
    None = 0,
    NS = 1,
    EW = 2,
    All = 3,
}

///Enum which passes constraint to the Deal struct for dealing specific types of hands. Right
///now is dismissed and will be dropped in a while.
pub enum Constraints<'a> {
    None,
    Bounds(&'a dyn Fn(&[Hand; NUMBER_OF_HANDS], &mut ShapeFactory) -> bool), // Pointer to type implementing Fn trait
    Predeal([(Seat, Option<Hand>); NUMBER_OF_HANDS]),
    BoundsAndPredeal(
        &'a dyn Fn(&[Hand; NUMBER_OF_HANDS], &mut ShapeFactory) -> bool,
        [(Seat, Option<Hand>); NUMBER_OF_HANDS],
    ),
}
impl<'a> Constraints<'a> {
    pub fn predeal(hands: Vec<(char, &str)>) -> Self {
        let mut predealt_hands: [(Seat, Option<Hand>); NUMBER_OF_HANDS] = [
            (Seat::North, Some(Hand::new())),
            (Seat::East, Some(Hand::new())),
            (Seat::South, Some(Hand::new())),
            (Seat::West, Some(Hand::new())),
        ];
        for (seat, hand) in hands {
            match seat {
                'N' => predealt_hands[0] = (Seat::North, Some(Hand::from_str(hand).unwrap())),
                'E' => predealt_hands[1] = (Seat::East, Some(Hand::from_str(hand).unwrap())),
                'S' => predealt_hands[2] = (Seat::South, Some(Hand::from_str(hand).unwrap())),
                'W' => predealt_hands[3] = (Seat::West, Some(Hand::from_str(hand).unwrap())),
                _ => (),
            }
        }
        Self::Predeal(predealt_hands)
    }
}

/// A builder for a dealer object. It's the standard way to create
/// a [`Dealer`] that deals a specific type of deal.
///
/// # Usage
/// ```
/// # use rusty_dealer::*;
/// let mut builder = DealerBuilder::new();
/// builder.predeal(Seat::North, Hand::from_str("SAKQHAKQDAKQCAKQJ").unwrap());
/// let dealer = builder.build().unwrap();
/// //North will have AKQ AKQ AKQ AKQJ.
/// println!("{}",dealer.deal().unwrap());
/// ```
pub struct DealerBuilder {
    // Function that decides if the deal is to be accepted
    // normally used to set things like at least a 9 card
    // fit in a major, but can still be used to do things like this
    // if hands.north.spades.len() < 6 && hands.north.hcp() > 13 {
    // do something...
    // }
    // even with a HandDescriptor:
    // if some_hand_descriptor.match(hands.north) {
    // do stuff ...
    // }
    accept: AcceptFunction,

    /// Descriptor of the hands we would like, e.g.
    hand_descriptors: HashMap<Seat, HandDescriptor>,
    /// Hands to predeal.
    predealt_hands: HashMap<Seat, Hand>,
    vulnerability: Vulnerability,
}

impl Default for DealerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DealerBuilder {
    pub fn new() -> Self {
        Self {
            accept: Box::new(|_: &Hands| true),
            hand_descriptors: HashMap::new(),
            predealt_hands: HashMap::new(),
            vulnerability: Vulnerability::default(),
        }
    }

    /// Set the cards that a particular [`Seat`] will be dealt. Will not fail right away if same card
    /// is dealt twice, but will fail in the building phase.
    pub fn predeal(&mut self, seat: Seat, hand: Hand) -> &mut Self {
        self.predealt_hands.insert(seat, hand);
        self
    }

    /// Sets a functions that will be used by the [`Dealer`] to check if the [`Deal`] is to be accepted.
    /// Do not set your hand types with this method (use the [DealerBuilder::with_hand_specification] method istead); but use it to set cross hand constraints.
    ///
    /// # Example
    /// ```
    /// # use rusty_dealer::*;
    /// let mut builder = DealerBuilder::new();
    /// builder.with_function(Box::new(|hands: &Hands| {
    ///          (hands.north().hearts() + hands.south().hearts()).len() >= 8
    ///          }
    ///      )
    /// );
    /// //This Dealer will only deal Deals in which North and South have a heart fit.
    /// let dealer = builder.build().unwrap();
    /// ```
    pub fn with_function(&mut self, accept_function: AcceptFunction) -> &mut Self {
        self.accept = accept_function;
        self
    }

    /// Method used to set hand specification for a [`Seat`]. See [`HandDescriptor`] for
    /// details.
    pub fn with_hand_specification(
        &mut self,
        seat: Seat,
        hand_description: HandDescriptor,
    ) -> &mut Self {
        self.hand_descriptors.insert(seat, hand_description);
        self
    }

    pub fn with_vulnerability(&mut self, vulnerability: Vulnerability) -> &mut Self {
        self.vulnerability = vulnerability;
        self
    }

    /// Builds the Dealer.
    /// **NOTE**: this will method will return an error if you try to predeal the same card twice.
    pub fn build(self) -> Result<impl Dealer, DealerError> {
        let mut deck = Cards::ALL;
        for hand in self.predealt_hands.values() {
            if !hand.cards.difference(deck).is_empty() {
                return Err(DealerError::new(
                    format!("card dealt twice: {}", hand.cards.difference(deck)).as_str(),
                ));
            }

            deck = deck.difference(hand.as_cards());
        }
        Ok(StandardDealer {
            predeal: self.predealt_hands,
            vulnerability: self.vulnerability,
            deck_starting_state: deck,
            accept_function: self.accept,
            hand_constraints: self.hand_descriptors,
            ..Default::default()
        })
    }
}

pub trait Dealer: std::fmt::Debug {
    fn deal(&self) -> Result<Deal, DealerError>;
}

#[derive(Debug)]
pub enum Subsequent {
    OutputConsequentially(u8),
    OutputAlwaysOne,
}

/// Struct that takes care of the dealing.
/// You won't interact much with this struct other that call the [`StandardDealer::deal`] method. Use the [`DealerBuilder`] instead to create a [`Dealer`] that
/// fits your needs.
pub struct StandardDealer {
    predeal: HashMap<Seat, Hand>,
    vulnerability: Vulnerability,
    deck_starting_state: Cards,
    hand_constraints: HashMap<Seat, HandDescriptor>,
    accept_function: AcceptFunction,
    // needed to print sequentially
    output_as_subsequent: Subsequent,
}

impl std::fmt::Debug for StandardDealer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dealer")
            .field("Predeal", &self.predeal)
            .field("Vulnerability", &self.vulnerability)
            .field("Hand Constraints", &self.hand_constraints)
            .finish()
    }
}

impl Default for StandardDealer {
    fn default() -> Self {
        Self {
            predeal: HashMap::new(),
            vulnerability: Vulnerability::default(),
            deck_starting_state: Cards::ALL,
            hand_constraints: HashMap::new(),
            accept_function: Box::new(|_: &Hands| true),
            output_as_subsequent: Subsequent::OutputAlwaysOne,
        }
    }
}

impl Dealer for StandardDealer {
    /// Deals a deal based on the parameters set via the constructor.
    fn deal(&self) -> Result<Deal, DealerError> {
        let mut hands: [Hand; 4] = [Hand::default(); 4];
        // This way to write the while loop ensures that we deal at least once
        // before evaluating the accept_function and the constraints.
        while {
            let mut deck = self.deck_starting_state;
            for seat in Seat::iter() {
                if let Some(hand) = self.predeal.get(&seat) {
                    let predeal_len = hand.as_cards().len();
                    if predeal_len < 13 {
                        let Some(cards_to_add) = deck.pick(13 - predeal_len as usize) else {
                            return Err(DealerError::new("The deck doesn't contain enough cards to deal all the hands. Check all the parameters and try to run again."));
                        };
                        hands[seat as usize].set_cards(hand.as_cards() + cards_to_add);
                    } else {
                        hands[seat as usize].set_cards(hand.as_cards());
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
                Subsequent::OutputConsequentially(num) => num,
                Subsequent::OutputAlwaysOne => 1,
            },
            ..Default::default()
        })
    }
}
impl StandardDealer {
    fn constraints_respected(&self, hands: &[Hand; NUMBER_OF_HANDS]) -> bool {
        if self.hand_constraints.is_empty() {
            true
        } else {
            self.hand_constraints
                .iter()
                .all(|(seat, hand_constraint)| hand_constraint.check(hands[*seat as usize]))
        }
    }
}

/// State tracker for the deal print output.
#[derive(Debug, Copy, Clone)]
pub enum Printer {
    Pbn,
    Lin,
    Short,
    Long,
}

///The central struct of the module: represents a bridge deal, with
///cards, vulnerability, ecc.
/// TODO: Should have a number, a dealer, a contract, ecc.
#[derive(Debug, Copy, Clone)]
pub struct Deal {
    vulnerability: Vulnerability,
    hands: [Hand; NUMBER_OF_HANDS],
    printer: Printer,
    number: u8,
}

impl dds::AsDDSDeal for Deal {
    fn as_dds_deal(&self) -> dds::DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, hand) in self.into_iter().enumerate() {
            for (index, suit) in hand.into_iter().enumerate() {
                remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
            }
        }
        remain_cards
    }
}

impl Default for Deal {
    fn default() -> Self {
        Deal::new()
    }
}
impl Deal {
    /// A new `Deal` with cards dealt randomly
    pub fn new() -> Self {
        Self {
            vulnerability: Vulnerability::None,
            hands: Self::deal(),
            printer: Printer::Short,
            number: 1,
        }
    }

    /// Creates a new deal with conditions
    pub fn new_with_conditions(constraints: &Constraints, factory: &mut ShapeFactory) -> Self {
        let mut hands = [Hand::new(); NUMBER_OF_HANDS];
        match constraints {
            Constraints::Bounds(f) => {
                while {
                    hands = Deal::deal();
                    !f(&hands, factory)
                } {}
            }
            Constraints::Predeal(predeal) => {
                Deal::predeal(*predeal, &mut hands);
            }
            Constraints::BoundsAndPredeal(f, predeal) => {
                while {
                    Deal::predeal(*predeal, &mut hands);
                    !f(&hands, factory)
                } {}
            }
            Constraints::None => hands = Deal::deal(),
        };
        Self {
            vulnerability: Vulnerability::None,
            hands,
            printer: Printer::Short,
            number: 1,
        }
    }
    fn predeal(
        predealt: [(Seat, Option<Hand>); NUMBER_OF_HANDS],
        hands: &mut [Hand; NUMBER_OF_HANDS],
    ) {
        let mut deck = Cards::ALL;
        for (_, hand_opt) in &predealt {
            if let Some(hand) = hand_opt {
                deck = deck.difference(hand.as_cards());
            }
        }
        for (seat, hand_opt) in predealt {
            if let Some(hand) = hand_opt {
                hands[seat as usize] = hand;
            } else {
                hands[seat as usize] = Hand {
                    cards: deck.pick(13).unwrap(),
                };
            }
        }
    }

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

    pub fn check(&self, f: impl Fn(&Deal) -> bool) -> bool {
        f(self)
    }

    pub fn set_vuln(&mut self, vuln: Vulnerability) {
        self.vulnerability = vuln;
    }

    pub fn west(&self) -> Hand {
        self.hands[3]
    }

    pub fn north(&self) -> Hand {
        self.hands[0]
    }

    pub fn east(&self) -> Hand {
        self.hands[1]
    }

    pub fn south(&self) -> Hand {
        self.hands[2]
    }

    fn set_print_style(&mut self, style: Printer) {
        self.printer = style;
    }

    pub fn long(&mut self) {
        self.set_print_style(Printer::Long);
    }

    pub fn pbn(&mut self) {
        self.set_print_style(Printer::Pbn);
    }

    pub fn short(&mut self) {
        self.set_print_style(Printer::Short);
    }

    pub fn lin(&mut self) {
        self.set_print_style(Printer::Lin);
    }

    pub fn as_string(&self) -> String {
        match self.printer {
            Printer::Pbn => self.as_pbn(),
            Printer::Lin => self.as_lin(self.number),
            Printer::Short => self.as_short(),
            Printer::Long => self.as_long(),
        }
    }

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
                stringa.push(',') // TODO: Wirte this and next block with iterators
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
        let west_len = self.hands[Seat::West as usize]
            .long_str()
            .split('\n')
            .map(str::len)
            .max()
            .unwrap();
        let east_len = self.hands[Seat::East as usize]
            .long_str()
            .split('\n')
            .map(str::len)
            .max()
            .unwrap();
        let north_len = self.hands[Seat::North as usize]
            .long_str()
            .split('\n')
            .map(str::len)
            .max()
            .unwrap();
        let south_len = self.hands[Seat::South as usize]
            .long_str()
            .split('\n')
            .map(str::len)
            .max()
            .unwrap();
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
        //stringa.push_str("\n");
        for (line_w, line_e) in self.hands[Seat::West as usize]
            .long_str()
            .split('\n')
            .zip(self.hands[Seat::East as usize].long_str().split('\n'))
        {
            stringa = format!(
                "{stringa}{line_w:<0$}{line_e:<1$}\n",
                if !line_w.is_empty() {
                    width - east_len
                } else {
                    width - east_len - 1
                },
                east_len
            )
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

    pub fn iter(&self) -> std::slice::Iter<Hand> {
        self.hands.iter()
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
#[test]
fn can_deal_test() {
    _ = Deal::new();
}

#[test]
fn deal_with_constraints_test() {
    for _ in 0..10 {
        let deal = Deal::new_with_conditions(
            &Constraints::Bounds(&|x: &[Hand; NUMBER_OF_HANDS], _y: &mut ShapeFactory| {
                x[1].diamonds().high_card_points() > 5
            }),
            &mut ShapeFactory::new(),
        );
        assert!(deal[1].diamonds().high_card_points() > 5);
    }
}

#[test]
fn deal_with_predeal_test() {
    let mut factory = ShapeFactory::new();
    let predeal = Constraints::predeal(vec![('N', "SKQT9HAQJD9873CA2"), ('S', "CKQJT9876S342D25")]);
    let deal = Deal::new_with_conditions(&predeal, &mut factory);
    assert_eq!(
        (
            Cards::from_str("SKSQSTS9HAHJHQD9D8D7D3CAC2").unwrap(),
            Cards::from_str("CKCQCJCTC9C8C7C6S3S4S2D2D5").unwrap()
        ),
        (deal.north().as_cards(), deal.south().as_cards())
    );
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
    let hand = Hand::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
    let mut builder = DealerBuilder::new();
    builder.predeal(Seat::North, hand);
    let dealer = builder.build().unwrap();
    let deal = dealer.deal().unwrap();
    assert_eq!(deal.north(), hand);
}

#[test]
fn dealer_deals_with_predeal_and_accept_function_test() {
    let hand = Hand::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
    let mut builder = DealerBuilder::new();
    builder
        .predeal(Seat::North, hand)
        .with_function(Box::new(|hands: &Hands| {
            hands.north().slen() + hands.south().slen() > 8
        }));
    let dealer = builder.build().unwrap();
    let deal = dealer.deal().unwrap();
    println!("{}", &deal);
    assert!(deal.north().slen() + deal.south().slen() > 8);
}
