// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use super::{
    parser::Parser, scanner::ScanningShapeError, Length, Modifier, ParsingShapeError, Pattern,
};
use itertools::Itertools;
use std::{cmp::Ordering, collections::VecDeque, ops::ControlFlow};

/// Represents a single shape description.
pub type ShapePattern = [u8; 4];

/// Represents the creator of shapes.
#[derive(Debug)]
pub(crate) struct ShapeCreator {
    /// Number of allocated slots for the shape as of now.
    pub allocated_slots: u8,
    /// Patterns to define the shape construction rules.
    pub patterns: VecDeque<Pattern>,
}

/// Represents errors that can occur during the interpretation of shapes.
#[derive(Debug)]
pub enum InterpretationShapeError {
    /// Indicates that there are too many allocated slots in a shape.
    TooMany,
    /// Indicates that there are not enough allocated slots in a shape.
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

#[derive(Debug, Copy, Clone, Default)]
struct PatternFormationChecker {
    accumulator: u8,
    flag_at_least: bool,
    flag_at_most: bool,
}

impl PatternFormationChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check(self) -> Result<(), InterpretationShapeError> {
        match self.accumulator.cmp(&13) {
            Ordering::Greater => {
                if self.flag_at_most {
                    Ok(())
                } else {
                    Err(InterpretationShapeError::TooMany)
                }
            }
            Ordering::Less => {
                if self.flag_at_least {
                    Ok(())
                } else {
                    Err(InterpretationShapeError::NotEnough)
                }
            }
            Ordering::Equal => Ok(()),
        }
    }
}

impl std::error::Error for InterpretationShapeError {}

impl TryFrom<Vec<Pattern>> for ShapeCreator {
    type Error = InterpretationShapeError;
    fn try_from(value: Vec<Pattern>) -> Result<Self, InterpretationShapeError> {
        let checker = PatternFormationChecker {
            accumulator: 0,
            flag_at_most: false,
            flag_at_least: false,
        };
        let allocated_slots = value.iter().fold(checker, pattern_length_adder);
        allocated_slots.check()?;
        Ok(Self {
            allocated_slots: 0,
            patterns: VecDeque::from(value),
        })
    }
}

/// Represents an error that can occur during the creation of shapes.
#[derive(Debug)]
pub struct CreationShapeError {
    /// The origin of the creation error.
    origin: CreationShapeErrorKind,
}

/// Represents the types of errors that can occur during the creation of shapes.
#[derive(Debug)]
pub enum CreationShapeErrorKind {
    /// Error during the interpretation of shapes.
    Interpretation(InterpretationShapeError),
    /// Error during the parsing of shapes.
    Parsing(ParsingShapeError),
    /// Error during the scanning of shapes.
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
    /// Builds a shape based on the provided pattern.
    pub fn build_shape(pattern: &str) -> Result<Vec<Vec<u8>>, CreationShapeError> {
        let parsed_input = Parser::parse_pattern(pattern)?;
        let mut shape_creator = ShapeCreator::try_from(parsed_input)?;
        let mut shape = Vec::new();
        let mut shapes = Vec::new();
        shape_creator.interpret(&mut shape, &mut shapes);
        Ok(shapes)
    }

    /// Recursive helper function to add elements to the shape based on the provided length and cap values.
    fn recur_adder_helper(
        &mut self,
        shape: &mut Vec<u8>,
        shapes: &mut Vec<Vec<u8>>,
        length: u8,
        cap: Option<u8>,
    ) {
        // Base case: we have a shape longer than 13.
        // We return doing nothing.
        if self.allocated_slots + length > 13 {
            return;
        }
        if let Some(cap) = cap {
            // If we capped the length of an AtMost element, we stop
            if length >= cap {
                // We push the length we have
                shape.push(length);
                self.allocated_slots += length;
                // We go ahead with the construction of the rest of the shape.
                self.interpret(shape, shapes);
                // We backtrack so the function up the call stack can push its length.
                self.allocated_slots -= length;
                shape.pop();
                return;
            }
        }

        // We recur to try to get
        // to the base case.
        self.recur_adder_helper(shape, shapes, length + 1, cap);

        // We retrace our step and try to go ahead with our length:
        // push the shape, interpret the rest.
        // pop the shape, remove the slots allocated and go on.
        self.allocated_slots += length;
        shape.push(length);
        self.interpret(shape, shapes);
        shape.pop();
        self.allocated_slots -= length;
    }

    /// Interprets the current pattern and constructs shapes accordingly.
    fn interpret(&mut self, shape: &mut Vec<u8>, shapes: &mut Vec<Vec<u8>>) {
        // If I have still pattern in the queue...
        if let Some(pattern) = self.patterns.pop_front() {
            // If we are dealing with the last pattern
            if let ControlFlow::Break(candidate) = Self::next_is_last_pattern(shape, &pattern) {
                // We check if we can create a shape with the last pattern
                if let Some(candidate) = candidate {
                    // If so, we push the shape to the shapes vector
                    let mut clone = shape.clone();
                    clone.push(candidate);
                    shapes.push(clone);
                }
                // Then, we always repush the pattern to the front and return
                self.patterns.push_front(pattern);
                return;
            }
            // Otherwise, we handle the pattern (Group or Suit), then we push it to the front
            // to backtrack and try the next pattern.
            self.handle_action_based_on_pattern(&pattern, shape, shapes);
            self.patterns.push_front(pattern);
        } else {
            // If we have no more patterns and we still have free places, we simply return as we are in a dead end.
        }
    }

    /// Handles the action based on the given pattern.
    fn handle_action_based_on_pattern(
        &mut self,
        pattern: &Pattern,
        shape: &mut Vec<u8>,
        shapes: &mut Vec<Vec<u8>>,
    ) {
        match *pattern {
            // If the pattern is an Exact suit length,
            // we push and continue interpreting the rest of the shape,
            // popping last to backtrack.
            Pattern::Suit(Length {
                length,
                modifier: Modifier::Exact,
            }) => {
                if self.allocated_slots + length < 14 {
                    self.allocated_slots += length;
                    shape.push(length);
                    self.interpret(shape, shapes);
                    self.allocated_slots -= length;
                    let _popped = shape.pop();
                }
            }
            // If the pattern is an AtLeast suit length,
            // we start start the recursion with no upper bound
            // and starting with the minimum length of the pattern.
            Pattern::Suit(Length {
                length,
                modifier: Modifier::AtLeast,
            }) => {
                self.recur_adder_helper(shape, shapes, length, None);
            }
            // If the pattern is an AtMost suit length,
            // we start start the recursion with the minimum length
            // and with some upper bound.
            Pattern::Suit(Length {
                length,
                modifier: Modifier::AtMost,
            }) => {
                self.recur_adder_helper(shape, shapes, 0, Some(length));
            }
            Pattern::Group(ref lengths) => {
                self.handle_group_pattern(lengths, shape, shapes);
            }
        }
    }

    /// Handles the group pattern by interpreting the lengths and updating the shapes.
    fn handle_group_pattern(
        &mut self,
        lengths: &[Length],
        shape: &mut Vec<u8>,
        shapes: &mut Vec<Vec<u8>>,
    ) {
        let group_len = lengths.len();
        for permutation in lengths.iter().permutations(group_len) {
            for suit in permutation {
                // Note to myself: is push front because we want to keep patterns not already handled last,
                // so we place our group members to the first place, where we are right now
                self.patterns.push_front(Pattern::Suit(*suit));
            }
            self.interpret(shape, shapes);
            for _ in 0..group_len {
                self.patterns.pop_front();
            }
        }
    }

    /// Short circuits if the last element closes the shape and adds it to the list of shapes.
    fn next_is_last_pattern(shape: &mut [u8], pattern: &Pattern) -> ControlFlow<Option<u8>> {
        if shape.len() == 3 {
            let candidate = 13u8.checked_sub(shape.iter().sum::<u8>());
            // If we have not owerlowed...
            if let Some(candidate) = candidate {
                // And the last pattern contains the residual length of the entire shape as now
                if pattern.contains(candidate) {
                    // We return the information that the candidate is valid
                    return ControlFlow::Break(Some(candidate));
                }
            }
            // else we return a None, since the candidate is not valid
            return ControlFlow::Break(None);
        }
        ControlFlow::Continue(())
    }
}

fn pattern_length_adder(
    mut accumulator: PatternFormationChecker,
    element: &Pattern,
) -> PatternFormationChecker {
    fn length_adder(
        mut accumulator: PatternFormationChecker,
        length: Length,
    ) -> PatternFormationChecker {
        match length {
            Length {
                modifier: Modifier::AtMost,
                length,
            } => {
                accumulator.accumulator += length;
                accumulator.flag_at_most = true;
                accumulator
            }
            Length {
                length,
                modifier: Modifier::AtLeast,
            } => {
                accumulator.accumulator += length;
                accumulator.flag_at_least = true;
                accumulator
            }
            Length { length, .. } => {
                accumulator.accumulator += length;
                accumulator
            }
        }
    }
    match element {
        Pattern::Suit(length) => length_adder(accumulator, *length),
        Pattern::Group(group) => {
            let group_acc = (*group)
                .iter()
                .fold(PatternFormationChecker::new(), |acc, &value| {
                    length_adder(acc, value)
                });
            accumulator.accumulator += group_acc.accumulator;
            accumulator.flag_at_least |= group_acc.flag_at_least;
            accumulator.flag_at_most |= group_acc.flag_at_most;
            accumulator
        }
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
    #[test]
    fn recursion3_test() {
        use super::{Length, Modifier, Pattern, ShapeCreator};
        let patterns = vec![
            Pattern::Group(vec![
                Length {
                    length: 3,
                    modifier: Modifier::Exact,
                },
                Length {
                    length: 2,
                    modifier: Modifier::Exact,
                },
            ]),
            Pattern::Group(vec![
                Length {
                    length: 7,
                    modifier: Modifier::Exact,
                },
                Length {
                    length: 1,
                    modifier: Modifier::Exact,
                },
            ]),
        ];
        let mut creator = ShapeCreator::try_from(patterns).unwrap();
        let mut shapes = Vec::new();
        let mut shape = Vec::new();
        creator.interpret(&mut shape, &mut shapes);
        shapes.sort();
        let mut res: Vec<_> = vec![[3, 2, 7, 1], [3, 2, 1, 7], [2, 3, 1, 7], [2, 3, 7, 1]];
        res.sort_unstable();
        assert_eq!(shapes, res);
    }
    #[test]
    fn check_strange_shapes() {
        let patterns = vec![
            Pattern::Suit(Length {
                length: 5,
                modifier: Modifier::Exact,
            }),
            Pattern::Suit(Length {
                length: 5,
                modifier: Modifier::AtMost,
            }),
            Pattern::Suit(Length {
                length: 4,
                modifier: Modifier::AtMost,
            }),
            Pattern::Suit(Length {
                length: 4,
                modifier: Modifier::AtMost,
            }),
        ];
        let mut creator = ShapeCreator::try_from(patterns).unwrap();
        let mut shapes = Vec::new();
        let mut shape = Vec::new();
        creator.interpret(&mut shape, &mut shapes);
        shapes.sort();
        let mut res: Vec<_> = vec![
            [5, 5, 3, 0],
            [5, 5, 2, 1],
            [5, 5, 1, 2],
            [5, 5, 0, 3],
            [5, 4, 4, 0],
            [5, 4, 3, 1],
            [5, 4, 2, 2],
            [5, 4, 1, 3],
            [5, 4, 0, 4],
            [5, 3, 4, 1],
            [5, 3, 3, 2],
            [5, 3, 2, 3],
            [5, 3, 1, 4],
            [5, 2, 4, 2],
            [5, 2, 3, 3],
            [5, 2, 2, 4],
            [5, 1, 4, 3],
            [5, 1, 3, 4],
            [5, 0, 4, 4],
        ];
        res.sort_unstable();
        assert_eq!(shapes, res);
    }
}
