use crate::prelude::{CreationShapeError, DealerError, LinParsingError};
use std::error::Error;

/// Error wrapper for the entire library, so we expose just this one at the highest level
#[non_exhaustive]
#[derive(Debug)]
pub enum SqueezerError {
    LinParsing(LinParsingError),
    CreationShape(CreationShapeError),
    DealingError(DealerError),
}

impl Error for SqueezerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SqueezerError::LinParsing(ref err) => Some(err),
            SqueezerError::CreationShape(ref err) => Some(err),
            SqueezerError::DealingError(ref err) => Some(err),
        }
    }
}

impl std::fmt::Display for SqueezerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.source().unwrap();
        write!(f, "squeezer encountered an error:\n\t{}", inner)
    }
}
