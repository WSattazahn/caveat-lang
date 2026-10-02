# Results

Installed Caveat Language 0.1.0-rc.7; Node 24.11.1. Doctor and agent demo exited 0. Doctor passed its runtime checks and warned that neither CLI name is on PATH; commands used the installed bin/caveat.mjs directly.

First attempt: first.cav and first.scenarios.json were written before any tracker validation/check/test/runtime execution. First validate exited 2; all six first scenarios failed at load. Diagnostic: line 19, column 1: expected comma before the state caveat at byte 17. These preserved files were never rewritten.

Self-directed corrections: has_caveat requires a declared state, so assess now copies latest(checks) into assessed, checks its caveats, and commits using latest(checks). A refused assess rolls back that state assignment with the complete transaction. The S1 observation ledger expectation was corrected to include both tool and checks@1, as sampling observes the reading too. No tracker policy was weakened.

Final commands and outcomes (all command stdout/stderr retained):

| Command | Exit | Result |
| --- | --- | --- |
| caveat-lang --version | 0 | completed |
| caveat-lang doctor --json | 0 | completed |
| caveat-lang demo agent --json | 0 | completed |
| caveat-lang validate @first.cav | 2 | failed; receipt retained |
| caveat-lang test @first.scenarios.json | 1 | failed; receipt retained |
| caveat-lang validate @tracker.cav | 0 | completed |
| caveat-lang check @tracker.cav | 0 | completed |
| caveat-lang test @tracker.scenarios.json | 1 | failed; receipt retained |
| caveat-lang test @tracker.scenarios.json | 0 | completed |
| caveat-lang explain @tracker.cav @events.jsonl | 0 | completed |
| caveat-lang dependents @tracker.cav checks @events.jsonl | 0 | completed |
| caveat-lang validate @invalid-because.cav | 2 | failed; receipt retained |
| caveat-lang validate @corrected-because.cav | 2 | failed; receipt retained |
| host-test.mjs | 0 | completed |
| caveat-lang --help | 0 | completed |
| caveat-lang validate @invalid-because.cav | 2 | failed; receipt retained |
| caveat-lang validate @corrected-because.cav | 0 | completed |
| caveat-lang explain @corrected-because.cav @diagnostic-events.jsonl | 0 | completed |
| caveat-lang dependents @tracker.cav correction @events.jsonl | 0 | completed |
| branch-test.mjs | 0 | completed |
| docs-read.mjs | 0 | completed |

Final tracker validate succeeded; check reported no warnings; test passed all 6 scenarios. Scenarios cover observation versus assessment, policy and input refusal, correction/revision, equal supporting readings and duplicate assessments, late qualification and save/restore. The runner verifies full-session atomicity on every refusal and continued shadow-session agreement after resumes.

The unchanged official assessment starter was loaded directly from node_modules. host.mjs exports async attempt(session, score), emits no output, and leaves ownership/closure to the caller. host-test.mjs verifies 85,101,92,40,85 => true,false,false,false,true and fresh-session boundaries 0,69,70,100,-1,101. At 101 the old approval remains visible but observation is refused; 92 observes but cannot reassess an in-force answer; 40 reopens and fails assessment; final 85 commits assessment@2 on observations@4.

Files: tracker.cav, tracker.scenarios.json; immutable first.cav, first.scenarios.json; events.jsonl; explanation.md; host.mjs, host-test.mjs; invalid-because.cav, corrected-because.cav, diagnostic-events.jsonl, diagnosis.md; FEATURE_REQUEST.md, branch-test.mjs; record.mjs, docs-read.mjs; receipts/commands.jsonl and numbered JSON/stdout/stderr receipts.

Preserved first hashes:

- first.cav SHA-256: 6638c00116ea0eaa40ac2ac0e2f9da605c49c29bca080af2a293f71eefbf8501
- first.scenarios.json SHA-256: e17026b21ab321cc4f13c17af71b9b118edef5482969425eabafefefb736fd81

Limitations: the shell ignored workdir and started at <DRIVE_ROOT>/; Set-Location failed due to sandbox ancestor metadata restrictions, and Node entrypoint loading initially failed with EPERM lstat on <REPO>/test-results. Local execution succeeded after an approved sandbox escalation. No network, parent repository, evaluator, other trials, posting or publishing was used. Public docs/API were the only package guidance. Some bootstrap exploratory command streams were available only as merged tool output; bootstrap receipts explicitly mark transcribed merged diagnostics and do not pretend to recover separate streams. Full CLI/runtime stdout/stderr capture begins with the receipt recorder.

There is no dedicated public whatif/fork facility in the documented commands/API. The documented separate-runtime save/restore alternative passed a live-preservation check. The local feature request describes the remaining lifecycle convenience and isolation contract; no trap injection, process-isolation or resource-exhaustion experiment was performed.
