//use crate::parser::ShapeParsingError;
use crate::prelude::*;
type TablesOrError<'a> = Result<(&'a [bool; SHAPE_COMBINATIONS], [u8; 4], [u8; 4]), DealerError>;

