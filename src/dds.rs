use std::ffi::c_int;

use crate::{ddsffi, prelude::*};

fn populate(deal: &Deal) -> [[u32; 4]; 4] {
    let mut remain_cards = [[0; 4]; 4];
    for (seat, hand) in deal.into_iter().enumerate() {
        for (index, suit) in hand.into_iter().enumerate() {
            remain_cards[seat][index] = suit.into_iter().map(|card| 1 << card.rank()).sum();
        }
    }
    remain_cards
}

pub enum Side {
    NS = 0,
    EW = 1,
}

fn empty_fut() -> ddsffi::futureTricks {
    ddsffi::futureTricks {
        nodes: 0,
        cards: 0,
        suit: [0; 13],
        rank: [0; 13],
        equals: [0; 13],
        score: [0; 13],
    }
}

pub fn dd_tricks(deal: &Deal, contract: &Contract) -> u8 {
    let (trump, first) = (contract.strain(), contract.leader());
    let c_deal = ddsffi::deal {
        trump: trump as c_int,
        first: first as c_int,
        currentTrickSuit: [0; 3],
        currentTrickRank: [0; 3],
        remainCards: populate(deal),
    };
    let mut future_tricks = empty_fut();
    let futp: *mut futureTricks = &mut future_tricks;
    unsafe { ddsffi::SolveBoard(c_deal, -1, 1, 1, futp, 0) };
    future_tricks.score[0] as u8
}
pub fn dd_score(deal: &Deal, contract: Contract) -> i32 {
    let tricks = dd_tricks(deal, &contract);
    contract.score(tricks)
}

