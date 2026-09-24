# `Position` stays contract-free; the opening leader is an explicit input

`Position` is the solver input snapshot, and the design calls it a pure function
of a `Deal`, a `PlayRecord` prefix and a `Strain`. That is true for every
non-empty prefix, but not for the opening position: with no cards on the table,
`to_play` is the opening leader, which the position's own fields cannot name. We
chose to take the leader as an explicit `Seat` input of
`Position::from_deal_prefix`, with a sibling
`from_deal_prefix_with_contract` that reads the strain and leader from a
`Contract`. `Position` itself never stores a contract, so the value type stays
free of scoring metadata and can be serialized, compared and handed to a solver
on its own.

## Considered Options

- **Take a `&Contract` in every constructor** — re-couples a position's creation
  to scoring metadata it does not otherwise need, and rules out building one for
  a contract the consumer has not materialised.
- **Store a leader or contract on `Position`** — adds a field that is redundant
  with the prefix for every non-empty position and would have to be kept in
  sync.
- **Make `trump`/`to_play` optional for a partially-known position** — the
  design deliberately keeps `trump` a mandatory `Strain`; an unknown strain is
  not a representable position.

## Consequences

- `from_deal_prefix` requires `leader: Seat`; callers holding a `Contract` use
  the sibling and pass none.
- The raw `Position::new` validates the shape of a snapshot past any point of
  the play, not a full deck: `table.len() <= 3`, each hand holds at most 13
  cards, the table and hands are distinct, the played count `52 - |hands|`
  leaves exactly `table.len()` cards on the table, and the table's plays are
  consecutive in rotation. `to_play` is the caller's claim, because at an empty
  table it cannot be recomputed without the leader.
- A fourth `table` card is rejected: a completed trick is represented as an
  empty `table` with the trick winner to play, so there is one shape per state.
