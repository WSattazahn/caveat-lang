# Published module-binding saves

These raw JSON strings came from the published `caveat-lang@1.0.0` and
`caveat-lang@1.4.1` packages, using each package's public Node loader and
shipped WASM. Tarball bytes and SHA-256 were matched to
`docs/releases/v1.0.0.md` and `docs/releases/v1.4.1.md` first.
`provenance.json` records those hashes and each save's exact byte hash.
Keep the saves as raw text: do not pretty-print or normalize their numbers.

`collision.bundle` has a module-local numeric `hold` and a root name.
Both releases write the same initial save; one event produces 10 in 1.0.0
and 14 in 1.4.1. The state fixtures follow that event. 1.4.1 also supplies
named and positional replay saves; 1.0.0 predates replay formats.

`legacy-only.bundle` calls a module's two-argument `hold` with one
argument, while the root has a one-argument `hold`. Correct scoped loading
refuses it. 1.4.1 captured the root function and accepted it, producing 13.
Its initial, state and both replay saves test continued historical restore
without allowing fresh loads to inherit the bug.

The `.bundle` extension keeps these intentional historical comparison
sources separate from the general corpus of programs expected to load.
