use std::cmp::Ordering;

use crate::shapeparser::*;

use super::{interpreter::CreationShapeError, scanner::Scanner};

#[derive(Debug)]
pub(super) struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

// Rough grammar rules:
//
// primary      -> NUMBER | "x"
// unary        -> primary("+" | "-")
// group        -> "(" unary (unary)+ ")"
// pattern      -> group | unary
// shape        -> pattern+
//
// The more or less correct grammar:
//
// <length> ::= [0-9]
// <unit> ::= <length> ( "+" | "-" )?
//          | "x"
// <group> ::= "(" <unit> <unit>+ ")"
// <shape> ::= <unit>* <group>* <unit>*

impl Parser {
    pub fn parse_pattern(pattern: &str) -> Result<Vec<Pattern>, CreationShapeError> {
        let scanner = Scanner::from(pattern);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Self::from(tokens);
        parser.parse().map_err(Into::into)
    }

    /// Guess what?! Parses!
    pub fn parse(&mut self) -> Result<Vec<Pattern>, ParsingShapeError> {
        let mut patterns = Vec::new();
        while !self.is_at_end() {
            patterns.push(self.group()?);
        }
        let pattern_length = patterns.iter().fold(0, |acc, pattern| acc + pattern.len());
        match pattern_length.cmp(&4) {
            Ordering::Less => Err(ParsingShapeError::ShapeTooShort),
            Ordering::Greater => Err(ParsingShapeError::ShapeTooLong),
            Ordering::Equal => Ok(patterns),
        }
    }

    /// Creates a new Parser
    pub fn from(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Advances the cursor and returns the previous token
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Returns whether we are at the end of the stream
    fn is_at_end(&self) -> bool {
        self.peek() == Token::Empty
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

    /// Parses group patterns like 3(532)
    fn group(&mut self) -> Result<Pattern, ParsingShapeError> {
        if self.check(Token::OpenParen) {
            self.advance();
            let mut group = Vec::with_capacity(4);
            loop {
                match self.peek() {
                    Token::CloseParen => {
                        self.advance();
                        if group.len() >= 2 {
                            return Ok(Pattern::Group(group));
                        } else {
                            return Err(ParsingShapeError::MalformedGroup);
                        }
                    }
                    Token::Empty => return Err(ParsingShapeError::UnmatchParenthesis),
                    _ => match self.suit() {
                        Ok(Pattern::Suit(length)) => {
                            group.push(length);
                        }
                        Err(error) => {
                            return Err(error);
                        }
                        _ => unreachable!("Parsed a Pattern::Group from the suit function!"),
                    },
                }
            }
        }
        self.suit()
    }

    /// Parses suit patterns
    fn suit(&mut self) -> Result<Pattern, ParsingShapeError> {
        match self.peek() {
            Token::Joker => {
                self.advance();
                Ok(Pattern::Suit(Length {
                    length: 0,
                    modifier: Modifier::AtLeast,
                }))
            }
            Token::Length(length) => {
                self.advance();
                if let Token::Modifier(modifier) = self.peek() {
                    self.advance();
                    Ok(Pattern::Suit(Length { length, modifier }))
                } else {
                    Ok(Pattern::Suit(Length {
                        length,
                        modifier: Modifier::Exact,
                    }))
                }
            }
            Token::OpenParen => Err(ParsingShapeError::NestedScope),
            Token::CloseParen => Err(ParsingShapeError::UnmatchParenthesis),
            Token::Modifier(modifier) => Err(ParsingShapeError::OrphanModifier(modifier)),
            Token::Empty => {
                unreachable!("Asked to parse an empty token, which should have been checked before")
            }
        }
    }
}

#[derive(Debug)]
pub enum ParsingShapeError {
    UnmatchParenthesis,
    OrphanModifier(Modifier),
    ShapeTooLong,
    ShapeTooShort,
    NestedScope,
    MalformedGroup,
}

impl std::fmt::Display for ParsingShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                ParsingShapeError::UnmatchParenthesis => String::from("non matching parentheses"),
                ParsingShapeError::OrphanModifier(modifier) =>
                    format!("orphan modifier: {}", modifier),
                ParsingShapeError::ShapeTooLong => String::from("shape has more than 13 cards"),
                ParsingShapeError::ShapeTooShort => String::from("shape has less than 13 cards"),
                ParsingShapeError::NestedScope => String::from("nested grouping not supported"),
                ParsingShapeError::MalformedGroup =>
                    String::from("group must contain at least two element"),
            }
        )
    }
}

impl std::error::Error for ParsingShapeError {}

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
                Modifier::AtLeast => "AtLeast: +".to_string(),
                Modifier::AtMost => "AtMost: -".to_string(),
                Modifier::Exact => "Exact".to_string(),
            }
        )
    }
}
