use crate::prelude::*;

pub struct Smartstack {
    //shape: Shape<'a>,
    evaluator: Evaluator,
    values: usize,
    predealt: bool,
    prepared: bool,
}

pub struct Evaluator {
    evaluator: Box<dyn Fn(&Cards) -> u8>,
}

impl Evaluator {
    pub fn new(values: &[u8]) -> Self {
        let mut vals = [2u8; 13];
        for i in 0..13 {
            match 13 - i <= values.len() {
                true => vals[i] = values[12 - i],
                false => vals[i] = 0u8,
            };
        }
        Self {
            evaluator: Box::new(move |x: &Cards| {
                x.into_iter()
                    .map(|y| vals[y.rank() as usize - 2])
                    .sum::<u8>()
            }),
        }
    }
    pub fn evaluate(&self, hand: &Cards) -> u8 {
        (self.evaluator)(hand)
    }
}

#[cfg(test)]
#[test]
fn evaluate_correctly_test() {
    let hcp = Evaluator::new(&[4u8, 3u8, 2u8, 1u8]);
    let mut deck = Cards::ALL;
    let hand = deck.pick(13).unwrap();
    assert_eq!(hcp.evaluate(&hand), hand.high_card_points() as u8);
}
