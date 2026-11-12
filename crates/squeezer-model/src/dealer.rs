// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! # Bridge deal generation
//!
//! This module owns deal generation in one place: the [`Dealer`] trait, its
//! builder ([`DealerBuilder`]), the rejection-sampling [`StandardDealer`], the
//! [`DealerError`] they return, and the cross-hand [`AcceptFunction`] seam.
//! The [`Deal`] type itself, with the rest of the record vocabulary, lives in
//! [`crate::record::deal`].
//!
//! ## Architecture Overview
//!
//! ```text
//! DealerBuilder -> StandardDealer -> Deal
//!      |               |             |
//!   Configure      Generate     Hand to the record layer
//! ```
//!
//! ## Key Features
//!
//! - **Predealing**: Assign specific cards to specific seats
//! - **Hand constraints**: Specify shape, HCP, etc. per seat
//! - **Custom acceptance**: Add arbitrary cross-hand validation logic
//! - **Bounded effort**: A rejection budget turns an impossible spec into a
//!   typed error instead of an endless loop
//!
//! ## Example Usage
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use squeezer_model::*;
//! # use squeezer_cards::*;
//! # use std::str::FromStr;
//!
//! // Simple deal with North predealed
//! let dealer = DealerBuilder::new()
//!     .predeal(Seat::North, Cards::from_str("SAKQHAKQDAKQCAKQJ")?)?
//!     .build()?;
//!
//! let deal = dealer.deal()?;
//! println!("{}", deal);
//!
//! // Complex constraint: NS 8+ card major fit
//! let dealer = DealerBuilder::new()
//!     .with_function(|deal: &Deal| {
//!         let ns_spades = deal.north().spades().len() + deal.south().spades().len();
//!         let ns_hearts = deal.north().hearts().len() + deal.south().hearts().len();
//!         ns_spades >= 8 || ns_hearts >= 8
//!     })
//!     .with_hand_constraint(
//!         Seat::North,
//!         HandConstraint::hcp(12..=37)
//!     )
//!     .build()?;
//! # Ok(())
//! # }
//! ```

use crate::prelude::*;

/// Type alias for cross-hand acceptance predicates.
///
/// The predicate receives the complete [`Deal`] and returns `true` if it meets
/// the criteria. Typically used for cross-hand constraints like "NS must have
/// an 8+ card spade fit" that cannot be expressed through individual hand
/// constraints.
///
/// The `Send + Sync` bound lets a dealer be shared across simulation threads;
/// plain (non-capturing or plain-data-capturing) closures satisfy it
/// automatically.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use squeezer_model::*;
/// # use squeezer_cards::*;
/// # use std::str::FromStr;
/// let accept: AcceptFunction = Box::new(|deal: &Deal| {
///     // Accept only if North-South have game-forcing values combined
///     deal.north().hcp() + deal.south().hcp() >= 25
/// });
/// # Ok(())
/// # }
/// ```
pub type AcceptFunction = Box<dyn Fn(&Deal) -> bool + Send + Sync>;

/// Default cap on rejection-sampling attempts.
///
/// The dealer never samples forever: once this many candidate deals have been
/// rejected it returns [`DealerError::Unsatisfiable`]. The value is chosen so
/// the tightest constraints shipped with the crate (a joint 18-20 HCP, six-card
/// minor and spades-longest filter, averaging ~10⁵ attempts) succeed with a
/// wide margin, while a genuinely impossible spec returns in bounded time
/// instead of hanging. Override it with [`DealerBuilder::with_attempt_budget`].
pub const DEFAULT_ATTEMPT_BUDGET: u32 = 2_000_000;

/// Error for anything the dealer or its conditioning can reject.
///
/// The historical free-form messages live in [`Self::Message`]. The
/// conditioning failures are typed, so a caller can tell an impossible
/// conditioning seed (a card fixed to two seats) from a rejection budget that
/// merely ran out ([`Self::Unsatisfiable`]). The shape and constraint variants
/// carry their module error as the [`std::error::Error::source`] rather than
/// flattening it into a string.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash)]
#[non_exhaustive]
pub enum DealerError {
    /// A free-form dealer message.
    Message(String),
    /// Conditioning fixed one card to two different seats.
    CardAttributedToTwoSeats {
        /// The offending card.
        card: Card,
        /// The seat seen first.
        first: Seat,
        /// The seat seen second.
        second: Seat,
    },
    /// Conditioning fixed more than 13 cards to one seat.
    ///
    /// This is the only size bound that needs its own variant: a hand holds
    /// at most 13 cards, so a per-suit overflow (`> 13` of one suit) and a
    /// deck overflow (`> 52` across seats) are already impossible once this
    /// bound and [`Self::CardAttributedToTwoSeats`] hold.
    TooManyKnownForSeat {
        /// The offending seat.
        seat: Seat,
        /// How many cards were fixed to it.
        count: u8,
    },
    /// Rejection sampling exhausted its bounded attempt budget.
    ///
    /// This is not a proof that no deal satisfies the constraints, only that
    /// the configured budget was too small to find one.
    Unsatisfiable {
        /// How many candidate deals were rejected before giving up.
        attempts: u32,
    },
    /// A shape pattern failed to scan, parse or interpret.
    Shape(CreationShapeError),
    /// A hand constraint could not be built.
    Constraint(ConstraintError),
    /// A hand string was not a valid run of cards.
    InvalidCards(InvalidCardError),
}

impl DealerError {
    #[must_use]
    #[inline]
    pub(crate) fn new<T: Into<String>>(msg: T) -> Self {
        Self::Message(msg.into())
    }
}

impl fmt::Display for DealerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(details) => write!(f, "{details}"),
            Self::CardAttributedToTwoSeats {
                card,
                first,
                second,
            } => write!(f, "card {card} is attributed to both {first} and {second}"),
            Self::TooManyKnownForSeat { seat, count } => {
                write!(
                    f,
                    "{seat} has {count} known cards, but a hand holds at most 13"
                )
            }
            Self::Unsatisfiable { attempts } => {
                write!(
                    f,
                    "no deal satisfied the constraints within {attempts} attempts"
                )
            }
            Self::Shape(cause) => write!(f, "{cause}"),
            Self::Constraint(cause) => write!(f, "{cause}"),
            Self::InvalidCards(cause) => write!(f, "{cause}"),
        }
    }
}

impl Error for DealerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Shape(cause) => Some(cause),
            Self::Constraint(cause) => Some(cause),
            Self::InvalidCards(cause) => Some(cause),
            Self::Message(_)
            | Self::CardAttributedToTwoSeats { .. }
            | Self::TooManyKnownForSeat { .. }
            | Self::Unsatisfiable { .. } => None,
        }
    }
}

impl From<CreationShapeError> for DealerError {
    fn from(value: CreationShapeError) -> Self {
        DealerError::Shape(value)
    }
}

impl From<ConstraintError> for DealerError {
    fn from(value: ConstraintError) -> Self {
        DealerError::Constraint(value)
    }
}

impl From<InvalidCardError> for DealerError {
    fn from(value: InvalidCardError) -> Self {
        DealerError::InvalidCards(value)
    }
}

/// Builder for creating a `Dealer` with specific constraints.
///
/// This is the primary way to create a dealer. It provides a fluent API for
/// configuring every aspect of deal generation: predealing, hand constraints
/// and acceptance functions.
///
/// # Builder Pattern
///
/// Methods can be chained and the builder is consumed by `.build()`:
///
/// # Ok(())
/// # }
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use squeezer_model::*;
/// # use squeezer_cards::*;
/// # use std::str::FromStr;
/// let dealer = DealerBuilder::new()
///     .predeal(Seat::North, Card::HQ.into())?
///     .with_hand_constraint(Seat::South, HandConstraint::hcp(2..15))
///     .with_function(|deal| /* validation */ true)
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// # Performance Considerations
///
/// - **Predealing**: Very fast - just removes cards from the available deck
/// - **Hand constraints**: Speed depends on constraint rarity (tight constraints = slower)
/// - **Accept functions**: Called only for candidates that pass every per-seat
///   constraint, so keep them fast
///
/// # Example: Complex Dealer
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use squeezer_model::*;
/// # use squeezer_cards::*;
/// # use std::str::FromStr;
/// let dealer = DealerBuilder::new()
///     .predeal(Seat::North, Cards::from_str("SAK")?)? // North has A-K of spades
///     .with_hand_constraint(
///         Seat::South,
///         HandConstraint::hcp(12..14)
///     )
///     .with_function(|deal| {
///         // NS must have 8+ spade fit
///         deal.north().spades().len() + deal.south().spades().len() >= 8
///     })
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct DealerBuilder {
    /// Function that validates the complete deal across all four hands
    accept: AcceptFunction,

    /// Available cards for random dealing (cards not predealed)
    deck: Cards,

    /// Per-seat constraints (shape, HCP, controls, etc.)
    hand_constraints: [Option<HandConstraint>; NUMBER_OF_HANDS],

    /// Cards to predeal to specific seats (can be partial hands)
    predealt_hands: [Option<Cards>; NUMBER_OF_HANDS],

    /// Cards fixed to a seat by conditioning beyond `predeal`: played cards
    /// (via [`DealerBuilder::with_played`]) and cards seeded directly (via
    /// [`DealerBuilder::with_known`]). The dealer's known set is this unioned
    /// with the predealt cards, per seat.
    conditioned_by: [Cards; NUMBER_OF_HANDS],

    /// The subset of `conditioned_by` that the seat has already played. Kept
    /// separately from `conditioned_by` so a candidate check can tell a card
    /// still in hand from one that is already gone.
    played_by: [Cards; NUMBER_OF_HANDS],

    /// Bounded rejection-sampling budget.
    attempt_budget: u32,
}

impl std::fmt::Debug for DealerBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DealerBuilder")
            .field("hand_constraints", &self.hand_constraints)
            .field("predealt_hands", &self.predealt_hands)
            .field("conditioned_by", &self.conditioned_by)
            .field("played_by", &self.played_by)
            .field("attempt_budget", &self.attempt_budget)
            .finish_non_exhaustive()
    }
}

impl Default for DealerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DealerBuilder {
    /// Creates a new builder with no constraints.
    ///
    /// The default configuration accepts all deals and has no predealing.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            accept: Box::new(|_: &Deal| true),
            hand_constraints: [None, None, None, None],
            predealt_hands: [None; 4],
            conditioned_by: [Cards::EMPTY; 4],
            played_by: [Cards::EMPTY; 4],
            deck: Cards::ALL,
            attempt_budget: DEFAULT_ATTEMPT_BUDGET,
        }
    }

    /// Predeals specific cards to a seat.
    ///
    /// Cards can be a partial hand (< 13 cards). The remaining cards will be
    /// dealt randomly from the available deck during generation.
    ///
    /// **Note**: This overwrites any previous predeal for this seat.
    ///
    /// # Errors
    ///
    /// Returns an error if any card in `hand` has already been dealt to another seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let builder = DealerBuilder::new()
    ///     .predeal(Seat::North, Cards::from_str("SAKQJ")?)?  // 5 spades
    ///     .predeal(Seat::South, Cards::from_str("HAK")?)?;   // 2 hearts
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn predeal(mut self, seat: Seat, hand: Cards) -> Result<Self, DealerError> {
        // Check for duplicate cards
        if !hand.difference(self.deck).is_empty() {
            return Err(DealerError::new(format!(
                "Cards already dealt: {}",
                hand.difference(self.deck)
            )));
        }

        // If this seat was already predealed, return those cards to the deck
        if let Some(cards_already_dealt) = self.predealt_hands[seat as usize] {
            self.deck += cards_already_dealt;
        }

        self.predealt_hands[seat as usize] = Some(hand);
        self.deck -= hand;
        Ok(self)
    }

    /// Predeals cards to all four seats at once.
    ///
    /// Each element can be `None` (no predeal for that seat) or `Some(cards)`.
    /// This is more efficient than calling `predeal()` four times separately.
    ///
    /// **Note**: This overwrites all previous predeals.
    ///
    /// # Errors
    ///
    /// Returns an error if any card appears in multiple hands.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let builder = DealerBuilder::new()
    ///     .predeal_all([
    ///         Some(Cards::from_str("SAKQJ")?),  // North
    ///         None,                              // East - no predeal
    ///         Some(Cards::from_str("HAK")?),     // South
    ///         None,                              // West - no predeal
    ///     ])?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn predeal_all(mut self, hands: [Option<Cards>; 4]) -> Result<Self, DealerError> {
        // Start with a full deck, removing previously predealed cards
        let mut tmp_deck = self.deck;
        for already_dealt_hand in self.predealt_hands.iter().flatten() {
            tmp_deck += *already_dealt_hand;
        }

        // Validate no duplicates and remove cards from deck
        for hand in hands.iter().flatten() {
            if hand.difference(tmp_deck).is_empty() {
                tmp_deck -= *hand;
            } else {
                return Err(DealerError::new(format!(
                    "Card dealt twice: {}",
                    hand.difference(tmp_deck)
                )));
            }
        }

        self.predealt_hands = hands;
        self.deck = tmp_deck;
        Ok(self)
    }

    /// Adds a single card to an existing predeal.
    ///
    /// This is useful for incremental updates, such as when analyzing a hand
    /// and learning about card locations one at a time.
    ///
    /// # Errors
    ///
    /// Returns an error if the card has already been dealt to any seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let mut builder = DealerBuilder::new();
    /// builder.add_predealt_card(Card::new(Suit::Spades,14), Seat::North)?;
    /// builder.add_predealt_card(Card::new(Suit::Spades, 13), Seat::North)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn add_predealt_card(&mut self, card: Card, seat: Seat) -> Result<(), DealerError> {
        let prev_deck = self.deck;
        let to_insert = card.into();
        self.deck -= to_insert;

        if prev_deck == self.deck {
            Err(DealerError::new("Card already dealt"))
        } else {
            *self.predealt_hands[seat as usize].get_or_insert(Cards::EMPTY) += to_insert;
            Ok(())
        }
    }

    /// Seeds every played card as a known card for the seat that played it.
    ///
    /// This is how a partially-known deal is modelled: the cards already
    /// played are fixed to their seats, and the rejection sampler fills each
    /// hand to 13 from the remaining deck. Calling it more than once unions
    /// the records; a card that ends up fixed to two different seats (played
    /// by two seats, or predealt to one and played by another) is rejected by
    /// [`DealerBuilder::build`], while a card both predealt and played by the
    /// same seat is simply one known card.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let record = PlayRecord::new(vec![
    ///     Play::new(Seat::West, Card::C2),
    ///     Play::new(Seat::North, Card::C3),
    /// ]);
    /// let dealer = DealerBuilder::new().with_played(&record).build()?;
    /// let deal = dealer.deal()?;
    /// assert!(deal.west().has_card(Card::C2));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn with_played(mut self, record: &PlayRecord) -> Self {
        for play in record.plays() {
            let seat = play.seat as usize;
            self.conditioned_by[seat] = self.conditioned_by[seat].insert(play.card);
            // Record the played subset too: a played card is fixed to the
            // seat's reconstructed original hand, but is no longer playable.
            self.played_by[seat] = self.played_by[seat].insert(play.card);
        }
        self
    }

    /// Seeds cards as known for a seat directly.
    ///
    /// The escape hatch for information that is not a `PlayRecord` (for
    /// example, cards revealed by a claim or an opponent's explanation). The
    /// cards join the predealt and played cards in the seat's known set and,
    /// unlike played cards, are treated as still in hand by
    /// [`Dealer::current_hand`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let known = Cards::from_str("HAK")?;
    /// let dealer = DealerBuilder::new().with_known(Seat::South, known).build()?;
    /// let deal = dealer.deal()?;
    /// assert!(deal.south().contains(known));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn with_known(mut self, seat: Seat, cards: Cards) -> Self {
        self.conditioned_by[seat as usize] = self.conditioned_by[seat as usize].union(cards);
        self
    }

    /// Sets the bounded rejection-sampling budget.
    ///
    /// The dealer rejects at most this many candidate deals before returning
    /// [`DealerError::Unsatisfiable`]. Defaults to
    /// [`DEFAULT_ATTEMPT_BUDGET`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let dealer = DealerBuilder::new().with_attempt_budget(500).build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[inline]
    pub fn with_attempt_budget(mut self, attempts: u32) -> Self {
        self.attempt_budget = attempts;
        self
    }

    /// Sets a custom acceptance function for cross-hand validation.
    ///
    /// This function runs after the per-seat hand constraints: the dealer calls
    /// it only for candidates whose every seat passed its constraint. A
    /// candidate that fails a seat's constraint is discarded before the
    /// function is reached. Use it for constraints that involve multiple hands,
    /// such as:
    /// - Partnership fits (e.g., "NS must have 8+ spade fit")
    /// - Combined HCP (e.g., "NS must have 25+ combined HCP")
    /// - Distributional requirements across hands
    ///
    /// **Performance note**: This function runs once per candidate that passes
    /// the per-seat constraints; rejected candidates never reach it. Keep it as
    /// fast as possible, since it is on the path of every accepted deal.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let builder = DealerBuilder::new()
    ///     .with_function(|deal: &Deal| {
    ///         // Require NS to have game-forcing values
    ///         let ns_hcp = deal.north().hcp() + deal.south().hcp();
    ///         ns_hcp >= 25
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn with_function<T: Fn(&Deal) -> bool + Send + Sync + 'static>(
        mut self,
        accept_function: T,
    ) -> Self {
        self.accept = Box::new(accept_function);
        self
    }

    /// Sets the hand constraint for a specific seat.
    ///
    /// This is the only per-seat constraint entry point: the
    /// [`HandConstraint`] parses its patterns once here and the dealer runs a
    /// single fused lookup per candidate.
    ///
    /// **Note**: This overwrites any previous constraint for this seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let builder = DealerBuilder::new()
    ///     .with_hand_constraint(
    ///         Seat::North,
    ///         HandConstraint::hcp(12..=17)
    ///     );
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hand_constraint(mut self, seat: Seat, constraint: HandConstraint) -> Self {
        self.hand_constraints[seat as usize] = Some(constraint);
        self
    }

    /// Sets hand constraints for all four seats at once.
    ///
    /// Each element can be `None` (no constraints) or `Some(constraint)`.
    ///
    /// **Note**: This overwrites all previous hand constraints.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let builder = DealerBuilder::new()
    ///     .with_hand_constraints_all([
    ///         Some(HandConstraint::hcp(12..14)),
    ///         None,  // East - no constraints
    ///         Some(HandConstraint::hcp(12..14)),
    ///         None,  // West - no constraints
    ///     ]);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hand_constraints_all(mut self, constraints: [Option<HandConstraint>; 4]) -> Self {
        self.hand_constraints = constraints;
        self
    }

    /// Builds the dealer from this configuration.
    ///
    /// Consumes the builder and returns a `StandardDealer` ready to generate deals.
    ///
    /// # Errors
    ///
    /// Returns [`DealerError`] when the conditioning is structurally
    /// impossible before any sampling happens: a card fixed to two different
    /// seats, or more than 13 known cards for a seat.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let dealer = DealerBuilder::new()
    ///     .predeal(Seat::North, Card::SK.into())?
    ///     .build()?;
    ///
    /// let deal = dealer.deal()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn build(self) -> Result<impl Dealer, DealerError> {
        let known = self.resolve_known()?;

        // The random deck is whatever the known cards leave. Computing it
        // here (rather than trusting the incremental `deck` bookkeeping) means
        // conditioning seeded by `with_played`/`with_known` is excluded too.
        let mut deck = Cards::ALL;
        for cards in known {
            deck -= cards;
        }

        // Resolve the fill order before moving the builder's fields out.
        let fill_order = self.fill_order();

        Ok(StandardDealer {
            predeal: self.predealt_hands,
            known,
            played: self.played_by,
            deck_starting_state: deck,
            accept_function: self.accept,
            fill_order,
            hand_constraints: self.hand_constraints,
            attempt_budget: self.attempt_budget,
        })
    }

    /// Computes the order in which [`StandardDealer`] fills and checks seats.
    ///
    /// Seats carrying a [`HandConstraint`] come first, so a candidate that
    /// cannot satisfy one of them is rejected after the first hand rather than
    /// the fourth. Within each tier the canonical North, East, South, West
    /// order is kept. Known cards never move a seat between tiers: a seat with
    /// thirteen known cards still belongs to the group its constraint puts it
    /// in, because its check outcome is constant for the run.
    ///
    /// The order is fixed for the dealer's lifetime; it is an implementation
    /// detail, not part of the builder's configuration.
    #[inline]
    fn fill_order(&self) -> [Seat; NUMBER_OF_HANDS] {
        let mut order = [Seat::North; NUMBER_OF_HANDS];
        let mut next = 0;
        for constrained in [true, false] {
            for seat in Seat::iter() {
                if self.hand_constraints[seat as usize].is_some() == constrained {
                    order[next] = seat;
                    next += 1;
                }
            }
        }
        order
    }

    /// Resolves the per-seat known set and validates it up front.
    ///
    /// `known[seat] = predeal[seat] ∪ conditioned_by[seat]`, deduplicated per
    /// seat. A card fixed to two different seats cannot be placed in a hand,
    /// so it is an error here rather than a silent misdeal.
    ///
    /// Two bounds from the design need no separate check once those hold: a
    /// per-suit overflow (`> 13` of one suit) is impossible when the whole
    /// hand is at most 13 cards, and a deck overflow (`> 52` across seats) is
    /// impossible when no card is shared and each seat is at most 13.
    #[inline]
    fn resolve_known(&self) -> Result<[Cards; NUMBER_OF_HANDS], DealerError> {
        let mut known = [Cards::EMPTY; NUMBER_OF_HANDS];
        for seat in Seat::iter() {
            let predealt = self.predealt_hands[seat as usize].unwrap_or(Cards::EMPTY);
            known[seat as usize] = predealt.union(self.conditioned_by[seat as usize]);
        }

        // Walk the seats once, accumulating what has already been assigned.
        // Any overlap is a card attributed to two seats.
        let mut assigned = Cards::EMPTY;
        for seat in Seat::iter() {
            let overlap = known[seat as usize] & assigned;
            if !overlap.is_empty() {
                let card = overlap.into_iter().next().expect("overlap is non-empty");
                let first = Seat::iter()
                    .find(|earlier| known[*earlier as usize].has_card(card))
                    .expect("overlap came from an earlier seat");
                return Err(DealerError::CardAttributedToTwoSeats {
                    card,
                    first,
                    second: seat,
                });
            }
            assigned += known[seat as usize];
        }

        for seat in Seat::iter() {
            let count = known[seat as usize].len();
            if count > 13 {
                return Err(DealerError::TooManyKnownForSeat { seat, count });
            }
        }

        Ok(known)
    }

    /// Sets both predealing and hand constraints for all seats at once.
    ///
    /// This is a convenience method combining `predeal_all()` and
    /// `with_hand_constraints_all()`.
    ///
    /// # Errors
    ///
    /// Returns an error if any cards are duplicated across hands.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use squeezer_model::*;
    /// # use squeezer_cards::*;
    /// # use std::str::FromStr;
    /// let config = [
    ///     (Some(Card::HQ.into()), Some(HandConstraint::hcp(12..14))),
    ///     (None, None),  // East - no constraints
    ///     (Some(Card::HK.into()), Some(HandConstraint::hcp(12..14))),
    ///     (None, None),  // West - no constraints
    /// ];
    /// let builder = DealerBuilder::new().with_complete_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn with_complete_config(
        self,
        config: [(Option<Cards>, Option<HandConstraint>); 4],
    ) -> Result<Self, DealerError> {
        let (cards, constraints): (Vec<_>, Vec<_>) = config.into_iter().unzip();
        // Unzipping a four-element array yields four-element vectors.
        self.with_hand_constraints_all(
            constraints
                .try_into()
                .expect("unzip keeps the four-element length"),
        )
        .predeal_all(
            cards
                .try_into()
                .expect("unzip keeps the four-element length"),
        )
    }
}

/// Trait for objects that can generate bridge deals.
///
/// This trait abstracts the deal generation process. The main implementation
/// is `StandardDealer`, but you could implement custom dealers with different
/// algorithms or constraints.
pub trait Dealer {
    /// Generates a single bridge deal satisfying all constraints.
    ///
    /// [`StandardDealer`] checks each seat's hand constraint as soon as that
    /// seat's cards are dealt: a failure discards the candidate immediately, so
    /// the remaining seats are not filled and the cross-hand acceptance function
    /// is not called. That function therefore only ever sees candidates that
    /// passed every per-seat constraint.
    ///
    /// For [`StandardDealer`], the seats are visited in its fixed fill order:
    /// seats with a [`HandConstraint`] first, then those without, keeping
    /// North, East, South, West inside each tier. Reordering how the hands of
    /// a fixed partition are drawn does not change the partition's
    /// distribution, so this only moves where the check cost lands.
    ///
    /// # Errors
    ///
    /// Returns `DealerError` if:
    /// - The deck doesn't have enough cards (invalid predeal configuration)
    /// - The bounded rejection budget is exhausted before a deal satisfies the
    ///   constraints ([`DealerError::Unsatisfiable`])
    ///
    /// # Performance
    ///
    /// Generation time varies based on constraint tightness. Tighter constraints
    /// require more candidate deals to be generated and rejected.
    fn deal(&self) -> Result<Deal, DealerError>;

    /// Returns the predealed cards for each seat.
    ///
    /// Used internally and for inspection/debugging. This is the raw
    /// predeal only; see [`Dealer::known`] for the full conditioning seed.
    fn predealt(&self) -> [Option<Cards>; NUMBER_OF_HANDS];

    /// Returns the known cards for each seat.
    ///
    /// The conditioning seed: predealt ∪ played ∪ directly-known, deduplicated
    /// per seat. These are the cards the sampler fixes before filling each
    /// hand to 13.
    fn known(&self) -> [Cards; NUMBER_OF_HANDS];

    /// Returns the cards `seat` can still play.
    ///
    /// This is [`Dealer::known`] minus the cards `seat` has already played:
    /// conditioning fixes a played card to the seat's reconstructed original
    /// hand, but that card is no longer available. Card-EV candidate validation
    /// uses this so a played card cannot pass as a candidate.
    fn current_hand(&self, seat: Seat) -> Cards;
}

/// Standard implementation of the `Dealer` trait.
///
/// This dealer uses rejection sampling: it generates random deals and accepts
/// only those that satisfy all constraints. The algorithm:
///
/// 1. Start with the deck left by the known cards
/// 2. For each seat in the dealer's fill order:
///    - Fill the seat from its known cards plus a random draw
///    - Check that seat's hand constraint; a failure discards the candidate
///      at once, without filling the remaining seats
/// 3. Check the acceptance function, which therefore only sees candidates that
///    passed every per-seat constraint
/// 4. If accepted, return the deal; otherwise, try again
///
/// # Fill order
///
/// The order is computed once when the dealer is built: seats carrying a
/// [`HandConstraint`] come first, then the unconstrained seats, with North,
/// East, South, West kept inside each tier. This makes the same constraint cost
/// roughly the same wherever it sits, because a doomed candidate dies after the
/// first hand instead of the third or fourth. Known cards never change a seat's
/// tier.
///
/// Any fixed fill order produces the same distribution of deals: it only
/// changes the order in which the hands of the partition are drawn from the
/// shared deck, not which partitions are accepted.
///
/// # Performance Characteristics
///
/// - **No constraints**: ~1-5 microseconds per deal
/// - **Light constraints**: ~10-50 microseconds
/// - **Tight constraints**: Can be much slower (milliseconds)
///
/// Very tight constraints (e.g., "exactly 6-5-2-0 with 15 HCP") may require
/// thousands of attempts and can be slow.
///
/// # Memory
///
/// The dealer is lightweight (~200 bytes) and cheap to create. The expensive
/// part is deal generation, not dealer creation.
pub struct StandardDealer {
    /// Cards to predeal to each seat
    predeal: [Option<Cards>; NUMBER_OF_HANDS],
    /// Cards fixed to each seat: `predeal ∪ played ∪ directly-known`.
    known: [Cards; NUMBER_OF_HANDS],
    /// Cards each seat has already played; a subset of `known`.
    played: [Cards; NUMBER_OF_HANDS],
    /// Available cards for random dealing
    deck_starting_state: Cards,
    /// The order in which seats are filled and checked for each candidate.
    ///
    /// Constrained seats first, then unconstrained ones, N, E, S, W inside
    /// each tier. Fixed when the dealer is built; see
    /// [`DealerBuilder::fill_order`].
    fill_order: [Seat; NUMBER_OF_HANDS],
    /// Per-seat constraints
    hand_constraints: [Option<HandConstraint>; NUMBER_OF_HANDS],
    /// Cross-hand validation function
    accept_function: AcceptFunction,
    /// Bounded rejection-sampling budget.
    attempt_budget: u32,
}

impl std::fmt::Debug for StandardDealer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StandardDealer")
            .field("predeal", &self.predeal)
            .field("known", &self.known)
            .field("played", &self.played)
            .field("fill_order", &self.fill_order)
            .field("hand_constraints", &self.hand_constraints)
            .field("attempt_budget", &self.attempt_budget)
            .finish_non_exhaustive()
    }
}

impl StandardDealer {
    /// Creates a new dealer with no constraints.
    ///
    /// Prefer using `DealerBuilder` for a more ergonomic API.
    #[must_use]
    pub fn new() -> Self {
        Self {
            predeal: [None; 4],
            known: [Cards::EMPTY; 4],
            played: [Cards::EMPTY; 4],
            deck_starting_state: Cards::ALL,
            // With no constraints every seat is in the same tier, so the
            // canonical positional order is also the constrained-first order.
            fill_order: [Seat::North, Seat::East, Seat::South, Seat::West],
            hand_constraints: [None, None, None, None],
            accept_function: Box::new(|_: &Deal| true),
            attempt_budget: DEFAULT_ATTEMPT_BUDGET,
        }
    }
}

impl Default for StandardDealer {
    fn default() -> Self {
        Self::new()
    }
}

impl Dealer for StandardDealer {
    #[inline]
    fn predealt(&self) -> [Option<Cards>; NUMBER_OF_HANDS] {
        self.predeal
    }

    #[inline]
    fn known(&self) -> [Cards; NUMBER_OF_HANDS] {
        self.known
    }

    #[inline]
    fn current_hand(&self, seat: Seat) -> Cards {
        self.known[seat as usize].difference(self.played[seat as usize])
    }

    /// Generates a deal using rejection sampling.
    ///
    /// This implementation fixes each seat's known cards, fills the remainder
    /// from the available deck, and checks each seat's constraint as soon as
    /// that hand is filled. It tries at most `attempt_budget` candidate deals.
    ///
    /// # Algorithm
    ///
    /// ```text
    /// for _ in 0..attempt_budget {
    ///     1. Start with the deck left by the known cards
    ///     2. For each seat:
    ///        - Start from its known cards
    ///        - Fill the remainder from the deck
    ///        - If its hand constraint fails, discard the candidate:
    ///          do not fill the remaining seats, do not run the closure
    ///     3. Check the acceptance function on the complete deal
    ///     4. If it accepts: return deal
    ///     5. Otherwise: try again
    /// }
    /// return Unsatisfiable
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`DealerError::Unsatisfiable`] when the budget is exhausted, or
    /// a deck error if the conditioning left too few cards to complete a hand.
    #[inline]
    fn deal(&self) -> Result<Deal, DealerError> {
        let mut hands: [Cards; 4] = [Cards::EMPTY; 4];
        // Hoist the RNG out of the rejection loop: creating a thread-local RNG
        // for every one of the four picks per attempt is needlessly expensive.
        let mut rng = rand::rng();

        'candidates: for _ in 0..self.attempt_budget {
            let mut deck = self.deck_starting_state;

            // Deal to each seat in the fixed fill order: known cards first,
            // then the remainder from the shared deck. Dealing from one deck
            // keeps the 52-card partition, so the hot path skips re-checking
            // it.
            for &seat in &self.fill_order {
                let known = self.known[seat as usize];
                let fill = 13 - usize::from(known.len());
                // Once the remaining cards exactly fill this seat, there is no
                // sampling decision left. Take them directly so the final hand
                // does not spend RNG draws re-discovering a deterministic set.
                let cards_to_add = if fill == usize::from(deck.len()) {
                    deck.take_all()
                } else {
                    let Some(cards_to_add) = deck.pick_rng::<_, FastSampler>(&mut rng, fill) else {
                        return Err(DealerError::new(
                            "Deck doesn't contain enough cards. Check predeal configuration.",
                        ));
                    };
                    cards_to_add
                };
                hands[seat as usize] = known + cards_to_add;

                // A seat that fails its constraint dooms the candidate, so
                // stop dealing and never reach the cross-hand closure.
                if self.hand_constraints[seat as usize]
                    .as_ref()
                    .is_some_and(|constraint| !constraint.check(hands[seat as usize]))
                {
                    continue 'candidates;
                }
            }

            let deal = Deal::from_cards_unchecked(hands);

            if (self.accept_function)(&deal) {
                return Ok(deal);
            }
        }

        Err(DealerError::Unsatisfiable {
            attempts: self.attempt_budget,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that basic deal generation works without panicking.
    #[test]
    fn test_basic_deal_generation() {
        let dealer = DealerBuilder::new().build().unwrap();
        let deal = dealer.deal().unwrap();

        // All hands should have exactly 13 cards
        for hand in deal {
            assert_eq!(hand.len(), 13);
        }
    }

    /// Tests predealing functionality.
    #[test]
    fn test_predeal() {
        let cards = Cards::from_str("SAKQJ").unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, cards)
            .unwrap()
            .build()
            .unwrap();

        let deal = dealer.deal().unwrap();

        // North should have all predealed cards
        assert!(deal.north().contains(cards));
    }

    /// Tests that predealing the same card twice fails.
    #[test]
    fn test_duplicate_predeal_fails() {
        let cards1 = Cards::from_str("SAK").unwrap();
        let cards2 = Cards::from_str("SAQ").unwrap(); // SA is duplicated

        let result = DealerBuilder::new()
            .predeal(Seat::North, cards1)
            .unwrap()
            .predeal(Seat::South, cards2);

        assert!(result.is_err());
    }
    /// Dealers are shareable across simulation threads: no `Cell`, and the
    /// acceptance predicate is `Send + Sync`.
    #[test]
    fn test_dealer_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StandardDealer>();
    }
    /// Tests that acceptance functions work correctly.
    #[test]
    fn test_accept_function() {
        let dealer = DealerBuilder::new()
            .with_function(|deal: &Deal| {
                // Only accept if North has at least 20 HCP
                deal.north().hcp() >= 20
            })
            .build()
            .unwrap();

        let deal = dealer.deal().unwrap();
        assert!(deal.north().hcp() >= 20);
    }
    /// A card predealt and played by the same seat is one known card, not a
    /// duplicate error.
    #[test]
    fn conditioning_dedups_same_seat_overlap() {
        let predeal = Cards::from_str("SAK").expect("predeal parses");
        let record = PlayRecord::new(vec![
            Play::new(Seat::North, Card::SA),
            Play::new(Seat::East, Card::H2),
        ]);
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, predeal)
            .expect("predeal is unique")
            .with_played(&record)
            .build()
            .expect("same-seat overlap is not a conflict");

        assert_eq!(dealer.known()[Seat::North as usize], predeal);
        assert_eq!(dealer.known()[Seat::East as usize], Cards::from(Card::H2));

        // A played card stays in the known (original-hand) seed but drops out
        // of the current hand.
        assert_eq!(dealer.current_hand(Seat::North), Cards::from(Card::SK));
        assert_eq!(dealer.current_hand(Seat::East), Cards::EMPTY);

        let deal = dealer.deal().expect("dealer deals");
        assert!(deal.north().contains(predeal));
        assert!(deal.east().has_card(Card::H2));
        // A played card is fixed to its seat, so no other hand can hold it.
        for seat in [Seat::North, Seat::South, Seat::West] {
            assert!(!deal[seat].has_card(Card::H2));
        }
    }
    /// `with_known` seeds the known set directly, without a `PlayRecord`.
    #[test]
    fn with_known_seeds_cards_directly() {
        let known = Cards::from_str("HAK").expect("known cards parse");
        let dealer = DealerBuilder::new()
            .with_known(Seat::South, known)
            .build()
            .expect("directly-known cards are valid");
        assert_eq!(dealer.known()[Seat::South as usize], known);
        let deal = dealer.deal().expect("dealer deals");
        assert!(deal.south().contains(known));
    }
    /// A card fixed to two different seats is a structural error.
    #[test]
    fn card_attributed_to_two_seats_is_an_error() {
        // Predealt to North, played by East.
        let record = PlayRecord::new(vec![Play::new(Seat::East, Card::SA)]);
        let Err(err) = DealerBuilder::new()
            .predeal(Seat::North, Cards::from(Card::SA))
            .expect("predeal is unique")
            .with_played(&record)
            .build()
        else {
            panic!("a card cannot be in two seats");
        };
        match err {
            DealerError::CardAttributedToTwoSeats {
                card,
                first,
                second,
            } => {
                assert_eq!(card, Card::SA);
                assert_eq!(first, Seat::North);
                assert_eq!(second, Seat::East);
            }
            other => panic!("expected a cross-seat conflict, got {other:?}"),
        }

        // Played by two different seats.
        let record = PlayRecord::new(vec![
            Play::new(Seat::North, Card::SA),
            Play::new(Seat::East, Card::SA),
        ]);
        let Err(err) = DealerBuilder::new().with_played(&record).build() else {
            panic!("a card cannot be played by two seats");
        };
        assert!(matches!(
            err,
            DealerError::CardAttributedToTwoSeats { card, .. } if card == Card::SA
        ));
    }
    /// More than 13 known cards for one seat is rejected before sampling.
    #[test]
    fn too_many_known_for_one_seat_is_an_error() {
        let fourteen = Cards::from_str("SAKQJT98765432H2").expect("14 distinct cards parse");
        assert_eq!(fourteen.len(), 14);
        let Err(err) = DealerBuilder::new()
            .with_known(Seat::North, fourteen)
            .build()
        else {
            panic!("a hand holds at most 13 cards");
        };
        assert!(matches!(
            err,
            DealerError::TooManyKnownForSeat {
                seat: Seat::North,
                count: 14
            }
        ));
    }
    /// Constraints run on the reconstructed 13-card hand, not the remainder.
    ///
    /// North has nine known black cards, so its remainder is four cards. The
    /// constraint can only be satisfied by the reconstructed hand: the four
    /// random cards must all be hearts.
    #[test]
    fn conditioning_constraints_check_reconstructed_full_hands() {
        let predeal = Cards::from_str("SAK").expect("predeal parses");
        let played: Vec<Play> = ["S2", "S3", "S4", "S5", "S6", "S7", "S8"]
            .iter()
            .map(|rank| Play::new(Seat::North, Card::from_str(rank).expect("card parses")))
            .collect();
        let record = PlayRecord::new(played);

        let dealer = DealerBuilder::new()
            .predeal(Seat::North, predeal)
            .expect("predeal is unique")
            .with_played(&record)
            .with_function(|deal: &Deal| deal.north().hearts().len() == 4)
            .build()
            .expect("conditioning is valid");

        let deal = dealer.deal().expect("dealer finds a 4-heart hand");
        assert_eq!(deal.north().len(), 13);
        assert_eq!(deal.north().hearts().len(), 4);
        assert!(deal.north().contains(predeal));
    }
    /// Nine spades fixed to North (two predealt, seven played); the caller adds
    /// the constraint under test.
    fn conditioned_north_spades_dealer(builder: DealerBuilder) -> impl Dealer {
        let predeal = Cards::from_str("SAK").expect("predeal parses");
        let played: Vec<Play> = ["S2", "S3", "S4", "S5", "S6", "S7", "S8"]
            .iter()
            .map(|rank| Play::new(Seat::North, Card::from_str(rank).expect("card parses")))
            .collect();
        builder
            .predeal(Seat::North, predeal)
            .expect("predeal is unique")
            .with_played(&PlayRecord::new(played))
            .build()
            .expect("conditioning is valid")
    }
    /// A `HandConstraint` also runs on the reconstructed 13-card hand: the
    /// nine known spades force the shape `9400`, so the four random cards must
    /// all be hearts.
    #[test]
    fn conditioning_hand_constraint_checks_reconstructed_full_hand() {
        let dealer = conditioned_north_spades_dealer(DealerBuilder::new().with_hand_constraint(
            Seat::North,
            HandConstraint::new(&["9400"], HcpRange::new(0, 37)).expect("shape parses"),
        ));
        let deal = dealer.deal().expect("dealer finds a 9-4-0-0 hand");
        assert_eq!(deal.north().spades().len(), 9);
        assert_eq!(deal.north().hearts().len(), 4);
    }
    /// A bounded budget returns a distinct error instead of sampling forever.
    #[test]
    fn exhausted_attempt_budget_is_a_bounded_error() {
        let dealer = DealerBuilder::new()
            .with_attempt_budget(10)
            .with_function(|_: &Deal| false)
            .build()
            .expect("a rejecting dealer still builds");
        let err = dealer
            .deal()
            .expect_err("no deal can satisfy the rejection");
        assert!(matches!(err, DealerError::Unsatisfiable { attempts: 10 }));
    }
    /// Tests adding predealt cards incrementally.
    #[test]
    fn test_incremental_predeal() {
        let mut builder = DealerBuilder::new();

        builder
            .add_predealt_card(Card::new(Suit::Spades, 14), Seat::North)
            .unwrap();

        builder
            .add_predealt_card(Card::new(Suit::Spades, 13), Seat::North)
            .unwrap();

        let dealer = builder.build().unwrap();
        let deal = dealer.deal().unwrap();

        assert!(deal.north().has_card(Card::new(Suit::Spades, 14)));
        assert!(deal.north().has_card(Card::new(Suit::Spades, 13)));
    }
    /// Tests that adding duplicate card fails.
    #[test]
    fn test_duplicate_incremental_predeal_fails() {
        let mut builder = DealerBuilder::new();

        builder
            .add_predealt_card(Card::new(Suit::Spades, 14), Seat::North)
            .unwrap();

        let result = builder.add_predealt_card(Card::new(Suit::Spades, 14), Seat::South);

        assert!(result.is_err());
    }
    /// Tests complete configuration.
    #[test]
    fn test_complete_config() {
        let cards_north = Cards::from_str("SAK").unwrap();
        let cards_south = Cards::from_str("HAK").unwrap();

        let config = [
            (Some(cards_north), None),
            (None, None),
            (Some(cards_south), None),
            (None, None),
        ];

        let dealer = DealerBuilder::new()
            .with_complete_config(config)
            .unwrap()
            .build()
            .unwrap();

        let deal = dealer.deal().unwrap();
        assert!(deal.north().contains(cards_north));
        assert!(deal.south().contains(cards_south));
    }
    #[test]
    fn can_deal_test() {
        _ = DealerBuilder::new()
            .build()
            .expect("unconstrained dealer builds")
            .deal()
            .expect("unconstrained dealer deals");
    }
    #[test]
    #[should_panic]
    #[allow(clippy::should_panic_without_expect)]
    fn predealing_twice_should_panic() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, hand)
            .unwrap()
            .predeal(Seat::West, hand)
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north(), hand);
    }
    #[test]
    fn dealer_builder_test() {
        let dealer_builder = DealerBuilder::new();
        _ = dealer_builder.build();
    }
    #[test]
    fn dealer_deals_test() {
        let db = DealerBuilder::new();
        let dealer = db.build().unwrap();
        _ = dealer.deal().unwrap();
    }
    #[test]
    fn dealer_deals_with_predeal_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, hand)
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north(), hand);
    }
    #[test]
    fn dealer_deals_with_predeal_all_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let hands = [Some(hand), None, None, None];
        let dealer = DealerBuilder::new()
            .predeal_all(hands)
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north(), hand);
    }
    #[test]
    fn dealer_deals_with_predeal_all_twice_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let hande = Cards::from_str("S432H432D432CT987").unwrap();
        let hand_2 = Cards::from_str("SAKQHAKQDAKQC6543").unwrap();
        let predeal_hands = [Some(hand), Some(hande), None, None];
        let dealer = DealerBuilder::new()
            .predeal_all(predeal_hands)
            .unwrap()
            .predeal_all([Some(hand_2), None, None, None])
            .unwrap()
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert_eq!(deal.north(), hand_2);
        assert_ne!(deal.east(), hande);
    }
    #[test]
    #[should_panic]
    fn dealer_deals_with_predeal_all_test_panic() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let hand_p = Cards::from_str("CQ").unwrap();
        let hands = [Some(hand), Some(hand_p), None, None];
        let _ = DealerBuilder::new().predeal_all(hands).unwrap();
    }
    #[test]
    fn dealer_deals_with_predeal_and_accept_function_test() {
        let hand = Cards::from_str("SAKQHAKQDAKQCAKQJ").unwrap();
        let dealer = DealerBuilder::new()
            .predeal(Seat::North, hand)
            .unwrap()
            .with_function(Box::new(|deal: &Deal| {
                deal.north().len_of_suit(Suit::Spades) + deal.south().len_of_suit(Suit::Spades) > 8
            }))
            .build()
            .unwrap();
        let deal = dealer.deal().unwrap();
        assert!(
            deal.north().len_of_suit(Suit::Spades) + deal.south().len_of_suit(Suit::Spades) > 8
        );
    }
}
