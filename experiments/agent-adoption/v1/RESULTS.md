# Agent adoption study v1 — results

Completed 2026-10-02 UTC (October 1 in the operator's local time). All three
final integrations passed the registered automatic objectives. Two initial
programs passed; the third initially failed to load and was repaired without
outside implementation help. These are descriptive artifact results, with
workspace-boundary deviations described below; they are not three clean,
isolated packet-only trials or evidence of an adoption improvement.

## Registered artifact and procedure

The [protocol](PROTOCOL.md), [task](TASK.md), scorer and 16 passing scorer controls
were frozen before execution in `247c7f4`; the candidate registration was
committed in `2b3cfe6`. Registration was prepared at 00:47:20 UTC; all three
fresh CLI contexts started at 00:47:48 UTC. Evaluation ended at 00:55:54 UTC.
The [registration](registration/registration.json) retains its original
`registered-not-run` status as a historical record, not current status.

- Registration SHA-256:
  `61c1377453452679d46199cade4d9d4f4129905522b76d0746ec7bb6087ef7e9`.
- Candidate: `caveat-lang@0.1.0-rc.7`, 810,225 bytes, SHA-256
  `eea2c9239f3d648a1af740843abee5ed1eac1c972c327793bb9af2ede36b9a1f`.
- Clean compiled source revision:
  `a24a6e91d75eeab466baa32a108ee9a242f43ac2`.
- Windows WASM SHA-256:
  `e0c1ff6921485ac342f4eced76c68f58b14615077838c6f2af7da725430c6ba3`.
- Codex CLI 0.159.2; Node 24.11.1; npm 11.6.2. No model or reasoning override.
  The equivalent non-JSON preflight reported `gpt-6.1-sol`, effort `none`;
  JSON trials did not independently report these settings. Per-trial model and
  effort fields remain null.

Each context received the same task and installed package. Project instruction
discovery and web search were disabled. Agents shared a Windows host and caches;
there was no operating-system isolation from other host files. Budgets were
20 minutes and 32 MiB per trial, with a separate 120-second evaluator deadline.
All three exited zero within those limits. No trial was replaced, restarted,
rescued, or given private evaluator feedback. Each agent repaired only from its
own diagnostics and the material it read. Evaluation used each installed
candidate's own library and runtime. Before/after manifests confirm that
scoring did not change the submissions or installed package files.

## Outcomes

| Trial | First program checks | Final program checks | Passing authored scenarios (first → final) | Host cases | Time | Semantics + reviewed dimensions | Total |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A01 | 15/15 | 15/15 | 6 → 8 | 10/10 | 479.758 s | 2 + 14 | 16/16 |
| A02 | 15/15 | 15/15 | 7 → 9 | 10/10 | 464.199 s | 2 + 14 | 16/16 |
| A03 | Load failure; 0/15 | 15/15 | 0 → 6 | 10/10 | 483.374 s | 1 + 14 | 15/16 |

A03 preserved six initial scenarios, but its initial program could not load, so
those scenarios were not passing. Program semantics receives one point for
all first-program checks passing and one for all final-program checks passing.
The other seven dimensions were reviewed manually against source, receipts and
external command ordering; their maximum is 14. The totals describe the rubric,
not a clean-run or production-readiness certification. The boundary deviations
are reported independently rather than silently changing the frozen rubric.

All three final programs separate neutral observation from assessment, retain
frozen decision grounds, reopen on new readings, withdraw corrected readings,
apply late caveats to future values, and resume saved history. Host integrations
perform required operations and require a fresh in-force approval, so a refused
observation cannot reuse a visible old approval. Each submission includes real
invalid-citation diagnostics, a grounded repair, an explanation distinguishing
grounds from lineage, and an unposted capability request. None invents public
`whatif` or claims that save/restore protects a live session against traps.

A01 and A02's first/final program hashes match in their evaluator reports;
they expanded scenarios without replacing their first artifacts. A03 first used
`has_caveat(latest(checks), stale)`, although that function expects a declared
state. It repaired this by binding the reading into a state before checking its
caveat. It also corrected an observation-ledger expectation to include the
sampled reading occurrence. Initial source, failed checks and repair commands
are retained. The separate diagnosis exercises include failed repairs too.

## Boundary and collection deviations

The Windows shell initially executed outside the requested working directory.
All three contexts saw unrelated root-directory listings. A02 also ran a broad
`rg --files` filename enumeration from the drive root; its returned listing was
truncated. These commands violated the intended packet-only read boundary.
Review found no command reading private repository, evaluator, gold fixture or
other-trial source contents. Filename exposure remains a real deviation and
prevents describing these as clean isolated trials. Later authoring and checks
used the intended workspace and installed public package.

The public archive omits those unrelated listings and minimizes local paths
and session identifiers. It preserves command order, source event references,
exit status, redaction reasons, original transcript hashes and original artifact
hashes. It does not present the redacted command export as the original raw
transcript. Complete originals remain in the ignored local receipt store.
Authored partial bootstrap receipts are distinguished from external captures.
See the [archive index](results/2026-10-02/README.md) and its redaction manifest.

After the runs ended, review identified a runner defect: output-capture I/O
exceptions could be suppressed. No capture failure was observed in the retained
runs, whose external JSONL parses and includes completed turns. The runner is
now corrected and six synthetic capture controls pass. The exact executed
runner remains archived; the [post-run amendment](RUNNER_AMENDMENT.md) explicitly
separates that fix from these unchanged registered results. Future execution
requires a fresh registration, including any improved workspace boundary.

## What this establishes

In this small synthetic exercise, all final artifacts implemented the requested
semantics and protected the host's attempt policy. Two first programs passed,
and one agent needed a self-directed syntax repair. Doctor, demo, the packaged
authoring guidance and the official starter were discovered and used.

There is no control group, cross-model sample, live evidence collection, or
statistical adoption claim. Fresh CLI context does not prove the model never
encountered Caveat before. Muse and Dot's earlier reports are background, not
additional trials. The current study also does not test a live desktop MCP
installation. Its Windows package is a measured study candidate, not an npm
publication or authorization to release.

The [evidence archive](results/2026-10-02/README.md) contains the first and final
artifacts, independent evaluator reports, manual scoring evidence and reviewed
command exports. The frozen task, scorer and registration stay alongside this
report so later work cannot silently redefine the observed result.
