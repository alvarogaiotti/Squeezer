// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! High-performance bridge hand analysis with SIMD optimization.
//!
//! This module analyzes player performance by comparing actual play against
//! double-dummy optimal play. The implementation uses SIMD instructions to
//! preprocess all 48 cards in a deal at once, then performs fast trick-winner
//! determination with minimal per-trick overhead.
//!
//! ## Performance Strategy
//!
//! 1. **Batch preprocessing**: All 48 cards are encoded once with trump information
//! 2. **SIMD operations**: Process 16 cards at a time using SSE/AVX
//! 3. **Branchless logic**: Per-trick evaluation uses conditional moves, not branches
//! 4. **Cache-friendly**: Preprocessed data is contiguous and aligned
#![allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]

use crate::{linparser::PlaySequence, prelude::*};
use dds::analyseplay::SolvedPlay;
use std::fmt;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m128i, _mm_add_epi16, _mm_and_si128, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_set1_epi8,
    _mm_set1_epi16, _mm_setzero_si128, _mm_srli_epi16, _mm_storeu_si128, _mm_unpackhi_epi8,
    _mm_unpacklo_epi8,
};

// ============================================================================
// Core Domain Types
// ============================================================================

/// Number of tricks in a bridge deal (0-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tricks(u8);

impl Tricks {
    pub const MAX: u8 = 13;

    #[inline]
    ///# Errors
    /// If the number of trick is more than 13
    pub const fn new(tricks: u8) -> Result<Self, TricksError> {
        if tricks <= Self::MAX {
            Ok(Self(tricks))
        } else {
            Err(TricksError::OutOfRange(tricks))
        }
    }

    #[inline]
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[inline]
    const fn new_unchecked(tricks: u8) -> Self {
        debug_assert!(tricks <= Self::MAX);
        Self(tricks)
    }
}

impl fmt::Display for Tricks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TricksError {
    OutOfRange(u8),
}

impl fmt::Display for TricksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(n) => write!(f, "trick count must be 0-13, got {n}"),
        }
    }
}

impl std::error::Error for TricksError {}

// ============================================================================

/// The difference in double-dummy tricks (0-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrickDelta(u8);

impl TrickDelta {
    pub const MAX: u8 = 13;
    pub const ZERO: Self = Self(0);

    #[inline]
    ///# Errors
    /// Errors is delta is more than 13
    pub const fn new(delta: u8) -> Result<Self, TrickDeltaError> {
        if delta <= Self::MAX {
            Ok(Self(delta))
        } else {
            Err(TrickDeltaError::OutOfRange(delta))
        }
    }

    #[inline]
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[inline]
    const fn new_unchecked(delta: u8) -> Self {
        debug_assert!(delta <= Self::MAX);
        Self(delta)
    }
}

impl fmt::Display for TrickDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrickDeltaError {
    OutOfRange(u8),
}

impl fmt::Display for TrickDeltaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(n) => write!(f, "trick delta must be 0-13, got {n}"),
        }
    }
}

impl std::error::Error for TrickDeltaError {}

// ============================================================================

/// Side of the table (declaring vs defending).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Side {
    Declaring,
    Defending,
}

impl Side {
    #[inline]
    #[must_use]
    pub fn from_seat(seat: Seat, declarer: Seat) -> Self {
        if seat == declarer || seat == declarer.partner() {
            Self::Declaring
        } else {
            Self::Defending
        }
    }
}

// ============================================================================

/// Performance evaluation of a single card play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CardResult {
    Optimal {
        tricks: Tricks,
    },
    Suboptimal {
        tricks: Tricks,
        tricks_lost: TrickDelta,
    },
}

impl CardResult {
    #[inline]
    #[must_use]
    pub const fn tricks(self) -> Tricks {
        match self {
            Self::Optimal { tricks } | Self::Suboptimal { tricks, .. } => tricks,
        }
    }

    #[inline]
    #[must_use]
    pub const fn tricks_lost(self) -> TrickDelta {
        match self {
            Self::Optimal { .. } => TrickDelta::ZERO,
            Self::Suboptimal { tricks_lost, .. } => tricks_lost,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_optimal(self) -> bool {
        matches!(self, Self::Optimal { .. })
    }
}

impl fmt::Display for CardResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Optimal { tricks } => write!(f, "Optimal ({tricks} tricks)"),
            Self::Suboptimal {
                tricks,
                tricks_lost,
            } => {
                write!(f, "{tricks} tricks (-{tricks_lost} from optimal)")
            }
        }
    }
}

// ============================================================================

/// A single card play with its performance evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardPlay {
    pub card: Card,
    pub result: CardResult,
    pub seat: Seat,
}

// ============================================================================

/// Complete play record for one player.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerRecord {
    plays: Vec<CardPlay>,
}

impl PlayerRecord {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            plays: Vec::with_capacity(13),
        }
    }

    #[inline]
    pub fn push(&mut self, play: CardPlay) {
        debug_assert!(self.plays.len() < 13);
        self.plays.push(play);
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.plays.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plays.is_empty()
    }

    #[inline]
    pub fn plays(&self) -> impl Iterator<Item = &CardPlay> {
        self.plays.iter()
    }

    #[must_use]
    pub fn statistics(&self) -> PlayerStats {
        let mut stats = PlayerStats::default();

        for play in &self.plays {
            stats.cards_played += 1;
            match play.result {
                CardResult::Optimal { .. } => stats.optimal_plays += 1,
                CardResult::Suboptimal { tricks_lost, .. } => {
                    stats.total_tricks_lost += u32::from(tricks_lost.get());
                }
            }
        }

        stats
    }
}

impl Default for PlayerRecord {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================

/// Performance statistics for a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerStats {
    pub cards_played: u32,
    pub optimal_plays: u32,
    pub total_tricks_lost: u32,
}

impl PlayerStats {
    #[inline]
    #[must_use]
    pub fn accuracy_percent(&self) -> f64 {
        if self.cards_played == 0 {
            0.0
        } else {
            (f64::from(self.optimal_plays) / f64::from(self.cards_played)) * 100.0
        }
    }

    #[inline]
    #[must_use]
    pub fn avg_tricks_lost(&self) -> f64 {
        if self.cards_played == 0 {
            0.0
        } else {
            f64::from(self.total_tricks_lost) / f64::from(self.cards_played)
        }
    }

    #[inline]
    #[must_use]
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            cards_played: self.cards_played + other.cards_played,
            optimal_plays: self.optimal_plays + other.optimal_plays,
            total_tricks_lost: self.total_tricks_lost + other.total_tricks_lost,
        }
    }
}

impl fmt::Display for PlayerStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{} optimal ({:.1}%), avg loss: {:.2}",
            self.optimal_plays,
            self.cards_played,
            self.accuracy_percent(),
            self.avg_tricks_lost()
        )
    }
}

// ============================================================================

/// Complete analysis of a hand's play.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HandAnalysis {
    pub contract: Contract,
    records: [PlayerRecord; 4],
}

impl HandAnalysis {
    #[inline]
    #[must_use]
    pub fn player(&self, seat: Seat) -> &PlayerRecord {
        &self.records[seat as usize]
    }

    #[inline]
    #[must_use]
    pub fn player_stats(&self, seat: Seat) -> PlayerStats {
        self.player(seat).statistics()
    }

    #[must_use]
    pub fn declaring_side_stats(&self) -> PlayerStats {
        let declarer = self.contract.declarer();
        let dummy = declarer.partner();
        self.player_stats(declarer)
            .combine(&self.player_stats(dummy))
    }

    #[must_use]
    pub fn defending_side_stats(&self) -> PlayerStats {
        let declarer = self.contract.declarer();
        let lho = declarer.next();
        let rho = declarer.previous();
        self.player_stats(lho).combine(&self.player_stats(rho))
    }
}

// ============================================================================
// SIMD Preprocessing
// ============================================================================

/// Preprocessed card data for fast comparison.
///
/// Encoding (u16):
/// - Trump cards: 256 + rank (values: 258-270)
/// - Non-trump: rank (values: 2-14)
///
/// Card encoding (u8):
/// - Card = rank + 16 * suit
/// - Suit extraction: card >> 4 (gives 0=spades, 1=hearts, 2=diamonds, 3=clubs)
/// - Rank extraction: card & 0x0F (gives 2-14)
///
/// This allows us to handle trump priority in a single preprocessing pass,
/// then adjust for lead suit per-trick with simple addition.
#[repr(align(64))] // Cache line alignment
#[derive(Debug)]
pub struct PreprocessedDeal {
    cards: [u8; 48],
}

impl PreprocessedDeal {
    /// Creates a new preprocessed deal using SIMD operations.
    #[must_use]
    pub fn new(cards: &[Card; 48], trump: Option<Suit>) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            // Self::new_scalar(cards, trump)
            if is_x86_feature_detected!("avx2") {
                unsafe { Self::new_avx2(cards, trump) }
            } else if is_x86_feature_detected!("sse2") {
                unsafe { Self::new_sse2(cards, trump) }
            } else {
                Self::new_scalar(cards, trump)
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            unsafe { Self::new_neon(cards, trump) }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self::new_scalar(cards, trump)
        }
    }

    /// Scalar fallback implementation.
    #[must_use]
    pub fn new_scalar(cards: &[Card; 48], trump: Option<Suit>) -> Self {
        let mut result = [0u8; 48];

        if let Some(trump_suit) = trump {
            let trump_id = trump_suit as u8; // 0=spades, 1=hearts, 2=diamonds, 3=clubs

            for i in 0..48 {
                let card = cards[i].as_u8();
                let suit = card >> 4; // Extract suit (0-3)

                // Branchless: add 256 if trump
                let is_trump = u8::from(suit == trump_id);
                result[i] = (is_trump << 7) + card;
            }
        } else {
            for i in 0..48 {
                result[i] = cards[i].as_u8();
            }
        }

        Self { cards: result }
    }

    /// SSE2 implementation (`x86_64`) - processes 16 cards at once.
    /// # Safety
    /// Requires SSE2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    #[must_use]
    pub unsafe fn new_sse2(cards: &[Card; 48], trump: Option<Suit>) -> Self {
        let mut result = [0u8; 48];

        if let Some(trump_suit) = trump {
            // Trump suit is 0-3 (spades=0, hearts=1, diamonds=2, clubs=3)
            let trump_id = trump_suit as u8;
            let trump_vec = _mm_set1_epi8(trump_id as i8);
            let mask_0f = _mm_set1_epi8(0x0F);

            // Process 16 cards at a time (3 iterations for 48 cards)
            for chunk in 0..3 {
                let offset = chunk * 16;

                // Load 16 cards as bytes
                let cards_u8 =
                    unsafe { _mm_loadu_si128(cards[offset..].as_ptr().cast::<__m128i>()) };

                // Extract suits: card >> 4 (gives 0-3)
                let suits = _mm_srli_epi16(cards_u8, 4);
                // Need to mask because srli_epi16 shifts 16-bit lanes, not bytes
                let suits_masked = _mm_and_si128(suits, mask_0f);

                // Extract ranks: card & 0x0F
                let ranks = _mm_and_si128(cards_u8, mask_0f);

                // Compare suits with trump (0-3)
                let is_trump_mask = _mm_cmpeq_epi8(suits_masked, trump_vec);

                // Process low 8 cards (convert u8->u16)
                let ranks_low = _mm_unpacklo_epi8(ranks, _mm_setzero_si128());
                let mask_low = _mm_unpacklo_epi8(is_trump_mask, is_trump_mask);

                // Add 256 where mask is set
                let bonus_256 = _mm_set1_epi16(256);
                let bonus_low = _mm_and_si128(mask_low, bonus_256);
                let result_low = _mm_add_epi16(ranks_low, bonus_low);

                // Process high 8 cards
                let ranks_high = _mm_unpackhi_epi8(ranks, _mm_setzero_si128());
                let mask_high = _mm_unpackhi_epi8(is_trump_mask, is_trump_mask);
                let bonus_high = _mm_and_si128(mask_high, bonus_256);
                let result_high = _mm_add_epi16(ranks_high, bonus_high);

                // Store results
                unsafe {
                    _mm_storeu_si128(result[offset..].as_mut_ptr().cast::<__m128i>(), result_low);
                };
                unsafe {
                    _mm_storeu_si128(
                        result[offset + 8..].as_mut_ptr().cast::<__m128i>(),
                        result_high,
                    );
                };
            }
        } else {
            // No trump case - just extract ranks
            for i in 0..48 {
                result[i] = cards[i].as_u8();
            }
        }

        Self { cards: result }
    }

    /// AVX2 implementation (`x86_64`) - processes 32 cards at once.
    /// # Safety
    /// Requires AVX2
    #[target_feature(enable = "avx2")]
    #[must_use]
    pub unsafe fn new_avx2(cards: &[Card; 48], trump: Option<Suit>) -> Self {
        use std::arch::x86_64::*;

        let mut result = [0u8; 48];

        if let Some(trump_suit) = trump {
            let trump_id = trump_suit as u8;

            let zero = _mm256_setzero_si256();
            let rank_mask = _mm256_set1_epi8(0x0F);
            let suit_mask = _mm256_set1_epi8(0xF0u8 as i8);
            let trump_vec = _mm256_set1_epi8(trump_id as i8);

            // ---- Load 32 cards ----
            let cards_v = _mm256_loadu_si256(cards.as_ptr().cast());
            // _mm256_storeu_si256(print.as_mut_ptr().cast(), cards_v);
            // println!("Cards");
            // for e in &print {
            //     println!("{e}");
            // }
            // _mm256_storeu_si256(print.as_mut_ptr().cast(), trump_vec);
            // println!("Trumps");
            // for e in &print {
            //     println!("{e}");
            // }

            // rank = low nibble
            let ranks = _mm256_and_si256(cards_v, rank_mask);

            // isolate high nibble and shift via lookup
            let suit_idx = _mm256_and_si256(cards_v, suit_mask);
            let suit_idx = _mm256_srli_epi16(suit_idx, 4);

            // _mm256_storeu_si256(print.as_mut_ptr().cast(), suit_idx);
            // println!("Suit match");
            // for e in &print {
            //     println!("{e}");
            // }
            // compare suits
            let is_trump = _mm256_cmpeq_epi8(suit_idx, trump_vec);

            // bonus byte: 1 or 0
            let bonus = _mm256_and_si256(is_trump, _mm256_set1_epi8(1));

            // _mm256_storeu_si256(print.as_mut_ptr().cast(), bonus);
            // println!("Bonus");
            // for e in &print {
            //     println!("{e}");
            // }

            let shift_bonus = _mm256_slli_epi16(bonus, 7);

            // _mm256_storeu_si256(print.as_mut_ptr().cast(), shift_bonus);
            // println!("Shift Bonus");
            // for e in &print {
            //     println!("{e:b}");
            // }
            // _mm256_storeu_si256(print.as_mut_ptr().cast(), bonus);
            // println!("Bonus");
            // for e in &print {
            //     println!("{e}");
            // }

            // // widen
            // let ranks_lo = _mm256_unpacklo_epi8(ranks, zero);
            // _mm256_storeu_si256(print.as_mut_ptr().cast(), ranks_lo);
            // println!("ranks");
            // for e in &print {
            //     println!("{e}");
            // }
            // let bonus_lo = _mm256_unpacklo_epi8(bonus, zero);
            // _mm256_storeu_si256(print.as_mut_ptr().cast(), bonus_lo);
            // println!("Bonus lo");
            // for e in &print {
            //     println!("{e}");
            // }
            // let bonus_hi = _mm256_slli_epi16(bonus_lo, 8);
            // _mm256_storeu_si256(print.as_mut_ptr().cast(), bonus_hi);
            // println!("Bonus hi");
            // for e in &print {
            //     println!("{e}");
            // }
            // let result_lo = _mm256_or_si256(ranks_lo, _mm256_slli_epi16(bonus_lo, 8));

            // let ranks_hi = _mm256_unpackhi_epi8(ranks, zero);
            // let bonus_hi = _mm256_unpackhi_epi8(bonus, zero);
            // let result_hi = _mm256_or_si256(ranks_hi, _mm256_slli_epi16(bonus_hi, 8));
            let result_p = _mm256_or_si256(ranks, _mm256_slli_epi16(bonus, 7));
            let result_p = _mm256_or_si256(result_p, cards_v);

            _mm256_storeu_si256(result.as_mut_ptr().cast(), result_p);

            // scalar tail
            for i in 32..48 {
                let card = cards[i].as_u8();
                let suit = card >> 4;
                let rank = card & 0x0F;
                let is_trump = (suit == trump_id) as u8;
                result[i] = (is_trump << 7) | rank as u8;
            }
        } else {
            for i in 0..48 {
                result[i] = cards[i].as_u8();
            }
        }

        Self { cards: result }
    }

    /// NEON implementation (ARM aarch64).
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    pub unsafe fn new_neon(cards: &[Card; 48], trump: Option<Suit>) -> Self {
        let mut result = [0u16; 48];

        if let Some(trump_suit) = trump {
            let trump_id = trump_suit as u8;
            let trump_vec = vdupq_n_u8(trump_id);
            let mask_0f = vdupq_n_u8(0x0F);
            let shift_4 = vdupq_n_s8(4);

            for chunk in 0..3 {
                let offset = chunk * 16;

                let cards_u8 = vld1q_u8(cards[offset..].as_ptr());

                // Extract suits (shift right 4)
                let suits = vshrq_n_u8(cards_u8, 4);

                // Extract ranks
                let ranks = vandq_u8(cards_u8, mask_0f);

                // Compare with trump
                let is_trump_mask = vceqq_u8(suits, trump_vec);

                // Widen to u16 and add bonus (low 8 cards)
                let ranks_low = vmovl_u8(vget_low_u8(ranks));
                let mask_low = vmovl_u8(vget_low_u8(is_trump_mask));
                let bonus_256 = vdupq_n_u16(256);
                let bonus_low = vandq_u16(mask_low, bonus_256);
                let result_low = vaddq_u16(ranks_low, bonus_low);

                // High 8 cards
                let ranks_high = vmovl_u8(vget_high_u8(ranks));
                let mask_high = vmovl_u8(vget_high_u8(is_trump_mask));
                let bonus_high = vandq_u16(mask_high, bonus_256);
                let result_high = vaddq_u16(ranks_high, bonus_high);

                vst1q_u16(result[offset..].as_mut_ptr(), result_low);
                vst1q_u16(result[offset + 8..].as_mut_ptr(), result_high);
            }
        } else {
            for i in 0..48 {
                result[i] = u16::from(cards[i].rank_bits());
            }
        }

        Self { cards: result }
    }

    /// Gets the preprocessed values for a specific trick.
    /// # Panics
    /// Panics if the slice we index into is not of length 4, something we hard code for.
    #[inline]
    #[must_use]
    pub fn get_trick(&self, trick_idx: usize) -> [u8; 4] {
        let offset = trick_idx * 4;
        self.cards[offset..offset + 4].try_into().expect("failed to convert preprocessed trick slice to array. Tricks should always have length 4, here it was not the case.")
    }
}

// ============================================================================
// Fast Trick Winner Determination
// ============================================================================

/// Finds the winner of a trick using preprocessed card strengths.
///
/// This is the hot path - called 12 times per hand.
/// # Panics
/// Will panic only if unable to find the highest card in the trick
#[inline]
#[must_use]
pub fn find_trick_winner(
    preprocessed: [u8; 4],
    original_cards: [Card; 4],
    lead_suit: Suit,
) -> usize {
    // Adjust for lead suit
    // Trump cards (>= 256) already have priority
    // Non-trump lead suit cards get +128 bonus
    let mut strengths = [0u8; 4];

    for i in 0..4 {
        let base = preprocessed[i];
        //println!("{i}: {} {base}", original_cards[i]);
        let suit = original_cards[i].suit();

        // Branchless: add 128 if not trump and is lead suit
        let is_not_trump = u8::from(base < 128);
        let is_lead = u8::from(suit == lead_suit);
        let lead_bonus = is_not_trump * is_lead * 64;

        strengths[i] = base + lead_bonus;
    }

    // Find maximum (compiler auto-vectorizes)
    strengths
        .iter()
        .enumerate()
        .max_by_key(|(_, s)| *s)
        .map(|(idx, _)| idx)
        .unwrap()
}

#[inline]
#[must_use]
pub fn find_trick_winner_naive(original_cards: [Card; 4], lead_suit: Suit, trump: Suit) -> usize {
    original_cards
        .iter()
        .enumerate()
        .fold((0, 0), |acc, (i, card)| {
            let is_trump = u8::from((trump as u8 ^ card.suit() as u8) & 0xF0 == 0);
            let is_lead = u8::from((lead_suit as u8 ^ card.suit() as u8) & 0xF0 == 0);

            let new_best: u8 = acc.1.max(card.as_u8() | (is_trump * 128) | (is_lead * 64));
            (
                (i as u8 * (new_best - acc.1)) + (acc.0 * u8::from((new_best - acc.1) != 0)),
                new_best - acc.1,
            )
        })
        .0 as usize
}

// ============================================================================
// Performance Evaluation
// ============================================================================

#[inline]
fn evaluate_play(prev_tricks: i32, curr_tricks: i32) -> CardResult {
    let delta = prev_tricks - curr_tricks;

    let tricks = Tricks::new_unchecked(
        curr_tricks
            .try_into()
            .unwrap_or_else(|_| panic!("DD tricks should be 0-13, got {curr_tricks}")),
    );

    let is_optimal = delta == 0;

    if is_optimal {
        CardResult::Optimal { tricks }
    } else {
        let tricks_lost = TrickDelta::new_unchecked(
            delta
                .unsigned_abs()
                .try_into()
                .unwrap_or_else(|_| panic!("Trick delta too large: {}", delta.abs())),
        );
        CardResult::Suboptimal {
            tricks,
            tricks_lost,
        }
    }
}

// ============================================================================
// Analysis Engine
// ============================================================================

/// Input for performance analysis.
#[derive(Debug, Clone)]
pub struct AnalysisInput {
    pub contract: Contract,
    pub play_sequence: PlaySequence,
    pub dd_results: SolvedPlay,
}

/// Analyzes player performance using SIMD-optimized preprocessing.
/// # Panics
/// Panics if DD result is empty
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn analyze_hand(input: AnalysisInput) -> HandAnalysis {
    let contract = input.contract;
    let trump = Suit::try_from(contract.strain()).ok();

    let mut cards_array: [Card; 48] = [Card::NONCE; 48];
    let seq_len = input.play_sequence.sequence.len();
    let tricks_to_analyze = seq_len / 4;
    let extra_cards = seq_len % 4;
    cards_array[0..seq_len].copy_from_slice(&input.play_sequence.sequence);

    // **KEY OPTIMIZATION**: Preprocess entire deal once with SIMD
    let preprocessed = PreprocessedDeal::new(&cards_array, trump);
    let mut records = [
        PlayerRecord::new(),
        PlayerRecord::new(),
        PlayerRecord::new(),
        PlayerRecord::new(),
    ];

    let mut dd_iter = input.dd_results.into_iter();
    let mut prev_tricks = dd_iter
        .next()
        .expect("DD results should include initial state");

    let mut current_leader = contract.leader();
    let mut trick_cards = [Card::NONCE; 4];

    // Process each trick
    for trick_idx in 0..tricks_to_analyze {
        let card_offset = trick_idx * 4;

        trick_cards.copy_from_slice(&cards_array[card_offset..card_offset + 4]);

        let trick_preprocessed = preprocessed.get_trick(trick_idx);
        let lead_suit = trick_cards[0].suit();
        //dbg!(trick_cards);
        //dbg!(trick_preprocessed);
        // Get DD results for this trick
        let trick_dd: Vec<i32> = dd_iter.by_ref().take(4).collect();
        assert_eq!(
            trick_dd.len(),
            4,
            "Missing DD results for trick {trick_idx}"
        );

        // Analyze each card play
        let seats = current_leader.iter_from();

        for ((&card, dd_tricks), seat) in trick_cards.iter().zip(trick_dd).zip(seats) {
            let result = evaluate_play(prev_tricks, dd_tricks);

            records[seat as usize].push(CardPlay { card, result, seat });
            prev_tricks = dd_tricks;
        }

        // Find winner using SIMD-friendly comparison
        let winner_position_in_trick =
            find_trick_winner(trick_preprocessed, trick_cards, lead_suit);
        //println!("{winner_position_in_trick}");
        current_leader = current_leader + winner_position_in_trick;
    }

    if extra_cards > 0 {
        let card_offset = tricks_to_analyze * 4;

        trick_cards[..extra_cards]
            .copy_from_slice(&cards_array[card_offset..card_offset + extra_cards]);
        let trick_dd = dd_iter.by_ref().take(extra_cards).collect_vec();
        // Analyze each card play
        let seats = current_leader.iter_from();

        for ((&card, dd_tricks), seat) in trick_cards.iter().zip(trick_dd).zip(seats) {
            let result = evaluate_play(prev_tricks, dd_tricks);

            records[seat as usize].push(CardPlay { card, result, seat });
            prev_tricks = dd_tricks;
        }
    }

    HandAnalysis { contract, records }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessed_deal_scalar() {
        let mut cards = [Card::new(Suit::Spades, 2); 48];
        cards[0] = Card::new(Suit::Spades, 14); // Ace of trump
        cards[1] = Card::new(Suit::Hearts, 14); // Ace of non-trump

        let preprocessed = PreprocessedDeal::new_scalar(&cards, Some(Suit::Spades));

        assert_eq!(preprocessed.cards[0], 142); // 256 + 14
        assert_eq!(preprocessed.cards[1], 14 ^ 0b0001_0000_u8);
    }

    #[test]
    fn test_side_from_seat() {
        let declarer = Seat::South;
        assert_eq!(Side::from_seat(Seat::South, declarer), Side::Declaring);
        assert_eq!(Side::from_seat(Seat::North, declarer), Side::Declaring);
        assert_eq!(Side::from_seat(Seat::East, declarer), Side::Defending);
        assert_eq!(Side::from_seat(Seat::West, declarer), Side::Defending);
    }

    #[test]
    fn test_evaluate_play_declarer() {
        // Declarer maintains tricks - optimal
        let result = evaluate_play(10, 10);
        assert!(result.is_optimal());

        // Declarer loses trick - suboptimal
        let result = evaluate_play(10, 9);
        assert_eq!(result.tricks_lost().get(), 1);
    }

    #[test]
    fn test_evaluate_play_defender() {
        let result = evaluate_play(10, 10);
        assert!(result.is_optimal());

        // Defender gives up trick - suboptimal
        let result = evaluate_play(9, 10);
        assert_eq!(result.tricks_lost().get(), 1);
    }

    #[test]
    fn test_player_stats() {
        let mut record = PlayerRecord::new();

        record.push(CardPlay {
            card: Card::new(Suit::Spades, 14),
            result: CardResult::Optimal {
                tricks: Tricks::new_unchecked(10),
            },
            seat: Seat::North,
        });

        record.push(CardPlay {
            card: Card::new(Suit::Hearts, 12),
            result: CardResult::Suboptimal {
                tricks: Tricks::new_unchecked(9),
                tricks_lost: TrickDelta::new_unchecked(1),
            },
            seat: Seat::North,
        });

        let stats = record.statistics();
        assert_eq!(stats.cards_played, 2);
        assert_eq!(stats.optimal_plays, 1);
        assert_eq!(stats.total_tricks_lost, 1);
        assert!(stats.accuracy_percent() - 50.0f64 < 0.001);
    }

    #[test]
    fn test_performance_difference() {
        let worse = evaluate_play(9, 7);
        let equal = evaluate_play(5, 5);
        let better = evaluate_play(3, 6);

        assert_eq!(
            better,
            CardResult::Suboptimal {
                tricks: Tricks(6),
                tricks_lost: TrickDelta(3)
            }
        );
        assert_eq!(equal, CardResult::Optimal { tricks: Tricks(5) });
        assert_eq!(
            worse,
            CardResult::Suboptimal {
                tricks: Tricks(7),
                tricks_lost: TrickDelta(2)
            }
        );
    }

    // #[test]
    // fn test_winner_suit() {
    //     let winner = find_trick_winner(
    //         [Card::new(Suit::Hearts, 10), Card::new(Suit::Hearts, 14)].into_iter(),
    //         Some(Suit::Spades),
    //         Seat::East,
    //     );
    //     assert_eq!(winner, (Seat::South, Card::HA));

    //     let winner = find_trick_winner(
    //         [
    //             Card::new(Suit::Hearts, 10),
    //             Card::new(Suit::Clubs, 2),
    //             Card::DA,
    //             Card::SA,
    //         ]
    //         .into_iter(),
    //         Some(Suit::Clubs),
    //         Seat::West,
    //     );
    //     assert_eq!(winner, (Seat::North, Card::C2));

    //     let winner = find_trick_winner(
    //         [
    //             Card::DA,
    //             Card::new(Suit::Clubs, 10),
    //             Card::new(Suit::Clubs, 2),
    //             Card::SA,
    //         ]
    //         .into_iter(),
    //         Some(Suit::Clubs),
    //         Seat::West,
    //     );
    //     assert_eq!(winner, (Seat::North, Card::C10));
    // }

    // #[test]
    // fn test_winner_nt() {
    //     let winner = find_trick_winner(
    //         [Card::new(Suit::Hearts, 10), Card::new(Suit::Hearts, 14)].into_iter(),
    //         None,
    //         Seat::West,
    //     );
    //     assert_eq!(winner, (Seat::North, Card::HA));

    //     let winner = find_trick_winner(
    //         [Card::new(Suit::Spades, 5), Card::new(Suit::Hearts, 14)].into_iter(),
    //         None,
    //         Seat::West,
    //     );
    //     assert_eq!(winner, (Seat::West, Card::S5));
    // }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_analyse_players_performance() {
        let contract = Contract::new(
            2,
            Strain::Spades,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled,
        );
        //  N   E   S   W
        //  S2  S3  S4  SA*
        //  H6  HK* H3  H5
        //  H2  H4  H8  HA*
        //  H7  HT  HJ  HQ*
        //  S5  S8  SK* H9
        //  S6  CJ  C7  CT
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
            Card::new(Suit::Hearts, 12), // W
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

        let analysis_input = AnalysisInput {
            contract,
            play_sequence: play_seq,
            dd_results: solved_play,
        };
        let players_performance = analyze_hand(analysis_input);

        println!("N  E  S  W");
        for i in 0..=4 {
            use itertools::*;
            println!(
                "{}",
                players_performance
                    .records
                    .iter()
                    .format_with("-", |elt, f| f(&format_args!("{}", elt.plays[i].card)))
            );
        }
        assert_eq!(
            players_performance.records[Seat::North as usize].plays[0].card,
            Card::S2
        );
        assert_eq!(
            players_performance.records[Seat::North as usize].plays[0].result,
            CardResult::Suboptimal {
                tricks: Tricks(8),
                tricks_lost: TrickDelta(1)
            }
        );
        assert_eq!(
            players_performance.records[Seat::East as usize].plays[1].card,
            Card::HK
        );
        assert_eq!(
            players_performance.records[Seat::East as usize].plays[2].card,
            Card::H4
        );
        assert_eq!(
            players_performance.records[Seat::East as usize].plays[2].result,
            CardResult::Suboptimal {
                tricks: Tricks(7),
                tricks_lost: TrickDelta(1)
            }
        );
        assert_eq!(
            players_performance.records[Seat::East as usize].plays[4].card,
            Card::S8
        );
        assert_eq!(
            players_performance.records[Seat::East as usize].plays[4].result,
            CardResult::Suboptimal {
                tricks: Tricks(8),
                tricks_lost: TrickDelta(1)
            }
        );
        assert_eq!(
            players_performance.records[Seat::East as usize].plays[5].card,
            Card::CJ
        );
        assert_eq!(
            players_performance.records[Seat::East as usize].plays[5].result,
            CardResult::Suboptimal {
                tricks: Tricks(9),
                tricks_lost: TrickDelta(1)
            }
        );
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
        let analysis_input = AnalysisInput {
            contract,
            play_sequence: play_sequence.clone(),
            dd_results: solved,
        };
        let p_performance = analyze_hand(analysis_input);
        let n_perf = p_performance.player_stats(Seat::North);
        let e_perf = p_performance.player_stats(Seat::East);
        println!("{} {}", n_perf.optimal_plays, n_perf.total_tricks_lost);
        println!("{} {}", e_perf.optimal_plays, e_perf.total_tricks_lost);
    }
}
