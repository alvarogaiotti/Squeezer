// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;

const W2_EVALUATOR: Evaluator = Evaluator::new(&[2, 2, 1, 1, 1]);
const CONTROLS_EVALUATOR: Evaluator = Evaluator::new(&[2, 1]);

/// Weak-two opening detector: 6-card major, 5-10 HCP, weak controls.
///
/// # Panics
///
/// Panics only if a fixed shape pattern stops parsing (a crate bug: the
/// patterns are literals covered by tests).
#[must_use]
pub fn weak2_hand_descriptor(hand: Cards) -> bool {
    let wk2 = HandConstraintBuilder::new()
        .add_shape("(63-)4-4-")
        .expect("fixed weak-two shape pattern parses")
        .with_range(5, 10)
        .build()
        .expect("fixed weak-two constraint is non-empty");
    wk2.check(hand)
        && ((hand.len_of_suit(Suit::Spades) == 6
            && W2_EVALUATOR.evaluate(hand.spades()) > 3
            && CONTROLS_EVALUATOR.evaluate(hand) < 4)
            || (hand.len_of_suit(Suit::Hearts) == 6
                && W2_EVALUATOR.evaluate(hand.hearts()) > 3
                && CONTROLS_EVALUATOR.evaluate(hand) < 4))
}
