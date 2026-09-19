# Card-payoff predealt / current-trick coupling

Status: resolved

## Problem

`CardPayoffSimulation` forces callers to understand dealer internals: the
candidate cards must already sit in a specific seat's predealt hand
(`src/simulation/payoff.rs`, builder ~L780-790, legacy ctor ~L862-875),
`run()` mutates `deal.hands()`, subtracts played cards, and injects a
per-option current trick (~L943-965). There is a latent non-ASCII
identifier typo (`insertion_poìnt`, ~L892) and a 0/48-card dead zone baked
into validation (~L857-860: empty trace forbidden, full deal forbidden).

## Direction

Deepen or document the seam: either the simulation owns mid-trick setup
behind a small interface (candidate cards + play trace in, solved tricks
out), or the current-trick algebra gets one tested home. Fix the typo and
revisit the dead zone either way.

## Non-goals

Scoring/pairwise accumulation; the shared batch-solve engine (separate,
already planned).

## Comments

- Decision 2026-09-19: explicit seam (small) over simulation-owned setup
  — the generic `D: Dealer` cannot construct predeals through the trait,
  so ownership stays with the caller but is now validated and documented.
- Done: `CardPayoffBuilder::build()` requires the predealt hand
  (missing predeal is an error, not a silent pass), validates trace
  length 1..=47 like `new()`, and shares `new()`'s set-based candidate
  check; fixed the `insertion_poìnt` typo (pure rename); documented the
  candidate/trace contract on builder methods and struct fields. Mid-trick
  algebra untouched (ownership redesign remains future work).
- Tests: `test_card_payoff_builder_validates_predealt_and_trace` (missing
  predeal, missing candidate, empty trace, valid build).
