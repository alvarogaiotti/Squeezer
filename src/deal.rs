use std::fmt::Debug;

use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Seat {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Default for Seat {
    fn default() -> Self {
        Seat::North
    }
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

impl Seat {
    pub fn next(self) -> Seat {
        self + 1
    }
    pub fn from_char(c: char) -> Result<Self, Box<dyn Error>> {
        match c {
            'N' => Ok(Self::North),
            'S' => Ok(Self::South),
            'W' => Ok(Self::West),
            'E' => Ok(Self::East),
            _ => Err(Box::new(DealerError::new("Is not a seat!"))),
        }
    }
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

impl std::ops::Add<usize> for Seat {
    type Output = Seat;

    fn add(self, rhs: usize) -> Self::Output {
        (self as usize + rhs).into()
    }
}

impl From<usize> for Seat {
    fn from(n: usize) -> Self {
        match n % NUMBER_OF_HANDS {
            x if x == Seat::North as usize => Seat::North,
            x if x == Seat::East as usize => Seat::East,
            x if x == Seat::South as usize => Seat::South,
            x if x == Seat::West as usize => Seat::West,
            _ => unreachable!(),
        }
    }
}

///Models vulnerability as an enum.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Vulnerability {
    None = 0,
    NS = 1,
    EW = 2,
    All = 3,
}

impl Default for Vulnerability {
    fn default() -> Self {
        Vulnerability::None
    }
}

///Enum which passes constraint to the Deal struct for dealing specific types of hands
pub enum Constraints<'a> {
    None,
    Bounds(&'a dyn Fn(&[Hand; 4], &mut ShapeFactory) -> bool), // Pointer to type implementing Fn trait
    Predeal([(Seat, Option<Hand>); NUMBER_OF_HANDS]),
    BoundsAndPredeal(
        &'a dyn Fn(&[Hand; 4], &mut ShapeFactory) -> bool,
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

type AcceptFunction = Box<(dyn Fn(&[Hand]) -> bool + Send + Sync)>;

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

    // Descriptor of the hands we would like, e.g.
    hand_descriptors: HashMap<Seat, HandDescriptor>,
    // Hands to predeal.
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
            accept: Box::new(|_: &[Hand]| true),
            hand_descriptors: HashMap::new(),
            predealt_hands: HashMap::new(),
            vulnerability: Vulnerability::default(),
        }
    }

    pub fn predeal(&mut self, seat: Seat, hand: Hand) -> &mut Self {
        self.predealt_hands.insert(seat, hand);
        self
    }

    pub fn with_function(&mut self, accept_function: AcceptFunction) -> &mut Self {
        self.accept = accept_function;
        self
    }

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

    pub fn build(self) -> impl Dealer {
        let mut deck = Cards::ALL;
        for (_, hand) in self.predealt_hands.iter() {
            deck = deck.difference(hand.cards);
        }
        StandardDealer {
            predeal: self.predealt_hands,
            vulnerability: self.vulnerability,
            deck_actual_state: deck,
            deck_starting_state: deck,
            accept_function: self.accept,
            hand_constraints: self.hand_descriptors,
            ..Default::default()
        }
    }
}

pub trait Dealer: Debug {
    fn deal(&self) -> Result<Deal, DealerError>;
}

#[derive(Debug)]
enum Subsequent {
    OutputConsequentially(usize),
    OutputAlwaysOne,
}
// Struct that takes care of the dealing.
pub struct StandardDealer {
    predeal: HashMap<Seat, Hand>,
    vulnerability: Vulnerability,
    deck_starting_state: Cards,
    deck_actual_state: Cards,
    hand_constraints: HashMap<Seat, HandDescriptor>,
    accept_function: AcceptFunction,
    output_as_subsequent: Subsequent,
}

impl Debug for StandardDealer {
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
            deck_actual_state: Cards::ALL,
            deck_starting_state: Cards::ALL,
            hand_constraints: HashMap::new(),
            accept_function: Box::new(|_: &[Hand]| true),
            output_as_subsequent: Subsequent::OutputAlwaysOne,
        }
    }
}

impl Dealer for StandardDealer {
    fn deal(&self) -> Result<Deal, DealerError> {
        let mut hands: [Hand; 4] = [Hand::default(); 4];
        // This way to write the while loop ensures that we deal at least once
        // before evaluating the accept_function and the constraints.
        while {
            let mut deck = self.deck_starting_state;
            for seat in Seat::iter() {
                if let Some(hand) = self.predeal.get(&seat) {
                    let predeal_len = hand.cards.len();
                    if predeal_len < 13 {
                        let cards_to_add = if let Some(cards) = deck.pick(13 - predeal_len) {
                            cards
                        } else {
                            return Err(DealerError::new("The deck doesn't contain enough cards to deal all the hands. Check all the parameters and try to run again."));
                        };
                        hands[seat as usize].set_cards(&(hand.cards + cards_to_add));
                    } else {
                        hands[seat as usize].set_cards(&hand.cards);
                    }
                } else {
                    hands[seat as usize] = if let Some(cards) = deck.pick(13) {
                        Hand { cards }
                    } else {
                        return Err(DealerError::new("The deck doesn't contain enough cards to deal all the hands. Check all the parameters and try to run again."));
                    };
                }
            }
            !((self.accept_function)(&hands) && self.check_if_hand_constraint_are_respected(&hands))
        } {}
        Ok(Deal {
            hands,
            ..Default::default()
        })
    }
}
impl StandardDealer {
    fn check_if_hand_constraint_are_respected(&self, hands: &[Hand]) -> bool {
        if self.hand_constraints.is_empty() {
            true
        } else {
            self.hand_constraints
                .iter()
                .all(|(seat, hand_constraint)| hand_constraint.check(&hands[*seat as usize]))
        }
    }
}

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
}

impl Default for Deal {
    fn default() -> Self {
        Deal::new()
    }
}
impl Deal {
    pub fn new() -> Self {
        Self {
            vulnerability: Vulnerability::None,
            hands: Self::deal(),
            printer: Printer::Short,
        }
    }
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
            _ => hands = Deal::deal(),
        };
        Self {
            vulnerability: Vulnerability::None,
            hands,
            printer: Printer::Short,
        }
    }
    fn predeal(
        predealt: [(Seat, Option<Hand>); NUMBER_OF_HANDS],
        hands: &mut [Hand; NUMBER_OF_HANDS],
    ) {
        let mut deck = Cards::ALL;
        for (_, hand_opt) in predealt.iter() {
            if let Some(hand) = hand_opt {
                deck = deck.difference(hand.cards);
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
    fn check(&self, f: impl FnOnce(&Deal) -> bool) -> bool {
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
            // 1 placeholder for future implementation
            Printer::Lin => self.as_lin(1),
            Printer::Short => self.as_short(),
            Printer::Long => self.as_long(),
        }
    }

    pub fn as_pbn(&self) -> String {
        let mut pbn = "[Deal \"N:".to_owned();
        pbn = format!(
            "{}",
            format_args!(
                "{}{}",
                pbn,
                self.into_iter()
                    .map(|hand| {
                        hand.into_iter()
                            .map(|holding| {
                                holding
                                    .into_iter()
                                    .map(|card| card.rankchar())
                                    .rev()
                                    .format("")
                            })
                            .format(".")
                            .to_string()
                    })
                    .format(" ")
            )
        );
        pbn.push_str("\"]");
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
                        holding
                            .into_iter()
                            .map(|card| card.rankchar())
                            .rev()
                            .format("")
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
            .map(|x| x.len())
            .max()
            .unwrap();
        let east_len = self.hands[Seat::East as usize]
            .long_str()
            .split('\n')
            .map(|x| x.len())
            .max()
            .unwrap();
        let north_len = self.hands[Seat::North as usize]
            .long_str()
            .split('\n')
            .map(|x| x.len())
            .max()
            .unwrap();
        let south_len = self.hands[Seat::South as usize]
            .long_str()
            .split('\n')
            .map(|x| x.len())
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
                if line_w.len() != 0 {
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
                .into_iter()
                .map(|string| format!("{string:^0$}", width - ns_len + string.len()))
                .format("\n"),
        );
        stringa
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
        write!(
            f,
            "\t\t{}\n{}\t\t\t{}\n\t\t{}",
            self.hands[Seat::North as usize],
            self.hands[Seat::West as usize],
            self.hands[Seat::East as usize],
            self.hands[Seat::South as usize],
        )
    }
}

impl<'a> IntoIterator for &'a Deal {
    type Item = Hand;
    type IntoIter = std::array::IntoIter<Hand, NUMBER_OF_HANDS>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.hands)
    }
}

#[cfg(test)]
#[test]
fn can_deal_test() {
    let deal = Deal::new();
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
    let mut deal = Deal::new_with_conditions(&predeal, &mut factory);
    assert_eq!(
        (
            Cards::from_str("SKSQSTS9HAHJHQD9D8D7D3CAC2").unwrap(),
            Cards::from_str("CKCQCJCTC9C8C7C6S3S4S2D2D5").unwrap()
        ),
        (deal.north().cards, deal.south().cards)
    );
}

#[test]
fn dealer_builder_test() {
    let dealer_builder = DealerBuilder::new();
    let dealer = dealer_builder.build();
}

#[test]
fn dealer_deals_test() {
    let db = DealerBuilder::new();
    let mut dealer = db.build();
    let deal = dealer.deal().unwrap();
}

#[test]
fn dealer_deals_with_predeal_test() {
    let hand = Hand::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
    let mut builder = DealerBuilder::new();
    builder.predeal(Seat::North, hand);
    let mut dealer = builder.build();
    let deal = dealer.deal().unwrap();
    assert_eq!(deal.north(), hand);
}

#[test]
fn dealer_deals_with_predeal_and_accept_function_test() {
    let hand = Hand::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
    let mut builder = DealerBuilder::new();
    builder
        .predeal(Seat::North, hand)
        .with_function(Box::new(|hands: &[Hand]| {
            hands[Seat::North as usize].slen() + hands[Seat::South as usize].slen() > 8
        }));
    let mut dealer = builder.build();
    let deal = dealer.deal().unwrap();
    println!("{}", &deal);
    assert!(deal.north().slen() + deal.south().slen() > 8);
}
