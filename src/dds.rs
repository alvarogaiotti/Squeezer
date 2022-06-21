use crate::prelude::*;

pub fn populate(deal: &Deal) -> [[u32; 4]; 4] {
    let mut remain_cards = [[0; 4]; 4];
    for (seat, hand) in deal.into_iter().enumerate() {
        for (index, suit) in hand.into_iter().enumerate() {
            remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
        }
    }
    remain_cards
}
