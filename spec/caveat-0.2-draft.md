# CAVEAT Language Specification — Draft 0.2 (working)

This draft records changes earned by executable/adversarial evidence. It does not supersede 0.1 yet.

## 1. Qualification topology
Draft 0.1's depth-only model fails on circular qualification. The runtime distinguishes `Finite { max_depth }` and `Cyclic`. Cyclic qualification MUST NOT be reported as ordinary finite depth. Dynamic/open-ended topology remains future work.

## 2. Resource semantics
Resource accounting is now executable rather than descriptive.

- `budget N` declares a deterministic unit ledger.
- `examine X cost N` consumes units and marks X examined only if the cost can be paid.
- The ledger preserves initial, spent, remaining, and exhausted state.
- `commit ... because budget` is valid only when the ledger is actually exhausted.
- Insufficient budget for an attempted examination is an evaluation error; it does not silently consume nonexistent resources.

The unit is intentionally abstract in 0.2. Domain mappings to time, money, tool calls, tokens, experiments, or human attention remain outside the core semantics.

## 3. Temporal history and commitment snapshots
The current graph answers what the program believes/contains now. It is insufficient for explaining why an earlier commitment was made.

Evaluation therefore maintains an ordered immutable history. Budget declarations, charged examinations, commitments, and reopenings are events with monotonically increasing sequence numbers.

Every commitment captures an `EpistemicSnapshot` containing a clone of the graph nodes, graph edges, and resource ledger as they existed immediately after creation of the commitment. Information introduced later MUST NOT appear retroactively in that snapshot.

This establishes two distinct semantic views:

- **current graph** — present epistemic state;
- **commitment snapshot** — epistemic state available at the decision event.

Reopening changes current state and appends a history event. It does not mutate the historical snapshot that justified the earlier commitment.

## 4. Reopening history
Repeated reopening remains visible as repeated history events. Draft 0.2 still needs to decide whether reopening an already-open commitment is semantically distinct from attaching further post-reopening evidence.

## 5. Contradiction
Supporting and opposing evidence can coexist without explosion at the representation layer. A formal paraconsistent inference system remains unresolved.

## 6. Errors
Duplicate symbols, undefined references, invalid resource amounts, unaffordable charged examinations, and false budget-exhaustion commitments are explicit evaluation errors.

## Rule
A semantic change enters Draft 0.2 because a program, proof, implementation constraint, or use case earned it — not because the language needs to appear novel.
