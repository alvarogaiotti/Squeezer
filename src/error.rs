// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::{CreationShapeError, DealerError, LinParsingError};
use std::error::Error;

/// Error wrapper for the entire library, so we expose just this one at the highest level.
/// Variants are self explanatory.
#[non_exhaustive]
#[derive(Debug)]
pub enum SqueezerError {
    LinParsing(LinParsingError),
    CreationShape(CreationShapeError),
    DealingError(DealerError),
    #[cfg(feature = "dds")]
    DDSError(dds::ddserror::DDSError),
    #[cfg(feature = "dds")]
    SeqError(dds::utils::SeqError),
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
        let inner = self.source().unwrap();
        write!(f, "squeezer encountered an error:\n\t{inner}")
    }
}

impl From<DealerError> for SqueezerError {
    fn from(value: DealerError) -> Self {
        Self::DealingError(value)
    }
}

impl From<LinParsingError> for SqueezerError {
    fn from(value: LinParsingError) -> Self {
        Self::LinParsing(value)
    }
}

#[cfg(feature = "dds")]
impl From<dds::ddserror::DDSError> for SqueezerError {
    fn from(value: dds::ddserror::DDSError) -> Self {
        Self::DDSError(value)
    }
}

#[cfg(feature = "dds")]
impl From<dds::utils::SeqError> for SqueezerError {
    fn from(value: dds::utils::SeqError) -> Self {
        Self::SeqError(value)
    }
}
