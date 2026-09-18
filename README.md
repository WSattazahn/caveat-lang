# CAVEAT

An experimental programming language for computation with claims, evidence, caveats, provisional commitments, and reopening.

CAVEAT explores a persistent epistemic graph rather than reducing every computation immediately to a single settled value. Contradictory positions may coexist; evidence carries provenance; commitments can retain unresolved caveats; later evidence can reopen earlier commitments.

## Status

CAVEAT 0.3 is executable: the Rust reference runtime parses source, evaluates the epistemic graph, exposes the CAVEAT Map, executes source-authored action plans, and builds WebAssembly browser sessions.

The current playable proof-of-use is **Moon Garden**, a short mobile-first mystery whose investigations, evidence, caveats, retained uncertainty, and commitment reopening are authored in `game/moon_garden.cav`. It now has both a polished 2D presentation and a separate **Moon Garden 3D** presentation driven by the same CAVEAT session. The earlier **The Door** scenario remains the world/action stress test.

**CAVEAT 3D 0.2** now consumes normalized Rust action-runtime executions (`Move`, `Inspect`, `Operate`, `Open`, `Observe`, `Stay`) and maps those commands to semantic place/entity/symbol presentation bindings. Moon Garden no longer needs normal per-action camera choreography. See `spec/caveat3d-0.2.md`. The remaining graphics problem is art-direction automation: semantic identifiers can now drive the scene, but attractive camera composition and assets still require authored presentation bindings.

## Repository layout

- `spec/` — versioned language specifications
- `runtime/` — reference runtime
- `tests/` — canonical semantic tests
- `examples/` — example CAVEAT programs
- `game/` — playable CAVEAT scenarios, including Moon Garden and The Door

> Do not save CAVEAT by redefining it. If the computational model collapses into an existing paradigm, record the result.
