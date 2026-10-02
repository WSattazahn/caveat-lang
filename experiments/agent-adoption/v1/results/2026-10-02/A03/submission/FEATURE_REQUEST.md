# Local feature request: managed hypothetical session branching

Installed package: caveat-lang 0.1.0-rc.7 (CLI --version and doctor receipts).
This is a local draft; no network or issue posting was performed.

Use case: after a live approval on confidence 85, preview observe 40 then assess and obtain the refusal and explanation without replacing the live approval or changing its save.

Inspected public surfaces:
- CLI --help lists test, explain, dependents, validate, check, replay, serve, init, doctor, demo agent, mcp and --version. There is no whatif/fork command.
- docs/reference/spec/caveat-serve-0.1.md lists dispatch, snapshot, explain, dependents, save, restore and close. restore replaces the live server's session.
- lib/session.d.mts exposes session dispatch/dispatchView, snapshot/view, save and close; runtime open, check and restore. It exposes no dedicated fork/whatif method.
- examples/agent-evidence/BRANCHING.md explicitly states there is no whatif command or Serve operation and that a public whatif API is deferred.
- README.md and docs/reference/docs/AI_AUTHORING.md describe session save/restore and link the branching alternative. Public docs/API text is archived by docs-read.mjs in receipts.

Existing alternative: save the live session, call loadRuntimeFromDirectory() again, restore the exact source/save in that separate runtime, send hypothetical events, inspect, then close the branch. branch-test.mjs demonstrates this and asserts unchanged live save and snapshot. Same-runtime restore makes another session sharing a WASM instance; it is not trap isolation. The docs also offer a separate Python CaveatServer process. I did not run the Python recipe or inject traps, and make no claim about process-wide failure or resource isolation.

Expected new capability: a documented managed hypothetical operation or fork facility that creates a disposable branch from a live checkpoint, runs events, returns their outcomes and explanation, and owns cleanup. Specify which failures are isolated and resource limits, rather than requiring callers to reconstruct this lifecycle.

Acceptance check: approve 85 on live; capture save and snapshot; preview observe 40 and assess; report accepted observation and policy refusal; close the branch; require byte-identical live save and deep-equal live snapshot, with its approval still in force. Also verify cleanup and document/test fatal and trap behavior without promising isolation beyond the tested boundary.
