use crate::prelude::*;
use dds::*;
use itertools::*;
use std::cmp::Ordering;

pub struct TraceSolved {
    pub trace: PlaySequence,
    pub results: SolvedPlay,
}

#[repr(u8)]
pub enum TrickPosition {
    First = 0,
    Second,
    Third,
    Last,
}

impl From<u8> for TrickPosition {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Self::First,
            1 => Self::Second,
            2 => Self::Third,
            3 => Self::Last,
            _ => unreachable!(),
        }
    }
}

/// Analyse the player performace based on the DDS play analysis
pub fn analyse_player_performance(
    player: Seat,
    contract: Contract,
    play_result_trace: TraceSolved,
) {
    let TraceSolved { trace, results } = play_result_trace;
    let mut tricks_iterator = results.into_iter().peekable();
    let mut correct_played = 0;
    let mut total_played = 0;
    let mut starting_position;
    let mut last_result = tricks_iterator
        .next()
        .expect("there should always be a result calculated before the attack");
    if contract.leader() == player {
        correct_played += (last_result
            == *tricks_iterator
                .peek()
                .expect("there should always be a result calculated after the attack"))
            as usize;
        starting_position = TrickPosition::First;
    } else {
        starting_position = (4 - contract.leader() as u8 + player as u8).into();
    };
    for (_trick, _dd_res) in trace
        .into_iter()
        .chunks(4)
        .into_iter()
        .zip(tricks_iterator.chunks(4).into_iter())
    {}
}

// fn max_for_trump(trump: Option<Suit>) -> Box<dyn Fn(Card, Card) -> Ordering> {
//     if let Some(trump) = trump {
//         Box::new(move |c1: Card, c2: Card| {
//             // Since the first card is always the winner,
//             // we can just check if the second card is of the trump suit
//             if c1.suit() != c2.suit() {
//                 if c2.suit() == trump {
//                     Ordering::Greater
//                 } else {
//                     Ordering::Less
//                 }
//             } else {
//                 c1.rank().cmp(&c2.rank())
//             }
//         })
//     } else {
//         Box::new(|c1: Card, c2: Card| {
//             if c1.suit() != c2.suit() {
//                 Ordering::Less
//             } else {
//                 c1.rank().cmp(&c2.rank())
//             }
//         })
//     }
// }
