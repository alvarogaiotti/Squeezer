// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! # Lavazza 3NT opening
//!
//! Deals one hand that satisfies the Lavazza 3NT opening: a long minor and
//! 26-32 Zar points. The Zar evaluator and the convention itself live outside
//! the library on purpose: neither is a reusable bridge primitive.

use squeezer::prelude::*;

/// Zar's 6-4-2-1 honour valuation.
const ZAR_EVALUATOR: Evaluator = Evaluator::new(&[6, 4, 2, 1]);

/// Removes points for honours that are wasted in short suits.
fn devaluate_short_honors(hand: Cards) -> u8 {
    [hand.spades(), hand.hearts(), hand.diamonds(), hand.clubs()]
        .into_iter()
        .map(|suit| match suit.len() {
            0..=1 => suit.kings().len() + suit.queens().len() + suit.jacks().len(),
            2 => suit.queens().len() + suit.jacks().len(),
            _ => 0u8,
        })
        .sum()
}

/// Sorts four suit holdings by length in ascending order using a sorting network.
fn sort_suit_lengths(suits: &mut [Cards; 4]) {
    fn cmp_swap(suits: &mut [Cards; 4], i: usize, j: usize) {
        if suits[i].len() > suits[j].len() {
            suits.swap(i, j);
        }
    }
    cmp_swap(suits, 0, 1);
    cmp_swap(suits, 2, 3);
    cmp_swap(suits, 0, 2);
    cmp_swap(suits, 1, 3);
    cmp_swap(suits, 1, 2);
}

/// Zar's length-and-concentration points.
fn evaluate_length_and_concentration(hand: Cards) -> u8 {
    let mut suits = [hand.spades(), hand.hearts(), hand.diamonds(), hand.clubs()];
    sort_suit_lengths(&mut suits);
    let [shortest, second_shortest, second_longest, longest] = suits;
    let sum_long = longest.len() + second_longest.len();
    let diff_long_short = longest.len() - shortest.len();
    let mut points = sum_long + diff_long_short;
    let weak_concentration = 10 < hand.hcp()
        && hand.hcp() < 15
        && longest.high_card_points() + second_longest.high_card_points() >= hand.hcp() - 1;
    let strong_concentration = 14 < hand.hcp()
        && longest.high_card_points()
            + second_longest.high_card_points()
            + second_shortest.high_card_points()
            >= hand.hcp() - 1;
    if weak_concentration || strong_concentration {
        points += 1;
    }

    if points == 25 && hand.spades().len() > 3 {
        points += 1;
    }
    points
}

/// Zar points: honour values plus length/concentration, net of short honours.
#[must_use]
fn zar_points(hand: Cards) -> u8 {
    ZAR_EVALUATOR.evaluate(hand) + evaluate_length_and_concentration(hand)
        - devaluate_short_honors(hand)
}

/// The Lavazza 3NT shape family: a long minor (seven, eight or nine cards).
///
/// # Errors
///
/// Returns [`ConstraintError`] if a fixed pattern is malformed, which would be
/// a bug in this example.
fn lavazza_shape() -> Result<HandConstraint, ConstraintError> {
    HandConstraintBuilder::new()
        .add_shape("(8x)xx")?
        .add_shape("(7x)xx")?
        .add_shape("(9x)xx")?
        .build()
}

/// Dealer for the Lavazza 3NT opening: long minor, 26-32 Zar points.
///
/// # Errors
///
/// Returns [`DealerError`] if the fixed shape pattern stops parsing or the
/// dealer cannot be built.
fn dealer_of_lavazza_3nt_opening(seat: Option<Seat>) -> Result<impl Dealer, DealerError> {
    let shape = lavazza_shape()?;
    DealerBuilder::new()
        .with_function(move |deal: &Deal| {
            if let Some(seat) = seat {
                shape.check(deal[seat as usize])
                    && (26..33).contains(&zar_points(deal[seat as usize]))
            } else {
                deal.iter()
                    .any(|hand| shape.check(*hand) && (26..33).contains(&zar_points(*hand)))
            }
        })
        .build()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dealer = dealer_of_lavazza_3nt_opening(Some(Seat::South))?;
    let deal = dealer.deal()?;

    println!("A Lavazza 3NT opening:\n{deal}");
    println!("South holds {} Zar points", zar_points(deal[Seat::South]));
    Ok(())
}
