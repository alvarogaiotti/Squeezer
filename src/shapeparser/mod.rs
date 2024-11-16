// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;
mod interpreter;
mod parser;
mod scanner;
pub use interpreter::*;
use parser::{Modifier, ParsingShapeError};

/// A pattern is a shape pattern, which is formed by 4 Lenght tokens.
/// It represent a set of possible shapes.
type Patterns = [Length; 4];

/// Pattern enum to represent different shape patterns.
///
/// - `Suit(Length)`: Represents a single length suit pattern.
/// - `Group(Vec<Length>)`: Represents a grouped pattern with multiple lengths. Grouped patterns are series of Suit patterns enclosed by a pair of round parenthesis.
#[derive(Debug)]
pub(crate) enum Pattern {
    Suit(Length),
    Group(Vec<Length>),
}

impl Pattern {
    /// Checks if the pattern contains a certain length.
    #[inline]
    pub fn contains(&self, num: u8) -> bool {
        match self {
            Self::Suit(Length {
                length,
                modifier: Modifier::Exact,
            }) => *length == num,
            Self::Suit(Length {
                length,
                modifier: Modifier::AtLeast,
            }) => num >= *length,
            Self::Suit(Length {
                length,
                modifier: Modifier::AtMost,
            }) => num <= *length,
            Self::Group(_) => false,
        }
    }

    /// Returns the length of the pattern, not of the contained variant.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Suit(_) => 1,
            Self::Group(patterns) => patterns.len(),
        }
    }
}

#[derive(Debug)]
struct PatternsAsShape {
    patterns: Patterns,
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Suit(len) => write!(f, "Pattern::Suit, with len:\n{len}"),
            Self::Group(vec) => write!(f, "Pattern::Group, with vec:\n{vec:#?}"),
        }
    }
}

trait Groupable {
    fn group(self) -> Vec<Length>;
}

impl Groupable for Pattern {
    fn group(self) -> Vec<Length> {
        match self {
            Pattern::Suit(len) => vec![len],
            Pattern::Group(lenghts) => lenghts,
        }
    }
}

macro_rules! token_as_int {
    ($token:expr, $variant:path) => {
        match $token {
            $variant(x) => x,
            _ => 0u8,
        }
    };
}

impl From<Vec<Length>> for PatternsAsShape {
    fn from(value: Vec<Length>) -> Self {
        use itertools::Itertools;
        assert_eq!(value.len(), 4);
        let mut patterns = [Length::at_least(0); 4];
        patterns.iter_mut().set_from(value);
        PatternsAsShape { patterns }
    }
}

/// Enum to represent different tokens in the shape definition string.
///
/// - `Length(u8)`: Represents a numerical length token.
/// - `Modifier(Modifier)`: Represents a modifier token.
/// - `Joker`: Represents a wildcard token.
/// - `OpenParen`: Represents an opening parenthesis token.
/// - `CloseParen`: Represents a closing parenthesis token.
/// - `Empty`: Represents an empty token.
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
enum Token {
    Length(u8),
    Modifier(Modifier),
    Joker,
    OpenParen,
    CloseParen,
    Empty,
}

impl Token {
    fn is_len_token(self) -> bool {
        matches!(self, Self::Length(_))
    }
    fn as_int(self) -> u8 {
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
                Token::Length(num) => format!("Token::Length({num})"),
                Token::Modifier(modifier) => format!("Token::Modifier({modifier})"),
                Token::Joker => "Token::Joker".to_owned(),
                Token::OpenParen => "Token::OpenParens".to_owned(),
                Token::CloseParen => "Token::ClosedParens".to_owned(),
                Token::Empty => "Token::Empty".to_owned(),
            }
        )
    }
}

/// Struct representing a Length with a numerical value and a Modifier.
#[derive(Debug, Copy, Clone)]
pub struct Length {
    length: u8,
    modifier: Modifier,
}

impl std::fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " length: {}\n modifier: {} ", self.length, self.modifier)
    }
}

impl Length {
    fn at_least(length: u8) -> Self {
        Self {
            length,
            modifier: Modifier::AtLeast,
        }
    }
    fn at_most(length: u8) -> Self {
        Self {
            length,
            modifier: Modifier::AtMost,
        }
    }
    fn exact(length: u8) -> Self {
        Self {
            length,
            modifier: Modifier::Exact,
        }
    }
    const AT_LEAST_0: Length = Self {
        length: 0,
        modifier: Modifier::AtLeast,
    };
}

#[cfg(test)]
mod test {
    use super::parser::Parser;
    use super::scanner::Scanner;
    #[test]
    fn new_parser_test() {
        let scanner = Scanner::from("(4+33)3-");
        let mut parser = Parser::from(scanner.scan_tokens().unwrap());
        println!("New parse pattern test: (4+33)3-");
        parser
            .parse()
            .unwrap()
            .into_iter()
            .for_each(|x| println!("{x:#?}"));
    }
    #[test]
    fn new_parser_pattern_test() {
        let scanner = Scanner::from("(4+3-3x)");
        let mut parser = Parser::from(scanner.scan_tokens().unwrap());
        println!("New parse pattern test (4+3-3x)");
        parser
            .parse()
            .unwrap()
            .into_iter()
            .for_each(|x| println!("{x:#?}"));
    }
    #[test]
    #[should_panic]
    fn new_parse_returns_orphan_modifier_test() {
        let scanner = Scanner::from("(+433x)");
        let mut parser = Parser::from(scanner.scan_tokens().unwrap());
        parser.parse().unwrap();
    }
    #[test]
    #[should_panic]
    fn new_parse_returns_unclosed_delimiter_test() {
        let scanner = Scanner::from("4+33x)");
        let mut parser = Parser::from(scanner.scan_tokens().unwrap());
        parser.parse().unwrap();
    }
    #[test]
    #[should_panic]
    fn new_parse_returns_unclosed_delimiter2_test() {
        let scanner = Scanner::from("(4+33x");
        let mut parser = Parser::from(scanner.scan_tokens().unwrap());
        parser.parse().unwrap();
    }
}
