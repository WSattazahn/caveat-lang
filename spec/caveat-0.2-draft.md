# CAVEAT Language Specification — Draft 0.2 (working)

This draft records changes earned by executable/adversarial evidence. It does not supersede 0.1 yet.

## 1. Qualification topology

Draft 0.1 treated qualification primarily in terms of depth. Adversarial cyclic qualification showed that this is insufficient.

The runtime now distinguishes at least:

- `Finite { max_depth }`
- `Cyclic`

A cyclic qualification graph MUST NOT be reported as having an ordinary finite qualification depth. Future work may add dynamic/open-ended topology when runtime generation exists.

## 2. Resource semantics — unresolved

Draft 0.1 names resource-bounded stopping and `BudgetExhausted`, but the runtime does not yet possess a resource ledger. Therefore budget exhaustion is currently representable as a reason but not causally enforced by execution.

Draft 0.2 must define:

- a resource budget primitive;
- costs for examination operations;
- what happens when an operation exceeds remaining budget;
- whether resource accounting is deterministic and replayable;
- how a budget-driven commitment differs from an `enough` commitment.

## 3. Reopening history

Repeated reopening currently produces multiple `Reopens` edges even when the commitment is already open. This preserves event visibility but does not encode event time or distinguish first reopening from subsequent evidence. Draft 0.2 should introduce an event/history model rather than overloading graph edges as an event log.

## 4. Contradiction

The first adversarial tests confirm that supporting and opposing evidence can coexist in the graph without explosion. This is representation-level contradiction tolerance only; a formal paraconsistent inference system remains unresolved.

## 5. Errors

Duplicate symbol definitions and references to undefined symbols are explicit evaluation errors in the reference runtime.

## Rule

A semantic change enters Draft 0.2 because a program, proof, implementation constraint, or use case earned it — not because the language needs to appear novel.
