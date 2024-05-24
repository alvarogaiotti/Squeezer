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

pub struct StringShapePattern {
    pattern: String,
}

impl StringShapePattern {
    #[must_use]
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_owned(),
        }
    }
}

pub enum ShapeDescriptor {
    SingleShape { shape_pattern: StringShapePattern }, // TODO: Make this a Vec<u8> already
    ClassOfShapes { shape_pattern: StringShapePattern },
}

impl ShapeDescriptor {
    #[must_use]
    pub fn from_string(pattern: &str) -> Self {
        if pattern.contains('(') {
            Self::ClassOfShapes {
                shape_pattern: StringShapePattern {
                    pattern: pattern.to_owned(),
                },
            }
        } else {
            Self::SingleShape {
                shape_pattern: StringShapePattern {
                    pattern: pattern.to_owned()
                },
            }
        }
    }

    #[must_use]
    pub fn new(pattern: &str) -> Self {
        if pattern.contains('(') {
            Self::ClassOfShapes {
                shape_pattern: StringShapePattern {
                    pattern: pattern.to_owned(),
                },
            }
        } else {
            Self::SingleShape {
                shape_pattern: StringShapePattern {
                    pattern: pattern.to_owned(),
                },
            }
        }
    }
}
