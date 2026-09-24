// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{prelude::*, shapeparser::CreationShapeError};

/// Error for anything the dealer or its conditioning can reject.
///
/// The historical free-form messages live in [`Self::Message`]. The
/// conditioning failures are typed, so a caller can tell an impossible
/// conditioning seed (a card fixed to two seats) from a rejection budget that
/// merely ran out ([`Self::Unsatisfiable`]). The shape and constraint variants
/// carry their module error as the [`std::error::Error::source`] rather than
/// flattening it into a string.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash)]
#[non_exhaustive]
pub enum DealerError {
    /// A free-form dealer message.
    Message(String),
    /// Conditioning fixed one card to two different seats.
    CardAttributedToTwoSeats {
        /// The offending card.
        card: Card,
        /// The seat seen first.
        first: Seat,
        /// The seat seen second.
        second: Seat,
    },
    /// Conditioning fixed more than 13 cards to one seat.
    ///
    /// This is the only size bound that needs its own variant: a hand holds
    /// at most 13 cards, so a per-suit overflow (`> 13` of one suit) and a
    /// deck overflow (`> 52` across seats) are already impossible once this
    /// bound and [`Self::CardAttributedToTwoSeats`] hold.
    TooManyKnownForSeat {
        /// The offending seat.
        seat: Seat,
        /// How many cards were fixed to it.
        count: u8,
    },
    /// Rejection sampling exhausted its bounded attempt budget.
    ///
    /// This is not a proof that no deal satisfies the constraints, only that
    /// the configured budget was too small to find one.
    Unsatisfiable {
        /// How many candidate deals were rejected before giving up.
        attempts: u32,
    },
    /// A shape pattern failed to scan, parse or interpret.
    Shape(CreationShapeError),
    /// A hand constraint could not be built.
    Constraint(ConstraintError),
}

impl DealerError {
    #[must_use]
    #[inline]
    pub(crate) fn new<T: Into<String>>(msg: T) -> Self {
        Self::Message(msg.into())
    }
}

impl fmt::Display for DealerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(details) => write!(f, "{details}"),
            Self::CardAttributedToTwoSeats {
                card,
                first,
                second,
            } => write!(f, "card {card} is attributed to both {first} and {second}"),
            Self::TooManyKnownForSeat { seat, count } => {
                write!(
                    f,
                    "{seat} has {count} known cards, but a hand holds at most 13"
                )
            }
            Self::Unsatisfiable { attempts } => {
                write!(
                    f,
                    "no deal satisfied the constraints within {attempts} attempts"
                )
            }
            Self::Shape(cause) => write!(f, "{cause}"),
            Self::Constraint(cause) => write!(f, "{cause}"),
        }
    }
}

impl Error for DealerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Shape(cause) => Some(cause),
            Self::Constraint(cause) => Some(cause),
            Self::Message(_)
            | Self::CardAttributedToTwoSeats { .. }
            | Self::TooManyKnownForSeat { .. }
            | Self::Unsatisfiable { .. } => None,
        }
    }
}

impl From<CreationShapeError> for DealerError {
    fn from(value: CreationShapeError) -> Self {
        DealerError::Shape(value)
    }
}

impl From<ConstraintError> for DealerError {
    fn from(value: ConstraintError) -> Self {
        DealerError::Constraint(value)
    }
}
