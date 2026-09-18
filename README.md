# CAVEAT

An experimental programming language for computation with claims, evidence, caveats, provisional commitments, and reopening.

CAVEAT explores a persistent epistemic graph rather than reducing every computation immediately to a single settled value. Contradictory positions may coexist; evidence carries provenance; commitments can retain unresolved caveats; later evidence can reopen earlier commitments.

## Status

CAVEAT 0.3 is executable: the Rust reference runtime parses source, evaluates the epistemic graph, exposes the CAVEAT Map, executes source-authored action plans, and builds WebAssembly browser sessions.

The current playable proof-of-use is **Moon Garden**, a short mobile-first mystery whose investigations, evidence, caveats, retained uncertainty, and commitment reopening are authored in `game/moon_garden.cav`. The earlier **The Door** scenario remains as the world/action and 3D stress test.

The next engine milestone is to generalize presentation primitives so more finished games can be authored from CAVEAT source without bespoke scene code.

## Repository layout

- `spec/` — versioned language specifications
- `runtime/` — reference runtime
- `tests/` — canonical semantic tests
- `examples/` — example CAVEAT programs
- `game/` — playable CAVEAT scenarios, including Moon Garden and The Door

> Do not save CAVEAT by redefining it. If the computational model collapses into an existing paradigm, record the result.
