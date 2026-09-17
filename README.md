# CAVEAT

An experimental programming language for computation with claims, evidence, caveats, provisional commitments, and reopening.

CAVEAT explores a persistent epistemic graph rather than reducing every computation immediately to a single settled value. Contradictory positions may coexist; evidence carries provenance; commitments can retain unresolved caveats; later evidence can reopen earlier commitments.

## Status

Draft 0.1 — semantics before syntax. The first implementation milestone is a Rust reference runtime with a hand-authored AST and Canonical Program 001.

## Repository layout

- `spec/` — versioned language specifications
- `runtime/` — reference runtime
- `tests/` — canonical semantic tests
- `examples/` — example CAVEAT programs
- `game/` — eventual game used as a language stress test

> Do not save CAVEAT by redefining it. If the computational model collapses into an existing paradigm, record the result.
