// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{linparser::PlaySequence, prelude::*};
use dds::analyseplay::SolvedPlay;
use itertools::Itertools;

/// A struct that contains the the sequence of cards played
/// and the dd tricks of every card once played.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TraceSolved {
    pub tricks: PlaySequence,
    pub results: SolvedPlay,
}

/// Represents a player's track of cards played in a board.
/// We use Option because sometimes we claim and do not play cards.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// Returns a [`Tricks`] from a `u8`, WITHOUT checking if in the correct range
    #[must_use]
    #[inline]
    fn new_unchecked(tricks: u8) -> Self {
        Self(tricks)
    }
}

/// Newtype wrapper for the double dummy difference between before and after the card is played.
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// Returns a [`TrickDifference`] from a `u8`, WITHOUT checking if in the correct range
    #[must_use]
    #[inline]
    pub fn new_unchecked(tricks_difference: u8) -> Self {
        Self(tricks_difference)
    }
}

#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Represents the performance of a card played
pub enum CardPerformance {
    /// Played a wrong card, losing [`TrickDifference`] tricks and making [`Tricks`] tricks
    Bad(Tricks, TrickDifference),
    /// Played the correct double dummy card, making [`Tricks`] tricks
    Good(Tricks),
}

impl CardPerformance {
    /// Returns the tricks of this [`CardPerformance`].
    #[inline]
    #[must_use]
    pub fn tricks(&self) -> Tricks {
        match *self {
            Self::Bad(tricks, _) | Self::Good(tricks) => tricks,
        }
    }

    /// Returns the difference of tricks (if any) of this [`CardPerformance`].
    #[inline]
    #[must_use]
    pub fn difference(&self) -> Option<TrickDifference> {
        match *self {
            Self::Bad(_, difference) => Some(difference),
            Self::Good(_) => None,
        }
    }
}

/// Represents  player's track of card's result played in a board.
/// We use Option because sometimes we claim and do not play cards.
/// The u8 represents the double dummy result of the card played.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
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
            if let CardPerformance::Bad(_, TrickDifference(delta)) = difference {
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

#[derive(Default, Copy, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Represents the accuracy of a single player, measured in correct cards played and number of
/// tricks lost
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
/// [`PlayerPlayRecord`] are the cards played by this player, [`PlayerResultTrace`] are the number
/// of double dummy tricks after the corresponding card is played.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Position {
    Defender,
    Declarer,
}

/// Analyzes the performance of players based on the contract and the result of playing a sequence of cards.
/// It evaluates the tricks and results of each player's card plays and determines the winner of each trick.
///
/// # Arguments
///
/// - `contract` - The contract being played
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
    let trump = Suit::try_from(contract.strain()).ok();

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
        let trick_cards: Vec<_> = trick.collect();
        for ((card, n_of_tricks), seat) in trick_cards
            .iter()
            .zip(results)
            .zip(winner.previous().iter_from())
        {
            let diff = performance_difference(previous_result, n_of_tricks);
            players_records[seat as usize].push_trick(*card, diff);
            previous_result = n_of_tricks;
        }
        (winner, _) = find_trick_winner_lut(trick_cards.into_iter(), trump, winner);
    }
    players_records
}
fn find_trick_winner_lut<I: Iterator<Item = Card>>(
    mut trick: I,
    trump: Option<Suit>,
    lead_seat: Seat,
) -> (Seat, Card) {
    // Pre-compute card values for faster comparison
    let mut best_card = trick.next().unwrap();
    let lead_suit = best_card.suit() as u8;

    let mut best_idx = 0;
    let mut best_value = card_value(best_card, trump, lead_suit);

    for (i, card) in trick.enumerate() {
        let value = card_value(card, trump, lead_suit);
        if value > best_value {
            best_idx = i + 1;
            best_card = card;
            best_value = value;
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    let winner_seat = match (lead_seat as u8 + best_idx as u8) % 4 {
        0 => Seat::North,
        1 => Seat::East,
        2 => Seat::South,
        3 => Seat::West,
        _ => unreachable!(),
    };

    (winner_seat, best_card)
}

#[inline]
fn card_value(card: Card, trump: Option<Suit>, lead_suit: u8) -> u16 {
    let suit = card.suit() as u8;
    let rank = card.rank();
    if let Some(trump) = trump {
        if suit == trump as u8 {
            // Trump cards: highest priority (256+) + rank
            256 + u16::from(rank)
        } else if suit == lead_suit {
            // Lead suit cards: medium priority (128+) + rank
            128 + u16::from(rank)
        } else {
            // Off-suit cards: lowest priority, just rank
            u16::from(rank)
        }
    } else if suit == lead_suit {
        // Lead suit cards: medium priority (128+) + rank
        128 + u16::from(rank)
    } else {
        // Off-suit cards: lowest priority, just rank
        u16::from(rank)
    }
}

fn performance_difference(previous_result: i32, result: i32) -> CardPerformance {
    let delta = previous_result - result;
    // If opponents played a wrong card and lost 1+ trick(s)
    if delta == 0i32 {
        CardPerformance::Good(Tricks(
            result
                .try_into()
                .expect("trick number should always be positive"),
        ))
    } else {
        CardPerformance::Bad(
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

        assert_eq!(better, CardPerformance::Bad(Tricks(6), TrickDifference(3)));
        assert_eq!(equal, CardPerformance::Good(Tricks(5)));
        assert_eq!(worse, CardPerformance::Bad(Tricks(7), TrickDifference(2)));
    }

    #[test]
    fn test_winner_suit() {
        let winner = find_trick_winner_lut(
            [Card::new(Suit::Hearts, 10), Card::new(Suit::Hearts, 14)].into_iter(),
            Some(Suit::Spades),
            Seat::East,
        );
        assert_eq!(winner, (Seat::South, Card::HA));

        let winner = find_trick_winner_lut(
            [
                Card::new(Suit::Hearts, 10),
                Card::new(Suit::Clubs, 2),
                Card::DA,
                Card::SA,
            ]
            .into_iter(),
            Some(Suit::Clubs),
            Seat::West,
        );
        assert_eq!(winner, (Seat::North, Card::C2));

        let winner = find_trick_winner_lut(
            [
                Card::DA,
                Card::new(Suit::Clubs, 10),
                Card::new(Suit::Clubs, 2),
                Card::SA,
            ]
            .into_iter(),
            Some(Suit::Clubs),
            Seat::West,
        );
        assert_eq!(winner, (Seat::North, Card::C10));
    }

    #[test]
    fn test_winner_nt() {
        let winner = find_trick_winner_lut(
            [Card::new(Suit::Hearts, 10), Card::new(Suit::Hearts, 14)].into_iter(),
            None,
            Seat::West,
        );
        assert_eq!(winner, (Seat::North, Card::HA));

        let winner = find_trick_winner_lut(
            [Card::new(Suit::Spades, 5), Card::new(Suit::Hearts, 14)].into_iter(),
            None,
            Seat::West,
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
        //  N   E   S   W
        //  S2  S3  S4  SA
        //  H5
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
            CardPerformance::Bad(Tricks(8), TrickDifference(1))
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
            CardPerformance::Bad(Tricks(7), TrickDifference(1))
        );
        assert_eq!(
            players_performance[Seat::East as usize].tricks[4].unwrap(),
            Card::S8
        );
        assert_eq!(
            players_performance[Seat::East as usize].results[4].unwrap(),
            CardPerformance::Bad(Tricks(8), TrickDifference(1))
        );
        assert_eq!(
            players_performance[Seat::East as usize].tricks[5].unwrap(),
            Card::CJ
        );
        assert_eq!(
            players_performance[Seat::East as usize].results[5].unwrap(),
            CardPerformance::Bad(Tricks(9), TrickDifference(1))
        );
    }

    #[test]
    fn test_compute_player_performance() {
        let p_performance = [
            PlayerResultTrace([Some(CardPerformance::Good(Tricks(10))); 12]),
            PlayerResultTrace([
                Some(CardPerformance::Good(Tricks(10))),
                Some(CardPerformance::Bad(Tricks(9), TrickDifference(1))),
                Some(CardPerformance::Good(Tricks(10))),
                Some(CardPerformance::Good(Tricks(10))),
                Some(CardPerformance::Bad(Tricks(9), TrickDifference(1))),
                Some(CardPerformance::Good(Tricks(10))),
                Some(CardPerformance::Good(Tricks(10))),
                Some(CardPerformance::Bad(Tricks(9), TrickDifference(1))),
                Some(CardPerformance::Good(Tricks(10))),
                Some(CardPerformance::Good(Tricks(10))),
                Some(CardPerformance::Bad(Tricks(9), TrickDifference(1))),
                Some(CardPerformance::Good(Tricks(10))),
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
    #[test]
    fn test_complete_analysis_pipeline() {
        use dds::analyseplay::PlayAnalyzer;
        //let lin = "pn|gattochef,sebyx,Inter2018,fede00|st||md|3SAQ432HQJT72DT3CQ,SKJH983D974CT9876,S965HK654DKJ6CAJ5,ST87HADAQ852CK432|rh||ah|Board 1|sv|o|mb|1C|an|2+|mb|1D|mb|1H|an|picche|mb|2D|mb|p|mb|p|mb|3H|mb|p|mb|3S|mb|p|mb|4S|mb|p|mb|p|mb|p|pg||pc|DA|pc|D3|pc|D9|pc|D6|pg||pc|HA|pc|H2|pc|H9|pc|H4|pg||pc|D8|pc|DT|pc|D7|pc|DJ|pg||pc|S5|pc|S7|pc|SA|pc|SJ|pg||pc|CQ|pc|CT|pc|CA|pc|C2|pg||pc|S6|pc|S8|pc|SQ|pc|SK|pg||pc|H8|mc|9|";

        let lin = "pn|simodra,fra97,matmont,thevava|st||md|3S34JH258TQKD2JQC7,S27TH69D679TKAC23,S6QH47JD458C468JA,|rh||ah|Board 1|sv|o|mb|p|mb|1S|mb|2H|mb|2S|mb|3H|mb|4S|mb|p|mb|p|mb|p|pg||pc|C7|pc|C3|pc|CA|pc|C5|pg||pc|H4|pc|HA|pc|H5|pc|H6|pg||pc|SA|pc|S3|pc|S2|pc|S6|pg||pc|SK|pc|S4|pc|S7|pc|SQ|pg||pc|D3|pc|D2|pc|DA|pc|D5|pg||pc|DK|pc|D4|pc|H3|pc|DJ|pg||pc|C2|pc|C4|pc|C9|pc|SJ|pg||pc|HK|mc|11|";
        //let lin = "st||md|3SK5HJ84DJT852CJT8,SAHAT9753DA93C752,SJ876432HQDK76CK9,SQT9HK62DQ4CAQ643|rh||ah|Board 1|sv|o|mb|2S|mb|p|mb|p|mb|3H|mb|p|mb|4H|mb|p|mb|p|mb|p|pc|cK|pc|cA|pc|c8|pc|c2|pg||";
        //let lin = "qx|c37|st||md|3SKJT2HK84DA6CAT75,S54HJT7532DK97C84,SAQ763H96DQJ54CKJ,S98HAQDT832CQ9632|rh||ah|Board 1|sv|n|mb|1S|mb|p|mb|2N|mb|p|mb|3C|mb|p|mb|3D|mb|p|mb|3S|mb|p|mb|3N|mb|p|mb|4D|mb|p|mb|4H|mb|p|mb|4S|mb|p|mb|p|mb|p|pc|c2";
        //let lin = "qx|o47|st||md|1SJ852HADKT742CJT5,SAT96HJ6D3CAK8743,SK743HK8543DQJ5CQ,SQHQT972DA986C962|rh||ah|Board 1|sv|n|mb|p|mb|1C|mb|d|mb|1H|mb|d|an|t/o|mb|2C|mb|2S|mb|3C|mb|3S|mb|p|mb|p|mb|p|pc|c2|pc|c5|pc|cA|pc|cQ|pg||pc|c3|pc|s3|pc|c6|pc|cT|pg||pc|h4|pc|h2|pc|hA|pc|h6|pg||pc|s2|pc|s6|pc|sK|pc|sQ|pg||pc|hK|pc|h7|pc|cJ|pc|hJ|pg||pc|dQ|pc|dA|pc|d2|pc|d3|pg||pc|hQ|pc|s5|pc|s9|pc|h3|pg||pc|cK|pc|s4|pc|c9|pc|d4|pg||pc|dJ|pc|d6|pc|d7|mc|8|pg||";
        let file = crate::linparser::LinFile::from_str(lin).unwrap();
        let deal = file.deals.first().unwrap();
        let dds_solver = dds::doubledummy::DoubleDummySolver::new();
        let contract = deal.contract().unwrap();
        let play_sequence = deal.play_sequence().unwrap();
        let players = file.players();
        let hands = deal.hands();
        let bidding = deal.bidding();
        println!(
            "Players: {} Dealer: {} Contract: {contract}\n {hands}\nBidding: {bidding}\nPlay sequence:\n{play_sequence}",
            players.iter().format(", "),
            deal.dealer(),
        );
        let dds_playsequence = play_sequence.try_into().unwrap();
        println!("{dds_playsequence:?}");
        let solved = dds_solver
            .analyze_play(deal, &contract, dds_playsequence)
            .unwrap();
        println!("{solved:?}");
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
