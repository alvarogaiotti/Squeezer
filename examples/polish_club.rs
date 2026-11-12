// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! # Polish Club 1♣ opening
//!
//! Deals one hand that satisfies the Polish Club 1♣ opening: weak balanced
//! 11-14, unbalanced with clubs 11-17, or any strong 18+ hand.
//!
//! This lives outside the library on purpose: it is one bidding convention,
//! not a reusable bridge primitive.

use squeezer::prelude::*;

/// Builds the Polish Club 1♣ opening constraint.
///
/// # Errors
///
/// Returns [`ConstraintError`] if an internal pattern is malformed. The
/// patterns are fixed tables, so this signals a bug in this example, not
/// caller input.
fn polish_club_hand_constraint() -> Result<HandConstraint, ConstraintError> {
    let weak1n = HandConstraintBuilder::new()
        .add_shape("(4432)")?
        .add_shape("(4333)")?
        .add_shape("(4414)")?
        .remove_shape("4441")?
        .add_shape("(5332)")?
        .remove_shape("(5x)(2+2+)")?
        .with_range(11, 14)
        .build()?;
    let unbal_with_clubs = HandConstraintBuilder::new()
        .with_longest(Suit::Clubs)?
        .remove_shape("(4x)x5")?
        .remove_shape("(5-5-5-)6+")?
        .with_range(11, 17)
        .build()?;
    let strong_any = HandConstraint::hcp(18..=37);
    HandConstraint::any(vec![weak1n, strong_any, unbal_with_clubs])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dealer = DealerBuilder::new()
        .with_hand_constraint(Seat::South, polish_club_hand_constraint()?)
        .build()?;

    println!("A Polish Club 1♣ opening:\n{}", dealer.deal()?);
    Ok(())
}
