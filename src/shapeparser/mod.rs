use crate::prelude::*;
mod interpreter;
mod parser;
mod scanner;

pub use parser::*;

type Patterns = [Length; 4];

#[derive(Debug)]
pub struct Shape {
    patterns: Patterns,
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Suit(len) => write!(f, "Pattern::Suit, with len:\n{}", len),
            Self::Group(vec) => write!(f, "Pattern::Group, with vec:\n{:#?}", vec),
        }
    }
}

trait Flattable {
    fn flat(self) -> Vec<Length>;
}

impl Flattable for Pattern {
    fn flat(self) -> Vec<Length> {
        match self {
            Pattern::Suit(len) => vec![len],
            Pattern::Group(lenghts) => lenghts.into_iter().collect(),
        }
    }
}

macro_rules! token_as_int {
    ($token:expr, $variant:path) => {
        match $token {
            $variant(x) => *x,
            _ => 0u8,
        }
    };
}

impl Into<Shape> for Vec<Length> {
    fn into(self) -> Shape {
        use itertools::*;
        assert_eq!(self.len(), 4);
        let mut patterns = [Length::at_least(0); 4];
        patterns.iter_mut().set_from(self);
        Shape { patterns }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum Token {
    Length(u8),
    Modifier(Modifier),
    Joker,
    OpenParen,
    CloseParen,
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
                Token::Modifier(Modifier::AtLeast) => "Token::Plus".to_string(),
                Token::Modifier(Modifier::AtMost) => "Token::Minus".to_string(),
                Token::Modifier(Modifier::Exact) => "Token::Modifier::Exact".to_string(),
                Token::Joker => "Token::Joker".to_string(),
                Token::OpenParen => "Token::OpenParens".to_string(),
                Token::CloseParen => "Token::ClosedParens".to_string(),
                Token::Empty => "Token::Empty".to_string(),
            }
        )
    }
}

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
    use super::scanner::Scanner;
    use super::Parser;
    #[test]
    fn new_parser_test() {
        let scanner = Scanner::new("(4+33)3-");
        let mut parser = Parser::new(scanner.scan_tokens().unwrap());
        println!("New parse pattern test: (4+33)3-");
        parser
            .parse()
            .unwrap()
            .into_iter()
            .for_each(|x| println!("{:#?}", x));
    }
    #[test]
    fn new_parser_pattern_test() {
        let scanner = Scanner::new("(4+3-3x)");
        let mut parser = Parser::new(scanner.scan_tokens().unwrap());
        println!("New parse pattern test (4+3-3x)");
        parser
            .parse()
            .unwrap()
            .into_iter()
            .for_each(|x| println!("{:#?}", x));
    }
    #[test]
    #[should_panic]
    fn new_parse_returns_orphan_modifier_test() {
        let scanner = Scanner::new("(+433x)");
        let mut parser = Parser::new(scanner.scan_tokens().unwrap());
        let result = parser.parse().unwrap();
    }
    #[test]
    #[should_panic]
    fn new_parse_returns_unclosed_delimiter_test() {
        let scanner = Scanner::new("4+33x)");
        let mut parser = Parser::new(scanner.scan_tokens().unwrap());
        let result = parser.parse().unwrap();
    }
    #[test]
    #[should_panic]
    fn new_parse_returns_unclosed_delimiter2_test() {
        let scanner = Scanner::new("(4+33x");
        let mut parser = Parser::new(scanner.scan_tokens().unwrap());
        let result = parser.parse().unwrap();
    }
}
