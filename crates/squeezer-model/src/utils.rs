// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Ready-made hand predicates built from the constraint vocabulary.
//!
//! These are convenience checks, not part of the constraint API: callers that
//! need a different threshold write their own [`HandConstraint`]. They live
//! here because they combine a shape constraint with the evaluator and the
//! card vocabulary, which the model crate owns.
//!
//! [`HandConstraint`]: crate::HandConstraint

use crate::{Evaluator, HandConstraintBuilder};
use squeezer_cards::{Cards, Suit};

/// Honour valuation for the weak-two suit: an honour's trick-taking weight.
const W2_EVALUATOR: Evaluator = Evaluator::new(&[2, 2, 1, 1, 1]);
/// Control valuation: an ace counts two, a king one.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn six_card_spade_suit_with_weak_honours_is_a_weak_two() {
        // KQJT98 543 432 2: six spades, 6 HCP, one control (the king).
        let hand = Cards::from_str("SKQJT98H543D432C2").expect("hand parses");
        assert!(weak2_hand_descriptor(hand));
    }

    #[test]
    fn five_card_spade_suit_is_not_a_weak_two() {
        // KQJT9 5432 432 2: the longest suit is one card short.
        let hand = Cards::from_str("SKQJT9H5432D432C2").expect("hand parses");
        assert!(!weak2_hand_descriptor(hand));
    }

    #[test]
    fn strong_hand_is_not_a_weak_two() {
        // KQJT98 A54 A32 2: the shape fits, but 14 HCP and five controls.
        let hand = Cards::from_str("SKQJT98HA54DA32C2").expect("hand parses");
        assert!(!weak2_hand_descriptor(hand));
    }
}
