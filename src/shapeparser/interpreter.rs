use super::{
    parser::Parser, scanner::ScanningShapeError, Length, Modifier, ParsingShapeError, Pattern,
};
use itertools::*;
use std::{cmp::Ordering, collections::VecDeque, ops::ControlFlow};

pub type ShapePattern = [u8; 4];

#[derive(Debug)]
pub(crate) struct ShapeCreator {
    pub free_places: u8,
    pub patterns: VecDeque<Pattern>,
}

#[derive(Debug)]
pub enum InterpretationShapeError {
    TooMany,
    NotEnough,
}

impl std::fmt::Display for InterpretationShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::TooMany => "shape has too many allocated slots",
                Self::NotEnough => "shape hasn't enough allocated slots",
            }
        )
    }
}

impl std::error::Error for InterpretationShapeError {}

impl TryFrom<Vec<Pattern>> for ShapeCreator {
    type Error = InterpretationShapeError;
    fn try_from(value: Vec<Pattern>) -> Result<Self, InterpretationShapeError> {
        fn check_modifier(to_be_checked: Modifier) -> impl Fn(&Pattern) -> bool {
            move |pattern| match *pattern {
                Pattern::Suit(Length {
                    length: _,
                    modifier,
                }) => modifier == to_be_checked,
                Pattern::Group(ref lengths) => lengths.iter().any(|length| {
                    let Length {
                        length: _,
                        modifier,
                    } = *length;
                    modifier == to_be_checked
                }),
            }
        }

        let allocated_slots = value.iter().fold(0u8, pattern_length_adder);
        match allocated_slots.cmp(&13) {
            Ordering::Greater => Err(InterpretationShapeError::TooMany),
            Ordering::Less => {
                // Implementation specific detail: we store jokers ('x') as 0 AtLeast.
                if value.iter().any(check_modifier(Modifier::AtMost))
                    && !value.iter().any(check_modifier(Modifier::AtLeast))
                {
                    Err(InterpretationShapeError::NotEnough)
                } else {
                    Ok(Self {
                        free_places: 13 - allocated_slots,
                        patterns: VecDeque::from(value),
                    })
                }
            }
            Ordering::Equal => Ok(Self {
                free_places: 13 - allocated_slots,
                patterns: VecDeque::from(value),
            }),
        }
    }
}

#[derive(Debug)]
pub struct CreationShapeError {
    origin: CreationShapeErrorKind,
}

#[derive(Debug)]
pub enum CreationShapeErrorKind {
    Interpretation(InterpretationShapeError),
    Parsing(ParsingShapeError),
    Scanning(ScanningShapeError),
}

impl std::fmt::Display for CreationShapeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreationShapeErrorKind::Interpretation(err) => err.fmt(f),
            CreationShapeErrorKind::Parsing(err) => err.fmt(f),
            CreationShapeErrorKind::Scanning(err) => err.fmt(f),
        }
    }
}

impl std::fmt::Display for CreationShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error creating shape: {}", self.origin)
    }
}
impl std::error::Error for CreationShapeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let error: &dyn std::error::Error = match self.origin {
            CreationShapeErrorKind::Interpretation(ref err) => err,
            CreationShapeErrorKind::Parsing(ref err) => err,
            CreationShapeErrorKind::Scanning(ref err) => err,
        };
        Some(error)
    }
}

impl From<ScanningShapeError> for CreationShapeError {
    fn from(value: ScanningShapeError) -> Self {
        Self {
            origin: CreationShapeErrorKind::Scanning(value),
        }
    }
}

impl From<ParsingShapeError> for CreationShapeError {
    fn from(value: ParsingShapeError) -> Self {
        Self {
            origin: CreationShapeErrorKind::Parsing(value),
        }
    }
}

impl From<InterpretationShapeError> for CreationShapeError {
    fn from(value: InterpretationShapeError) -> Self {
        Self {
            origin: CreationShapeErrorKind::Interpretation(value),
        }
    }
}

impl ShapeCreator {
    pub fn build_shape(pattern: &str) -> Result<Vec<Vec<u8>>, CreationShapeError> {
        let parsed_input = Parser::parse_pattern(pattern)?;
        let mut shape_creator = ShapeCreator::try_from(parsed_input)?;
        let mut shape = Vec::new();
        let mut shapes = Vec::new();
        shape_creator.interpret(&mut shape, &mut shapes);
        Ok(shapes)
    }

    fn recur_adder_helper(
        &mut self,
        shape: &mut Vec<u8>,
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
                self.interpret(shape, shapes);
                // We backtrack, to restart e push lesser length values with free places
                shape.pop();
                return;
            }
        }
        if self.free_places == 0 {
            shape.push(length);
            self.interpret(shape, shapes);
            let _popped = shape.pop();
            return;
        }
        self.free_places -= 1;
        self.recur_adder_helper(shape, shapes, length + 1, cap);
        shape.push(length);
        self.free_places += 1;
        self.interpret(shape, shapes);
        let _popped = shape.pop();
    }

    fn interpret(&mut self, shape: &mut Vec<u8>, shapes: &mut Vec<Vec<u8>>) {
        if let Some(pattern) = self.patterns.pop_front() {
            if let ControlFlow::Break(_) =
                Self::shortcircuit_if_last_closes_shape(shape, &pattern, shapes)
            {
                self.patterns.push_front(pattern);
                return;
            }
            self.handle_action_based_on_pattern(&pattern, shape, shapes);
            self.patterns.push_front(pattern);
        } else if self.free_places == 0 {
            shapes.push(shape.clone());
        }
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
        &mut self,
        pattern: &Pattern,
        shape: &mut Vec<u8>,
        shapes: &mut Vec<Vec<u8>>,
    ) {
        match *pattern {
            Pattern::Suit(Length {
                length,
                modifier: Modifier::AtLeast,
            }) => {
                self.recur_adder_helper(shape, shapes, length, None);
            }
            Pattern::Suit(Length {
                length,
                modifier: Modifier::AtMost,
            }) => {
                self.recur_adder_helper(shape, shapes, 0, Some(length));
            }
            Pattern::Suit(Length {
                length,
                modifier: Modifier::Exact,
            }) => {
                shape.push(length);
                self.interpret(shape, shapes);
                let _popped = shape.pop();
            }
            Pattern::Group(ref lengths) => {
                self.handle_group_pattern(lengths, shape, shapes);
            }
        }
    }

    fn handle_group_pattern(
        &mut self,
        lengths: &Vec<Length>,
        shape: &mut Vec<u8>,
        shapes: &mut Vec<Vec<u8>>,
    ) {
        let group_len = lengths.len();
        for permutation in lengths.iter().permutations(group_len) {
            for suit in permutation.into_iter().rev() {
                self.patterns.push_front(Pattern::Suit(*suit))
            }
            self.interpret(shape, shapes);
            for _ in 0..group_len {
                self.patterns.pop_front();
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
                let mut clone = shape.clone();
                clone.push(candidate);
                shapes.push(clone);
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

#[cfg(test)]
mod test {
    use super::ShapeCreator;
    use super::{Length, Modifier, Pattern};

    #[test]
    fn test_recursion() {
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
        let mut creator = ShapeCreator::try_from(patterns).unwrap();
        let mut shapes = Vec::new();
        let mut shape = Vec::new();
        creator.interpret(&mut shape, &mut shapes);
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
        let mut creator = ShapeCreator::try_from(patterns).unwrap();
        let mut shapes = Vec::new();
        let mut shape = Vec::new();
        creator.interpret(&mut shape, &mut shapes);
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
