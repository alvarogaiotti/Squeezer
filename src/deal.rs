use crate::prelude::*;

///Seat enum, still to be understood how to use it,
///but i know it will be used
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum Seat {
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

///Enum which passes constraint to the Deal struct for dealing specific types of hands
pub enum Constraints<'a> {
    None,
    Bounds(
        &'a dyn Fn(&[Hand; 4], &mut ShapeFactory) -> bool,
        &'a mut ShapeFactory<'a>,
    ), // Pointer to type implementing Fn trait
    Predeal([(Seat, Option<Hand>); NUMBER_OF_HANDS]),
    BoundsAndPredeal(
        &'a dyn Fn(&[Hand; 4], &mut ShapeFactory) -> bool,
        &'a mut ShapeFactory<'a>,
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

///The principal struct of the module: represents a bridge deal, with
///cards, vulnerability, ecc.
/// TODO: Should have a number, a dealer, a contract, ecc.
#[derive(Debug)]
pub struct Deal {
    vulnerability: Vulnerability,
    hands: [Hand; NUMBER_OF_HANDS],
    printer: Box<dyn DealPrinter>,
}
impl Deal {
    pub fn new() -> Self {
        Self {
            vulnerability: Vulnerability::None,
            hands: Self::deal(),
            printer: Box::new(ShortStrPrinter {}),
        }
    }
    pub fn new_with_conditions(constraints: Constraints) -> Self {
        let mut hands = [Hand::new(); NUMBER_OF_HANDS];
        match constraints {
            Constraints::Bounds(f, factory) => {
                while {
                    hands = Deal::deal();
                    !f(&hands, factory)
                } {}
            }
            Constraints::Predeal(predeal) => {
                Deal::predeal(predeal, &mut hands);
            }
            Constraints::BoundsAndPredeal(f, factory, predeal) => {
                while {
                    Deal::predeal(predeal, &mut hands);
                    !f(&hands, factory)
                } {}
            }
            _ => hands = Deal::deal(),
        };
        Self {
            vulnerability: Vulnerability::None,
            hands,
            printer: Box::new(ShortStrPrinter {}),
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
    pub fn print(&self) {
        println!("{}", self.printer.print(&self.hands));
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
        format!("{}|sv|{}|rh||ah|Board {}", stringa, data3, board_n)
    }
}

pub trait DealPrinter: std::fmt::Debug {
    fn print(&self, hands: &[Hand; NUMBER_OF_HANDS]) -> String;
}
pub trait PrintFormat {
    fn pbn(&mut self);
    fn long(&mut self);
    fn short(&mut self);
}
impl PrintFormat for Deal {
    fn pbn(&mut self) {
        self.printer = Box::new(PbnPrinter {});
    }
    fn long(&mut self) {
        self.printer = Box::new(LongStrPrinter {});
    }
    fn short(&mut self) {
        self.printer = Box::new(ShortStrPrinter {});
    }
}
#[derive(Debug, Clone, Copy)]
struct PbnPrinter {}
impl DealPrinter for PbnPrinter {
    fn print(&self, hands: &[Hand; NUMBER_OF_HANDS]) -> String {
        format!(
            "[Deal:\"N:{}\"]",
            hands
                .iter()
                .map(|hand| format!(
                    "{}",
                    hand.into_iter()
                        .map(|cards| format!(
                            "{}",
                            cards
                                .into_iter()
                                .map(|card| card.rankchar())
                                .rev()
                                .format("")
                        ))
                        .format(".")
                ))
                .format(" ")
        )
    }
}
#[derive(Debug, Clone, Copy)]
struct ShortStrPrinter {}
impl DealPrinter for ShortStrPrinter {
    fn print(&self, hands: &[Hand; NUMBER_OF_HANDS]) -> String {
        format!(
            "\t\t{}\n{}\t\t\t{}\n\t\t{}",
            hands[0], hands[3], hands[1], hands[2],
        )
    }
}
#[derive(Debug, Clone, Copy)]
struct LongStrPrinter {}
impl DealPrinter for LongStrPrinter {
    fn print(&self, hands: &[Hand; NUMBER_OF_HANDS]) -> String {
        let mut stringa = String::new();
        for line in hands[0].long_str().split('\n') {
            stringa = format!("{stringa}\t   {}\n", line);
        }
        for (line_w, line_e) in hands[3]
            .long_str()
            .split('\n')
            .zip(hands[1].long_str().split('\n'))
        {
            stringa = format!("{stringa}{:<20}{line_e}\n", line_w)
        }
        for line in hands[2].long_str().split('\n') {
            stringa = format!("{stringa}\t   {}\n", line);
        }
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
            self.hands[0], self.hands[3], self.hands[1], self.hands[2],
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
        let deal = Deal::new_with_conditions(Constraints::Bounds(
            &|x: &[Hand; NUMBER_OF_HANDS], _y: &mut ShapeFactory| {
                x[1].diamonds().high_card_points() > 5
            },
            &mut ShapeFactory::new(),
        ));
        assert!(deal[1].diamonds().high_card_points() > 5);
    }
}

#[test]
fn deal_with_predeal_test() {
    let predeal = Constraints::predeal(vec![
        ('N', "SKSQSTS9HAHJHQD9D8D7D3CAC2"),
        ('S', "CKCQCJCTC9C8C7C6S3S4S2D2D4D5"),
    ]);
    let mut deal = Deal::new_with_conditions(predeal);
    deal.long();
    deal.print();
    assert_eq!(
        (
            Cards::from_str("SKSQSTS9HAHJHQD9D8D7D3CAC2").unwrap(),
            Cards::from_str("CKCQCJCTC9C8C7C6S3S4S2D2D4D5").unwrap()
        ),
        (deal.north().cards, deal.south().cards)
    );
}
#[test]
fn lin_test() {
    let deal = Deal::new();
    println!("{}", deal.as_lin(1));
}
#[test]
fn pbn_test() {
    let mut deal = Deal::new();
    deal.pbn();
    deal.print();
}
