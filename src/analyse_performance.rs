use dds::*;
use crate::prelude::*;
use itertools::*;
use std::cmp::Ordering;

pub struct TraceSolved {
    pub trace : PlaySequence,
    pub results: SolvedPlay
}

#[repr(u8)]
pub enum TrickPosition {
    First = 0,
    Second,
    Third,
    Last
}

impl From<u8> for TrickPosition {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Self::First,
            1 => Self::Second,
            2 => Self::Third,
            3 => Self::Last,
        }
    }
}


/// Analyse the player performace based on the DDS play analysis
pub fn analyse_player_performance(player: Seat, contract: Contract, play_result_trace: TraceSolved) {
    let TraceSolved {trace, results} = play_result_trace;
    let mut tricks_iterator = results.into_iter().peek();
    let mut correct_played = 0;
    let mut total_played = 0;
    let mut starting_position;
    if contract.leader() == player {
        let optimal = tricks_iterator.next();
        if let Some(tricks) = tricks_iterator.peek() {
            correct_played += optimal == tricks as usize;
        } else {
            unreachable!();
        }
        starting_position = TrickPosition::First;
    } else {
        starting_position = (4 - contract.leader() as u8 + player as u8).into();
    }

    for (trick, dd_res) in trace.into_iter().chunks(4).into_iter().zip(&tricks_iterator.chunks(4).into_iter()) {
        if &dd_res.nth(starting_position as usize) >= &dd_res.nth(starting_position as usize - 1) {
            correct_played += 1
        } 
    }
    
}

fn max_for_suit(trump: Option<Suit>) -> impl Fn(Card, Card) -> bool {
    let default = |c1: Card, c2: Card| if c1.suit() != c2.suit() {
        // Remember that suit are stored in reverse order as a type
        // e.g. Spades < Hearts
        c1.suit() < c2.suit()
    } else {
        c1.rank() > c2.rank()
    };
    if let Some(trump) = trump {
        |c1, c2| if c1.suit() == trump {
            if c2.suit() != trump {
                Ordering::Greater
            } else {
                c1.rank() > c2.rank()
            }
        } else {
            default(c1,c2)
        }
    } else {
        default
    }
}