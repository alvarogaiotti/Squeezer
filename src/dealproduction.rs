// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{prelude::*, shapeparser::CreationShapeError};

///Error for wrong Shape pattern passed to `ShapeFactory`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash)]
pub struct DealerError {
    details: String,
}

impl DealerError {
    #[must_use]
    #[inline]
    pub(crate) fn new<T: Into<String>>(msg: T) -> Self {
        Self {
            details: msg.into(),
        }
    }
}

impl fmt::Display for DealerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for DealerError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<CreationShapeError> for DealerError {
    fn from(value: CreationShapeError) -> Self {
        DealerError {
            details: value.to_string(),
        }
    }
}
