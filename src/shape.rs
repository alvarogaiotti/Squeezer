use std::{cell::RefCell, rc::Rc};

use crate::prelude::*;

///Error for wrong Shape pattern passed to ShapeFactory.
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

///A Shape producing struct, with some sort of cache. Really doubious.
#[derive(Debug, PartialEq, Eq)]
pub struct ShapeFactory<'a, 'b> {
    cache_table: HashMap<&'b str, Shape<'a>>,
    op_cache: HashMap<(&'a Shape<'a>, &'a Shape<'a>, &'b str), &'a Shape<'a>>,
}

impl<'a, 'b> ShapeFactory<'a, 'b> {
    pub const JOKER: char = 'x';

    pub fn new() -> Self {
        ShapeFactory {
            cache_table: HashMap::new(),
            op_cache: HashMap::new(),
        }
    }
    pub fn new_shape(
        &'b mut self,
        shape: Option<&'b str>,
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
                    father: Rc::new(RefCell::new(self)),
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
                father: Rc::new(RefCell::new(self)),
            };
            let pattern: Vec<char> = vec!['4', '3', '3', '3'];
            let parsed: Vec<u8> = Vec::new();
            instance.insert(pattern, parsed)?;
            self.cache_table.insert("4333", instance);
            Ok(self.cache_table.get("4333").unwrap().clone())
        }
    }
    fn add(&'a self, lhs: &'a Shape<'a>, rhs: &'a Shape<'a>, op: &str) -> &Shape {
        if let Some(shape) = self.op_cache.get(&(lhs, rhs, "+")) {
            return shape.clone();
        } else {
            let mut min_ls = [0u8; 4];
            let mut max_ls = [0u8; 4];
            for index in 0..4 {
                min_ls[index] = u8::min(lhs.min_ls[index], rhs.min_ls[index]);
                max_ls[index] = u8::max(lhs.max_ls[index], rhs.max_ls[index]);
            }
            let mut table = [false; SHAPE_COMBINATIONS];
            for (i, bit) in lhs
                .table
                .iter()
                .zip(rhs.table.iter())
                .map(|(&x, &y)| x | y)
                .enumerate()
            {
                table[i] = bit;
            }
            let shape = self.from_table(table, LenHint::Lenghts { min_ls, max_ls });
            self.op_cache.insert((lhs, rhs, "+"), &shape);
            self.op_cache.insert((rhs, lhs, "+"), &shape);
            &shape
        }
    }
    fn sub(&'a self, lhs: &'a Shape<'a>, rhs: &'a Shape<'a>, op: &str) -> &Shape {
        if let Some(shape) = self.op_cache.get(&(lhs, rhs, "-")) {
            return shape.clone();
        } else {
            let mut min_ls = [0u8; 4];
            let mut max_ls = [0u8; 4];
            for index in 0..4 {
                min_ls[index] = u8::min(lhs.min_ls[index], rhs.min_ls[index]);
                max_ls[index] = u8::max(lhs.max_ls[index], rhs.max_ls[index]);
            }
            let mut table = [false; SHAPE_COMBINATIONS];
            for (i, bit) in lhs
                .table
                .iter()
                .zip(rhs.table.iter())
                .map(|(&x, &y)| x ^ y)
                .enumerate()
            {
                table[i] = bit;
            }
            let shape = self.from_table(table, LenHint::Lenghts { min_ls, max_ls });
            self.op_cache.insert((lhs, rhs, "-"), &shape);
            self.op_cache.insert((rhs, lhs, "-"), &shape);
            &shape
        }
    }
    pub fn from_table(
        &'a mut self,
        table: [bool; SHAPE_COMBINATIONS],
        len_hints: LenHint,
    ) -> &Shape {
        match len_hints {
            LenHint::Lenghts { min_ls, max_ls } => {
                let shape = Shape {
                    table,
                    min_ls,
                    max_ls,
                    father: Rc::new(RefCell::new(self)),
                };
                let stringa = format!(
                    "{}{}{}{}{}{}{}{}",
                    min_ls[0],
                    min_ls[1],
                    min_ls[2],
                    min_ls[3],
                    max_ls[0],
                    max_ls[1],
                    max_ls[2],
                    max_ls[3],
                )
                .as_str();
                self.cache_table.insert(stringa.clone(), shape);
                &shape
            }
            LenHint::None => {
                let mut shape = Shape {
                    table,
                    min_ls: [0, 0, 0, 0],
                    max_ls: [0, 0, 0, 0],
                    father: Rc::new(RefCell::new(self)),
                };
                let min_ls = [13, 13, 13, 13];
                let max_ls = [0, 0, 0, 0];
                for nonflat in (0..14).permutations(4) {
                    if shape.table[shape.flatten(nonflat)] {
                        for (dim, coord) in nonflat.iter().enumerate() {
                            shape.min_ls[dim] = u8::min(shape.min_ls[dim], *coord);
                            shape.max_ls[dim] = u8::max(shape.min_ls[dim], *coord);
                        }
                    }
                }
                let stringa = format!(
                    "{}{}{}{}{}{}{}{}",
                    min_ls[0],
                    min_ls[1],
                    min_ls[2],
                    min_ls[3],
                    max_ls[0],
                    max_ls[1],
                    max_ls[2],
                    max_ls[3],
                )
                .as_str();
                self.cache_table.insert(stringa.clone(), shape);
                self.cache_table.get(stringa.clone()).unwrap()
            }
        }
    }
}

type Father<'a, 'b> = Rc<RefCell<&'a mut ShapeFactory<'a, 'b>>>;

///A shape, a set of shape really: uses a 4D table to keep track of the
///various distributions, so membership check is a matter of checking if a array
///has a bool set to true. Implements some sort of cache which I will be rewriting in the near
///future.
///TODO:Implement a better cache instead of a HashMap.
#[derive(Debug, Clone, Eq)]
pub struct Shape<'a, 'b> {
    min_ls: [u8; 4],
    max_ls: [u8; 4],
    table: [bool; SHAPE_COMBINATIONS],
    father: Father<'a, 'b>,
}
impl Hash for Shape<'_, '_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.table.hash(hasher);
    }
}

impl PartialEq for Shape<'_> {
    fn ne(&self, other: &Self) -> bool {
        self.table != other.table
    }

    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
    }
}

impl Shape<'_> {
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
                        Ok(RANKS + 1)
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
                        Ok(RANKS + 1)
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
            self.insert(it.clone(), new_parsed)?;
        }
        Ok(())
    }
    fn insert1(&mut self, shape: Vec<u8>, safe: bool) -> Result<(), Box<dyn Error>> {
        let jokers = any(shape.iter(), |&x| x == RANKS + 1);
        let pre_set: u8 = shape.iter().filter(|&&x| x != RANKS + 1).sum();
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
}

///Enum used to pass hint for suit lenghts
///to the Shape::from_table method
pub enum LenHint {
    None,
    Lenghts { min_ls: [u8; 4], max_ls: [u8; 4] },
}

///Trait for checking membership, used in particular to check if a
///Hand shape is matched by a Shape instance
pub trait Membership<Contenuto> {
    fn includes(&self, contenuto: &Contenuto) -> bool;
}

impl Membership<Hand> for Shape<'_> {
    fn includes(&self, contenuto: &Hand) -> bool {
        self.table[self.flatten(contenuto.shape())] as bool
    }
}
impl<'a> std::ops::Add for Shape<'a> {
    type Output = &'a Shape<'a>;
    fn add(self, rhs: Self) -> Self::Output {
        self.father.borrow_mut().add(&self, &rhs, "+")
    }
}

impl<'a> std::ops::Sub for Shape<'a> {
    type Output = &'a Shape<'a>;
    fn sub(self, rhs: Self) -> Self::Output {
        &*self.father.borrow_mut().sub(&self, &rhs, "+")
    }
}

#[cfg(test)]
#[test]
#[should_panic]
fn shape_error_unbal_parentheses_test() {
    let mut factory = ShapeFactory::new();
    let mut res = factory.new_shape(None).unwrap();
    let pattern: Vec<char> = "4(333".chars().collect();
    res.insert(pattern, Vec::new()).unwrap();
}

#[test]
fn shape_parens_interpretation_working_test() {
    let mut factory = ShapeFactory::new();
    let res = factory.new_shape(Some("4(34)2")).unwrap();
    let mut true_arr = Vec::<usize>::new();
    for (i, data) in res.table.iter().enumerate() {
        if *data {
            true_arr.push(i)
        }
    }
    assert_eq!(vec![11622 as usize, 11804 as usize], true_arr);
}

#[test]
fn membership_shape_hand_test() {
    let mut factory = ShapeFactory::new();
    let res = factory.new_shape(Some("4(34)2")).unwrap();
    let deck = Cards::ALL;
    let clubs = deck.clubs().pick(2).unwrap();
    let diamonds = deck.diamonds().pick(4).unwrap();
    let hearts = deck.hearts().pick(3).unwrap();
    let spades = deck.spades().pick(4).unwrap();
    let hand = Cards::EMPTY
        .union(spades)
        .union(clubs)
        .union(diamonds)
        .union(hearts);

    let cards = Hand { cards: hand };
    assert!(res.includes(&cards))
}
