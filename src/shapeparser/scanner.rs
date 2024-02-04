use crate::shapeparser::*;
pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    cursor: usize,
}

impl Scanner {
    pub fn from(string: &str) -> Self {
        Self {
            source: string.chars().collect(),
            tokens: Vec::new(),
            cursor: 0,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, ScanningShapeError> {
        while !self.is_at_end() {
            self.scan_token()?;
        }
        self.tokens.push(Token::Empty);
        Ok(self.tokens)
    }

    pub fn is_at_end(&self) -> bool {
        self.cursor >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), ScanningShapeError> {
        let c = self.advance();

        match c {
            '(' => self.add_token(Token::OpenParen),
            ')' => self.add_token(Token::CloseParen),
            '+' => self.add_token(Token::Modifier(Modifier::AtLeast)),
            '-' => self.add_token(Token::Modifier(Modifier::AtMost)),
            'x' => self.add_token(Token::Joker),
            length if length.is_ascii_hexdigit() => {
                // SAFETY: Bounds already checked
                let length = length.to_digit(16).unwrap() as u8;
                if length <= 13 {
                    self.add_token(Token::Length(length))
                } else {
                    return Err(ScanningShapeError::SuitTooLong(length));
                }
            }

            _ => return Err(ScanningShapeError::UnknownChar(c)),
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
pub enum ScanningShapeError {
    UnknownChar(char),
    SuitTooLong(u8),
}

impl std::fmt::Display for ScanningShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                ScanningShapeError::UnknownChar(char) => format!("unknown char {}", char),
                ScanningShapeError::SuitTooLong(num) => format!("suit is too long: {}", num),
            }
        )
    }
}

impl std::error::Error for ScanningShapeError {}
