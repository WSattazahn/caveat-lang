# CAVEAT Language Specification — Draft 0.2 (working)

This draft records changes earned by executable/adversarial evidence. It does not supersede 0.1 yet.

## 1. Qualification topology
Draft 0.1's depth-only model fails on circular qualification. The runtime distinguishes `Finite { max_depth }` and `Cyclic`. Cyclic qualification MUST NOT be reported as ordinary finite depth. Dynamic/open-ended topology remains future work.

## 2. Resource semantics
`budget N` declares a deterministic ledger. `examine X cost N` consumes units only when payable. Initial, spent, remaining, and exhausted state are preserved. `commit ... because budget` is valid only after actual exhaustion. Resource units remain domain-abstract.

## 3. Temporal history and commitment snapshots
Evaluation maintains ordered immutable history. Every commitment captures an `EpistemicSnapshot` of nodes, edges, and resource state at decision time. Later information cannot retroactively enter that snapshot. Reopening changes current state and appends history without rewriting the original commitment.

## 4. Explicit inference
CAVEAT 0.2 uses named local inference rules rather than claiming a complete novel logic. `rule R when A, B => C; infer R;` records support dependencies and an inference-history event. Contradiction elsewhere in the graph does not imply arbitrary conclusions. This is conservative non-explosive behavior, not yet a complete formal paraconsistent calculus.

## 5. Dependency-sensitive qualification impact
A caveat directly qualifying a node may be relevant to conclusions that depend on that node. The runtime can now compute `QualificationImpact` by traversing downstream `Supports` dependencies from each directly qualified node.

For `p -> q -> r` with caveat `c` qualifying `q`, impact contains `q` and `r`, but not upstream `p` and not unrelated nodes.

This is deliberately an **impact trace**, not automatic mutation. Draft 0.2 does not yet assert that every downstream conclusion is invalid, blocked, or itself directly qualified. Domains may need different propagation policies. The trace supplies the provenance needed for such policies without infecting the entire graph by default.

## 6. Reopening history
Repeated reopening remains visible as repeated history events. The distinction between first reopening and additional evidence while already open remains unresolved.

## 7. Errors
Duplicate symbols/rules, undefined references, invalid resource amounts, unaffordable examinations, and false budget-exhaustion commitments are explicit evaluation errors.

## Rule
A semantic change enters Draft 0.2 because a program, proof, implementation constraint, or use case earned it — not because the language needs to appear novel.
