// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;

/// Struct for evaluating card holdings with a particular logic:
/// for example: controls, or aces, or 6421, or other counting methods.
/// Could be modified to accept a i32 for more flexibility to the counting style.
/// es. give half a point for a ten ecc.
///
/// # Example
///
/// ```
/// # use squeezer::{Evaluator,Cards, DealerError};
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
///  //                         A=4, K=3, Q=2, J=1
///  let hcp = Evaluator::new(&[4u8, 3u8, 2u8, 1u8]);
///  let mut deck = Cards::ALL;
///  let hand = deck.pick(13).unwrap();
///  assert_eq!(hcp.evaluate(hand), hand.high_card_points());
/// # Ok(())
/// # }
/// ```
pub struct Evaluator {
    evaluator: Box<dyn Fn(Cards) -> u8>,
}

impl Evaluator {
    #[must_use]
    pub fn new(values: &[u8]) -> Self {
        let mut vals = [0u8; 13];
        // Since the iteration starts from the top (e.g. A), we populate in
        // a linear fashion, using enumerate to take the values that comes in when
        // new is called with the standard parameters like values: &[4,3,2,1]
        //                                                          A,K,Q,J,...
        //                                                          [8,6,2,1,0...]

        vals[..values.len()].copy_from_slice(values);
        Self {
            evaluator: Box::new(move |x: Cards| {
                x.into_iter()
                    .map(|y| vals[14 - y.rank() as usize])
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
