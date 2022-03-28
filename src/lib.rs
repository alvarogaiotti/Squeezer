use itertools::{any, Itertools};
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

use bridge_deck::{Card, Cards, Suit};

const SUITS: usize = 4;
const SHAPE_COMBINATIONS: usize = 14usize.pow(4);
const RANKS: u8 = 13;

///Simple error that conveys the information of a wrong
///Shape input.
#[derive(Debug)]
struct WrongShapeError {
    details: String,
}

impl WrongShapeError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for WrongShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}
impl Error for WrongShapeError {
    fn description(&self) -> &str {
        &self.details
    }
}

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

///Represent a bridge hand: 13 cards, with different and various properties.
///The majority of this properties were implemented in the bridge_deck crate,
///github version.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Hand {
    cards: Cards,
}

impl Hand {
    pub fn shape(self) -> Vec<u8> {
        let spades = self.cards.spades().len() as u8;
        let hearts = self.cards.hearts().len() as u8;
        let diamonds = self.cards.diamonds().len() as u8;
        let clubs = self.cards.clubs().len() as u8;
        vec![spades, hearts, diamonds, clubs]
    }
    pub fn new() -> Self {
        let mut deck = Cards::ALL;
        Hand {
            cards: deck.pick(13).unwrap(),
        }
    }

    pub fn spades(self) -> Cards {
        self.cards.spades()
    }
    pub fn hearts(self) -> Cards {
        self.cards.hearts()
    }
    pub fn diamonds(self) -> Cards {
        self.cards.diamonds()
    }
    pub fn clubs(self) -> Cards {
        self.cards.clubs()
    }
    pub fn hcp(self) -> usize {
        self.cards.high_card_points()
    }
    pub fn from_str(hand: &str) -> Result<Hand, String> {
        Ok(Hand {
            cards: Cards::from_str(hand)?,
        })
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cards)
    }
}

impl<'a> IntoIterator for &'a Hand {
    type Item = Cards;
    type IntoIter = HandIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HandIterator {
            hand: &self.cards,
            index: 0,
        }
    }
}

///Iterator for the Hand struct. I couldn't do better ;(
pub struct HandIterator<'a> {
    hand: &'a Cards,
    index: usize,
}

impl<'a> Iterator for HandIterator<'a> {
    type Item = Cards;
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => self.hand.spades(),
            1 => self.hand.hearts(),
            2 => self.hand.diamonds(),
            3 => self.hand.clubs(),
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}

///A Shape producing struct, with some sort of cache. Really doubious.
pub struct ShapeFactory<'a> {
    cache_table: HashMap<&'a str, Shape>,
}

impl<'a> ShapeFactory<'a> {
    pub const JOKER: char = 'x';

    pub fn new() -> Self {
        ShapeFactory {
            cache_table: HashMap::new(),
        }
    }
    pub fn new_shape(
        &'a mut self,
        shape: Option<&'a str>,
    ) -> Result<Shape, Box<(dyn Error + 'static)>> {
        if let Some(pattern) = shape {
            //if a pattern is provided
            if let Some(instance) = self.cache_table.get_mut(pattern)
            //search it in the cache table
            {
                Ok(instance.clone())
            } else {
                //if not found, produce it
                let mut obj = Shape {
                    min_ls: [13, 13, 13, 13],
                    max_ls: [13, 13, 13, 13],
                    table: [false; SHAPE_COMBINATIONS],
                    op_cache: HashMap::new(),
                };
                let shape_pattern: Vec<char> = pattern.chars().into_iter().collect();
                let parsed: Vec<u8> = Vec::new();
                obj.insert(shape_pattern, parsed)?;
                self.cache_table.insert(pattern, obj);
                Ok(self.cache_table.get_mut(pattern).unwrap().clone())
            }
        } else {
            //if not a pattern, return a void Shape
            let mut instance = Shape {
                min_ls: [4, 3, 3, 3],
                max_ls: [4, 3, 3, 3],
                table: [false; SHAPE_COMBINATIONS],
                op_cache: HashMap::new(),
            };
            let pattern: Vec<char> = vec!['4', '3', '3', '3'];
            let parsed: Vec<u8> = Vec::new();
            instance.insert(pattern, parsed)?;
            self.cache_table.insert("4333", instance);
            Ok(self.cache_table.get("4333").unwrap().clone())
        }
    }
}

///A shape, a set of shape really: uses a 4D table to keep track of the
///various distributions, so membership check is a matter of checking if a array
///has a bool set to true. Implements some sort of cache which I will be rewriting in the near
///future.
///TODO:Implement a better cache instead of a HashMap.
#[derive(Debug, Clone, Eq)]
pub struct Shape {
    min_ls: [u8; 4],
    max_ls: [u8; 4],
    table: [bool; SHAPE_COMBINATIONS],
    op_cache: HashMap<(&'static str, [bool; SHAPE_COMBINATIONS]), Shape>,
}
impl Hash for Shape {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.table.hash(hasher);
    }
}

impl PartialEq for Shape {
    fn ne(&self, other: &Self) -> bool {
        self.table != other.table
    }

    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
    }
}

impl Shape {
    pub fn new(_shape: Option<&str>) -> Self {
        todo!()
    }

    fn insert(
        &mut self,
        mut it: Vec<char>,
        parsed: Vec<u8>,
    ) -> Result<(), Box<dyn Error + 'static>> {
        let head: Vec<u8>;
        if it.len() == 0 {
            return self.insert1(parsed, false);
        }
        if let Some('(') = it.first() {
            let closing = if let Some(index) = it.iter().position(|&x| x == ')') {
                Ok(index)
            } else {
                Err(WrongShapeError::new("Unbalanced parentheses."))
            };
            let closing: usize = closing?;
            head = (it[1..closing])
                .into_iter()
                .map(|&x| {
                    if x == ShapeFactory::JOKER {
                        Ok(15u8)
                    } else {
                        match x.to_digit(10) {
                            Some(value) => Ok(value as u8),
                            None => Err(WrongShapeError::new(
                                "Shape pattern contains unknown chars.",
                            )),
                        }
                    }
                })
                .map(|x| x.unwrap())
                .collect();
            it = (it[closing + 1..]).to_vec();
        } else {
            head = (it[0..1])
                .into_iter()
                .map(|&x| {
                    if x == ShapeFactory::JOKER {
                        Ok(15u8)
                    } else {
                        match x.to_digit(10) {
                            Some(value) => Ok(value as u8),
                            None => Err(WrongShapeError::new(
                                "Shape pattern contains unknown chars.",
                            )),
                        }
                    }
                })
                .map(|x| x.unwrap())
                .collect();
            it = (it[1..]).to_vec();
        }
        for perm in head.iter().permutations(head.len()) {
            let mut new_parsed: Vec<u8> = parsed.to_vec();
            new_parsed.extend(perm);
            return self.insert(it, new_parsed);
        }
        Ok(())
    }
    fn insert1(&mut self, shape: Vec<u8>, safe: bool) -> Result<(), Box<dyn Error>> {
        let jokers = any(shape.iter(), |&x| x == 15);
        let pre_set: u8 = shape.iter().sum();
        if !jokers {
            if pre_set == 13 {
                for suit in Suit::ALL {
                    let suit = *suit as usize;
                    self.min_ls[suit] = u8::min(self.min_ls[suit], shape[suit]);
                    self.max_ls[suit] = u8::max(self.max_ls[suit], shape[suit]);
                }
                self.table[self.flatten(shape)] = true;
                return Ok(());
            } else if safe == true {
                return Err(Box::new(WrongShapeError::new(
                    "Wrong number of cards in shape.",
                )));
            }
        } else {
            if pre_set > 13 {
                return Err(Box::new(WrongShapeError::new("Invalid ambiguous shape.")));
            }
            for (i, l) in shape.iter().enumerate() {
                if l == &(RANKS + 1) {
                    for ll in 0..13 - pre_set + 1 {
                        let mut new_shape: Vec<u8> = shape[..i].to_vec();
                        new_shape.push(ll);
                        new_shape.extend(shape[i + 1..].to_vec());
                        self.insert1(new_shape, false)?
                    }
                }
            }
        }
        Ok(())
    }

    fn flatten(&self, shape: Vec<u8>) -> usize {
        let (s, h, d, c) = shape.iter().map(|&x| x as usize).next_tuple().unwrap();
        ((((s * (RANKS + 1) as usize + h) * (RANKS + 1) as usize) + d) * (RANKS + 1) as usize) + c
    }

    fn from_table(table: [bool; SHAPE_COMBINATIONS], len_hints: LenHint) -> Self {
        match len_hints {
            LenHint::Lenghts { min_ls, max_ls } => Self {
                table,
                min_ls,
                max_ls,
                op_cache: HashMap::new(),
            },
            LenHint::None => Self {
                table,
                min_ls: [0, 0, 0, 0],
                max_ls: [0, 0, 0, 0],
                op_cache: HashMap::new(),
            },
        }
    }
}

///Enum used to pass hint for suit lenghts
///to the Shape::from_table method
enum LenHint {
    None,
    Lenghts { min_ls: [u8; 4], max_ls: [u8; 4] },
}

///Trait for checking membership, used in particular to check if a
///Hand shape is matched by a Shape instance
trait Membership<Contenuto> {
    fn include(self, contenuto: Contenuto) -> bool;
}

impl Membership<Hand> for Shape {
    fn include(self, contenuto: Hand) -> bool {
        self.table[self.flatten(contenuto.shape())] as bool
    }
}
impl std::ops::Add for Shape {
    type Output = Shape;
    fn add(mut self, rhs: Self) -> Self::Output {
        if let Some(shape) = self.op_cache.get(&("+", rhs.table)) {
            return shape.clone();
        } else {
            let mut min_ls = [0u8; 4];
            let mut max_ls = [0u8; 4];
            for index in 0..=4 {
                min_ls[index] = u8::min(self.min_ls[index], rhs.min_ls[index]);
                max_ls[index] = u8::max(self.max_ls[index], rhs.max_ls[index]);
            }
            let mut table = [false; SHAPE_COMBINATIONS];
            for (i, bit) in self
                .table
                .iter()
                .zip(rhs.table.iter())
                .map(|(&x, &y)| x | y)
                .enumerate()
            {
                table[i] = bit;
            }
            let shape = Shape::from_table(table, LenHint::Lenghts { min_ls, max_ls });
            self.op_cache.insert(("+", rhs.table.clone()), shape);
            self.op_cache.get(&("+", rhs.table)).unwrap().clone()
        }
    }
}

impl std::ops::Sub for Shape {
    type Output = Shape;
    fn sub(mut self, rhs: Self) -> Self::Output {
        if let Some(shape) = self.op_cache.get(&("-", rhs.table)) {
            return shape.clone();
        } else {
            let mut min_ls = [0u8; 4];
            let mut max_ls = [0u8; 4];
            for index in 0..=4 {
                min_ls[index] = u8::min(self.min_ls[index], rhs.min_ls[index]);
                max_ls[index] = u8::max(self.max_ls[index], rhs.max_ls[index]);
            }
            let mut table = [false; SHAPE_COMBINATIONS];
            for (i, bit) in self
                .table
                .iter()
                .zip(&mut rhs.table.iter())
                .map(|(&x, &y)| x | y)
                .enumerate()
            {
                table[i] = bit;
            }
            let shape = Shape::from_table(table, LenHint::Lenghts { min_ls, max_ls });
            self.op_cache.insert(("-", rhs.table), shape);
            self.op_cache.get(&("-", rhs.table)).unwrap().clone()
        }
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
#[test]
#[should_panic]
fn shape_error_unbal_parentheses_test() {
    let mut factory = ShapeFactory::new();
    let mut res = factory.new_shape(None).unwrap();
    let pattern: Vec<char> = "4(333".chars().collect();
    res.insert(pattern, Vec::new()).unwrap();
}

#[test]
fn shape_parens_ok_test() {
    let mut factory = ShapeFactory::new();
    let mut res = factory.new_shape(None).unwrap();
    let pattern: Vec<char> = "4(34)2".chars().collect();
    let shape = res.insert(pattern, Vec::new()).unwrap();
}
