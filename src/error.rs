// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::{CreationShapeError, DealerError, ParseLinError};
use std::error::Error;

/// Error wrapper for the entire library, so we expose just this one at the highest level.
/// Variants are self explanatory.
#[non_exhaustive]
#[derive(Debug)]
pub enum SqueezerError {
    LinParsing(ParseLinError),
    CreationShape(CreationShapeError),
    DealingError(DealerError),
    #[cfg(feature = "dds")]
    DDSError(dds::ddserror::DdsError),
    #[cfg(feature = "dds")]
    SeqError(dds::utils::BuildSequenceError),
}

impl Error for SqueezerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SqueezerError::LinParsing(ref err) => Some(err),
            SqueezerError::CreationShape(ref err) => Some(err),
            SqueezerError::DealingError(ref err) => Some(err),
            #[cfg(feature = "dds")]
            SqueezerError::DDSError(ref err) => Some(err),
            #[cfg(feature = "dds")]
            SqueezerError::SeqError(ref err) => Some(err),
        }
    }
}

impl std::fmt::Display for SqueezerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(inner) = self.source() {
            write!(f, "squeezer encountered an error:\n\t{inner}")
        } else {
            write!(
                f,
                "squeezer encountered an error:\n\tUnknown: file a issue on github."
            )
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
