# Results

Installed Caveat Language 0.1.0-rc.7, Node 24.11.1. Doctor ok=true with PATH warnings; agent demo passed. Used the installed CLI by direct Node file invocation. No network, publishing, issue posting, parent repository, evaluator, other trial, or external project inspection.

## Preserved first attempt and final tracker

first.cav and first.scenarios.json (source first.cav) were copied/written before the first validation attempt. First validate exited 0 and all six preregistered scenarios passed. They have not been rewritten. first-manifest.json records their hashes and receipts.

No tracker source repair was required: tracker.cav remains byte-identical to first.cav. tracker.scenarios.json names tracker.cav. Two later capacity scenarios were added; all eight final scenarios pass. Six scenarios were registered before any tracker evaluation. They include actual checkpoint, same_as and resume steps and cover the required policies. The runner checks every refusal for complete atomicity and all final saves for restore equality.

Final validate: exit 0. Check: exit 0, no warnings. Final test: exit 0, eight passed. Explain, dependents, JSON explain, and replay on events.jsonl: exit 0.

## Host integration

host.mjs exports async attempt(session, score), prints nothing and does not close the caller session. Success requires both required operations accepted, an approved current in-force commitment, and a commit journal entry made by that attempt. No reopening event or starter modification is added.

host-test.mjs passed: scores 85, 101, 92, 40, 85 return true, false, false, false, true. 101 rejects while the old approval stays visible and the complete save remains unchanged. 92 records support but assess refuses Already assessed. 40 reopens and assess refuses weak evidence. Final 85 commits assessment@2. Additional fresh-session scores 0, 69, 70, 100, -1, 102, NaN and Infinity passed. host-results.json retains full outcomes/snapshots. Starter SHA-256 was identical before/after: 4119af19e00135d0d7a1207f2e68ec515cecd0bc9932d9dce8ecb4136c5bcc02.

## Diagnostic and missing-capability investigation

The separate invalid claim citation fails with the exact version diagnostic in diagnosis.md. An attempted bare-evidence correction also failed; its receipt is retained. The corrected expression citation loads and replays successfully. These failures did not alter the tracker or first attempt.

Public whatif/fork facility absent on inspected installed surfaces. FEATURE_REQUEST.md cites those surfaces, the documented save/restore alternative and a future acceptance check. branch-test.mjs demonstrates the separate-loader alternative and preserves the live save and snapshot. No issue posted.

## Commands and exit statuses

Each numbered JSON receipt contains the command arguments, exit status, unmodified captured stdout/stderr. Matching .stdout.txt/.stderr.txt files expose the raw streams. receipts/commands.jsonl is the index. Nonzero diagnostic receipts are retained.

| Receipt | Command | Exit |
| --- | --- | --- |
| receipts/001.json | node node_modules/caveat-lang/bin/caveat.mjs --version | 0 |
| receipts/002.json | node node_modules/caveat-lang/bin/caveat.mjs doctor --json | 0 |
| receipts/003.json | node node_modules/caveat-lang/bin/caveat.mjs demo agent | 0 |
| receipts/004.json | node node_modules/caveat-lang/bin/caveat.mjs --help | 0 |
| receipts/005.json | node build-scenarios.mjs | 0 |
| receipts/006.json | node node_modules/caveat-lang/bin/caveat.mjs validate first.cav | 0 |
| receipts/007.json | node node_modules/caveat-lang/bin/caveat.mjs test first.scenarios.json | 0 |
| receipts/008.json | node host-test.mjs | 0 |
| receipts/009.json | node node_modules/caveat-lang/bin/caveat.mjs validate tracker.cav | 0 |
| receipts/010.json | node node_modules/caveat-lang/bin/caveat.mjs check tracker.cav | 0 |
| receipts/011.json | node node_modules/caveat-lang/bin/caveat.mjs test tracker.scenarios.json | 0 |
| receipts/012.json | node node_modules/caveat-lang/bin/caveat.mjs explain tracker.cav events.jsonl | 0 |
| receipts/013.json | node node_modules/caveat-lang/bin/caveat.mjs dependents tracker.cav checks events.jsonl | 0 |
| receipts/014.json | node node_modules/caveat-lang/bin/caveat.mjs explain --json tracker.cav events.jsonl | 0 |
| receipts/015.json | node node_modules/caveat-lang/bin/caveat.mjs replay tracker.cav events.jsonl | 0 |
| receipts/016.json | node node_modules/caveat-lang/bin/caveat.mjs validate invalid-because.cav | 2 |
| receipts/017.json | node node_modules/caveat-lang/bin/caveat.mjs validate corrected-because.cav | 2 |
| receipts/018.json | node node_modules/caveat-lang/bin/caveat.mjs validate corrected-because.cav | 0 |
| receipts/020.json | node branch-test.mjs | 0 |
| receipts/021.json | node extend-scenarios.mjs | 0 |
| receipts/022.json | node node_modules/caveat-lang/bin/caveat.mjs test tracker.scenarios.json | 0 |
| receipts/023.json | node node_modules/caveat-lang/bin/caveat.mjs replay corrected-because.cav diagnosis.events.jsonl | 0 |

## Files

- tracker.cav, tracker.scenarios.json: final source and eight scenarios.
- first.cav, first.scenarios.json, first-manifest.json: preserved initial candidate and recorded hashes.
- events.jsonl, explanation.md: decision/correction/revision evidence and exact grounds versus lineage.
- host.mjs, host-test.mjs, host-results.json: integration and verification.
- invalid-because.cav, corrected-because.cav, diagnosis.events.jsonl, diagnosis.md: separate diagnostic exercise.
- branch-test.mjs, branch-results.json, FEATURE_REQUEST.md: capability investigation and working alternative.
- receipts/, receipt.mjs, read-public.mjs: command receipts and capture helpers.
- build-scenarios.mjs, extend-scenarios.mjs: initial and later scenario authoring scripts. Do not rerun the initial builder over the preserved files.

## Remaining limitations

The managed shell initially started in <DRIVE_ROOT>/ despite workdir, then denied Set-Location to the exact allowed workspace. A sandbox escalation succeeded. The global caveat command was not on PATH; direct installed CLI invocation worked. Shell discovery and some document-inspection calls were made directly through the tool API; their complete raw stdout/stderr were not all saved to disk. The initial discovery calls also preceded the receipt helper. The tool conversation retains those calls; bootstrap-limitations.json records the exact failed commands and known errors without pretending to be complete stream captures. All installed CLI executions, including diagnostic failures, have complete numbered receipts.

The tests cover authored policy and bounded history (16 readings, 8 revisions), not authenticity of supplied scores or arbitrary runtime correctness. Branch tests verify state preservation for ordinary accepted/rejected hypothetical events; they do not establish process isolation, trap containment, cancellation or resource-exhaustion behavior. No external issue search was performed because network access was excluded.
