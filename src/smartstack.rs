use crate::prelude::*;

pub struct Smartstack {
    //shape: Shape<'a>,
    evaluator: Evaluator,
    values: usize,
    predealt: bool,
    prepared: bool,
}

pub struct Evaluator {}

impl Evaluator {
    pub fn new(values: &[u8]) -> impl Fn(&Cards) -> u8 {
        let mut vals = [2u8; 13];
        for i in 0..13 {
            match 13 - i <= values.len() {
                true => vals[i] = values[12 - i],
                false => vals[i] = 0u8,
            };
        }
        move |x: &Cards| {
            x.into_iter()
                .map(|y| vals[y.rank() as usize - 2])
                .sum::<u8>()
        }
    }
}

#[cfg(test)]
#[test]
fn evaluate_correctly_test() {
    let hcp = Evaluator::new(&[4u8, 3u8, 2u8, 1u8]);
    let eval = Evaluator::new(&[6, 4, 2, 1]);
    let mut deck = Cards::ALL;
    let hand = deck.pick(13).unwrap();
    assert_eq!(hcp(&hand), hand.high_card_points() as u8);
}
