// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! The errors the simulation and analysis layers own.
//!
//! These replace the dissolved root crate's crate-wide boundary error for the
//! moved code. A simulation error names what a Monte Carlo run can fail on; an
//! analysis error carries the analyser's own error type as a parameter, so the
//! seam stays generic over backends.

use squeezer_model::{DealerError, PositionError};
use squeezer_solver::SolverError;
use std::fmt;

/// Why a simulation could not be configured or run.
///
/// The variants preserve their cause: [`std::error::Error::source`] returns the
/// position, dealer or solver error, so a caller can drill into the layer that
/// failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum SimulationError {
    /// A builder field was missing or inconsistent before any work started.
    InvalidInput {
        /// Where the input was rejected, for diagnostics.
        context: &'static str,
        /// What was wrong with it.
        message: String,
    },
    /// A sampled deal could not be turned into the position to solve.
    Position {
        /// Index of the deal in `0..num_deals`, so the caller knows which
        /// sampled deal the play trace disagrees with.
        deal_index: usize,
        /// Why the deal and prefix do not form a position.
        source: PositionError,
    },
    /// The dealer could not produce a deal satisfying the constraints.
    Dealing(DealerError),
    /// The solver backed rejected a position or failed to solve it.
    Solver(SolverError),
}

impl SimulationError {
    /// Builds an [`InvalidInput`](SimulationError::InvalidInput) error.
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

    /// Builds a [`Position`](SimulationError::Position) error for a sampled
    /// deal whose derived position is inconsistent with its play prefix.
    #[must_use]
    pub fn position(deal_index: usize, source: PositionError) -> Self {
        Self::Position { deal_index, source }
    }
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { context, message } => {
                write!(f, "invalid input ({context}): {message}")
            }
            Self::Position { deal_index, source } => {
                write!(
                    f,
                    "cannot derive a position for deal {deal_index}: {source}"
                )
            }
            Self::Dealing(error) => write!(f, "dealing failed: {error}"),
            Self::Solver(error) => write!(f, "solver failed: {error}"),
        }
    }
}

impl std::error::Error for SimulationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidInput { .. } => None,
            Self::Position { source, .. } => Some(source),
            Self::Dealing(error) => Some(error),
            Self::Solver(error) => Some(error),
        }
    }
}

impl From<DealerError> for SimulationError {
    fn from(error: DealerError) -> Self {
        Self::Dealing(error)
    }
}

impl From<SolverError> for SimulationError {
    fn from(error: SolverError) -> Self {
        Self::Solver(error)
    }
}

/// Why a recorded play and its DD sequence cannot be analysed together.
///
/// The DD sequence is one entry per recorded play plus the starting position,
/// so a truncated sequence is rejected before it can panic or silently drop
/// the plays it does not cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnalysisInputError {
    /// The recorded play claims more cards than a deal holds.
    PlaySequenceTooLong {
        /// How many cards the record claims.
        cards: usize,
    },
    /// The DD sequence is empty, so it has no starting position.
    MissingInitialTricks,
    /// The DD sequence ended before the recorded play did.
    MissingTrickResult {
        /// Zero-based index of the first recorded card with no DD result.
        play_index: usize,
    },
}

impl fmt::Display for AnalysisInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlaySequenceTooLong { cards } => {
                write!(f, "the play records {cards} cards, but a deal has 52")
            }
            Self::MissingInitialTricks => {
                write!(f, "the DD sequence has no starting position")
            }
            Self::MissingTrickResult { play_index } => {
                write!(
                    f,
                    "the DD sequence has no result for recorded play {play_index}"
                )
            }
        }
    }
}

impl std::error::Error for AnalysisInputError {}

/// Why a recorded board could not be analysed.
///
/// `E` is the analyser's own error type, so [`analyze_board`](crate::performance_analysis::analyze_board)
/// is generic over the backend rather than requiring a crate-wide error to
/// absorb every analyser error.
#[derive(Debug)]
#[non_exhaustive]
pub enum AnalysisError<E> {
    /// The board lacked the contract or the recorded play analysis needs.
    InvalidInput {
        /// Where the input was rejected, for diagnostics.
        context: &'static str,
        /// What was wrong with it.
        message: String,
    },
    /// The recorded play and the DD sequence disagree about how much play
    /// there is.
    PlaySequence(AnalysisInputError),
    /// The analyser rejected the board or failed to produce a DD sequence.
    Analyser(E),
}

impl<E> AnalysisError<E> {
    /// Builds an [`InvalidInput`](AnalysisError::InvalidInput) error.
    #[must_use]
    pub fn invalid_input(context: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            context,
            message: message.into(),
        }
    }

    /// Wraps the analyser's own error.
    #[must_use]
    pub fn analyser(error: E) -> Self {
        Self::Analyser(error)
    }
}

impl<E: fmt::Display> fmt::Display for AnalysisError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { context, message } => {
                write!(f, "invalid input ({context}): {message}")
            }
            Self::PlaySequence(error) => {
                write!(
                    f,
                    "performance analysis rejected the play sequence: {error}"
                )
            }
            Self::Analyser(error) => write!(f, "performance analysis failed: {error}"),
        }
    }
}

impl<E> std::error::Error for AnalysisError<E>
where
    E: fmt::Display + fmt::Debug + std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidInput { .. } => None,
            Self::PlaySequence(error) => Some(error),
            Self::Analyser(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn invalid_input_display_includes_context_and_message() {
        let error = SimulationError::invalid_input("payoff builder", "num_deals must be set");
        let text = error.to_string();
        assert!(text.contains("payoff builder"), "got {text}");
        assert!(text.contains("num_deals must be set"), "got {text}");
        assert!(error.source().is_none(), "invalid input has no source");
    }

    #[test]
    fn simulation_errors_expose_their_cause() {
        let error = SimulationError::from(DealerError::Unsatisfiable { attempts: 7 });
        assert!(matches!(error, SimulationError::Dealing(_)));
        assert!(error.source().is_some(), "dealer errors carry a source");

        let error = SimulationError::from(SolverError::NoCards);
        assert!(matches!(error, SimulationError::Solver(_)));
        assert!(error.source().is_some(), "solver errors carry a source");
    }

    #[test]
    fn analysis_errors_expose_the_analyser_error() {
        let error: AnalysisError<SolverError> = AnalysisError::analyser(SolverError::NoCards);
        assert!(matches!(error, AnalysisError::Analyser(_)));
        assert!(error.source().is_some(), "the analyser error is the source");
        assert!(error.to_string().contains("no card"), "got {error}");

        let error: AnalysisError<SolverError> =
            AnalysisError::invalid_input("performance analysis", "board records no play");
        assert!(error.source().is_none(), "invalid input has no source");
    }

    #[test]
    fn analysis_errors_expose_a_play_sequence_error() {
        let error: AnalysisError<SolverError> =
            AnalysisError::PlaySequence(AnalysisInputError::MissingInitialTricks);
        assert!(matches!(error, AnalysisError::PlaySequence(_)));
        assert!(
            error.source().is_some(),
            "the play-sequence error is the source"
        );
        assert!(
            error.to_string().contains("no starting position"),
            "got {error}"
        );
    }
}
