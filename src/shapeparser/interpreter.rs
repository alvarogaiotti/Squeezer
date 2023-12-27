use super::{Length, Modifier, Pattern};
use crate::Shapes;
use itertools::*;
use std::{collections::VecDeque, ops::ControlFlow};

trait Interpret {
    fn interpret(self) -> Shapes;
}
struct ShapeCreator {
    pub allocated_slots: u8,
    pub iterators: VecDeque<Pattern>,
}

impl From<Vec<Pattern>> for ShapeCreator {
    fn from(value: Vec<Pattern>) -> Self {
        let allocated_slots = value.iter().fold(0u8, pattern_length_adder);
        Self {
            allocated_slots,
            iterators: VecDeque::from(value),
        }
    }
}
fn input_debug() {
    let mut stringa = String::new();
    let _ = std::io::stdin().read_line(&mut stringa);
}

impl ShapeCreator {
    fn recur_adder_helper(
        free_places: u8,
        shape: &mut Vec<u8>,
        patterns: &mut VecDeque<Pattern>,
        shapes: &mut Vec<Vec<u8>>,
        length: u8,
        cap: Option<u8>,
    ) {
        if let Some(cap) = cap {
            // If we capped the length of a AtMost element, we stop
            if length >= cap {
                // We push the length we have
                shape.push(length);
                // We go ahead with the costruction of shapes
                Self::recur(free_places, shape, patterns, shapes);
                // We backtrack, to restart e push lesser length values with free places
                shape.pop();
                return;
            }
        }
        if free_places == 0 {
            shape.push(length);
            Self::recur(free_places, shape, patterns, shapes);
            let _popped = shape.pop();
            return;
        }
        Self::recur_adder_helper(free_places - 1, shape, patterns, shapes, length + 1, cap);
        shape.push(length);
        Self::recur(free_places, shape, patterns, shapes);
        let _popped = shape.pop();
    }

    fn recur(
        free_places: u8,
        shape: &mut Vec<u8>,
        patterns: &mut VecDeque<Pattern>,
        shapes: &mut Vec<Vec<u8>>,
    ) {
        if let Some(pattern) = patterns.pop_front() {
            if let ControlFlow::Break(_) =
                Self::shortcircuit_if_last_closes_shape(shape, &pattern, shapes)
            {
                patterns.push_front(pattern);
                return;
            }
            Self::handle_action_based_on_pattern(&pattern, free_places, shape, patterns, shapes);
            patterns.push_front(pattern);
        } else if free_places == 0 {
            shapes.push(shape.clone());
        }
    }

    fn interpret(&mut self) {
        let _shape: Vec<u8> = Vec::with_capacity(4);
    }

    fn cap_at_suit_size(free_places: u8, len: u8) -> (u8, u8) {
        Self::cap_at_custom_size(free_places, len, 13)
    }
    fn cap_at_custom_size(mut free_places: u8, len: u8, cap: u8) -> (u8, u8) {
        assert!(len < cap);
        let new_len = (len + free_places).clamp(0, cap);
        free_places = if let Some(result) = free_places.checked_sub(cap - len) {
            result
        } else {
            0
        };
        (free_places, new_len)
    }

    fn handle_action_based_on_pattern(
        pattern: &Pattern,
        free_places: u8,
        shape: &mut Vec<u8>,
        patterns: &mut VecDeque<Pattern>,
        shapes: &mut Vec<Vec<u8>>,
    ) {
        match *pattern {
            Pattern::Suit(Length {
                length,
                modifier: Modifier::AtLeast,
            }) => {
                Self::recur_adder_helper(free_places, shape, patterns, shapes, length, None);
            }
            Pattern::Suit(Length {
                length,
                modifier: Modifier::AtMost,
            }) => {
                Self::recur_adder_helper(free_places, shape, patterns, shapes, 0, Some(length));
            }
            Pattern::Suit(Length {
                length,
                modifier: Modifier::Exact,
            }) => {
                shape.push(length);
                Self::recur(free_places, shape, patterns, shapes);
                let _popped = shape.pop();
            }
            Pattern::Group(ref lengths) => {
                Self::handle_group_pattern(lengths, patterns, free_places, shape, shapes);
            }
        }
    }

    fn handle_group_pattern(
        lengths: &Vec<Length>,
        patterns: &mut VecDeque<Pattern>,
        free_places: u8,
        shape: &mut Vec<u8>,
        shapes: &mut Vec<Vec<u8>>,
    ) {
        let group_len = lengths.len();
        for permutation in lengths.iter().permutations(group_len) {
            for suit in permutation.into_iter().rev() {
                patterns.push_front(Pattern::Suit(*suit))
            }
            Self::recur(free_places, shape, patterns, shapes);
            for _ in 0..group_len {
                patterns.pop_front();
            }
        }
    }

    fn shortcircuit_if_last_closes_shape(
        shape: &mut Vec<u8>,
        pattern: &Pattern,
        shapes: &mut Vec<Vec<u8>>,
    ) -> ControlFlow<()> {
        if shape.len() == 3 {
            let candidate = 13 - shape.iter().sum::<u8>();
            if pattern.contains(candidate) {
                shape.push(candidate);
                shapes.push(shape.clone());
                shape.pop();
                return ControlFlow::Break(());
            } else {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}

fn pattern_length_adder(accumulator: u8, element: &Pattern) -> u8 {
    match element {
        Pattern::Suit(Length {
            length: _,
            modifier: Modifier::AtMost,
        }) => accumulator,
        Pattern::Suit(Length {
            length,
            modifier: _,
        }) => accumulator + *length,
        Pattern::Group(group) => accumulator + (*group).iter().fold(0, group_length_adder),
    }
}

fn group_length_adder(accumulator: u8, length: &Length) -> u8 {
    match length {
        Length {
            length: _,
            modifier: Modifier::AtMost,
        } => accumulator,
        Length {
            length,
            modifier: _,
        } => accumulator + *length,
    }
}
mod test {

    #[test]
    fn test_recursion() {
        use super::ShapeCreator;
        use super::{Length, Modifier, Pattern};
        let patterns = vec![
            Pattern::Suit(Length {
                length: 3,
                modifier: Modifier::Exact,
            }),
            Pattern::Group(vec![
                Length {
                    length: 3,
                    modifier: Modifier::AtLeast,
                },
                Length {
                    length: 3,
                    modifier: Modifier::AtMost,
                },
                Length {
                    length: 4,
                    modifier: Modifier::Exact,
                },
            ]),
        ];
        let mut creator = ShapeCreator::from(patterns);
        let mut shapes = Vec::new();
        let mut shape = Vec::new();
        ShapeCreator::recur(
            13 - creator.allocated_slots,
            &mut shape,
            &mut creator.iterators,
            &mut shapes,
        );
        shapes.sort();
        let mut res = vec![
            vec![3, 6, 0, 4],
            vec![3, 6, 4, 0],
            vec![3, 4, 6, 0],
            vec![3, 4, 0, 6],
            vec![3, 0, 4, 6],
            vec![3, 0, 6, 4],
            vec![3, 5, 1, 4],
            vec![3, 5, 4, 1],
            vec![3, 4, 5, 1],
            vec![3, 4, 1, 5],
            vec![3, 1, 4, 5],
            vec![3, 1, 5, 4],
            vec![3, 4, 2, 4],
            vec![3, 4, 2, 4],
            vec![3, 4, 4, 2],
            vec![3, 4, 4, 2],
            vec![3, 2, 4, 4],
            vec![3, 2, 4, 4],
            vec![3, 3, 3, 4],
            vec![3, 3, 3, 4],
            vec![3, 3, 4, 3],
            vec![3, 3, 4, 3],
            vec![3, 4, 3, 3],
            vec![3, 4, 3, 3],
        ];
        res.sort();
        assert_eq!(shapes, res);
    }
    #[test]
    fn recursion2_test() {
        use super::{Length, Modifier, Pattern, ShapeCreator};
        use itertools::*;
        let patterns = vec![Pattern::Group(vec![
            Length {
                length: 3,
                modifier: Modifier::Exact,
            },
            Length {
                length: 3,
                modifier: Modifier::AtLeast,
            },
            Length {
                length: 3,
                modifier: Modifier::AtMost,
            },
            Length {
                length: 4,
                modifier: Modifier::Exact,
            },
        ])];
        let mut creator = ShapeCreator::from(patterns);
        let mut shapes = Vec::new();
        let mut shape = Vec::new();
        ShapeCreator::recur(
            13 - creator.allocated_slots,
            &mut shape,
            &mut creator.iterators,
            &mut shapes,
        );
        shapes.sort();
        let mut res: Vec<Vec<u8>> = Vec::new();
        res.extend(vec![3, 6, 0, 4].into_iter().permutations(4));
        res.extend(vec![3, 5, 1, 4].into_iter().permutations(4));
        res.extend(vec![3, 4, 2, 4].into_iter().permutations(4));
        res.extend(vec![3, 3, 3, 4].into_iter().permutations(4));
        res.sort();
        assert_eq!(shapes, res);
    }
}
