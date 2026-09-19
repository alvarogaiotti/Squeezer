# Card-payoff predealt / current-trick coupling

Status: needs-triage

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
