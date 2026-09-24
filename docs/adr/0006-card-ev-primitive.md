# One all-cards primitive: card evaluation, with lead analysis as a view

Card evaluation is built on a single double-dummy all-legal solve per sampled
deal (`solutions=3`), which returns the value of every legal card at once.
`CardEvaluation` is the primitive. `CardPayoffSimulation` is deleted rather than
kept as a second path that must agree, and lead analysis becomes a view over the
same primitive at the opening position — set percentage, cumulative trick
distributions, and per-suit grouping are derived, not separately solved. Values
are raw bridge points by default, with an extensible `Denomination` for
comparisons against a baseline (shared with the contract payoff matrix).
