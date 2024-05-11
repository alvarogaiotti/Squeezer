use crate::prelude::*;
use dds::*;
use itertools::*;

/// A struct that contains the the sequence of cards played
/// and the dd tricks of every card once played.
pub struct TraceSolved {
    pub tricks: PlaySequence,
    pub results: SolvedPlay,
}

/// Represents a player's track of cards played in a board.
/// We use Option because sometimes we claim and do not play cards.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayerPlayTrace([Option<Card>; 13]);

impl std::ops::Index<usize> for PlayerPlayTrace {
    type Output = Option<Card>;
    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl std::ops::IndexMut<usize> for PlayerPlayTrace {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tricks(u8);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrickDifference(u8);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CardPerformance {
    Better(Tricks, TrickDifference),
    Worse(Tricks, TrickDifference),
    Equal(Tricks),
}

/// Represents a player's track of card's result played in a board.
/// We use Option because sometimes we claim and do not play cards.
/// The u8 represents the double dummy result of the card played.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayerResultTrace([Option<CardPerformance>; 12]);

impl std::ops::Index<usize> for PlayerResultTrace {
    type Output = Option<CardPerformance>;
    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl std::ops::IndexMut<usize> for PlayerResultTrace {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

/// Represents a player's track of card's result played in a board.
pub struct PlayerPlayRecord {
    pub tricks: PlayerPlayTrace,
    pub results: PlayerResultTrace,
    trick_counter: usize,
}

impl PlayerPlayRecord {
    /// Create a new player play record.
    pub fn new() -> Self {
        Self {
            tricks: PlayerPlayTrace::default(),
            results: PlayerResultTrace::default(),
            trick_counter: 0,
        }
    }

    /// Method use to push a card with the associated result into
    /// the play record of the player.
    pub fn push_trick(&mut self, card: Card, result: CardPerformance) {
        self.tricks[self.trick_counter] = Some(card);
        self.results[self.trick_counter] = Some(result);
        self.trick_counter += 1;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents the position of a player for this trick.
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
impl From<usize> for TrickPosition {
    fn from(value: usize) -> Self {
        match value % 4 {
            0 => Self::First,
            1 => Self::Second,
            2 => Self::Third,
            3 => Self::Last,
            _ => unreachable!(),
        }
    }
}

/// Analyse the players performace based on the DDS play analysis
pub fn analyse_players_performance(contract: Contract, play_result_trace: TraceSolved) {
    let TraceSolved { tricks, results } = play_result_trace;
    let mut results_iterator = results.into_iter();
    let mut players_records: [PlayerPlayRecord; 4] =
        std::array::from_fn(|_| PlayerPlayRecord::new());
    let (winner_notrump, winner_trump);
    let winner_function: &dyn Fn(Card, Card, Seat, Seat) -> Seat =
        if matches!(contract.strain(), Strain::NoTrumps) {
            winner_notrump = winner_nt;
            &winner_notrump
        } else {
            let trump = contract.strain().try_into().expect("it's not NoTrump");
            winner_trump =
                move |previous_card: Card, card: Card, winner: Seat, actual_player: Seat| {
                    if previous_card.suit() != card.suit() {
                        if card.suit() == trump {
                            actual_player
                        } else {
                            winner
                        }
                    } else if previous_card.rank() > card.rank() {
                        winner
                    } else {
                        actual_player
                    }
                };
            &winner_trump
        };

    let mut winner = contract.leader();
    let mut dd_result = results_iterator
        .next()
        .expect("there should always be a result calculated before the attack");

    for (trick, results) in tricks
        .into_iter()
        .chunks(4)
        .into_iter()
        .zip(results_iterator.chunks(4).into_iter())
    {
        let mut card_result_iterator = trick.into_iter().zip(results.into_iter());
        let (first_card, result) = card_result_iterator.next().expect("there should always be a leader, otherwise the tricks iterator should not have entered another loop.");

        // Loop body, but winner of the last trick is the actual player
        let diff = performance_difference(dd_result, result);
        players_records[winner as usize].push_trick(first_card, diff);
        let mut previous_card = first_card;
        dd_result = result;

        let mut seat_iterator = winner.iter_from();
        for (card, result) in card_result_iterator {
            let actual_player = seat_iterator.next().unwrap();
            winner = winner_function(previous_card, card, actual_player, winner);
            let diff = performance_difference(dd_result, result);
            players_records[actual_player as usize].push_trick(card, diff);
            previous_card = card;
            dd_result = result;

        }
    }
}

fn winner_nt(previous_card: Card, card: Card, winner: Seat, actual_player: Seat) -> Seat {
    if previous_card.suit() != card.suit() {
        winner
    } else if previous_card.rank() > card.rank() {
        winner
    } else {
        actual_player
    }
}

fn performance_difference(dd_result: i32, result: i32) -> CardPerformance {
    let diff = match dd_result - result {
        x if x < 0i32 => CardPerformance::Worse(
            Tricks(
                result
                    .try_into()
                    .expect("tricks number should always be positive"),
            ),
            TrickDifference(
                x.abs()
                    .try_into()
                    .expect("difference beween tricks cannot exceed 13"),
            ),
        ),
        x if x > 0i32 => CardPerformance::Better(
            Tricks(
                result
                    .try_into()
                    .expect("tricks number should always be positive"),
            ),
            TrickDifference(
                x.try_into()
                    .expect("difference beween tricks cannot exceed 13"),
            ),
        ),
        _ => CardPerformance::Equal(Tricks(
            result
                .try_into()
                .expect("trick number should always be positive"),
        )),
    };
    diff
}

/// Returns the player position in the trick.
fn player_position(leader: Seat, player: Seat) -> TrickPosition {
    (4 - leader as u8 + player as u8).into()
}

// fn winners(play_result_trace: PlaySequence, contract: Contract) -> Vec<Seat> {
//     let strain = contract.strain();
//     play_result_trace
//         .into_iter()
//         .chunks(4)
//         .into_iter()
//         .map(|chunk| {
//             if strain == Strain::NoTrumps {
//                 max_no_trump(chunk.into_iter())
//             } else {
//                 max_trump(
//                     chunk.into_iter(),
//                     strain.try_into().expect("just checked: it's not NoTrumps"),
//                 )
//             }
//         })
//         .collect()
// }

// fn max_no_trump<I>(trick: I) -> Seat
// where
//     I: Iterator<Item = Card>,
// {
//     trick
//         .enumerate()
//         .max_by(|(_, card1), (_, card2)| {
//             if card1.suit() != card2.suit() {
//                 Ordering::Greater
//             } else {
//                 card1.rank().cmp(&card2.rank())
//             }
//         })
//         .expect("shouldn't be calling this function whithout a populated trick")
//         .0
//         .into()
// }

// fn max_trump<I>(trick: I, trump: Suit) -> Seat
// where
//     I: Iterator<Item = Card>,
// {
//     trick
//         .enumerate()
//         .max_by(|(_, card1), (_, card2)| {
//             if card1.suit() != card2.suit() {
//                 if card2.suit() == trump {
//                     Ordering::Greater
//                 } else {
//                     Ordering::Less
//                 }
//             } else {
//                 card1.rank().cmp(&card2.rank())
//             }
//         })
//         .expect("shouldn't be calling this function whithout a populated trick")
//         .0
//         .into()
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use dds::*;

    #[test]
    fn calculate_correct_positions() {
        let vec: Vec<Seat> = vec![1u8, 2, 3, 0, 1].into_iter().map(Into::into).collect();
        let player: Vec<Seat> = vec![0u8, 2, 4, 2, 8].into_iter().map(Into::into).collect();
        for (index, (start, player)) in vec.iter().zip(player.iter()).enumerate() {
            assert_eq!(
                TrickPosition::from(3usize + index),
                player_position(*start, *player)
            );
        }
    }
}
