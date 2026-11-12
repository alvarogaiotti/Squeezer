// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Bridge hand analysis: compare recorded play against double-dummy results.
//!
//! The analysis loop resolves each trick's winner with the canonical rule in
//! `squeezer-model` (through [`find_trick_winner_naive`]) and inspects each
//! recorded card once. The preprocessed/SIMD path in [`PreprocessedDeal`] and
//! [`find_trick_winner`] is a supported benchmark harness: it is exported so
//! external measurements can reproduce the comparison, but [`analyze_hand`]
//! never constructs it.
//!
//! ## Performance strategy
//!
//! 1. **Stack trick buffers**: a complete trick is a fixed four-card array
//! 2. **One DD-sequence pass**: each recorded card is inspected once
//! 3. **Reserved records**: a player's `Vec` reserves a full hand up front, so
//!    pushing a card during analysis never reallocates
#![allow(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]

use crate::prelude::*;
use squeezer_solver::PlayAnalyser;
use std::fmt;

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
#[non_exhaustive]
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
#[non_exhaustive]
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
///
/// Every variant is explicitly double-dummy-relative (ADR-0003): "optimal"
/// means optimal against the fully-known deal, never the correct real-world
/// play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CardResult {
    DdOptimal {
        dd_tricks: Tricks,
    },
    DdSuboptimal {
        dd_tricks: Tricks,
        dd_tricks_lost: TrickDelta,
    },
}

impl CardResult {
    #[inline]
    #[must_use]
    pub const fn dd_tricks(self) -> Tricks {
        match self {
            Self::DdOptimal { dd_tricks } | Self::DdSuboptimal { dd_tricks, .. } => dd_tricks,
        }
    }

    #[inline]
    #[must_use]
    pub const fn dd_tricks_lost(self) -> TrickDelta {
        match self {
            Self::DdOptimal { .. } => TrickDelta::ZERO,
            Self::DdSuboptimal { dd_tricks_lost, .. } => dd_tricks_lost,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_dd_optimal(self) -> bool {
        matches!(self, Self::DdOptimal { .. })
    }
}

impl fmt::Display for CardResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DdOptimal { dd_tricks } => write!(f, "DD-optimal ({dd_tricks} tricks)"),
            Self::DdSuboptimal {
                dd_tricks,
                dd_tricks_lost,
            } => {
                write!(f, "{dd_tricks} tricks (-{dd_tricks_lost} from DD-optimal)")
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
                CardResult::DdOptimal { .. } => stats.dd_optimal_plays += 1,
                CardResult::DdSuboptimal { dd_tricks_lost, .. } => {
                    stats.dd_total_tricks_lost += u32::from(dd_tricks_lost.get());
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
///
/// Every double-dummy-relative quantity is named with a leading `dd_`
/// qualifier (ADR-0003), matching [`CardResult::dd_tricks_lost`]. `CONTEXT.md`
/// calls that quantity "DD deviation": the tricks a card lost against the
/// deal's DD-optimal result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerStats {
    pub cards_played: u32,
    pub dd_optimal_plays: u32,
    pub dd_total_tricks_lost: u32,
}

impl PlayerStats {
    #[inline]
    #[must_use]
    pub fn dd_accuracy_percent(&self) -> f64 {
        if self.cards_played == 0 {
            0.0
        } else {
            (f64::from(self.dd_optimal_plays) / f64::from(self.cards_played)) * 100.0
        }
    }

    #[inline]
    #[must_use]
    pub fn dd_avg_tricks_lost(&self) -> f64 {
        if self.cards_played == 0 {
            0.0
        } else {
            f64::from(self.dd_total_tricks_lost) / f64::from(self.cards_played)
        }
    }

    #[inline]
    #[must_use]
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            cards_played: self.cards_played + other.cards_played,
            dd_optimal_plays: self.dd_optimal_plays + other.dd_optimal_plays,
            dd_total_tricks_lost: self.dd_total_tricks_lost + other.dd_total_tricks_lost,
        }
    }
}

impl fmt::Display for PlayerStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{} DD-optimal ({:.1}%), avg DD loss: {:.2}",
            self.dd_optimal_plays,
            self.cards_played,
            self.dd_accuracy_percent(),
            self.dd_avg_tricks_lost()
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
/// # Purpose
///
/// This type is a supported benchmark harness, not part of [`analyze_hand`].
/// It encodes all 48 cards once so [`find_trick_winner`] can be measured
/// against the canonical [`find_trick_winner_naive`] rule. The production
/// analysis path never constructs one; the type stays public so external
/// measurement tools can reproduce the encoding and the comparison without
/// reimplementing either.
///
/// Encoding (u8):
/// - `card = rank + 16 * suit` (rank 2-14; suit 0=spades, 1=hearts,
///   2=diamonds, 3=clubs), so non-trump values are 2-62
/// - A trump card adds 128, so every trump outranks every non-trump card
/// - A non-trump card in the lead suit adds 64 at comparison time
/// - Suit extraction: `card >> 4`; rank extraction: `card & 0x0F`
///
/// This handles trump priority in one preprocessing pass, then adjusts for the
/// lead suit per trick with simple addition. The scalar constructors define
/// this contract; [`PreprocessedDeal::new_sse2`] is an unverified measurement
/// experiment that does not implement it, so never pass its output to
/// [`find_trick_winner`].
#[repr(align(64))] // Cache line alignment
#[derive(Debug)]
pub struct PreprocessedDeal {
    cards: [u8; 48],
}

impl PreprocessedDeal {
    /// Creates a new preprocessed deal.
    ///
    /// Scalar only: the former AVX2/SSE2/NEON auto-dispatch is gone. The
    /// SSE2 path disagreed with scalar on trump encoding, the NEON path did
    /// not compile, and the naive winner matched SIMD performance anyway
    /// (see `find_trick_winner_naive`). The arch-specific constructors
    /// below survive solely as benchmark harnesses.
    #[must_use]
    pub fn new(cards: &[Card; 48], trump: Option<Suit>) -> Self {
        Self::new_scalar(cards, trump)
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

                // Branchless: add 128 if trump, so it outranks every card
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
    ///
    /// Benchmark experiment only; see [`PreprocessedDeal`] for why its output
    /// is not interchangeable with the scalar encoding.
    /// # Safety
    /// The caller must ensure SSE2 is available at runtime (for example with
    /// `is_x86_feature_detected!("sse2")`); the target-feature attribute does
    /// not check it.
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
    ///
    /// Benchmark harness only; see [`PreprocessedDeal`].
    /// # Safety
    /// The caller must ensure AVX2 is available at runtime (for example with
    /// `is_x86_feature_detected!("avx2")`); the target-feature attribute does
    /// not check it.
    #[target_feature(enable = "avx2")]
    #[must_use]
    pub unsafe fn new_avx2(cards: &[Card; 48], trump: Option<Suit>) -> Self {
        use std::arch::x86_64::{
            _mm256_and_si256, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_or_si256,
            _mm256_set1_epi8, _mm256_slli_epi16, _mm256_srli_epi16, _mm256_storeu_si256,
        };

        let mut result = [0u8; 48];

        if let Some(trump_suit) = trump {
            let trump_id = trump_suit as u8;

            let rank_mask = _mm256_set1_epi8(0x0F);
            let suit_mask = _mm256_set1_epi8(0xF0u8 as i8);
            let trump_vec = _mm256_set1_epi8(trump_id as i8);

            // ---- Load 32 cards ----
            let cards_v = unsafe { _mm256_loadu_si256(cards.as_ptr().cast()) };
            // }
            // }

            // rank = low nibble
            let ranks = _mm256_and_si256(cards_v, rank_mask);

            // isolate high nibble and shift via lookup
            let suit_idx = _mm256_and_si256(cards_v, suit_mask);
            let suit_idx = _mm256_srli_epi16(suit_idx, 4);

            // }
            // compare suits
            let is_trump = _mm256_cmpeq_epi8(suit_idx, trump_vec);

            // bonus byte: 1 or 0
            let bonus = _mm256_and_si256(is_trump, _mm256_set1_epi8(1));

            // }

            // }
            // }

            // let ranks_lo = _mm256_unpacklo_epi8(ranks, zero);
            // }
            // let bonus_lo = _mm256_unpacklo_epi8(bonus, zero);
            // }
            // let bonus_hi = _mm256_slli_epi16(bonus_lo, 8);
            // }
            // let result_lo = _mm256_or_si256(ranks_lo, _mm256_slli_epi16(bonus_lo, 8));

            // let ranks_hi = _mm256_unpackhi_epi8(ranks, zero);
            // let bonus_hi = _mm256_unpackhi_epi8(bonus, zero);
            // let result_hi = _mm256_or_si256(ranks_hi, _mm256_slli_epi16(bonus_hi, 8));
            let result_p = _mm256_or_si256(ranks, _mm256_slli_epi16(bonus, 7));
            let result_p = _mm256_or_si256(result_p, cards_v);

            unsafe { _mm256_storeu_si256(result.as_mut_ptr().cast(), result_p) };

            // scalar tail
            for i in 32..48 {
                let card = cards[i].as_u8();
                let suit = card >> 4;
                let is_trump = u8::from(suit == trump_id);
                result[i] = (is_trump << 7) | card;
            }
        } else {
            for i in 0..48 {
                result[i] = cards[i].as_u8();
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
/// # Purpose
///
/// This is the benchmarked alternative to [`find_trick_winner_naive`], whose
/// canonical rule the analysis path actually uses. It is a supported public
/// entry point for measurement, not a hot path called by [`analyze_hand`].
/// `original_cards` must be the same four cards encoded in `preprocessed`.
///
/// # Panics
/// Will panic only if unable to find the highest card in the trick
#[inline]
#[must_use]
pub fn find_trick_winner(
    preprocessed: [u8; 4],
    original_cards: [Card; 4],
    lead_suit: Suit,
) -> usize {
    // Adjust for the lead suit. Trump cards already carry the 128 bit, so they
    // outrank every non-trump card; a non-trump lead-suit card gets +64.
    let mut strengths = [0u8; 4];

    for i in 0..4 {
        let base = preprocessed[i];
        let suit = original_cards[i].suit();

        // Branchless: add 64 if not trump and is lead suit
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
        .expect("the four strengths always have a maximum")
}

/// Finds the winner of a single trick as the index into `original_cards`.
///
/// This is the rule the analysis path uses: [`analyze_hand`] calls it for every
/// trick. The rule itself lives in
/// [`trick_winner`], which this delegates to. `lead_suit` is redundant with
/// `original_cards[0].suit()` — every caller passes exactly that — and is
/// retained only for signature compatibility; it is not consulted. A `None`
/// trump means no trump. The name `naive` records its origin as the reference
/// implementation used to validate [`find_trick_winner`]; it is not a
/// degraded path.
#[inline]
#[must_use]
pub fn find_trick_winner_naive(
    original_cards: [Card; 4],
    _lead_suit: Suit,
    trump: Option<Suit>,
) -> usize {
    // The canonical rule works on `Play`s and returns the winning `Seat`; this
    // analysis path has no seats, so bind the four positions to seats in order
    // and map the winner back.
    let trick: [Play; 4] =
        std::array::from_fn(|index| Play::new(Seat::from(index), original_cards[index]));
    trick_winner(&trick, trump.map_or(Strain::NoTrumps, Strain::from)) as usize
}

// ============================================================================
// Performance Evaluation
// ============================================================================

#[inline]
fn evaluate_play(prev_tricks: u8, curr_tricks: u8) -> CardResult {
    let delta = i32::from(prev_tricks) - i32::from(curr_tricks);

    let tricks = Tricks::new_unchecked(curr_tricks);

    let is_dd_optimal = delta == 0;

    if is_dd_optimal {
        CardResult::DdOptimal { dd_tricks: tricks }
    } else {
        let dd_tricks_lost = TrickDelta::new_unchecked(
            delta
                .unsigned_abs()
                .try_into()
                .unwrap_or_else(|_| panic!("Trick delta too large: {}", delta.abs())),
        );
        CardResult::DdSuboptimal {
            dd_tricks: tricks,
            dd_tricks_lost,
        }
    }
}

// ============================================================================
// Analysis Engine
// ============================================================================

/// Input for performance analysis.
///
/// `dd_results` is the backend-agnostic output of the solver seam's
/// [`PlayAnalyser`]: the **declarer's cumulative double-dummy trick total** at
/// the starting position and after each recorded play, so a board with `n`
/// plays has `n + 1` entries (DDS's `SolvedPlay.tricks` is that declarer
/// total, not a side-to-play count). The difference between consecutive
/// entries is the [`CardResult::dd_tricks_lost`] DD deviation for the card
/// just played. It carries no `dds` types, so the analysis is
/// backend-independent.
///
/// The caller contract is that every entry is a trick count in `0..=13` and
/// that the sequence holds the starting position plus one value per recorded
/// card. [`analyze_hand`] feeds the values to `Tricks::new_unchecked` (only
/// debug-checked), so an out-of-range value is a caller bug; a sequence that
/// is too short to cover the recorded play is an [`AnalysisInputError`], not a
/// panic.
#[derive(Debug, Clone)]
pub struct AnalysisInput {
    pub contract: Contract,
    pub play_sequence: PlayRecord,
    pub dd_results: Vec<u8>,
}

/// Analyses a recorded board against double-dummy through the solver seam.
///
/// The seam's [`PlayAnalyser`] produces the backend-agnostic per-play trick
/// counts; [`analyze_hand`] turns them into per-seat statistics. The board
/// must carry both a contract and a recorded play.
///
/// # Errors
///
/// Returns [`AnalysisError::Analyser`] with the analyser's own error when the
/// board cannot be analysed, [`AnalysisError::InvalidInput`] when it lacks a
/// contract or a play, and [`AnalysisError::PlaySequence`] when the analyser's
/// DD sequence does not cover the recorded play.
pub fn analyze_board<A>(
    analyser: &A,
    board: &Board,
) -> Result<HandAnalysis, AnalysisError<A::Error>>
where
    A: PlayAnalyser,
{
    let contract = board.contract().ok_or_else(|| {
        AnalysisError::invalid_input("performance analysis", "board has no contract")
    })?;
    let play_sequence = board.play().cloned().ok_or_else(|| {
        AnalysisError::invalid_input("performance analysis", "board records no play")
    })?;
    let dd_results = analyser
        .analyse_play(board)
        .map_err(AnalysisError::analyser)?;
    analyze_hand(AnalysisInput {
        contract,
        play_sequence,
        dd_results,
    })
    .map_err(AnalysisError::PlaySequence)
}

/// Turns a backend-agnostic DD sequence into per-play results and per-seat
/// statistics.
///
/// The winner of each trick is re-derived from the cards and the contract's
/// strain, so the recorded play is expected to be a legal, complete
/// playthrough (as [`Board`] guarantees). A short final trick is analysed card
/// by card, so an unplayed deal need not end on a completed trick.
///
/// See [`AnalysisInput`] for the `dd_results` contract: the sequence must hold
/// the starting position plus one value per recorded card. Extra values are
/// ignored.
///
/// # Errors
///
/// Returns [`AnalysisInputError::PlaySequenceTooLong`] when the record claims
/// more than 52 cards, [`AnalysisInputError::MissingInitialTricks`] when the
/// DD sequence is empty, and [`AnalysisInputError::MissingTrickResult`] when it
/// ends before the recorded play does.
///
/// # Panics
///
/// Panics only on an internal invariant violation: the sequence length is
/// checked above, so no input reaches the hot loop's `expect` calls. They stay
/// explicit so a future edit that removes the check fails loudly instead of
/// analysing a wrong trick count.
#[allow(clippy::needless_pass_by_value)]
pub fn analyze_hand(input: AnalysisInput) -> Result<HandAnalysis, AnalysisInputError> {
    let AnalysisInput {
        contract,
        play_sequence,
        dd_results,
    } = input;
    let trump = contract.strain().as_suit();

    let mut cards_array: [Card; 52] = [Card::NONCE; 52];
    let seq_len = play_sequence.len();
    if seq_len > cards_array.len() {
        return Err(AnalysisInputError::PlaySequenceTooLong { cards: seq_len });
    }
    let tricks_to_analyze = seq_len / 4;
    let extra_cards = seq_len % 4;

    // The play sequence has the same fixed upper bound as the bridge record.
    // Copy it directly into the scratch array instead of materialising a
    // temporary `Vec<Card>` on every hand analysis.
    for (destination, card) in cards_array[..seq_len].iter_mut().zip(play_sequence.cards()) {
        *destination = card;
    }

    let mut records = [
        PlayerRecord::new(),
        PlayerRecord::new(),
        PlayerRecord::new(),
        PlayerRecord::new(),
    ];

    // The analysis indexes DD results by play, so the initial position plus
    // every recorded card must be present. Checking the length once keeps the
    // hot loop on the allocation-free path.
    let recorded_results = dd_results.len();
    if recorded_results == 0 {
        return Err(AnalysisInputError::MissingInitialTricks);
    }
    if recorded_results <= seq_len {
        return Err(AnalysisInputError::MissingTrickResult {
            play_index: recorded_results - 1,
        });
    }

    let mut dd_iter = dd_results.into_iter();
    let mut prev_tricks = dd_iter
        .next()
        .expect("the DD sequence is non-empty, checked above");

    let mut current_leader = contract.leader();
    let mut trick_cards = [Card::NONCE; 4];

    // Process each trick
    for trick_idx in 0..tricks_to_analyze {
        let card_offset = trick_idx * 4;

        trick_cards.copy_from_slice(&cards_array[card_offset..card_offset + 4]);

        let lead_suit = trick_cards[0].suit();
        // Get DD results for this trick. A complete trick has a fixed width,
        // so keep the values on the stack rather than allocating a `Vec` for
        // every iteration.
        let mut trick_dd = [0u8; 4];
        for value in &mut trick_dd {
            *value = dd_iter
                .next()
                .expect("the sequence covers every recorded play, checked above");
        }

        // Analyze each card play
        let seats = current_leader.iter_from();

        for ((&card, dd_tricks), seat) in trick_cards.iter().zip(trick_dd).zip(seats) {
            let result = evaluate_play(prev_tricks, dd_tricks);

            records[seat as usize].push(CardPlay { card, result, seat });
            prev_tricks = dd_tricks;
        }

        // Find the winner with the canonical rule; the preprocessed path is
        // benchmark-only.
        let winner_position_in_trick = find_trick_winner_naive(trick_cards, lead_suit, trump);
        current_leader = current_leader + winner_position_in_trick;
    }

    if extra_cards > 0 {
        let card_offset = tricks_to_analyze * 4;

        trick_cards[..extra_cards]
            .copy_from_slice(&cards_array[card_offset..card_offset + extra_cards]);

        for (offset, seat) in current_leader.iter_from().enumerate().take(extra_cards) {
            let dd_tricks = dd_iter
                .next()
                .expect("the sequence covers every recorded play, checked above");
            let result = evaluate_play(prev_tricks, dd_tricks);
            let card = trick_cards[offset];

            records[seat as usize].push(CardPlay { card, result, seat });
            prev_tricks = dd_tricks;
        }
    }

    Ok(HandAnalysis { contract, records })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The contract shared by the synthetic hand-analysis tests: 1H by South.
    fn hearts_contract() -> Contract {
        Contract::new(
            1,
            Strain::Hearts,
            Seat::South,
            Vulnerable::No,
            Doubled::NotDoubled,
        )
    }

    #[test]
    fn test_preprocessed_deal_scalar() {
        let mut cards = [Card::new(Suit::Spades, 2); 48];
        cards[0] = Card::new(Suit::Spades, 14); // Ace of trump
        cards[1] = Card::new(Suit::Hearts, 14); // Ace of non-trump

        let preprocessed = PreprocessedDeal::new_scalar(&cards, Some(Suit::Spades));

        assert_eq!(preprocessed.cards[0], 142); // trump bonus 128 + spade ace (14)
        assert_eq!(preprocessed.cards[1], 0b0000_1110_u8 ^ 0b0001_0000_u8);
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
        // Declarer maintains tricks - DD-optimal
        let result = evaluate_play(10, 10);
        assert!(result.is_dd_optimal());

        // Declarer loses trick - DD-suboptimal
        let result = evaluate_play(10, 9);
        assert_eq!(result.dd_tricks_lost().get(), 1);
    }

    #[test]
    fn test_evaluate_play_defender() {
        let result = evaluate_play(10, 10);
        assert!(result.is_dd_optimal());

        // Defender gives up trick - DD-suboptimal
        let result = evaluate_play(9, 10);
        assert_eq!(result.dd_tricks_lost().get(), 1);
    }

    #[test]
    fn test_player_stats() {
        let mut record = PlayerRecord::new();

        record.push(CardPlay {
            card: Card::new(Suit::Spades, 14),
            result: CardResult::DdOptimal {
                dd_tricks: Tricks::new_unchecked(10),
            },
            seat: Seat::North,
        });

        record.push(CardPlay {
            card: Card::new(Suit::Hearts, 12),
            result: CardResult::DdSuboptimal {
                dd_tricks: Tricks::new_unchecked(9),
                dd_tricks_lost: TrickDelta::new_unchecked(1),
            },
            seat: Seat::North,
        });

        let stats = record.statistics();
        assert_eq!(stats.cards_played, 2);
        assert_eq!(stats.dd_optimal_plays, 1);
        assert_eq!(stats.dd_total_tricks_lost, 1);
        assert!(stats.dd_accuracy_percent() - 50.0f64 < 0.001);
    }

    #[test]
    fn test_performance_difference() {
        let worse = evaluate_play(9, 7);
        let equal = evaluate_play(5, 5);
        let better = evaluate_play(3, 6);

        assert_eq!(
            better,
            CardResult::DdSuboptimal {
                dd_tricks: Tricks(6),
                dd_tricks_lost: TrickDelta(3)
            }
        );
        assert_eq!(
            equal,
            CardResult::DdOptimal {
                dd_tricks: Tricks(5)
            }
        );
        assert_eq!(
            worse,
            CardResult::DdSuboptimal {
                dd_tricks: Tricks(7),
                dd_tricks_lost: TrickDelta(2)
            }
        );
    }

    #[test]
    fn test_preprocessed_winner_agrees_with_naive() {
        // Revived from commented-out tests written against an older
        // signature: pins the preprocessed winner to the naive one plus
        // expected positions, across trump and no-trump cases.
        let cases: [([Card; 4], Suit, Option<Suit>, usize); 4] = [
            (
                [
                    Card::new(Suit::Hearts, 10),
                    Card::new(Suit::Hearts, 14),
                    Card::new(Suit::Clubs, 2),
                    Card::new(Suit::Spades, 14),
                ],
                Suit::Hearts,
                Some(Suit::Spades),
                3,
            ),
            (
                [
                    Card::new(Suit::Spades, 11),
                    Card::new(Suit::Spades, 13),
                    Card::new(Suit::Spades, 12),
                    Card::new(Suit::Spades, 9),
                ],
                Suit::Spades,
                Some(Suit::Spades),
                1,
            ),
            (
                [
                    Card::new(Suit::Hearts, 10),
                    Card::new(Suit::Hearts, 14),
                    Card::new(Suit::Clubs, 2),
                    Card::new(Suit::Diamonds, 5),
                ],
                Suit::Hearts,
                None,
                1,
            ),
            (
                [
                    Card::new(Suit::Clubs, 10),
                    Card::new(Suit::Hearts, 14),
                    Card::new(Suit::Clubs, 2),
                    Card::new(Suit::Clubs, 13),
                ],
                Suit::Clubs,
                None,
                3,
            ),
        ];
        for (trick, lead_suit, trump, expected) in cases {
            let mut padded = [Card::NONCE; 48];
            padded[..4].copy_from_slice(&trick);
            let preprocessed = PreprocessedDeal::new_scalar(&padded, trump);
            let fast = find_trick_winner(preprocessed.get_trick(0), trick, lead_suit);
            let naive = find_trick_winner_naive(trick, lead_suit, trump);
            assert_eq!(fast, naive, "divergence on {trick:?}");
            assert_eq!(fast, expected, "wrong winner for {trick:?}");
        }
    }

    #[cfg(feature = "dds")]
    #[test]
    fn dd_example_hand_analysis_is_all_optimal() {
        // DDS's own hand-0 oracle is 46 results, all 8 (pinned by the seam
        // test), so every recorded play is DD-optimal and no tricks are lost.
        // This is the re-baselined DDS 3.1 semantics for the analysis.
        let board = squeezer_solver::test_fixtures::example_board_hand0_play();
        let analysis =
            analyze_board(&squeezer_solver::DdsSolver, &board).expect("example hand 0 analyses");

        let mut total = 0;
        for seat in Seat::iter() {
            let stats = analysis.player_stats(seat);
            total += stats.cards_played;
            assert_eq!(
                stats.dd_optimal_plays, stats.cards_played,
                "{seat} should be DD-optimal throughout"
            );
            assert_eq!(stats.dd_total_tricks_lost, 0, "{seat} lost no DD tricks");
        }
        assert_eq!(total, 45, "the whole recorded play is analysed");

        // The per-side statistics are kept.
        assert_eq!(analysis.declaring_side_stats().cards_played, 22);
        assert_eq!(analysis.defending_side_stats().cards_played, 23);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn a_backend_agnostic_dd_sequence_drives_the_analysis() {
        // Pure-function test: the DD sequence is synthetic, so the analysis is
        // exercised without a solver. Only the deltas between consecutive
        // values matter, and they are what `CardResult` reports.
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
        let cards = vec![
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
        ];
        let play_sequence = PlayRecord::new(squeezer_solver::test_fixtures::attribute_seats(
            &cards, contract,
        ));
        let dd_results = vec![
            7, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9,
        ];

        let analysis = analyze_hand(AnalysisInput {
            contract,
            play_sequence,
            dd_results,
        })
        .expect("the DD sequence covers every recorded play");

        let play = |seat: Seat, index: usize| {
            *analysis
                .player(seat)
                .plays()
                .nth(index)
                .expect("index is within the recorded plays")
        };

        assert_eq!(play(Seat::North, 0).card, Card::S2);
        assert_eq!(
            play(Seat::North, 0).result,
            CardResult::DdSuboptimal {
                dd_tricks: Tricks(8),
                dd_tricks_lost: TrickDelta(1)
            }
        );
        assert_eq!(play(Seat::East, 1).card, Card::HK);
        assert_eq!(play(Seat::East, 2).card, Card::H4);
        assert_eq!(
            play(Seat::East, 2).result,
            CardResult::DdSuboptimal {
                dd_tricks: Tricks(7),
                dd_tricks_lost: TrickDelta(1)
            }
        );
        assert_eq!(play(Seat::East, 4).card, Card::S8);
        assert_eq!(
            play(Seat::East, 4).result,
            CardResult::DdSuboptimal {
                dd_tricks: Tricks(8),
                dd_tricks_lost: TrickDelta(1)
            }
        );
        assert_eq!(play(Seat::East, 5).card, Card::CJ);
        assert_eq!(
            play(Seat::East, 5).result,
            CardResult::DdSuboptimal {
                dd_tricks: Tricks(9),
                dd_tricks_lost: TrickDelta(1)
            }
        );
    }

    #[cfg(feature = "dds")]
    #[test]
    fn analyze_board_rejects_a_board_without_a_play() {
        let board = squeezer_solver::test_fixtures::example_board();
        let error = analyze_board(&squeezer_solver::DdsSolver, &board)
            .expect_err("a board with no play cannot be analysed");
        assert!(matches!(error, AnalysisError::InvalidInput { .. }));
    }

    #[test]
    fn analyze_hand_handles_an_empty_and_partial_play() {
        let contract = hearts_contract();

        let empty = analyze_hand(AnalysisInput {
            contract,
            play_sequence: PlayRecord::new(Vec::new()),
            dd_results: vec![7],
        })
        .expect("the opening position is the only required result");
        assert!(empty.player(Seat::North).is_empty());
        assert!(empty.player(Seat::East).is_empty());
        assert!(empty.player(Seat::South).is_empty());
        assert!(empty.player(Seat::West).is_empty());

        let partial = analyze_hand(AnalysisInput {
            contract,
            play_sequence: PlayRecord::new(vec![
                Play::new(Seat::West, Card::SA),
                Play::new(Seat::North, Card::HA),
            ]),
            dd_results: vec![7, 8, 6],
        })
        .expect("both cards of the short final trick have a DD result");
        let west = partial.player(Seat::West).plays().next().copied();
        let north = partial.player(Seat::North).plays().next().copied();
        assert_eq!(west.map(|play| play.card), Some(Card::SA));
        assert_eq!(
            west.map(|play| play.result),
            Some(CardResult::DdSuboptimal {
                dd_tricks: Tricks(8),
                dd_tricks_lost: TrickDelta(1),
            })
        );
        assert_eq!(north.map(|play| play.card), Some(Card::HA));
        assert_eq!(
            north.map(|play| play.result),
            Some(CardResult::DdSuboptimal {
                dd_tricks: Tricks(6),
                dd_tricks_lost: TrickDelta(2),
            })
        );
    }

    #[test]
    fn analyze_hand_reports_missing_dd_results_in_a_short_final_trick() {
        let contract = hearts_contract();
        let error = analyze_hand(AnalysisInput {
            contract,
            play_sequence: PlayRecord::new(vec![
                Play::new(Seat::West, Card::SA),
                Play::new(Seat::North, Card::HA),
            ]),
            dd_results: vec![7, 8],
        })
        .expect_err("the second card of the short trick has no DD result");

        assert_eq!(
            error,
            AnalysisInputError::MissingTrickResult { play_index: 1 }
        );
    }

    #[test]
    fn analyze_hand_requires_the_initial_dd_position() {
        let contract = hearts_contract();
        let error = analyze_hand(AnalysisInput {
            contract,
            play_sequence: PlayRecord::new(Vec::new()),
            dd_results: Vec::new(),
        })
        .expect_err("an empty DD sequence has no starting position");

        assert_eq!(error, AnalysisInputError::MissingInitialTricks);
    }

    #[test]
    fn analyze_hand_rejects_a_play_longer_than_a_deal() {
        let contract = hearts_contract();
        let error = analyze_hand(AnalysisInput {
            contract,
            play_sequence: PlayRecord::new(vec![Play::new(Seat::West, Card::SA); 53]),
            dd_results: vec![7; 54],
        })
        .expect_err("a deal has only 52 cards");

        assert_eq!(error, AnalysisInputError::PlaySequenceTooLong { cards: 53 });
    }

    #[test]
    fn analyze_board_keeps_a_short_dd_sequence_as_a_typed_error() {
        /// An analyser that returns a fixed DD sequence, as a broken backend
        /// might.
        struct ScriptedAnalyser(Vec<u8>);

        impl PlayAnalyser for ScriptedAnalyser {
            type Error = std::convert::Infallible;

            fn analyse_play(&self, _board: &Board) -> Result<Vec<u8>, std::convert::Infallible> {
                Ok(self.0.clone())
            }
        }

        let board = squeezer_solver::test_fixtures::example_board_hand0_play();

        let error = analyze_board(&ScriptedAnalyser(Vec::new()), &board)
            .expect_err("an empty DD sequence cannot cover a recorded play");
        assert!(matches!(
            error,
            AnalysisError::PlaySequence(AnalysisInputError::MissingInitialTricks)
        ));

        let error = analyze_board(&ScriptedAnalyser(vec![7, 8]), &board)
            .expect_err("two values cannot cover a 45-card play");
        assert!(matches!(
            error,
            AnalysisError::PlaySequence(AnalysisInputError::MissingTrickResult { play_index: 1 })
        ));
    }

    #[test]
    fn analyze_hand_reports_a_complete_trick_without_dd_results() {
        let contract = hearts_contract();
        let error = analyze_hand(AnalysisInput {
            contract,
            play_sequence: PlayRecord::new(vec![
                Play::new(Seat::West, Card::SA),
                Play::new(Seat::North, Card::HA),
                Play::new(Seat::East, Card::DA),
                Play::new(Seat::South, Card::CA),
            ]),
            dd_results: vec![7],
        })
        .expect_err("a complete trick needs four DD results");

        assert_eq!(
            error,
            AnalysisInputError::MissingTrickResult { play_index: 0 }
        );
    }
}
