// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The auction: an ordered list of calls with no stored seats.
//!
//! Attribution is positional: the `i`-th call was made by `dealer + i`. Keeping
//! seats out of the data means a [`Auction`] can be sliced, reversed or built by
//! an importer without inventing seat tags, while still exposing the rotation
//! through [`Auction::seat`].

use std::num::NonZeroU8;

use crate::contract::{Contract, Doubled, Seat, Strain, Vulnerability, Vulnerable};
use crate::prelude::*;

/// A single call in an auction: a pass, a double, a redouble, or a bid.
///
/// The `Bid` variant names a level and strain rather than a full
/// [`Contract`], because a call carries no declarer or vulnerability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Call {
    /// A pass.
    Pass,
    /// A double of the last bid.
    Double,
    /// A redouble of the last double.
    Redouble,
    /// A contract bid: a level (1-7) and a strain.
    Bid {
        /// The bid level, guaranteed non-zero.
        level: NonZeroU8,
        /// The denomination.
        strain: Strain,
    },
}

impl Call {
    /// True for a contract bid (`Pass`, `Double` and `Redouble` are false).
    #[must_use]
    pub const fn is_bid(&self) -> bool {
        matches!(self, Self::Bid { .. })
    }

    /// The bid's level, if this is a bid.
    #[must_use]
    pub const fn level(&self) -> Option<NonZeroU8> {
        match self {
            Self::Bid { level, .. } => Some(*level),
            _ => None,
        }
    }

    /// The bid's strain, if this is a bid.
    #[must_use]
    pub const fn strain(&self) -> Option<Strain> {
        match self {
            Self::Bid { strain, .. } => Some(*strain),
            _ => None,
        }
    }
}

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "Pass"),
            Self::Double => write!(f, "Double"),
            Self::Redouble => write!(f, "Redouble"),
            Self::Bid { level, strain } => write!(f, "{level}{strain}"),
        }
    }
}

/// A call plus the metadata LIN attaches to it.
///
/// `alert` records the `!` marker; `annotation` records an `an|...|`
/// explanation. Rooms and other file metadata stay in the importer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordedCall {
    /// The call itself.
    pub call: Call,
    /// Whether the call was alerted.
    pub alert: bool,
    /// Optional explanation attached to the call.
    pub annotation: Option<String>,
}

impl RecordedCall {
    /// A plain, unalerted call.
    #[must_use]
    pub const fn new(call: Call) -> Self {
        Self {
            call,
            alert: false,
            annotation: None,
        }
    }
}

/// Why a sequence of calls is not a legal auction.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuctionError {
    /// The auction starts with a `Double` or `Redouble`.
    NonStarter {
        /// The offending call.
        call: Call,
        /// The seat that made it.
        seat: Seat,
    },
    /// A bid that does not outrank the previous bid.
    Insufficient {
        /// The highest bid so far.
        previous: Call,
        /// The bid that failed to outrank it.
        next: Call,
        /// The seat that made the insufficient bid.
        seat: Seat,
    },
    /// A `Double` or `Redouble` that the seat is not entitled to make.
    IllegalDouble {
        /// The offending call.
        call: Call,
        /// The seat that made it.
        seat: Seat,
    },
    /// A call made after the auction had already ended.
    CallAfterEnd {
        /// The offending call.
        call: Call,
        /// The seat that made it.
        seat: Seat,
    },
}

impl std::fmt::Display for AuctionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonStarter { call, seat } => {
                write!(f, "{seat} cannot open the auction with {call}")
            }
            Self::Insufficient {
                previous,
                next,
                seat,
            } => write!(f, "{seat} bid {next}, which does not outrank {previous}"),
            Self::IllegalDouble { call, seat } => {
                write!(f, "{seat} is not entitled to {call}")
            }
            Self::CallAfterEnd { call, seat } => {
                write!(f, "{seat} called {call} after the auction had ended")
            }
        }
    }
}

impl std::error::Error for AuctionError {}

/// An ordered auction with no stored seats.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Auction {
    calls: Vec<RecordedCall>,
}

impl Auction {
    /// Wraps an already-ordered list of calls.
    #[must_use]
    pub const fn new(calls: Vec<RecordedCall>) -> Self {
        Self { calls }
    }

    /// Appends a call.
    pub fn push(&mut self, call: RecordedCall) {
        self.calls.push(call);
    }

    /// The calls in order.
    #[must_use]
    pub fn calls(&self) -> &[RecordedCall] {
        &self.calls
    }

    /// Number of calls.
    #[must_use]
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// True when no call has been made.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }

    /// Iterates over the calls in order.
    pub fn iter(&self) -> std::slice::Iter<'_, RecordedCall> {
        self.calls.iter()
    }

    /// The index one past the call that ended the auction, or `None` when the
    /// calls never reach an ending.
    ///
    /// The auction ends on the fourth opening pass (a natural pass-out) or on
    /// the third consecutive pass after any bid, double or redouble. A double
    /// or redouble resets the closing pass count, so `1S X Pass Pass` is still
    /// open while `1S X Pass Pass Pass` is over. The single predicate lives
    /// here so validation, contract derivation and both format paths cannot
    /// drift apart.
    fn ended_at(&self) -> Option<usize> {
        let mut closing_passes = 0;
        let mut saw_non_pass = false;

        for (index, recorded) in self.calls.iter().enumerate() {
            if recorded.call == Call::Pass {
                closing_passes += 1;
                let required = if saw_non_pass { 3 } else { 4 };
                if closing_passes == required {
                    return Some(index + 1);
                }
            } else {
                saw_non_pass = true;
                closing_passes = 0;
            }
        }

        None
    }

    /// True when the calls form a complete auction.
    ///
    /// A complete auction is a natural pass-out (four opening passes) or any
    /// bid, double or redouble followed by three consecutive passes. A cut
    /// auction — the calls a file recorded before the auction finished — is not
    /// complete. Calls after the ending call do not make an illegal auction
    /// complete; [`Auction::validate`] reports them.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.ended_at() == Some(self.calls.len())
    }

    /// The seat that made the `index`-th call, given the `dealer`.
    ///
    /// Returns `None` when `index` is past the end of the auction.
    #[must_use]
    pub fn seat(&self, index: usize, dealer: Seat) -> Option<Seat> {
        (index < self.calls.len()).then(|| dealer + index)
    }

    /// The final contract implied by this auction, if it produced one.
    ///
    /// The declarer is the first seat on the winning side to have bid the final
    /// strain; `vulnerability` supplies the per-seat answer. Returns `None` for
    /// a passed-out auction, and for an auction that continued after the call
    /// that ended it: that auction is illegal, and [`Auction::validate`]
    /// reports the offending call rather than letting it name a contract.
    #[must_use]
    pub fn contract(&self, dealer: Seat, vulnerability: Vulnerability) -> Option<Contract> {
        // An auction that kept going after its ending call has no contract to
        // derive. Refusing here keeps `Pass Pass Pass Pass 1S` from ever being
        // read as a 1S contract, even by a caller that skipped validation.
        if self.ended_at().is_some_and(|end| end < self.calls.len()) {
            return None;
        }

        let (final_index, final_level, final_strain) = self
            .calls
            .iter()
            .enumerate()
            .filter_map(|(index, recorded)| match recorded.call {
                Call::Bid { level, strain } => Some((index, level.get(), strain)),
                _ => None,
            })
            .reduce(|best, candidate| {
                let (_, best_level, best_strain) = best;
                let (_, level, strain) = candidate;
                if (level, strain) > (best_level, best_strain) {
                    candidate
                } else {
                    best
                }
            })?;

        // Doubled state comes from the calls after the final bid.
        let doubled =
            self.calls[final_index + 1..]
                .iter()
                .fold(Doubled::NotDoubled, |state, recorded| match recorded.call {
                    Call::Double => Doubled::Doubled,
                    Call::Redouble => Doubled::Redoubled,
                    Call::Pass | Call::Bid { .. } => state,
                });

        let winning_seat = dealer + final_index;
        let declarer =
            self.calls
                .iter()
                .enumerate()
                .find_map(|(index, recorded)| match recorded.call {
                    Call::Bid { strain, .. }
                        if strain == final_strain
                            && (dealer + index).is_same_line(&winning_seat) =>
                    {
                        Some(dealer + index)
                    }
                    _ => None,
                })?;

        Some(Contract::new(
            final_level,
            final_strain,
            declarer,
            Vulnerable::new(vulnerability, declarer),
            doubled,
        ))
    }

    /// Checks that the calls form a legal auction in four-seat rotation.
    ///
    /// A double is legal only immediately after an opponent's bid, and a
    /// redouble only immediately after an opponent's double; passes do not
    /// change the entitlement. `NonStarter` is reserved for a double or
    /// redouble that opens the auction with no prior call.
    ///
    /// The auction may be cut — a sequence that stops before its closing
    /// passes is legal — but no call may follow the one that ended it. A
    /// natural pass-out ends the auction after four opening passes, and any
    /// bid, double or redouble ends it after three consecutive passes.
    ///
    /// # Errors
    ///
    /// Returns [`AuctionError`] when the auction opens with a double or
    /// redouble, a bid does not outrank the previous one, a double/redouble is
    /// made by a side not entitled to it, or a call follows the end of the
    /// auction.
    pub fn validate(&self, dealer: Seat) -> Result<(), AuctionError> {
        if let Some(end) = self.ended_at()
            && let Some(recorded) = self.calls.get(end)
        {
            return Err(AuctionError::CallAfterEnd {
                call: recorded.call,
                seat: dealer + end,
            });
        }

        let mut highest_bid: Option<Call> = None;
        // The most recent non-pass call and the seat that made it; this is what
        // decides whether the next double or redouble is entitled.
        let mut last_non_pass: Option<(Call, Seat)> = None;

        for (index, recorded) in self.calls.iter().enumerate() {
            let seat = dealer + index;
            match recorded.call {
                Call::Pass => {}
                Call::Bid { level, strain } => {
                    let bid = Call::Bid { level, strain };
                    if let Some(previous) = highest_bid {
                        let (Some(prev_level), Some(prev_strain)) =
                            (previous.level(), previous.strain())
                        else {
                            unreachable!("highest_bid is always a bid")
                        };
                        if (level.get(), strain) <= (prev_level.get(), prev_strain) {
                            return Err(AuctionError::Insufficient {
                                previous,
                                next: bid,
                                seat,
                            });
                        }
                    }
                    highest_bid = Some(bid);
                    last_non_pass = Some((bid, seat));
                }
                Call::Double => {
                    let entitled = matches!(
                        last_non_pass,
                        Some((Call::Bid { .. }, bidder)) if !seat.is_same_line(&bidder)
                    );
                    if !entitled {
                        return Err(if last_non_pass.is_none() {
                            AuctionError::NonStarter {
                                call: recorded.call,
                                seat,
                            }
                        } else {
                            AuctionError::IllegalDouble {
                                call: recorded.call,
                                seat,
                            }
                        });
                    }
                    last_non_pass = Some((recorded.call, seat));
                }
                Call::Redouble => {
                    let entitled = matches!(
                        last_non_pass,
                        Some((Call::Double, doubler)) if !seat.is_same_line(&doubler)
                    );
                    if !entitled {
                        return Err(if last_non_pass.is_none() {
                            AuctionError::NonStarter {
                                call: recorded.call,
                                seat,
                            }
                        } else {
                            AuctionError::IllegalDouble {
                                call: recorded.call,
                                seat,
                            }
                        });
                    }
                    last_non_pass = Some((recorded.call, seat));
                }
            }
        }

        Ok(())
    }
}

impl<'a> IntoIterator for &'a Auction {
    type Item = &'a RecordedCall;
    type IntoIter = std::slice::Iter<'a, RecordedCall>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bid(level: u8, strain: Strain) -> RecordedCall {
        RecordedCall::new(Call::Bid {
            level: NonZeroU8::new(level).expect("test level is non-zero"),
            strain,
        })
    }

    fn pass() -> RecordedCall {
        RecordedCall::new(Call::Pass)
    }

    #[test]
    fn seat_rotates_from_dealer() {
        let auction = Auction::new(vec![pass(), pass(), pass(), pass()]);
        assert_eq!(auction.seat(0, Seat::West), Some(Seat::West));
        assert_eq!(auction.seat(1, Seat::West), Some(Seat::North));
        assert_eq!(auction.seat(4, Seat::West), None);
    }

    #[test]
    fn opening_double_is_a_non_starter() {
        let auction = Auction::new(vec![RecordedCall::new(Call::Double)]);
        assert!(matches!(
            auction.validate(Seat::North),
            Err(AuctionError::NonStarter { .. })
        ));
    }

    #[test]
    fn insufficient_bid_is_rejected() {
        let auction = Auction::new(vec![bid(1, Strain::Spades), bid(1, Strain::Hearts)]);
        assert!(matches!(
            auction.validate(Seat::North),
            Err(AuctionError::Insufficient { .. })
        ));
    }

    #[test]
    fn doubling_partner_is_rejected() {
        // North bids 1S, South (partner) doubles: not entitled.
        let auction = Auction::new(vec![
            bid(1, Strain::Spades),
            pass(),
            RecordedCall::new(Call::Double),
        ]);
        assert!(matches!(
            auction.validate(Seat::North),
            Err(AuctionError::IllegalDouble { .. })
        ));
    }

    #[test]
    fn legal_auction_passes_validation() {
        // North 1S, East pass, South 2S, West pass, North 4S, three passes.
        let auction = Auction::new(vec![
            bid(1, Strain::Spades),
            pass(),
            bid(2, Strain::Spades),
            pass(),
            bid(4, Strain::Spades),
            pass(),
            pass(),
            pass(),
        ]);
        assert!(auction.validate(Seat::North).is_ok());
    }

    #[test]
    fn four_opening_passes_complete_the_auction() {
        let auction = Auction::new(vec![pass(), pass(), pass(), pass()]);
        assert!(auction.is_complete());
    }

    #[test]
    fn three_closing_passes_after_a_bid_complete_the_auction() {
        // North 1S, then three passes: the auction is over.
        let auction = Auction::new(vec![bid(1, Strain::Spades), pass(), pass(), pass()]);
        assert!(auction.is_complete());
    }

    #[test]
    fn a_cut_auction_is_not_complete() {
        for calls in [
            Vec::new(),
            vec![pass(), pass(), pass()],
            vec![bid(1, Strain::Spades)],
            vec![bid(1, Strain::Spades), pass(), pass()],
        ] {
            assert!(
                !Auction::new(calls.clone()).is_complete(),
                "these calls must be cut: {calls:?}"
            );
        }
    }

    #[test]
    fn a_double_resets_the_closing_pass_count() {
        // 1S X Pass Pass is cut; the third pass completes the auction.
        assert!(
            !Auction::new(vec![
                bid(1, Strain::Spades),
                RecordedCall::new(Call::Double),
                pass(),
                pass(),
            ])
            .is_complete()
        );
        assert!(
            Auction::new(vec![
                bid(1, Strain::Spades),
                RecordedCall::new(Call::Double),
                pass(),
                pass(),
                pass(),
            ])
            .is_complete()
        );
    }

    #[test]
    fn a_bid_after_four_opening_passes_is_rejected() {
        let auction = Auction::new(vec![pass(), pass(), pass(), pass(), bid(1, Strain::Spades)]);
        let error = auction
            .validate(Seat::North)
            .expect_err("the auction passed out after four passes");
        assert!(matches!(error, AuctionError::CallAfterEnd { .. }));
        assert!(
            error.to_string().contains("after the auction"),
            "the error should say the auction ended: {error}"
        );
    }

    #[test]
    fn a_call_after_the_third_closing_pass_is_rejected() {
        // North 1S, three passes close the auction; South's 2S comes too late.
        let auction = Auction::new(vec![
            bid(1, Strain::Spades),
            pass(),
            pass(),
            pass(),
            bid(2, Strain::Spades),
        ]);
        assert!(matches!(
            auction.validate(Seat::North),
            Err(AuctionError::CallAfterEnd { .. })
        ));
    }

    #[test]
    fn a_passed_out_auction_validates() {
        let auction = Auction::new(vec![pass(), pass(), pass(), pass()]);
        assert!(auction.validate(Seat::North).is_ok());
    }

    #[test]
    fn a_redouble_also_resets_the_closing_pass_count() {
        // North 1S, East X, South XX, then West and North pass: still cut.
        let cut = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Double),
            RecordedCall::new(Call::Redouble),
            pass(),
            pass(),
        ]);
        assert!(!cut.is_complete());
        assert!(cut.validate(Seat::North).is_ok());

        let complete = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Double),
            RecordedCall::new(Call::Redouble),
            pass(),
            pass(),
            pass(),
        ]);
        assert!(complete.is_complete());
        assert!(complete.validate(Seat::North).is_ok());
    }

    #[test]
    fn a_doubled_auction_validates_only_after_three_closing_passes() {
        let cut = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Double),
            pass(),
        ]);
        assert!(cut.validate(Seat::North).is_ok());
        let complete = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Double),
            pass(),
            pass(),
            pass(),
        ]);
        assert!(complete.validate(Seat::North).is_ok());
        // A fourth pass follows the call that already ended the auction.
        let over = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Double),
            pass(),
            pass(),
            pass(),
            pass(),
        ]);
        assert!(matches!(
            over.validate(Seat::North),
            Err(AuctionError::CallAfterEnd { .. })
        ));
    }

    #[test]
    fn contract_names_the_first_side_to_bid_the_strain() {
        // North 1S, East pass, South 2S, West pass, North 4S, three passes.
        let auction = Auction::new(vec![
            bid(1, Strain::Spades),
            pass(),
            bid(2, Strain::Spades),
            pass(),
            bid(4, Strain::Spades),
            pass(),
            pass(),
            pass(),
        ]);
        let contract = auction
            .contract(Seat::North, Vulnerability::None)
            .expect("auction produced a contract");
        assert_eq!(contract.declarer(), Seat::North);
        assert_eq!(contract.level(), 4);
        assert_eq!(contract.strain(), Strain::Spades);
    }

    #[test]
    fn passed_out_auction_has_no_contract() {
        let auction = Auction::new(vec![pass(), pass(), pass(), pass()]);
        assert!(auction.contract(Seat::North, Vulnerability::None).is_none());
    }

    #[test]
    fn an_auction_that_continued_after_its_end_has_no_contract() {
        // The 1S follows the fourth opening pass: the auction was over, so it
        // must not name a 1S contract even before validation runs.
        let after_pass_out =
            Auction::new(vec![pass(), pass(), pass(), pass(), bid(1, Strain::Spades)]);
        assert!(
            after_pass_out
                .contract(Seat::North, Vulnerability::None)
                .is_none()
        );

        // The 2S follows the third closing pass.
        let after_closing_pass = Auction::new(vec![
            bid(1, Strain::Spades),
            pass(),
            pass(),
            pass(),
            bid(2, Strain::Spades),
        ]);
        assert!(
            after_closing_pass
                .contract(Seat::North, Vulnerability::None)
                .is_none()
        );

        // A cut auction still derives the contract of its highest bid.
        let cut = Auction::new(vec![bid(1, Strain::Spades), pass(), pass()]);
        assert_eq!(
            cut.contract(Seat::North, Vulnerability::None)
                .map(|contract| contract.level()),
            Some(1)
        );
    }

    #[test]
    fn contract_records_the_doubled_state() {
        // North 1S, East doubles, then it is passed out.
        let auction = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Double),
            pass(),
            pass(),
            pass(),
        ]);
        assert!(auction.validate(Seat::North).is_ok());
        let contract = auction
            .contract(Seat::North, Vulnerability::None)
            .expect("auction produced a contract");
        assert_eq!(contract.declarer(), Seat::North);
        assert_eq!(contract.level(), 1);
        assert_eq!(contract.strain(), Strain::Spades);
    }

    #[test]
    fn doubling_after_a_redouble_is_illegal_not_a_non_starter() {
        // N 1S, E doubles, S redoubles, W doubles: W is not entitled, but the
        // auction did have a prior call, so this is an illegal double.
        let auction = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Double),
            RecordedCall::new(Call::Redouble),
            RecordedCall::new(Call::Double),
        ]);
        assert!(matches!(
            auction.validate(Seat::North),
            Err(AuctionError::IllegalDouble { .. })
        ));
    }

    #[test]
    fn redoubling_without_a_double_is_illegal_not_a_non_starter() {
        // N 1S, E redoubles: there was a prior bid, so it is not a non-starter.
        let auction = Auction::new(vec![
            bid(1, Strain::Spades),
            RecordedCall::new(Call::Redouble),
        ]);
        assert!(matches!(
            auction.validate(Seat::North),
            Err(AuctionError::IllegalDouble { .. })
        ));
    }
}
