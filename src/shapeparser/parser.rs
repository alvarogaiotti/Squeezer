use crate::shapeparser::*;
#[derive(Debug)]
enum ParserState {
    Grouping,
    Linear,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    state: ParserState,
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
    /// Guess what?! Parses!
    pub fn parse(&mut self) -> Result<Vec<Pattern>, ShapeParsingError> {
        let mut patterns = Vec::new();
        while !self.is_at_end() {
            patterns.push(self.group()?);
        }
        Ok(patterns)
    }
    /// Creates a new Parser
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            state: ParserState::Linear,
        }
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
    fn group(&mut self) -> Result<Pattern, ShapeParsingError> {
        if self.check(Token::OpenParen) {
            self.advance();
            let mut group = Vec::with_capacity(4);
            loop {
                match self.peek() {
                    Token::CloseParen => {
                        self.advance();
                        return Ok(Pattern::Group(group));
                    }
                    Token::Empty => return Err(ShapeParsingError::UnmatchParenthesis),
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
    fn suit(&mut self) -> Result<Pattern, ShapeParsingError> {
        println!("In suit, next token:\n{}", self.peek());
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
                    println!(
                        "In suit, emitting:\n{}",
                        Pattern::Suit(Length { length, modifier })
                    );
                    Ok(Pattern::Suit(Length { length, modifier }))
                } else {
                    println!(
                        "In suit, emitting:\n{}",
                        Pattern::Suit(Length {
                            length,
                            modifier: Modifier::Exact
                        })
                    );
                    Ok(Pattern::Suit(Length {
                        length,
                        modifier: Modifier::Exact,
                    }))
                }
            }
            Token::OpenParen => Err(ShapeParsingError::NestedScope),
            Token::CloseParen => Err(ShapeParsingError::UnmatchParenthesis),
            Token::Modifier(modifier) => Err(ShapeParsingError::OrphanModifier(modifier)),
            Token::Empty => Err(ShapeParsingError::ShapeTooShort),
        }
    }
}

#[derive(Debug)]
pub enum ShapeParsingError {
    UnmatchParenthesis,
    UnknownChar(char),
    OrphanModifier(Modifier),
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
                ShapeParsingError::OrphanModifier(modifier) =>
                    format!("orphan modifier: {}", modifier),
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
                Modifier::AtLeast => "AtLeast: +".to_string(),
                Modifier::AtMost => "AtMost: -".to_string(),
                Modifier::Exact => "Exact".to_string(),
            }
        )
    }
}
