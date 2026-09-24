# No single ground truth for a decision

Bridge decisions depend on information the deal does not carry — count signals,
tempo, inferences from an opponent's card — so no evaluation is "the correct
play". We expose two complementary evaluations and privilege neither: DD
deviation (how the played card compares to the DD optimum of that exact,
fully-known deal) and card expected value (a card's average DD outcome over a
distribution of deals consistent with the information the consumer supplies).
The first answers "what did these cards allow here"; the second answers "what
is the best long-run choice given this conditioning". Naming must never imply
absoluteness: no "correct", no "ground truth", no unqualified "optimal";
DD-relative results are always labelled as such (e.g. `CardResult::Optimal` is
to be renamed).
