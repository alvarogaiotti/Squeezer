# Squeezer

A workspace for bridge deal generation, double-dummy solving, simulation, and
performance analysis. This glossary fixes the vocabulary the code and docs
use; it is not a spec.

## Crates

There is no umbrella crate. Each library crate owns one vocabulary and depends
only on the crates beneath it; the split is recorded in
[ADR-0008](docs/adr/0008-crate-split.md).

| Crate | Vocabulary |
| --- | --- |
| `squeezer-cards` | `Suit`, `Card`, `Cards`, the samplers |
| `squeezer-model` | shapes and constraints, the contract vocabulary, `Deal`, the dealer, and the recorded-board types |
| `squeezer-solver` | the solver seam, its context pool, and the DDS backend behind a feature |
| `squeezer-io` | LIN and PBN import and the file-container writers |
| `squeezer-bbo` | BBO hand retrieval, behind `blocking` and `async` |
| `squeezer-sim` | simulation and performance analysis |

## Language

### Deals and records

**Deal**:
The four seats' thirteen-card holdings, with no contract and no play.
_Avoid_: board (that is a recorded Deal).

**Board**:
A Deal together with its dealer, vulnerability, auction, contract, and play —
the unit a LIN or PBN file describes.
_Avoid_: hand (that is one player's cards), record (alone).

**Seat**:
One of the four positions: North, East, South, West.

**Auction**:
The ordered sequence of calls; seats are positional (`seat(i) = dealer + i`).
_Avoid_: bidding.

**Call**:
A bid, pass, double, or redouble.

**Bid**:
A Call that names a level and Strain, as opposed to Pass, Double, or
Redouble.

**Contract**:
The final call together with its Strain, Doubled state, declarer, and
Vulnerability.

**Play** / **PlayRecord**:
The cards played, each attributed to a Seat, grouped into tricks.
_Avoid_: sequence (ambiguous).

**Claim**:
An early end to the play, recorded as the number of tricks claimed.

**Position**:
A partially played state: the strain, whose turn it is, the cards on the table,
and the remaining cards per Seat. The input to double-dummy solving.
_Avoid_: deal (that is un-played).

**Legal play**:
A card a Seat may play from a Position, respecting follow-suit: if the player
holds the led suit, only those cards; otherwise the whole hand.

**Known cards**:
Predealt plus already-played cards fixed to a Seat; the conditioning seed.

**Fill order**:
The order in which the dealer fills, and therefore checks, the four seats for
one candidate. Not the canonical seat order of a Deal.
_Avoid_: deal order, seat order.

### Double-dummy evaluation

**Solver context**:
A stateful double-dummy solver handle owning its transposition table.

**Pool chunk**:
A contiguous slice of the indices passed to `SolverPool::map`, handled by one
rayon job with one solver context. The pool creates at most one chunk per
worker, so the chunk is the unit of context creation and table reuse.
_Avoid_: batch (the retired DDS batch entry points), partition.

**Transposition table (TT)**:
A solver's memo of already-solved sub-positions. It is owned per solver context,
survives across solves that are similar, and may be cleared explicitly.
_Avoid_: cache, memo table.

**DD-optimal**:
The card(s) maximising double-dummy tricks for the side to play in one exact,
fully-known Deal. Relative to that Deal only.
_Avoid_: optimal, best (unqualified), correct.

**DD deviation**:
The tricks lost by a played card against that Deal's DD-optimal result.

**Card expected value (card EV)**:
The average DD outcome of playing a card across a distribution of deals
consistent with the information the consumer supplies. A long-run
recommendation, not a claim about a specific Deal.
_Avoid_: correct play, ground truth.

**Denomination**:
How an outcome is valued: raw bridge points by default, or a comparison to a
baseline under a scoring (IMPs, matchpoints).

**DD table**:
The double-dummy trick count for every strain and every Seat in one Deal — the
input to par and to a scored report, not a claim about any played card.
_Avoid_: results table, score table.

**Par**:
The double-dummy contract and score at which neither side can improve,
computed from a DD table.
