# Architecture follow-ups: deferred slices

Deferred during the improve-codebase-architecture passes (deal-constraint
deepening, Q13 unifications, simulation solve engine). Each ticket is
independently implementable; none blocks the others.

Grilled decisions already locked (see issue files): simulation engine seam
(function + two wrappers, multi-thread solver, `DealingError` mapping,
`MAXNOOFBOARDS` chunking, single-threaded test verification).
