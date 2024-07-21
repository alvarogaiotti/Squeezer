// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;
use dds::SolvedPlay;
use itertools::Itertools;

/// A struct that contains the the sequence of cards played
/// and the dd tricks of every card once played.
#[non_exhaustive]
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

impl Tricks {
    /// Returns a [`Tricks`] from a `u8`, checking if in the correct range
    /// # Errors
    /// Errors when `tricks` is not in the range `0..=13`
    #[inline]
    pub fn new(tricks: u8) -> Result<Self, DealerError> {
        if (0..=13).contains(&tricks) {
            Ok(Tricks(tricks))
        } else {
            Err(DealerError::new(format!(
                "trick number can be only in the range 0..=13: got {tricks}"
            )))
        }
    }

    /// Returns a [`Tricks`] from a `u8`, WHITHOUT checking if in the correct range
    #[must_use]
    #[inline]
    pub fn new_unchecked(tricks: u8) -> Self {
        Self(tricks)
    }
}

/// Newtype wrapper for the double dummy difference between before and after the card is played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrickDifference(u8);

impl TrickDifference {
    /// Returns a [`TrickDifference`] from a `u8`, checking if in the correct range
    /// # Errors
    /// Errors when `trick_difference` is not in the range `1..=13`
    #[inline]
    pub fn new(tricks_difference: u8) -> Result<Self, DealerError> {
        if (1..=13).contains(&tricks_difference) {
            Ok(TrickDifference(tricks_difference))
        } else {
            Err(DealerError::new(format!(
                "trick number can be only in the range 1..=13: got {tricks_difference}"
            )))
        }
    }

    /// Returns a [`TrickDifference`] from a `u8`, WHITHOUT checking if in the correct range
    #[must_use]
    #[inline]
    pub fn new_unchecked(tricks_difference: u8) -> Self {
        Self(tricks_difference)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CardPerformance {
    Error(Tricks, TrickDifference),
    Correct(Tricks),
}

impl CardPerformance {
    /// Returns the tricks of this [`CardPerformance`].
    #[inline]
    #[must_use]
    pub fn tricks(&self) -> Tricks {
        match *self {
            Self::Error(tricks, _) | Self::Correct(tricks) => tricks,
        }
    }

    /// Returns the difference of this [`CardPerformance`].
    #[inline]
    #[must_use]
    pub fn difference(&self) -> Option<TrickDifference> {
        match *self {
            Self::Error(_, difference) => Some(difference),
            Self::Correct(_) => None,
        }
    }
}

/// Represents  player's track of card's result played in a board.
/// We use Option because sometimes we claim and do not play cards.
/// The u8 represents the double dummy result of the card played.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayerResultTrace([Option<CardPerformance>; 12]);

impl PlayerResultTrace {
    #[must_use]
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            data: &self.0[..],
            pointer: 0,
        }
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        let mut counter = 0;
        while let Some(option) = self.0.get(counter) {
            if option.is_some() {
                counter += 1;
            } else {
                break;
            }
        }
        counter
    }

    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[allow(clippy::cast_lossless, clippy::cast_possible_truncation)]
    #[inline]
    /// Computes the performance of a player, taking a [`PlayerAccuracy`]
    /// representing the accuracy of a player since now and updating it with
    /// the result of the this hand
    pub fn compute_player_performance(&self, accuracy: &mut PlayerAccuracy) {
        // The accumulator is (number_of_correct_cards_played, tricks_lost)
        let result = self.iter().fold((0, 0), |accumulator, difference| {
            if let CardPerformance::Error(_, TrickDifference(delta)) = difference {
                (accumulator.0, delta + accumulator.1)
            } else {
                (accumulator.0 + 1u32, accumulator.1)
            }
        });

        accuracy.tricks_lost += result.1 as u32;
        accuracy.cards_played += self.len() as u32;
        accuracy.correct_cards += result.0;
    }
}

impl<'a> IntoIterator for &'a PlayerResultTrace {
    type Item = &'a CardPerformance;
    type IntoIter = self::Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a> {
    data: &'a [Option<CardPerformance>],
    pointer: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a CardPerformance;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.data.get(self.pointer);
        self.pointer += 1;
        if let Some(item) = item {
            item.as_ref()
        } else {
            None
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct PlayerAccuracy {
    correct_cards: u32,
    tricks_lost: u32,
    cards_played: u32,
}

impl PlayerAccuracy {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            correct_cards: 0,
            tricks_lost: 0,
            cards_played: 0,
        }
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayerPlayRecord {
    pub tricks: PlayerPlayTrace,
    pub results: PlayerResultTrace,
    trick_counter: usize,
}

impl PlayerPlayRecord {
    /// Create a new player play record.
    #[must_use]
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

/// Represents the position of a player for this trick.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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

enum Position {
    Defender,
    Declarer,
}

/// Analyzes the performance of players based on the contract and the result of playing a sequence of cards.
/// It evaluates the tricks and results of each player's card plays and determines the winner of each trick.
///
/// # Arguments
///
/// - `contract` - The contract being played (e.g., Suit and Level)
/// - `play_result_trace` - The traced sequence of played cards and their corresponding solver results
///
/// # Returns
///
/// An array of `PlayerPlayRecord` instances representing the performance of each player.
///
/// # Panics
///
/// Panics if there is no specified winner for a trick.
#[must_use]
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
    let delta = previous_result - result;
    // If opponents played a wrong card and lost 1+ trick(s)
    if delta == 0i32 {
        CardPerformance::Correct(Tricks(
            result
                .try_into()
                .expect("trick number should always be positive"),
        ))
    } else {
        CardPerformance::Error(
            Tricks(
                result
                    .try_into()
                    .expect("tricks number should always be positive"),
            ),
            TrickDifference(
                delta
                    .abs()
                    .try_into()
                    .expect("difference beween tricks cannot exceed 13"),
            ),
        )
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
            CardPerformance::Error(Tricks(6), TrickDifference(3))
        );
        assert_eq!(equal, CardPerformance::Correct(Tricks(5)));
        assert_eq!(worse, CardPerformance::Error(Tricks(7), TrickDifference(2)));
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
            CardPerformance::Error(Tricks(8), TrickDifference(1))
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
            CardPerformance::Error(Tricks(7), TrickDifference(1))
        );
        assert_eq!(
            players_performance[Seat::East as usize].tricks[4].unwrap(),
            Card::S8
        );
        assert_eq!(
            players_performance[Seat::East as usize].results[4].unwrap(),
            CardPerformance::Error(Tricks(8), TrickDifference(1))
        );
        assert_eq!(
            players_performance[Seat::East as usize].tricks[5].unwrap(),
            Card::CJ
        );
        assert_eq!(
            players_performance[Seat::East as usize].results[5].unwrap(),
            CardPerformance::Error(Tricks(9), TrickDifference(1))
        );
    }

    #[test]
    fn test_compute_player_performance() {
        let p_performance = [
            PlayerResultTrace([Some(CardPerformance::Correct(Tricks(10))); 12]),
            PlayerResultTrace([
                Some(CardPerformance::Correct(Tricks(10))),
                Some(CardPerformance::Error(Tricks(9), TrickDifference(1))),
                Some(CardPerformance::Correct(Tricks(10))),
                Some(CardPerformance::Correct(Tricks(10))),
                Some(CardPerformance::Error(Tricks(9), TrickDifference(1))),
                Some(CardPerformance::Correct(Tricks(10))),
                Some(CardPerformance::Correct(Tricks(10))),
                Some(CardPerformance::Error(Tricks(9), TrickDifference(1))),
                Some(CardPerformance::Correct(Tricks(10))),
                Some(CardPerformance::Correct(Tricks(10))),
                Some(CardPerformance::Error(Tricks(9), TrickDifference(1))),
                Some(CardPerformance::Correct(Tricks(10))),
            ]),
        ];
        let mut n_perf = PlayerAccuracy::new();
        let mut e_perf = PlayerAccuracy::new();
        p_performance[0].compute_player_performance(&mut n_perf);
        p_performance[1].compute_player_performance(&mut e_perf);
        assert_eq!(12, n_perf.cards_played);
        assert_eq!(0, n_perf.tricks_lost);
        assert_eq!(12, n_perf.correct_cards);
        assert_eq!(12, e_perf.cards_played);
        assert_eq!(4, e_perf.tricks_lost);
        assert_eq!(8, e_perf.correct_cards);
    }
    #[cfg(feature = "dds")]
    #[test]
    fn test_complete_analysis_pipeline() {
        use dds::PlayAnalyzer;
        let lin = "pn|gattochef,sebyx,Inter2018,fede00|st||md|3SAQ432HQJT72DT3CQ,SKJH983D974CT9876,S965HK654DKJ6CAJ5,ST87HADAQ852CK432|rh||ah|Board 1|sv|o|mb|1C|an|2+|mb|1D|mb|1H|an|picche|mb|2D|mb|p|mb|p|mb|3H|mb|p|mb|3S|mb|p|mb|4S|mb|p|mb|p|mb|p|pg||pc|DA|pc|D3|pc|D9|pc|D6|pg||pc|HA|pc|H2|pc|H9|pc|H4|pg||pc|D8|pc|DT|pc|D7|pc|DJ|pg||pc|S5|pc|S7|pc|SA|pc|SJ|pg||pc|CQ|pc|CT|pc|CA|pc|C2|pg||pc|S6|pc|S8|pc|SQ|pc|SK|pg||pc|H8|mc|9|";

        //let lin = "pn|simodra,fra97,matmont,thevava|st||md|3S34JH258TQKD2JQC7,S27TH69D679TKAC23,S6QH47JD458C468JA,|rh||ah|Board 1|sv|o|mb|p|mb|1S|mb|2H|mb|2S|mb|3H|mb|4S|mb|p|mb|p|mb|p|pg||pc|C7|pc|C3|pc|CA|pc|C5|pg||pc|H4|pc|HA|pc|H5|pc|H6|pg||pc|SA|pc|S3|pc|S2|pc|S6|pg||pc|SK|pc|S4|pc|S7|pc|SQ|pg||pc|D3|pc|D2|pc|DA|pc|D5|pg||pc|DK|pc|D4|pc|H3|pc|DJ|pg||pc|C2|pc|C4|pc|C9|pc|SJ|pg||pc|HK|mc|11|";
        let deal = crate::LinDeal::from_str(lin).unwrap();
        let dds_solver = crate::dds::DoubleDummySolver {};
        let contract = deal.contract().unwrap();
        let play_sequence = deal.play_sequence().unwrap();
        let players = deal.players();
        let hands = deal.hands();
        let bidding = deal.bidding().unwrap();
        println!(
            "Players: {} Dealer: {} Contract: {contract}\n {hands}\nBidding: {bidding}\nPlay sequence:\n{play_sequence}",
            players.iter().format(", "), deal.dealer(),
        );
        let dds_playsequence = play_sequence.try_into().unwrap();
        println!("{dds_playsequence:?}");
        let solved = dds_solver
            .analyze_play(&deal, &contract, dds_playsequence)
            .unwrap();
        let p_performance = analyse_players_performance(
            contract,
            TraceSolved {
                tricks: (*deal.play_sequence().unwrap()).clone(),
                results: solved,
            },
        );
        let mut n_perf = PlayerAccuracy::new();
        let mut e_perf = PlayerAccuracy::new();
        p_performance[0]
            .results
            .compute_player_performance(&mut n_perf);
        p_performance[1]
            .results
            .compute_player_performance(&mut e_perf);
        println!("{} {}", n_perf.correct_cards, n_perf.tricks_lost);
        println!("{} {}", e_perf.correct_cards, e_perf.tricks_lost);
    }
}
