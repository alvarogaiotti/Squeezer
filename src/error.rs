// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The crate-wide error boundary.
//!
//! Every module owns a precise, public error (see `ConstraintError` for the
//! template: typed variants that preserve their cause). [`SqueezerError`]
//! wraps them all so a caller can match on the failure domain at the crate
//! boundary and drill into the module error when it needs the detail. The
//! wrapped error is the [`std::error::Error::source`] of the wrapper, so the
//! chain reaches the root cause without any variant flattening a payload.
//!
//! `DdsError` stays in the `dds` crate and is not re-exported: the solver seam
//! ([`SolverError`]) is feature-independent, so the core error never names DDS.
//! Only the [`Solver`](SqueezerError::Solver) variant's construction touches
//! `DdsError`, and only under the `dds` feature.

#[cfg(any(feature = "bbo", feature = "bbo_async"))]
use crate::bbohelpers::BboError;
use crate::dealproduction::DealerError;
use crate::hand_constraint::ConstraintError;
#[cfg(feature = "lin")]
use crate::linparser::ParseLinError;
#[cfg(feature = "dds")]
use crate::performance_analysis::{TrickDeltaError, TricksError};
use crate::record::{AuctionError, BoardError, DealError, PositionError};
use crate::shapeparser::CreationShapeError;
use crate::solver::SolverError;

/// Error wrapper for the entire library, so we expose just this one at the
/// highest level.
///
/// Variants are self-explanatory; each wraps its module's typed error as the
/// variant payload. Feature-gated variants match the availability of their
/// source, so a build only names errors it can actually produce.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SqueezerError {
    /// Caller-supplied configuration is missing or inconsistent.
    #[error("invalid input ({context}): {message}")]
    InvalidInput {
        /// Where the input was rejected, for diagnostics.
        context: &'static str,
        /// What was wrong with it.
        message: String,
    },
    /// A LIN file could not be imported.
    #[cfg(feature = "lin")]
    #[error("LIN parsing failed: {0}")]
    LinParsing(#[from] ParseLinError),
    /// A shape pattern failed to scan, parse or interpret.
    #[error("shape creation failed: {0}")]
    CreationShape(#[from] CreationShapeError),
    /// A hand constraint could not be built.
    #[error("invalid hand constraint: {0}")]
    ConstraintError(#[from] ConstraintError),
    /// The dealer or its conditioning rejected a deal.
    #[error("dealing failed: {0}")]
    DealingError(#[from] DealerError),
    /// A backend-agnostic solver failure; feature-independent, so the core
    /// error never names DDS.
    #[error("solver failed: {0}")]
    Solver(#[from] SolverError),
    /// Four hands did not form a valid deal.
    #[error("invalid deal: {0}")]
    DealError(#[from] DealError),
    /// The parts of a recorded board did not fit together.
    #[error("invalid board: {0}")]
    BoardError(#[from] BoardError),
    /// A sequence of calls was not a legal auction.
    #[error("invalid auction: {0}")]
    AuctionError(#[from] AuctionError),
    /// A position could not be derived from a board.
    #[error("invalid position: {0}")]
    PositionError(#[from] PositionError),
    /// A BBO download or login failed.
    #[cfg(any(feature = "bbo", feature = "bbo_async"))]
    #[error("BBO request failed: {0}")]
    Bbo(#[from] BboError),
    /// A trick count was outside 0-13.
    #[cfg(feature = "dds")]
    #[error("invalid trick count: {0}")]
    TricksError(#[from] TricksError),
    /// A trick delta was outside 0-13.
    #[cfg(feature = "dds")]
    #[error("invalid trick delta: {0}")]
    TrickDeltaError(#[from] TrickDeltaError),
}

impl SqueezerError {
    /// Builds an [`InvalidInput`](SqueezerError::InvalidInput) boundary error.
    ///
    /// A single constructor keeps the `context`/`message` pair consistent
    /// across the simulation builders.
    #[must_use]
    pub fn invalid_input(context: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            context,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Card;
    use crate::hand::HcpRange;
    use crate::hand_constraint::HandConstraint;
    use crate::record::auction::Call;
    use crate::record::deal::Seat;
    use crate::shapeparser::ShapeCreator;
    use std::error::Error;

    /// Builds a malformed-pattern error through the real parser, so the test
    /// exercises the same cause a caller would see.
    fn creation_shape_error() -> CreationShapeError {
        ShapeCreator::build_shape("not a shape!!").expect_err("pattern is malformed")
    }

    #[test]
    fn invalid_input_display_includes_context_and_message() {
        let error = SqueezerError::invalid_input("payoff builder", "num_deals must be set");
        let text = error.to_string();
        assert!(text.contains("payoff builder"), "got {text}");
        assert!(text.contains("num_deals must be set"), "got {text}");
        assert!(error.source().is_none(), "invalid input has no source");
    }

    #[test]
    fn wrapped_errors_expose_the_module_error_as_source() {
        let error = SqueezerError::from(DealerError::Unsatisfiable { attempts: 7 });
        assert!(matches!(error, SqueezerError::DealingError(_)));
        let source = error.source().expect("wrapped errors carry a source");
        assert!(
            source.downcast_ref::<DealerError>().is_some(),
            "source should be the module error itself"
        );
    }

    #[test]
    fn display_includes_the_wrapped_payload() {
        let error = SqueezerError::from(DealerError::Unsatisfiable { attempts: 42 });
        assert!(error.to_string().contains("42"), "got {error}");
    }

    #[test]
    fn constraint_source_chain_reaches_the_parser_cause() {
        let constraint = HandConstraint::new(&["not a shape!!"], HcpRange::new(0, 37))
            .expect_err("pattern is malformed");
        let error = SqueezerError::from(constraint);
        assert!(matches!(error, SqueezerError::ConstraintError(_)));

        let module = error.source().expect("wrapped constraint");
        assert!(module.downcast_ref::<ConstraintError>().is_some());
        let cause = module.source().expect("constraint preserves its cause");
        assert!(cause.downcast_ref::<CreationShapeError>().is_some());
    }

    #[test]
    fn conditioning_unsatisfiable_flows_through_the_boundary() {
        let error = SqueezerError::from(DealerError::Unsatisfiable { attempts: 10 });
        match error {
            SqueezerError::DealingError(DealerError::Unsatisfiable { attempts }) => {
                assert_eq!(attempts, 10);
            }
            other => panic!("expected a dealer unsatisfiable, got {other:?}"),
        }
    }

    /// Asserts a converted error matched the expected variant and still
    /// exposes its module error as the source.
    fn assert_wraps(error: &SqueezerError, is_variant: fn(&SqueezerError) -> bool) {
        assert!(is_variant(error), "wrong variant: {error:?}");
        assert!(error.source().is_some(), "missing source: {error:?}");
    }

    #[test]
    fn every_core_module_error_converts_to_its_variant() {
        assert_wraps(&SqueezerError::from(creation_shape_error()), |e| {
            matches!(e, SqueezerError::CreationShape(_))
        });
        assert_wraps(
            &SqueezerError::from(
                HandConstraint::new(&["not a shape!!"], HcpRange::new(0, 37))
                    .expect_err("pattern is malformed"),
            ),
            |e| matches!(e, SqueezerError::ConstraintError(_)),
        );
        assert_wraps(&SqueezerError::from(DealerError::new("dealer")), |e| {
            matches!(e, SqueezerError::DealingError(_))
        });
        assert_wraps(&SqueezerError::from(SolverError::NoCards), |e| {
            matches!(e, SqueezerError::Solver(_))
        });
        assert_wraps(
            &SqueezerError::from(DealError::DuplicatedCard(Card::S2)),
            |e| matches!(e, SqueezerError::DealError(_)),
        );
        assert_wraps(
            &SqueezerError::from(BoardError::DuplicatePlay(Card::S2)),
            |e| matches!(e, SqueezerError::BoardError(_)),
        );
        assert_wraps(
            &SqueezerError::from(AuctionError::IllegalDouble {
                call: Call::Pass,
                seat: Seat::North,
            }),
            |e| matches!(e, SqueezerError::AuctionError(_)),
        );
        assert_wraps(&SqueezerError::from(PositionError::MissingContract), |e| {
            matches!(e, SqueezerError::PositionError(_))
        });
    }

    #[test]
    fn shape_error_stays_typed_through_the_dealer() {
        let dealer = DealerError::from(creation_shape_error());
        assert!(matches!(dealer, DealerError::Shape(_)));
        assert!(
            dealer.source().is_some(),
            "the dealer error should keep the parser cause"
        );

        let boundary = SqueezerError::from(dealer);
        assert!(matches!(boundary, SqueezerError::DealingError(_)));
        let module = boundary.source().expect("wrapped dealer error");
        assert!(module.downcast_ref::<DealerError>().is_some());
        let cause = module.source().expect("dealer keeps the parser cause");
        assert!(cause.downcast_ref::<CreationShapeError>().is_some());
    }

    #[test]
    fn hand_type_builder_remove_shape_preserves_the_parser_cause() {
        use crate::hand::HandType;
        let mut builder = HandType::builder();
        let error: CreationShapeError = builder
            .remove_shape("not a shape!!")
            .expect_err("pattern is malformed");
        assert!(error.source().is_some(), "parser cause survives");
    }

    #[cfg(feature = "lin")]
    #[test]
    fn lin_error_converts_to_its_variant() {
        use crate::linparser::ParseLinErrorKind;
        let error = SqueezerError::from(ParseLinError::new(
            ParseLinErrorKind::Syntax,
            "garbage".to_owned(),
        ));
        assert!(matches!(error, SqueezerError::LinParsing(_)));
        assert!(error.source().is_some());
    }

    #[cfg(any(feature = "bbo", feature = "bbo_async"))]
    #[test]
    fn bbo_error_converts_to_its_variant() {
        let error = SqueezerError::from(BboError::Login);
        assert!(matches!(error, SqueezerError::Bbo(_)));
        assert!(error.source().is_some());
    }

    #[cfg(feature = "dds")]
    #[test]
    fn trick_errors_convert_to_their_variants() {
        let tricks = SqueezerError::from(TricksError::OutOfRange(14));
        assert!(matches!(tricks, SqueezerError::TricksError(_)));
        assert!(tricks.source().is_some());

        let delta = SqueezerError::from(TrickDeltaError::OutOfRange(14));
        assert!(matches!(delta, SqueezerError::TrickDeltaError(_)));
        assert!(delta.source().is_some());
    }
}
