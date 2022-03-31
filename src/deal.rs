use crate::prelude::*;

///Seat enum, still to be understood how to use it,
///but i know it will be used
#[derive(Debug, PartialEq, Hash, Eq)]
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
}

impl std::ops::Add<usize> for Seat {
    type Output = Seat;

    fn add(self, rhs: usize) -> Self::Output {
        (self as usize + rhs).into()
    }
}

impl From<usize> for Seat {
    fn from(n: usize) -> Self {
        match n % 4 {
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
    NONE = 0,
    NS = 1,
    EW = 2,
    ALL = 3,
}

///Enum which passes constraint to the Deal struct for dealing specific types of hands
pub enum Constraints<'a> {
    None,
    Bounds(&'a dyn Fn(&[Hand; 4], &mut ShapeFactory) -> bool), // Pointer to type implementing Fn trait
}

///The principal struct of the module: represents a bridge deal, with
///cards, vulnerability, ecc.
/// TODO: Should have a number, a dealer, a contract, ecc.
#[derive(Debug, Clone, Copy)]
pub struct Deal {
    vulnerability: Vulnerability,
    hands: [Hand; 4],
}

impl Deal {
    pub fn new(func: Constraints, factory: &mut ShapeFactory) -> Self {
        let mut hands: [Hand; 4];
        match func {
            Constraints::Bounds(f) => {
                while {
                    hands = Deal::deal();
                    !f(&hands, factory)
                } {}
            }
            _ => {
                hands = Deal::deal();
            }
        };
        Self {
            vulnerability: Vulnerability::NONE,
            hands,
        }
    }

    pub fn deal() -> [Hand; 4] {
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
        f(&self)
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
            "{} {} {} {}",
            self.hands[0], self.hands[1], self.hands[2], self.hands[3],
        )
    }
}

impl<'a> IntoIterator for &'a Deal {
    type Item = Hand;
    type IntoIter = std::array::IntoIter<Hand, 4>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.hands)
    }
}

#[cfg(test)]
#[test]
fn can_deal_test() {
    let deal = Deal::new(Constraints::None, &mut ShapeFactory::new());
}

#[test]
fn deal_with_constraints_test() {
    for _ in 0..10 {
        let deal = Deal::new(
            Constraints::Bounds(&|x: &[Hand; 4], y: &mut ShapeFactory| {
                x[1].diamonds().high_card_points() > 5
            }),
            &mut ShapeFactory::new(),
        );
        assert!(deal[1].diamonds().high_card_points() > 5);
    }
}
