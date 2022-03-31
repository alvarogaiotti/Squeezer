use crate::prelude::*;

///Error for wrong Shape pattern passed to ShapeFactory.
#[derive(Debug)]
pub struct DealerError {
    details: String,
}

impl DealerError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}
impl fmt::Display for DealerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for DealerError {
    fn description(&self) -> &str {
        &self.details
    }
}

///A Shape producing struct, with some sort of cache. Really doubious.
#[derive(Debug, PartialEq, Eq)]
pub struct ShapeFactory<'a> {
    table: [bool; SHAPE_COMBINATIONS],
    cache_table: HashSet<&'a str>,
    op_cache: HashSet<(&'a str, &'a str)>,
    min_ls: [u8; 4],
    max_ls: [u8; 4],
    not_in_cache: HashMap<&'a str, [bool; SHAPE_COMBINATIONS]>,
}

impl<'a> ShapeFactory<'a> {
    pub const JOKER: char = 'x';

    pub fn new() -> Self {
        ShapeFactory {
            table: [false; SHAPE_COMBINATIONS],
            cache_table: HashSet::new(),
            op_cache: HashSet::new(),
            min_ls: [13, 13, 13, 13],
            max_ls: [0, 0, 0, 0],
            not_in_cache: HashMap::new(),
        }
    }
    pub fn new_shape(&mut self, shape: Option<&'a str>) -> Result<(), Box<(dyn Error + 'static)>> {
        if let Some(pattern) = shape {
            //if a pattern is provided
            if let Some(_) = self.cache_table.get(pattern)
            //search it in the cache table
            {
                Ok(())
            } else {
                //if not found, produce it
                let shape_pattern: Vec<char> = pattern.chars().into_iter().collect();
                self.insert(shape_pattern)?;
                self.cache_table.insert(pattern);
                Ok(())
            }
        } else {
            //if not a pattern, return a void Shape
            let pattern: Vec<char> = vec!['4', '3', '3', '3'];
            self.insert(pattern)?;
            self.cache_table.insert("4333");
            Ok(())
        }
    }

    //pub fn from_table(
    //    &'a mut self,
    //    table: [bool; SHAPE_COMBINATIONS],
    //    len_hints: LenHint,
    //) -> &Shape {
    //    match len_hints {
    //        LenHint::Lenghts { min_ls, max_ls } => {
    //            let shape = Shape {
    //                table,
    //                min_ls,
    //                max_ls,
    //                father: Rc::new(RefCell::new(self)),
    //            };
    //            let stringa = format!(
    //                "{}{}{}{}{}{}{}{}",
    //                min_ls[0],
    //                min_ls[1],
    //                min_ls[2],
    //                min_ls[3],
    //                max_ls[0],
    //                max_ls[1],
    //                max_ls[2],
    //                max_ls[3],
    //            )
    //            .as_str();
    //            self.cache_table.insert(stringa.clone(), shape);
    //            &shape
    //        }
    //        LenHint::None => {
    //            let mut shape = Shape {
    //                table,
    //                min_ls: [0, 0, 0, 0],
    //                max_ls: [0, 0, 0, 0],
    //                father: Rc::new(RefCell::new(self)),
    //            };
    //            let min_ls = [13, 13, 13, 13];
    //            let max_ls = [0, 0, 0, 0];
    //            for nonflat in (0..14).permutations(4) {
    //                if shape.table[shape.flatten(nonflat)] {
    //                    for (dim, coord) in nonflat.iter().enumerate() {
    //                        shape.min_ls[dim] = u8::min(shape.min_ls[dim], *coord);
    //                        shape.max_ls[dim] = u8::max(shape.min_ls[dim], *coord);
    //                    }
    //                }
    //            }
    //            let stringa = format!(
    //                "{}{}{}{}{}{}{}{}",
    //                min_ls[0],
    //                min_ls[1],
    //                min_ls[2],
    //                min_ls[3],
    //                max_ls[0],
    //                max_ls[1],
    //                max_ls[2],
    //                max_ls[3],
    //            )
    //            .as_str();
    //            self.cache_table.insert(stringa.clone(), shape);
    //            self.cache_table.get(stringa.clone()).unwrap()
    //        }
    //    }
    //}
    pub fn insert(&mut self, mut it: Vec<char>) -> Result<(), Box<dyn Error + 'static>> {
        let mut parsed: Vec<u8> = Vec::new();
        let mut collected: Vec<Vec<u8>> = Vec::new();
        let patterns = ShapeFactory::get_pattern(&mut it, &mut parsed, &mut collected)?;
        for pattern in patterns {
            self.insert1(pattern.clone(), false)?
        }
        Ok(())
    }

    fn table_from_pattern(
        shape: Vec<u8>,
        table: &mut [bool; SHAPE_COMBINATIONS],
        safe: bool,
    ) -> Result<(&[bool; SHAPE_COMBINATIONS], [u8; 4], [u8; 4]), Box<dyn Error + 'static>> {
        let jokers = any(shape.iter(), |&x| x == RANKS + 1);
        let pre_set: u8 = shape.iter().filter(|&&x| x != RANKS + 1).sum();
        let mut min_ls = [0u8; 4];
        let mut max_ls = [0u8; 4];
        if !jokers {
            if pre_set == 13 {
                for suit in Suit::ALL {
                    let suit = *suit as usize;
                    min_ls[suit] = u8::min(min_ls[suit], shape[suit]);
                    max_ls[suit] = u8::max(max_ls[suit], shape[suit]);
                }
                table[ShapeFactory::flatten(shape)] = true;
                return Ok((table, min_ls, max_ls));
            } else if safe == true {
                return Err(Box::new(DealerError::new(
                    "Wrong number of cards in shape.",
                )));
            };
        } else {
            if pre_set > 13 {
                return Err(Box::new(DealerError::new("Invalid ambiguous shape.")));
            }
            for (i, l) in shape.iter().enumerate() {
                if l == &(RANKS + 1) {
                    for ll in 0..13 - pre_set + 1 {
                        let mut new_shape: Vec<u8> = shape[..i].to_vec();
                        new_shape.push(ll);
                        new_shape.extend(shape[i + 1..].to_vec());
                        ShapeFactory::table_from_pattern(new_shape, table, false)?;
                    }
                };
            }
        };
        Ok((table, min_ls, max_ls))
    }

    fn insert1(&mut self, shape: Vec<u8>, safe: bool) -> Result<(), Box<dyn Error>> {
        let mut table = [false; SHAPE_COMBINATIONS];
        let (table, min_ls, max_ls) = ShapeFactory::table_from_pattern(shape, &mut table, safe)?;
        for suit in Suit::ALL {
            let suit = *suit as usize;
            self.min_ls[suit] = u8::min(self.min_ls[suit], min_ls[suit]);
            self.max_ls[suit] = u8::max(self.max_ls[suit], max_ls[suit]);
        }
        for (i, bit) in table.iter().enumerate() {
            self.table[i] |= bit;
        }
        return Ok(());
    }
    fn get_pattern(
        it: &mut Vec<char>,
        parsed: &mut Vec<u8>,
        collected: &'a mut Vec<Vec<u8>>,
    ) -> Result<&'a Vec<Vec<u8>>, Box<dyn Error + 'static>> {
        if it.len() == 0 {
            collected.push(parsed.clone());
            Ok(collected)
        } else {
            let head: Vec<u8>;
            if let Some('(') = it.first() {
                let closing = if let Some(index) = it.iter().position(|&x| x == ')') {
                    Ok(index)
                } else {
                    Err(DealerError::new("Unbalanced parentheses."))
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
                                None => {
                                    Err(DealerError::new("Shape pattern contains unknown chars."))
                                }
                            }
                        }
                    })
                    .map(|x| x.unwrap())
                    .collect();
                *it = (it[closing + 1..]).to_vec();
            } else {
                head = (it[0..1])
                    .into_iter()
                    .map(|&x| {
                        if x == ShapeFactory::JOKER {
                            Ok(RANKS + 1)
                        } else {
                            match x.to_digit(10) {
                                Some(value) => Ok(value as u8),
                                None => {
                                    Err(DealerError::new("Shape pattern contains unknown chars."))
                                }
                            }
                        }
                    })
                    .map(|x| x.unwrap())
                    .collect();
                *it = (it[1..]).to_vec();
            }
            for perm in head.iter().permutations(head.len()) {
                parsed.extend(perm.clone());
                ShapeFactory::get_pattern(&mut it.clone(), parsed, collected)?;
                parsed.drain(parsed.len() - perm.len()..);
            }
            Ok(collected)
        }
    }

    fn flatten(shape: Vec<u8>) -> usize {
        let (s, h, d, c) = shape.iter().map(|&x| x as usize).next_tuple().unwrap();
        ((((s * (RANKS + 1) as usize + h) * (RANKS + 1) as usize) + d) * (RANKS + 1) as usize) + c
    }
    fn remove(&mut self, rhs: &'a str) {
        let mut table = [false; SHAPE_COMBINATIONS];
        let mut collected: Vec<Vec<u8>> = Vec::new();
        let _all_well =
            ShapeFactory::get_pattern(&mut rhs.chars().collect(), &mut Vec::new(), &mut collected);
        for pattern in collected {
            if let Ok((table, _min_ls, _max_ls)) =
                ShapeFactory::table_from_pattern(pattern, &mut table, false)
            {
                for (i, &bit) in table.iter().enumerate() {
                    if self.table[i] == true && bit == true {
                        self.table[i] = false
                    }
                }
            }
            self.op_cache.insert((rhs, "-"));
        }
    }

    fn update_lenghts(&mut self) {}
}

///Enum used to pass hint for suit lenghts
///to the Shape::from_table method
pub enum LenHint {
    None,
    Lenghts { min_ls: [u8; 4], max_ls: [u8; 4] },
}

///Trait for checking membership, used in particular to check if a
///Hand shape is matched by a Shape instance
pub trait Membership<'a, Contenuto> {
    fn includes(&self, contenuto: &Contenuto) -> bool;
    fn is_not_in(&mut self, non_contenuto: &Contenuto, pattern: &'a str) -> bool;
}

impl<'a> Membership<'a, Hand> for ShapeFactory<'a> {
    fn includes(&self, contenuto: &Hand) -> bool {
        self.table[ShapeFactory::flatten(contenuto.shape())] as bool
    }
    fn is_not_in(&mut self, non_contenuto: &Hand, pattern: &'a str) -> bool {
        if let Some(table) = self.not_in_cache.get(pattern) {
            table[ShapeFactory::flatten(non_contenuto.shape())]
        } else {
            let mut collected: Vec<Vec<u8>> = Vec::new();
            let mut patterns = ShapeFactory::get_pattern(
                &mut pattern.chars().collect(),
                &mut Vec::new(),
                &mut collected,
            )
            .unwrap();
            let mut pattern_table = [false; SHAPE_COMBINATIONS];
            let mut shape_table = [false; SHAPE_COMBINATIONS];
            for worked_pattern in collected {
                let (table, _, _) =
                    ShapeFactory::table_from_pattern(worked_pattern, &mut pattern_table, false)
                        .unwrap();
                for (i, bit) in table.iter().enumerate() {
                    shape_table[i] |= bit;
                }
            }
            let res = shape_table[ShapeFactory::flatten(non_contenuto.shape())];
            self.not_in_cache.insert(pattern.clone(), shape_table);
            res
        }
    }
}
impl<'a> std::ops::Add<&'a str> for &mut ShapeFactory<'a> {
    type Output = Self;
    fn add(self, rhs: &'a str) -> Self::Output {
        if let Some(_) = self.op_cache.get(&(rhs, "+")) {
            self
        } else {
            let pattern: Vec<char> = rhs.chars().collect();
            self.insert(pattern).unwrap();
            self.op_cache.insert((rhs.clone(), "+"));
            self
        }
    }
}

impl<'a> std::ops::Sub<&'a str> for &mut ShapeFactory<'a> {
    type Output = Self;
    fn sub(self, rhs: &'a str) -> Self::Output {
        if let Some(_) = self.op_cache.get(&(rhs, "-")) {
            self
        } else {
            self.remove(rhs);
            self
        }
    }
}

#[cfg(test)]
#[test]
fn factory_get_pattern_test() {
    let mut collected: Vec<Vec<u8>> = Vec::new();

    let mut infos = ShapeFactory::get_pattern(
        &mut "4333".chars().collect(),
        &mut Vec::new(),
        &mut collected,
    )
    .unwrap();
    assert_eq!(collected.pop().unwrap(), vec![4u8, 3u8, 3u8, 3u8]);
}
#[test]
fn shape_creation_test() {
    let mut factory = ShapeFactory::new();
    factory.insert("4333".chars().collect()).unwrap();
    assert!(factory.table[11609] == true);
}
#[test]
#[should_panic]
fn shape_error_unbal_parentheses_test() {
    let mut factory = ShapeFactory::new();
    let pattern: Vec<char> = "4(333".chars().collect();
    factory.insert(pattern).unwrap();
}

#[test]
fn shape_parens_interpretation_working_test() {
    let mut factory = ShapeFactory::new();
    factory.new_shape(Some("4(34)2")).unwrap();
    let mut true_arr = Vec::<usize>::new();
    for (i, data) in factory.table.iter().enumerate() {
        if *data {
            true_arr.push(i)
        }
    }
    assert_eq!(vec![11622 as usize, 11804 as usize], true_arr);
}

#[test]
fn membership_shape_hand_test() {
    let mut factory = ShapeFactory::new();
    factory.new_shape(Some("4(34)2")).unwrap();
    let deck = Cards::ALL;
    let clubs = deck.clubs().pick(2).unwrap();
    let diamonds = deck.diamonds().pick(4).unwrap();
    let hearts = deck.hearts().pick(3).unwrap();
    let spades = deck.spades().pick(4).unwrap();
    let cards = Cards::EMPTY
        .union(spades)
        .union(clubs)
        .union(diamonds)
        .union(hearts);

    let hand = Hand { cards };
    //println!("{}", ShapeFactory::flatten(hand.shape()));
    assert!(factory.includes(&hand));
}
