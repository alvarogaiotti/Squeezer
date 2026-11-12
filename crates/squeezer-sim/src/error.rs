// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! The errors the simulation and analysis layers own.
//!
//! These replace the root crate's former crate-wide boundary error for the
//! moved code. A simulation error names the three things a Monte Carlo run can
//! fail on; an analysis error carries the analyser's own error type as a
//! parameter, so the seam stays generic over backends.

use squeezer_model::DealerError;
use squeezer_solver::SolverError;
use std::fmt;

/// Why a simulation could not be configured or run.
///
/// The variants preserve their cause: [`SimulationError::source`] returns the
/// dealer or solver error, so a caller can drill into the layer that failed.
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
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { context, message } => {
                write!(f, "invalid input ({context}): {message}")
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
}
