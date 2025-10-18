// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::SqueezerError;

mod card_payoff;
mod lead_sim;
mod payoff;

pub use card_payoff::*;
pub use lead_sim::*;
pub use payoff::*;

pub trait SimulationResult {
    type DataFormat;
    fn report(&self);
    fn results(self) -> Self::DataFormat;
}

pub trait Simulation<T: SimulationResult> {
    /// # Errors
    /// Errors on error from the solver
    /// or for operations to create input for the solver from a deal
    fn run(self) -> Result<T, SqueezerError>;
}
