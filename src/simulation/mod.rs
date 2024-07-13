use crate::SqueezerError;

mod lead_sim;
mod payoff;

pub use lead_sim::*;
pub use payoff::*;

pub trait SimulationResult {
    fn report(&self);
}

pub trait Simulation<T: SimulationResult> {
    /// # Errors
    /// Errors on error from the solver
    /// or for operations to create input for the solver from a deal
    fn run(&self) -> Result<T, SqueezerError>;
}
