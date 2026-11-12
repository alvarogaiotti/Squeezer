// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Ready-made hand predicates built from the card vocabulary.
//!
//! These are convenience checks, not part of the constraint API: callers that
//! need a different threshold write their own [`HandConstraint`]. They live
//! here because they combine shape, honour valuation and the card vocabulary,
//! which the model crate owns.
//!
//! [`HandConstraint`]: crate::HandConstraint

use squeezer_cards::{Cards, Suit};

/// Weighted honour count over aces, kings, queens, jacks and tens.
///
/// The private successor to the deleted public `Evaluator`: fixed to the five
/// honour ranks, so there is no input to truncate or silently zero-fill.
fn honour_points(hand: Cards, weights: [u8; 5]) -> u8 {
    let [aces, kings, queens, jacks, tens] = weights;
    aces * hand.aces().len()
        + kings * hand.kings().len()
        + queens * hand.queens().len()
        + jacks * hand.jacks().len()
        + tens * hand.tens().len()
}

/// Honour weights for the weak-two suit: an ace or king is worth two, a
/// queen, jack or ten one.
fn weak_two_honours(holding: Cards) -> u8 {
    honour_points(holding, [2, 2, 1, 1, 1])
}

/// Bridge controls: an ace counts two, a king one.
fn controls(hand: Cards) -> u8 {
    honour_points(hand, [2, 1, 0, 0, 0])
}

/// Weak-two opening detector: a complete hand with a 6-card major, 5-10 HCP,
/// enough honour strength in that suit and fewer than four controls.
///
/// The side suits are deliberately unconstrained: `6-4-2-1` and `6-3-2-2` are
/// as much a weak two as `6-3-3-1`. The previous `(63-)4-4-` pattern pinned
/// the six-card and short suits to the first two seats, which rejected a hand
/// such as 6-4-2-1 with the four-card suit in hearts.
#[must_use]
pub fn weak2_hand_descriptor(hand: Cards) -> bool {
    if hand.len() != 13 || !(5..=10).contains(&hand.hcp()) || controls(hand) >= 4 {
        return false;
    }
    [Suit::Spades, Suit::Hearts]
        .into_iter()
        .any(|suit| hand.len_of_suit(suit) == 6 && weak_two_honours(hand.in_suit(suit)) > 3)
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
    fn six_four_two_one_is_a_weak_two() {
        // KQJT98 5432 32 2: six spades, four hearts, six HCP.
        let hand = Cards::from_str("SKQJT98H5432D32C2").expect("hand parses");
        assert!(weak2_hand_descriptor(hand));
    }

    #[test]
    fn six_three_two_two_is_a_weak_two() {
        // KQJT98 543 32 32: six spades, three hearts, six HCP.
        let hand = Cards::from_str("SKQJT98H543D32C32").expect("hand parses");
        assert!(weak2_hand_descriptor(hand));
    }

    #[test]
    fn both_majors_six_cards_checks_both() {
        // 987654 KQJT98 2 -: worth a weak two in hearts, even though spades
        // is also six cards.
        let hand = Cards::from_str("S987654HKQJT98D2").expect("hand parses");
        assert!(weak2_hand_descriptor(hand));
    }

    #[test]
    fn partial_hand_is_not_a_weak_two() {
        // Nine cards with a six-card major is not a complete bridge hand.
        let hand = Cards::from_str("SKQJT98H543").expect("hand parses");
        assert!(!weak2_hand_descriptor(hand));
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
