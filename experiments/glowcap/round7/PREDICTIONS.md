# Round 7 predictions

Recorded by Claude for the owner, as the protocol allows ("The new change
requests", Predictions), after CR13–CR16 were written and before any author
started. The predictor has read the requests, the earlier rounds' results and
earlier programs, but no round-7 implementation, because none exists. A
prediction that fails is reported as failed.

| CR | predicted | why |
| --- | --- | --- |
| CR13: the slime forgets | TypeScript smaller diff and fewer failed runs | a six-entry window ordered by acceptance is a list operation; Caveat values are counts and sets with no list, so the window needs per-slot state or withdrawal bookkeeping, and the six-entry caps on `reopenedBy` and `history` sit outside what the decision journal keeps |
| CR14: the player can mark a mushroom | tie | one boolean, two events and one `why` row on both sides; Caveat's `reject` rules and TypeScript's guards are the same size |
| CR15: a mushroom out of reach | Caveat smaller diff | a fifth entity and three rejections are declarations under repetition, while the TypeScript side touches each place that assumes every mushroom can be absorbed and tasted |
| CR16: the journal | TypeScript clearly better | an ordered, capped list of entries is ordinary data in TypeScript; Caveat has no list value, so the journal is either per-slot shadow state or rebuilt in the adapter. Current caveats on old entries are the one part that may come free on the Caveat side |
| Mutation score | Caveat higher on M5 (cite evidence never read) | the runtime rejects a `because` the value never read (Explanations 0.1); TypeScript checks nothing there. No prediction for M1–M4 |
| Explanation drift | TypeScript lower on CR13 and CR16, Caveat lower on CR15 | follows the change-cost predictions |
