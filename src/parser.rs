use std::{future, iter::Peekable};

use crate::prelude::*;

macro_rules! token_as_int {
    ($token:expr, $variant:path) => {
        match $token {
            $variant(x) => *x,
            _ => 0u8,
        }
    };
}

pub(crate) enum ShapeParsingError {
    UnmatchParenthesis,
    UnknownChar(char),
    OrphanModifier(char),
    ShapeTooLong,
    ShapeTooShort,
    NestedScope,
    SuitTooLong,
}

impl ShapeParsingError {
    pub(crate) fn as_string(&self) -> String {
        match *self {
            ShapeParsingError::UnmatchParenthesis => String::from("non matching parentheses"),
            ShapeParsingError::UnknownChar(char) => format!("unknown char {}", char),
            ShapeParsingError::OrphanModifier(char) => format!("orphan modifier: {}", char),
            ShapeParsingError::ShapeTooLong => String::from("shape has more than 13 cards"),
            ShapeParsingError::ShapeTooShort => String::from("shape has less than 13 cards"),
            ShapeParsingError::NestedScope => String::from("nested grouping not supported"),
            ShapeParsingError::SuitTooLong => String::from("suit cannot have more than 9 cards"),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum Token {
    Length(u8),
    Plus,
    Minus,
    Joker,
    OpenParens,
    CloseParens,
    Empty,
}

impl Token {
    fn is_len_token(&self) -> bool {
        matches!(self, Self::Length(_))
    }
    fn as_int(&self) -> u8 {
        assert!(self.is_len_token());
        token_as_int!(self, Token::Length)
    }
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Length(num) => format!("Token::Length({})", num),
                Token::Plus => "Token::Plus".to_string(),
                Token::Minus => "Token::Minus".to_string(),
                Token::Joker => "Token::Joker".to_string(),
                Token::OpenParens => "Token::OpenParens".to_string(),
                Token::CloseParens => "Token::ClosedParens".to_string(),
                Token::Empty => "Token::Empty".to_string(),
            }
        )
    }
}

pub enum LengthModifier {
    AtLeast,
    AtMost,
    Exact,
}

pub struct Length {
    length: u8,
    modifiers: LengthModifier,
}

impl Length {
    fn at_least(length: u8) -> Self {
        Self {
            length,
            modifiers: LengthModifier::AtLeast,
        }
    }
    fn at_most(length: u8) -> Self {
        Self {
            length,
            modifiers: LengthModifier::AtMost,
        }
    }
    fn exact(length: u8) -> Self {
        Self {
            length,
            modifiers: LengthModifier::Exact,
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum Scope {
    Nested,
    Linear,
}

pub trait ToPeekableStream<I>: std::iter::Iterator
where
    I: Iterator,
{
    fn peekable(self) -> Peekable<I>;
}

pub struct Parser<I: Iterator<Item = char>> {
    stream: Peekable<I>,
    previous_token: Token,
    lengths: [Option<Length>; 4],
    scope: Scope,
}

impl<I: Iterator<Item = char>> Parser<I> {
    pub fn new(stream: I) -> Self {
        Self {
            stream: stream.peekable(),
            previous_token: Token::Empty,
            lengths: [None, None, None, None],
            scope: Scope::Linear,
        }
    }

    pub fn parse(&mut self) -> Result<[LenRange; 4], DealerError> {
        loop {
            let current_token = self.parse_single_token()?;

            if let Some(mut_ref_to_current_none) = self.lengths.iter_mut().find(|x| x.is_none()) {
                *mut_ref_to_current_none = match self.previous_token {
                    Token::Length(length) => match current_token {
                        Token::Plus => Some(Length {
                            length,
                            modifiers: LengthModifier::AtLeast,
                        }),
                        Token::Minus => Some(Length {
                            length,
                            modifiers: LengthModifier::AtMost,
                        }),
                        _ => Some(Length {
                            length,
                            modifiers: LengthModifier::Exact,
                        }),
                    },
                    Token::Joker => Some(Length {
                        length: 0,
                        modifiers: LengthModifier::AtLeast,
                    }),
                    _ => unreachable!(),
                };

                if current_token == Token::Empty {
                    break;
                }
            }
        }
        Ok([LenRange::default(); 4])
    }

    fn parse_single_token(&mut self) -> Result<Token, DealerError> {
        let current_token = match self.stream.next() {
            Some(token) => match token {
                '0' => Token::Length(0),
                '1' => Token::Length(1),
                '2' => Token::Length(2),
                '3' => Token::Length(3),
                '4' => Token::Length(4),
                '5' => Token::Length(5),
                '6' => Token::Length(6),
                '7' => Token::Length(7),
                '8' => Token::Length(8),
                '9' => Token::Length(9),
                '+' => match self.previous_token {
                    Token::Length(_) => Token::Plus, // In theory not possible as we have peeked to check
                    _ => return Err(DealerError::from(ShapeParsingError::OrphanModifier(token))),
                },
                '-' => match self.previous_token {
                    Token::Length(_) => Token::Minus, // In theory not possible as we have peeked to check
                    _ => return Err(DealerError::from(ShapeParsingError::OrphanModifier(token))),
                },
                'x' => Token::Joker,
                '(' => {
                    if self.scope == Scope::Nested {
                        return Err(DealerError::from(ShapeParsingError::NestedScope));
                    }
                    self.scope = Scope::Nested;
                    Token::OpenParens
                }
                ')' => {
                    if self.scope == Scope::Linear {
                        return Err(DealerError::from(ShapeParsingError::UnmatchParenthesis));
                    }
                    self.scope = Scope::Linear;
                    Token::CloseParens
                }
                _ => return Err(DealerError::from(ShapeParsingError::UnknownChar(token))),
            },
            None => {
                if self.scope == Scope::Nested {
                    return Err(DealerError::from(ShapeParsingError::UnmatchParenthesis));
                } else {
                    Token::Empty
                }
            }
        };
        Ok(current_token)
    }
}

#[cfg(test)]
#[test]
fn parse_pattern_test() {
    let mut parser = Parser::new("(4+3-3x)".chars());
    parser.parse().unwrap();
    assert_eq!(1, 1);
}
#[test]
#[should_panic]
fn parse_returns_orphan_modifier_test() {
    let mut parser = Parser::new("(+433x)".chars());
    parser.parse().unwrap();
}
#[test]
#[should_panic]
fn parse_returns_unclosed_delimiter_test() {
    let mut parser = Parser::new("4+33x)".chars());
    parser.parse().unwrap();
}
#[test]
#[should_panic]
fn parse_returns_unclosed_delimiter2_test() {
    let mut parser = Parser::new("(4+33x".chars());
    parser.parse().unwrap();
}
