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

impl Seat {
    pub fn next(self) -> Seat {
        self + 1
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
    Bounds(&'a dyn Fn(&Deal) -> bool), // Pointer to type implementing Fn trait
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
    pub fn new(func: Constraints) -> Self {
        let mut deal: Deal;
        match func {
            Constraints::Bounds(f) => {
                while {
                    deal = Self {
                        vulnerability: Vulnerability::NONE,
                        hands: Deal::deal(),
                    };
                    !f(&deal)
                } {}
            }
            _ => {
                deal = Self {
                    vulnerability: Vulnerability::NONE,
                    hands: Deal::deal(),
                }
            }
        };
        deal
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
        let west = Hand {
            cards: deck.pick(13).unwrap(),
        };
        [north, east, south, west]
    }
    fn check(self, f: impl FnOnce(&Deal) -> bool) -> bool {
        f(&self)
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
    let deal = Deal::new(Constraints::None);
    println!("{}", deal);
}

#[test]
fn deal_with_constraints_test() {
    for _ in 1..11 {
        let deal = Deal::new(Constraints::Bounds(&|x: &Deal| {
            x[1].diamonds().high_card_points() > 5
        }));
        assert!(deal[1].diamonds().high_card_points() > 5);
    }
}
