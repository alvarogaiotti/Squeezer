use std::{future, iter::Peekable};

use crate::prelude::*;

/// Rough grammar rules:
/// shape       -> suit* group* suit* {4};
/// pattern     -> suit | group[n];
/// suit        -> length modifier?;
/// length      -> "0".."C";
/// modifier    -> "+" | "-";
/// group[n]    -> "(" | suit suit+ | ")" {4};

type Patterns = [Length; 4];

#[derive(Debug)]
pub struct Shape {
    patterns: Patterns,
}

enum Pattern {
    Suit(Length),
    Group(Vec<Length>),
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

pub struct Parse {
    tokens: Vec<Token>,
    current: usize,
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

impl Parse {
    pub fn parse(mut self) -> Shape {
        self.shape().unwrap()
    }
    /// Creates a new Parser
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Advances the cursor and returns the previous token
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Are we at the end of the Token stream?
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Empty)
    }

    /// Returns the next token without advancing the cursor
    fn peek(&self) -> Token {
        // SAFETY: we know the sequence is not ended
        *self.tokens.get(self.current).unwrap()
    }

    /// Returns the previous Token
    fn previous(&self) -> Token {
        // SAFETY: we know the sequence is not ended
        *self.tokens.get(self.current - 1).unwrap()
    }

    /// Checks if the Token provided matches the next Token
    fn check(&self, token: Token) -> bool {
        if self.is_at_end() {
            return false;
        };
        std::mem::discriminant(&self.peek()) == std::mem::discriminant(&token)
    }

    /// Checks if the Token provided matches the next Token and advances the cursor
    fn is_same(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            return true;
        }
        false
    }

    /// Parses the final Shape
    fn shape(&mut self) -> Result<Shape, ShapeParsingError> {
        let pattern = self.pattern()?;

        Ok(pattern.into())
    }

    fn pattern(&mut self) -> Result<Vec<Length>, ShapeParsingError> {
        let mut suits = Vec::with_capacity(4);
        while let Some(group) = self.group()? {
            suits.extend(group)
        }
        Ok(suits)
    }

    fn group(&mut self) -> Result<Pattern, ShapeParsingError> {
        if self.is_same(Token::OpenParen) {
            let mut group = Vec::with_capacity(4);
            while let Some(suit) = self.suit() {
                group.push(suit)
            }
            if !self.is_same(Token::CloseParen) {
                return Err(ShapeParsingError::UnmatchParenthesis);
            }
            return Ok(Pattern::Group(group));
        }
        self.suit()
    }

    fn suit(&mut self) -> Pattern {
        self.modifier().map(|x| Pattern::Suit(x))
    }

    fn modifier(&mut self) -> Option<Length> {
        if let Some(mut length) = self.length() {
            while self.is_same(Token::Modifier(Modifier::Exact)) {
                match self.previous() {
                    Token::Modifier(modifier) => {
                        length.modifier = modifier;
                    }
                    _ => unreachable!(),
                }
            }
            Some(length)
        } else {
            None
        }
    }

    fn length(&mut self) -> Option<Length> {
        if self.is_same(Token::Length(Default::default())) {
            Some(Length {
                length: self.previous().as_int(),
                modifier: Modifier::Exact,
            })
        } else {
            None
        }
    }
}

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    cursor: usize,
}

impl Scanner {
    pub fn new(string: &str) -> Self {
        Self {
            source: string.chars().collect(),
            tokens: Vec::new(),
            cursor: 0,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, ShapeParsingError> {
        while !self.is_at_end() {
            self.scan_token()?;
        }
        self.tokens.push(Token::Empty);
        Ok(self.tokens)
    }

    pub fn is_at_end(&self) -> bool {
        self.cursor >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), ShapeParsingError> {
        let c = self.advance();

        match c {
            '(' => self.add_token(Token::OpenParen),
            ')' => self.add_token(Token::CloseParen),
            '+' => self.add_token(Token::Modifier(Modifier::AtLeast)),
            '-' => self.add_token(Token::Modifier(Modifier::AtMost)),
            'x' => self.add_token(Token::Joker),
            length if length.is_ascii_hexdigit() => {
                // SAFETY: Bounds already checked
                self.add_token(Token::Length(length.to_digit(16).unwrap() as u8))
            }

            _ => return Err(ShapeParsingError::UnknownChar(c)),
        }

        Ok(())
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn advance(&mut self) -> char {
        self.cursor += 1;

        // SAFETY: The function is called only when we are sure that we ar not at the end of the
        // stream
        *self.source.get(self.cursor - 1).unwrap()
    }

    /// Returns the cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns next char without advancing the cursor
    pub fn peek(&self) -> Option<&char> {
        self.source.get(self.cursor)
    }

    /// Returns whether the string is exhausted or not
    pub fn exhausted(&self) -> bool {
        self.cursor == self.source.len()
    }

    /// Returns next character, if available, advancing the cursor
    pub fn pop(&mut self) -> Option<&char> {
        match self.source.get(self.cursor) {
            Some(character) => {
                self.cursor += 1;

                Some(character)
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub enum ShapeParsingError {
    UnmatchParenthesis,
    UnknownChar(char),
    OrphanModifier(char),
    ShapeTooLong,
    ShapeTooShort,
    NestedScope,
}

impl std::fmt::Display for ShapeParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                ShapeParsingError::UnmatchParenthesis => String::from("non matching parentheses"),
                ShapeParsingError::UnknownChar(char) => format!("unknown char {}", char),
                ShapeParsingError::OrphanModifier(char) => format!("orphan modifier: {}", char),
                ShapeParsingError::ShapeTooLong => String::from("shape has more than 13 cards"),
                ShapeParsingError::ShapeTooShort => String::from("shape has less than 13 cards"),
                ShapeParsingError::NestedScope => String::from("nested grouping not supported"),
            }
        )
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum Modifier {
    AtLeast,
    AtMost,
    Exact,
}

impl std::fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Modifier::AtLeast => "AtLeast".to_string(),
                Modifier::AtMost => "AtMost".to_string(),
                Modifier::Exact => "Exact".to_string(),
            }
        )
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
                        Token::Modifier(Modifier::AtLeast) => Some(Length {
                            length,
                            modifier: Modifier::AtLeast,
                        }),
                        Token::Modifier(Modifier::AtMost) => Some(Length {
                            length,
                            modifier: Modifier::AtMost,
                        }),
                        _ => Some(Length {
                            length,
                            modifier: Modifier::Exact,
                        }),
                    },
                    Token::Joker => Some(Length {
                        length: 0,
                        modifier: Modifier::AtLeast,
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
                    Token::Length(_) => Token::Modifier(Modifier::AtLeast), // In theory not possible as we have peeked to check
                    _ => return Err(DealerError::from(ShapeParsingError::OrphanModifier(token))),
                },
                '-' => match self.previous_token {
                    Token::Length(_) => Token::Modifier(Modifier::AtMost), // In theory not possible as we have peeked to check
                    _ => return Err(DealerError::from(ShapeParsingError::OrphanModifier(token))),
                },
                'x' => Token::Joker,
                '(' => {
                    if self.scope == Scope::Nested {
                        return Err(DealerError::from(ShapeParsingError::NestedScope));
                    }
                    self.scope = Scope::Nested;
                    Token::OpenParen
                }
                ')' => {
                    if self.scope == Scope::Linear {
                        return Err(DealerError::from(ShapeParsingError::UnmatchParenthesis));
                    }
                    self.scope = Scope::Linear;
                    Token::CloseParen
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
#[test]
fn new_parser_test() {
    let scanner = Scanner::new("(4+33)3-");
    let parser = Parse::new(scanner.scan_tokens().unwrap());
    let result = parser.parse();
    println!("{result:?}");
}
