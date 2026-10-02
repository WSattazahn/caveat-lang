# Results

Installed caveat-lang 0.1.0-rc.7 on Node 24.11.1. Version, doctor and agent demo all exit 0 (receipts 001–003). Doctor reports ok=true; both CLI names are absent from PATH, so commands use node node_modules/caveat-lang/bin/caveat.mjs directly. No network, external posting, publishing, parent repository inspection, or other agent was used.

First attempt: first.cav and first.scenarios.json were written before the first tracker validation. The scenario source is first.cav. Validation exits 0 (004), and all seven preregistered scenarios pass (005). Both preserved files remain untouched. No tracker source repair was necessary. Final tracker.cav is byte-identical to first.cav. Final scenarios add S8/S9 for reading/decision history capacity; all nine pass (020).

Self-directed corrections were limited to the separate diagnostic exercise: add a required event, then replace an invalid direct evidence citation with the state actually used. All failed CLI receipts are retained (012, 013, 015). The final invalid-claim diagnostic is deliberate; the corrected example validates and runs.

Final command outcomes (prefix: node node_modules/caveat-lang/bin/caveat.mjs):

| Command | Exit | Receipt |
| --- | --- | --- |
| validate tracker.cav | 0 | 006 |
| check tracker.cav | 0, no warnings | 007 |
| test tracker.scenarios.json | 0, nine pass | 020 |
| explain tracker.cav events.jsonl --json | 0 | 009 |
| dependents tracker.cav checks events.jsonl --json | 0 | 010 |
| replay tracker.cav events.jsonl | 0 | 011 |
| validate diagnosis-invalid.cav | 2, deliberate claim citation | 015 |
| validate diagnosis-corrected.cav | 0 | 016 |
| node integration-test.mjs | 0 | 017 |
| node branch-test.mjs | 0 | 021 |

The unchanged installed assessment.cav is loaded directly by the caller test. host.mjs exports async attempt(session, score), does not print or close the supplied session, and checks both required operations, approval in force and commitment during the attempt. The sequence 85,101,92,40,85 returns true,false,false,false,true. The 101 refusal retains the old approval with a byte-identical save; 92's observe succeeds but assess refuses Already assessed; 40 reopens but assess refuses weak evidence; the last 85 commits assessment@2. Eight other boundary scores are tested. Full results, snapshots and starter SHA-256 are in host-results.json. Calls are sequential; concurrent callers must serialize attempts for one session.

Files: tracker.cav / tracker.scenarios.json; preserved first.cav / first.scenarios.json; host.mjs / integration-test.mjs / host-results.json; events.jsonl / explanation.md; diagnosis.md / diagnosis-invalid.cav / diagnosis-corrected.cav / diagnosis-snapshot.json; FEATURE_REQUEST.md / branch-test.mjs / branch-results.json; receipts/commands.jsonl and numbered unedited CLI stdout/stderr; public-docs.mjs and receipt.mjs are receipt helpers. author.mjs records how the first candidate was created: do not rerun it, since it would replace the preserved files. extend-scenarios.mjs records the later scenario addition; do not rerun it, since it would duplicate IDs.

Hashes:

- first.cav: 02f1d8b3296af82e43d87ec1e25eb1b7e989b6fe55d2f4bb1bbe35528be79e4b
- tracker.cav: 02f1d8b3296af82e43d87ec1e25eb1b7e989b6fe55d2f4bb1bbe35528be79e4b
- first.scenarios.json: 63cbda14dba8ee03bdbb4c28635de2081d6c05aa34d96b6ad2e455b7cfd678b5
- installed starter: 4119af19e00135d0d7a1207f2e68ec515cecd0bc9932d9dce8ecb4136c5bcc02

Remaining limitations: capacity is intentionally 16 readings / 8 decisions; exhaustion refuses atomically. Scores are supplied values, not authenticated evidence. The hypothetical alternative demonstrates ordinary branch behavior only, without a trap or process isolation test. Public whatif/fork is absent; the local request describes the need.

Environment/receipt limitation: early tool commands unexpectedly ran from <DRIVE_ROOT>/ despite workdir, and Set-Location to the named workspace was denied. The first broad rg therefore searched outside the intended scope accidentally; its output was truncated and it was interrupted. No such results were used as guidance. Elevated commands explicitly entering only the named workspace then worked. Bootstrap failures predated the receipt helper; retained bootstrap excerpts are labeled partial, and their complete stdout/stderr could not be recovered. Later CLI and script subprocess outputs are unedited and complete. Public documentation was reread into receipt 018 to preserve full text. Shell authoring/read wrapper outputs were visible in tool transcripts but were not all separately captured as subprocess receipts; this is an unmet part of the request to record every command, not a claim of complete bootstrap logging.
