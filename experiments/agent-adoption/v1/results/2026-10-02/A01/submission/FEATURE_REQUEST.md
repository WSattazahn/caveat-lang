# Local request: public hypothetical-session operation

Package: caveat-lang 0.1.0-rc.7; runtime revision a24a6e91d75eeab466baa32a108ee9a242f43ac2; WASM SHA-256 e0c1ff6921485ac342f4eced76c68f58b14615077838c6f2af7da725430c6ba3. No issue was posted.

## Use case and expected behavior

After observe confidence 85; assess in the unchanged official assessment.cav, a host wants to try observe confidence 40; assess and inspect the refusal while retaining the live approval and complete live session. A public hypothetical operation should accept a checkpoint and event list, return outcomes and explanation, and leave the live save/snapshot/view unchanged. Its lifecycle, trap isolation, cancellation and resource limits should be explicit. This is a requested interface, not an existing callable API.

## Public surfaces inspected

- Installed CLI --help lists test, explain, dependents, validate, check, replay, serve, init, doctor, demo agent and mcp. No whatif/fork command is listed (receipts/004.stdout.txt).
- [Session API declarations](node_modules/caveat-lang/lib/session.d.mts) expose runtime.open(source), runtime.restore(source, saved), and session.dispatch, dispatchView, snapshot, view, save, close (plus text variants). No session fork/whatif method is declared.
- [Node loader declarations](node_modules/caveat-lang/lib/node.d.mts) expose loadRuntimeFromDirectory.
- [Serve specification](node_modules/caveat-lang/docs/reference/spec/caveat-serve-0.1.md) exposes dispatch, snapshot, view, explain, dependents, save, restore and close. Restore replaces the current session.
- [MCP guide](node_modules/caveat-lang/docs/MCP.md) lists caveat_validate, caveat_check, caveat_test, caveat_explain, caveat_dependents; it explicitly has no persistent-session or whatif tools.
- [Official branching guide](node_modules/caveat-lang/examples/agent-evidence/BRANCHING.md) says "There is no whatif command or Serve operation in this candidate" (whatif is code-formatted in the original), and documents branching through save/restore in a separate runtime. It says a public whatif API is deferred.

Full inspected documentation and declarations are retained in the read-public receipt. Scope of the absence finding is these public installed commands, declared APIs, and documentation; no private implementation or external project was inspected.

## Existing alternative and actual evidence

branch-test.mjs uses session.save(), another loadRuntimeFromDirectory() invocation, and branchRuntime.restore(theExactSource, saved). It dispatches the hypothetical events only on that branch. It verifies equal runtime identities, the restored checkpoint, policy/reject on assess after 40, and byte-identical live save plus deep-equal live snapshot. branch-results.json retains the snapshots and refusal.

The official guide also documents a second Python CaveatServer process. That Python alternative was inspected but not run. A second session on the same WASM runtime shares trap fate; it is not the documented isolation alternative. Our demonstration used separate loader calls in one Node process. It did not test traps, process isolation, host side effects, or resource exhaustion.

## Acceptance check for a future facility

Start a live official starter session at approved assessment@1, retain its save/snapshot/view, and run hypothetical observe 40; assess through the new documented facility. Require accepted observation, policy/reject assessment, a branch report showing reopened assessment@1, and exact equality of all three retained live representations. Release the branch, then verify the live session still dispatches normally. Add documented failure/trap/cancellation tests matching the promised isolation and lifecycle contract.

The manual alternative works for this use case; the missing capability is a dedicated public operation and its verified lifecycle contract, not inability to save/restore.
