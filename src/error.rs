// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    linparser::ParseLinError,
    prelude::{CreationShapeError, DealerError},
};
use std::error::Error;

/// Error wrapper for the entire library, so we expose just this one at the highest level.
/// Variants are self explanatory.
#[non_exhaustive]
#[derive(Debug)]
pub enum SqueezerError {
    InvalidInput(String),
    LinParsing(ParseLinError),
    CreationShape(CreationShapeError),
    DealingError(DealerError),
    #[cfg(feature = "dds")]
    DDSError(dds::ddserror::DdsError),
    #[cfg(feature = "dds")]
    SeqError(dds::utils::BuildSequenceError),
    Generic(String),
}

impl Error for SqueezerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SqueezerError::LinParsing(err) => Some(err),
            SqueezerError::CreationShape(err) => Some(err),
            SqueezerError::DealingError(err) => Some(err),
            #[cfg(feature = "dds")]
            SqueezerError::DDSError(err) => Some(err),
            #[cfg(feature = "dds")]
            SqueezerError::SeqError(err) => Some(err),
            SqueezerError::Generic(_) => None,
            SqueezerError::InvalidInput(_) => None,
        }
    }
}

impl std::fmt::Display for SqueezerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(inner) = self.source() {
            write!(f, "squeezer encountered an error:\n\t{inner}")
        } else {
            match self {
                SqueezerError::Generic(err) => write!(f, "squeezer encountered an error:\n\t{err}"),
                SqueezerError::InvalidInput(_) => {
                    write!(f, "squeezer encountered an error:\n\tInvalid input")
                }
                _ => unreachable!(
                    "only SqueezerError::Generic and InvalidInput should return None from the source method"
                ),
            }
        }
    }
}

impl From<DealerError> for SqueezerError {
    fn from(value: DealerError) -> Self {
        Self::DealingError(value)
    }
}

impl From<ParseLinError> for SqueezerError {
    fn from(value: ParseLinError) -> Self {
        Self::LinParsing(value)
    }
}

#[cfg(feature = "dds")]
impl From<dds::ddserror::DdsError> for SqueezerError {
    fn from(value: dds::ddserror::DdsError) -> Self {
        Self::DDSError(value)
    }
}

#[cfg(feature = "dds")]
impl From<dds::utils::BuildSequenceError> for SqueezerError {
    fn from(value: dds::utils::BuildSequenceError) -> Self {
        Self::SeqError(value)
    }
}
