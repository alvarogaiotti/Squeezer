use crate::prelude::*;

/// Struct for evaluating card holdings with a particular logic:
/// for example: controls, or aces, or 6421, or other counting methods.
/// Could be modified to accept a i32 for more flexibility to the counting style.
/// es. give half a point for a ten ecc.
///
/// # Example
///
/// ```
/// use squeezer::{Evaluator,Cards};
///
/// let hcp = Evaluator::new(&[4u8, 3u8, 2u8, 1u8]);
/// let mut deck = Cards::ALL;
/// let hand = deck.pick(13).unwrap();
/// assert_eq!(hcp.evaluate(hand), { hand.high_card_points() });
/// ```
pub struct Evaluator {
    evaluator: Box<dyn Fn(Cards) -> u8>,
}

impl Evaluator {
    #[must_use]
    pub fn new(values: &[u8]) -> Self {
        let mut vals = [0u8; 13];
        // Starting from 13 so we populate vals with rank indexes
        // using enumerate to take the values that comes in when
        // new is calle with the standard parameters like values: &[4,3,2,1]
        //                                                          A,K,Q,J,...

        let iter_range = (13 - values.len()..13).rev();

        for (position_in_values, rank_equivalent_for_vals) in iter_range.enumerate() {
            vals[rank_equivalent_for_vals] = values[position_in_values]
        }
        Self {
            evaluator: Box::new(move |x: Cards| {
                x.into_iter()
                    .map(|y| vals[y.rank() as usize - 2])
                    .sum::<u8>()
            }),
        }
    }
    #[must_use]
    pub fn evaluate(&self, cards: Cards) -> u8 {
        (self.evaluator)(cards)
    }
}

#[cfg(test)]
#[test]
fn evaluate_correctly_test() {
    let hcp = Evaluator::new(&[4u8, 3u8, 2u8, 1u8]);
    let mut deck = Cards::ALL;
    let hand = deck.pick(13).unwrap();
    assert_eq!(hcp.evaluate(hand), { hand.high_card_points() });
}
