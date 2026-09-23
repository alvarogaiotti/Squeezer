// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The played cards: a flat, ordered sequence attributed to seats.
//!
//! Seats are stored per play rather than per trick, so a claimed or partial
//! final trick is simply a short trailing group and no contract is needed to
//! attribute the cards. Tricks are the derived groups of four.

use crate::prelude::*;

/// One card, played by one seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Play {
    /// The seat that played the card.
    pub seat: Seat,
    /// The card played.
    pub card: Card,
}

impl Play {
    /// A play of `card` by `seat`.
    #[must_use]
    pub const fn new(seat: Seat, card: Card) -> Self {
        Self { seat, card }
    }
}

/// The cards played on a board, in order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayRecord {
    plays: Vec<Play>,
}

impl PlayRecord {
    /// Wraps an already-ordered list of plays.
    #[must_use]
    pub const fn new(plays: Vec<Play>) -> Self {
        Self { plays }
    }

    /// Appends a play.
    pub fn push(&mut self, play: Play) {
        self.plays.push(play);
    }

    /// The plays in order.
    #[must_use]
    pub fn plays(&self) -> &[Play] {
        &self.plays
    }

    /// Number of cards played.
    #[must_use]
    pub fn len(&self) -> usize {
        self.plays.len()
    }

    /// True when no card has been played.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plays.is_empty()
    }

    /// The most recent play, if any.
    #[must_use]
    pub fn last(&self) -> Option<&Play> {
        self.plays.last()
    }

    /// Iterates over the plays in order.
    pub fn iter(&self) -> std::slice::Iter<'_, Play> {
        self.plays.iter()
    }

    /// Iterates over the played cards, dropping seat attribution.
    ///
    /// This is the view DDS's `PlayTraceBin` consumes.
    pub fn cards(&self) -> impl Iterator<Item = Card> + '_ {
        self.plays.iter().map(|play| play.card)
    }

    /// Iterates over the tricks: consecutive groups of four plays.
    ///
    /// The final group may be short when the play was claimed or cut off.
    pub fn tricks(&self) -> impl Iterator<Item = &[Play]> + '_ {
        self.plays.chunks(4)
    }
}

impl<'a> IntoIterator for &'a PlayRecord {
    type Item = &'a Play;
    type IntoIter = std::slice::Iter<'a, Play>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards() -> Vec<Card> {
        vec![Card::S2, Card::S3, Card::S4, Card::S5, Card::H2, Card::H3]
    }

    #[test]
    fn tricks_group_plays_in_fours_with_a_short_tail() {
        let plays = cards()
            .into_iter()
            .enumerate()
            .map(|(index, card)| Play::new(Seat::from(index), card))
            .collect();
        let record = PlayRecord::new(plays);

        let tricks: Vec<_> = record.tricks().collect();
        assert_eq!(tricks.len(), 2);
        assert_eq!(tricks[0].len(), 4);
        assert_eq!(tricks[1].len(), 2);
    }

    #[test]
    fn cards_iterator_drops_seat_attribution() {
        let record = PlayRecord::new(vec![
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::S3),
        ]);
        assert_eq!(record.cards().collect::<Vec<_>>(), vec![Card::S2, Card::S3]);
    }
}
