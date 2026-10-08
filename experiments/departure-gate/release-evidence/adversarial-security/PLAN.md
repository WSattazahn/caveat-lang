# Bounded adversarial pass on the frozen Caveat draft

Candidate: unpublished `590fae59aa4295ac5209f930a1e58ac152bd4603`. Comparison: the verified published rc.15 package at `3a88ba0f80d563b4493840dc7bd7e195329b8302`. Exact JS/WASM/build identities and offline provenance are in `cells.json` and `identity-notes.md`. Reuse these artifacts without a rebuild or semantic modification.

This pass tests save validation, required withdrawal reasons, transactional restoration and archive reporting. It does not test prompt injection, an agent's instruction hierarchy, external tool authority, credentials or exfiltration. Fixtures use synthetic labels and bounded positive numeric values. No external endpoint or credential is needed.

## Case families

- **M/A:** genuine marker/journal saves and real simultaneous reading/self-withdrawal departures. Test malformed markers; distinguish syntactically valid unresolved archive roots from restore errors; mutate required archive records/nodes, sequence fields and duplicate payloads. Re-query retained archives after restoring a save and with immediate/delayed draining. Positive controls include exact save round trips, complete archives, identical duplicates and explicitly unauthenticated coherent payload edits.
- **W:** shared withdrawal restore rules, write-once reasons, explicit host-trust limits, policy/final-binding rollback, required own grounds and readable holders, and a short reachable reason chain. Separate rc.15 retention from approved draft departure.
- **P:** the missing intersection identified by source review: a genuine post-collection save that still contains an independently required withdrawal and a departure marker. Exercise malformed ledger/graph/marker edits, authentic restore, rollback and valid older-save continuation.

Each family preserves its inventory and expectations before the first run. Probe source and inputs are retained; a harness error is preserved and corrected explicitly, not recast as runtime success or silently discarded. Fresh result paths prevent overwriting attempts. `record-run.py` preserves raw stdout/stderr, exact argv, status and source/artifact hashes. There is no timing experiment or process-count claim beyond these bounded correctness cases.

## Interpret results against the correct contract

1. Shared guarantees: observed concrete withdrawal endpoints, the original reason/sequence/event after repetition, required reasons/qualifications, valid-save restoration and atomic rejection.
2. Approved draft behavior: legitimately departed records leave current snapshots, markers summarize compacted provenance, and archived withdrawals retain historical relationships. Rc.15 lacks those archive APIs; their absence is not a failed comparison.
3. Documented trust limits: restore checks source possibility and consistency, not historical authenticity. A source-possible edited save or consistent supplied archive payload can be accepted. Archive `complete` is scoped to supplied records and referenced dependencies, with `authenticated: false`.
4. A regression requires a concrete reproducer contradicting an applicable requirement. If found, preserve the input/output, minimize without modifying the runtime and use earlier draft artifacts only to locate the divergence. Do not repair semantics in this investigation.

The completed peak-memory and high-degree experiments remain unchanged. No push, PR mutation, merge, publication or release is authorized.

## Historical findings

The exact source and expectations for Muse F256/F358/F362/F363/F364 are not currently available. Their summary is preserved as context, not converted into invented test expectations. `historical-findings.json` records the bounded search and unresolved mapping. The user has been asked for their location. Repository-derived probes have new IDs and do not count as replaying those findings.
