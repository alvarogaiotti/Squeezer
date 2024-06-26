use crate::{prelude::*, shapeparser::CreationShapeError};

///Error for wrong Shape pattern passed to `ShapeFactory`.
#[derive(Debug)]
pub struct DealerError {
    details: String,
}

impl DealerError {
    #[must_use]
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_owned(),
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