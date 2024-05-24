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

/// Newtype wrapper for a trick, represented with a u8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tricks(u8);

/// Newtype wrapper for the double dummy difference between before and after the card is played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrickDifference(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Analyzes the performance of players based on the contract and the result of playing a sequence of cards.
/// It evaluates the tricks and results of each player's card plays and determines the winner of each trick.
///
/// # Arguments
///
/// * `contract` - The contract being played (e.g., Suit and Level)
/// * `play_result_trace` - The traced sequence of played cards and their corresponding solver results
///
/// # Returns
///
/// An array of `PlayerPlayRecord` instances representing the performance of each player.
///
/// # Panics
///
/// Panics if there is no specified winner for a trick.
/// It may also panic when attempting to convert the contract into a trump type if it is not a No Trump contract.
pub fn analyse_players_performance(
    contract: Contract,
    play_result_trace: TraceSolved,
) -> [PlayerPlayRecord; 4] {
    let TraceSolved { tricks, results } = play_result_trace;
    let mut results_iterator = results.into_iter();
    let mut players_records: [PlayerPlayRecord; 4] =
        std::array::from_fn(|_| PlayerPlayRecord::new());
    let (winner_notrump, winner_trump);
    // Determine the winner based on whether it's a No Trump contract or not
    let winner_function: &dyn Fn(Card, Card, Seat, Seat) -> (Seat, Card) =
        if matches!(contract.strain(), Strain::NoTrumps) {
            winner_notrump = winner_nt;
            &winner_notrump
        } else {
            // If it's a Trump contract, set the winner accordingly
            let trump = contract.strain().try_into().expect("it's not NoTrump");
            winner_trump = curry_winner_trump(trump);
            &winner_trump
        };

    let mut winner = contract.leader();
    let mut previous_result = results_iterator
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

        // Process the first card of each trick
        let diff = performance_difference(previous_result, result);
        players_records[winner as usize].push_trick(first_card, diff);
        let mut winner_card = first_card;
        previous_result = result;
        let mut seat_iterator = winner.iter_from();

        // Iterate through the rest of the cards in the trick
        for (card, result) in card_result_iterator {
            let actual_player = seat_iterator.next().unwrap();
            (winner, winner_card) = winner_function(winner_card, card, winner, actual_player);
            let diff = performance_difference(previous_result, result);
            players_records[actual_player as usize].push_trick(card, diff);
            previous_result = result;
        }
    }
    players_records
}

fn curry_winner_trump(trump: Suit) -> impl Fn(Card, Card, Seat, Seat) -> (Seat, Card) {
    move |previous_card: Card, card: Card, winner: Seat, actual_player: Seat| {
        if previous_card.suit() != card.suit() {
            if card.suit() == trump {
                (actual_player, card)
            } else {
                (winner, previous_card)
            }
        } else if previous_card.rank() > card.rank() {
            (winner, previous_card)
        } else {
            (actual_player, card)
        }
    }
}

fn winner_nt(previous_card: Card, card: Card, winner: Seat, actual_player: Seat) -> (Seat, Card) {
    // Short circuits.
    if previous_card.suit() != card.suit() || previous_card.rank() > card.rank() {
        (winner, previous_card)
    } else {
        (actual_player, card)
    }
}

fn performance_difference(previous_result: i32, result: i32) -> CardPerformance {
    match previous_result - result {
        x if x < 0i32 => CardPerformance::Better(
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
        x if x > 0i32 => CardPerformance::Worse(
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
    }
}

/// Returns the player position in the trick.
fn player_position(leader: Seat, player: Seat) -> TrickPosition {
    (4 - leader as u8 + player as u8).into()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    #[test]
    fn test_performance_difference() {
        let worse = performance_difference(9, 7);
        let equal = performance_difference(5, 5);
        let better = performance_difference(3, 6);

        assert_eq!(
            better,
            CardPerformance::Better(Tricks(6), TrickDifference(3))
        );
        assert_eq!(equal, CardPerformance::Equal(Tricks(5)));
        assert_eq!(worse, CardPerformance::Worse(Tricks(7), TrickDifference(2)));
    }

    #[test]
    fn test_winner_suit() {
        let winner_fn = curry_winner_trump(Suit::Clubs);
        let winner = winner_fn(
            Card::new(Suit::Hearts, 10),
            Card::new(Suit::Hearts, 14),
            Seat::North,
            Seat::South,
        );
        assert_eq!(winner, (Seat::South, Card::HA));

        let winner = winner_fn(
            Card::new(Suit::Hearts, 10),
            Card::new(Suit::Clubs, 2),
            Seat::North,
            Seat::South,
        );
        assert_eq!(winner, (Seat::South, Card::C2));

        let winner = winner_fn(
            Card::new(Suit::Clubs, 10),
            Card::new(Suit::Clubs, 2),
            Seat::North,
            Seat::South,
        );
        assert_eq!(winner, (Seat::North, Card::C10));
    }

    #[test]
    fn test_winner_nt() {
        let winner = winner_nt(
            Card::new(Suit::Hearts, 10),
            Card::new(Suit::Hearts, 14),
            Seat::North,
            Seat::South,
        );
        assert_eq!(winner, (Seat::South, Card::HA));

        let winner = winner_nt(
            Card::new(Suit::Spades, 5),
            Card::new(Suit::Hearts, 14),
            Seat::West,
            Seat::North,
        );
        assert_eq!(winner, (Seat::West, Card::S5));
    }

    #[test]
    fn test_analyse_players_performance() {
        let contract = Contract::new(
            2,
            Strain::Spades,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled,
        );
        let play_seq = PlaySequence::new(vec![
            Card::new(Suit::Spades, 2), // N
            Card::new(Suit::Spades, 3),
            Card::new(Suit::Spades, 4),
            Card::new(Suit::Spades, 14), // Winner W
            Card::new(Suit::Hearts, 5),  // W
            Card::new(Suit::Hearts, 6),
            Card::new(Suit::Hearts, 13), // Winner East
            Card::new(Suit::Hearts, 3),
            Card::new(Suit::Hearts, 4), // E
            Card::new(Suit::Hearts, 8),
            Card::new(Suit::Hearts, 14), // Winner W
            Card::new(Suit::Hearts, 2),
            Card::new(Suit::Hearts, 12), // W, Winner W
            Card::new(Suit::Hearts, 7),
            Card::new(Suit::Hearts, 10),
            Card::new(Suit::Hearts, 11),
            Card::new(Suit::Hearts, 9), // W
            Card::new(Suit::Spades, 5),
            Card::new(Suit::Spades, 8),
            Card::new(Suit::Spades, 13), // Winner S
            Card::new(Suit::Clubs, 7),   // S
            Card::new(Suit::Clubs, 10),
            Card::new(Suit::Spades, 6), // Winner N
            Card::new(Suit::Clubs, 11),
        ]);
        let solved_play = SolvedPlay::from_seq(vec![
            7, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9,
        ]);

        let trace_solved = TraceSolved {
            tricks: play_seq,
            results: solved_play,
        };
        let players_performance = analyse_players_performance(contract, trace_solved);
        assert_eq!(
            players_performance[Seat::North as usize].tricks[0].unwrap(),
            Card::S2
        );
        assert_eq!(
            players_performance[Seat::North as usize].results[0].unwrap(),
            CardPerformance::Better(Tricks(8), TrickDifference(1))
        );
        assert_eq!(
            players_performance[Seat::East as usize].tricks[1].unwrap(),
            Card::HK
        );
        assert_eq!(
            players_performance[Seat::East as usize].tricks[2].unwrap(),
            Card::H4
        );
        assert_eq!(
            players_performance[Seat::East as usize].results[2].unwrap(),
            CardPerformance::Worse(Tricks(7), TrickDifference(1))
        );
        assert_eq!(
            players_performance[Seat::East as usize].tricks[4].unwrap(),
            Card::S8
        );
        assert_eq!(
            players_performance[Seat::East as usize].results[4].unwrap(),
            CardPerformance::Better(Tricks(8), TrickDifference(1))
        );
        assert_eq!(
            players_performance[Seat::East as usize].tricks[5].unwrap(),
            Card::CJ
        );
        assert_eq!(
            players_performance[Seat::East as usize].results[5].unwrap(),
            CardPerformance::Better(Tricks(9), TrickDifference(1))
        );
    }
}
