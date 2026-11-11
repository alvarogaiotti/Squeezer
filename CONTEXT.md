# Squeezer

A library for bridge deal generation, double-dummy solving, simulation, and
performance analysis. This glossary fixes the vocabulary the code and docs
use; it is not a spec.

## Language

### Deals and records

**Deal**:
The four 13-card hands, with no contract and no play.
_Avoid_: board (that is a recorded Deal).

**Board**:
A Deal together with its dealer, vulnerability, auction, contract, and play —
the unit a LIN or PBN file describes.
_Avoid_: hand (that is one player's cards), record (alone).

**Hand**:
One seat's cards.
_Avoid_: using "hand" for a Deal or a Board.

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

**Par**:
The double-dummy contract and score at which neither side can improve,
computed from a DD table.
