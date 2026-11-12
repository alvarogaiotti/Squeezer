// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The played cards: a flat, ordered sequence attributed to seats.
//!
//! Seats are stored per play rather than per trick, so a claimed or partial
//! final trick is simply a short trailing group and no contract is needed to
//! attribute the cards. Tricks are the derived groups of four.

use crate::contract::{Contract, Seat};
use crate::prelude::*;

use super::position::trick_winner;

/// A card group before the final one holds fewer than four cards.
///
/// The winner of each complete trick leads the next, so a short group anywhere
/// but last would leave the attribution without the seat that leads the next
/// trick. Callers outside the model crate (the LIN and PBN importers) get this
/// from [`attribute_plays`] and [`validate_trick_lengths`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortTrickNotFinal;

impl fmt::Display for ShortTrickNotFinal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("only the final trick of a play may hold fewer than four cards")
    }
}

impl Error for ShortTrickNotFinal {}

/// Checks that only the final group of a recorded play is short.
///
/// # Errors
///
/// Returns [`ShortTrickNotFinal`] when a group before the last holds fewer
/// than four cards.
pub fn validate_trick_lengths(groups: &[Vec<Card>]) -> Result<(), ShortTrickNotFinal> {
    if groups.iter().rev().skip(1).any(|group| group.len() < 4) {
        return Err(ShortTrickNotFinal);
    }
    Ok(())
}

/// Attributes a flat, trick-grouped card order to seats.
///
/// Both importers record the cards in play order, grouped by trick, and neither
/// stores a seat per card. The opening leader is the contract's leader; within
/// a trick the seats rotate, and the winner of each complete trick leads the
/// next one. A short final group is allowed: a claim or a cut-off record ends
/// the play mid-trick, and no later trick can follow it.
///
/// # Errors
///
/// Returns [`ShortTrickNotFinal`] when a group before the last is short.
pub fn attribute_plays(
    groups: &[Vec<Card>],
    contract: Contract,
) -> Result<PlayRecord, ShortTrickNotFinal> {
    validate_trick_lengths(groups)?;

    let mut plays = Vec::new();
    let mut leader = contract.leader();
    for group in groups {
        let trick: Vec<Play> = group
            .iter()
            .enumerate()
            .map(|(offset, card)| Play::new(leader + offset, *card))
            .collect();
        if trick.len() == 4 {
            leader = trick_winner(&trick, contract.strain());
        }
        plays.extend(trick);
    }
    Ok(PlayRecord::new(plays))
}

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
