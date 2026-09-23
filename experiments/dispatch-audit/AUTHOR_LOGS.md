# Frozen author-log rejection-origin audit

This is retrospective, source-corroborated **text inference**, not structured runtime attribution. No author candidate was rerun. SMOKE is excluded. Only recorded author checks in `runs/*/checks/[0-9]+.jsonl` are counted; inputs, console transcripts, verifier/scoring runs, and final-result verdicts are not additional observations.

Run `node experiments/dispatch-audit/author-logs.mjs` from the repository root to regenerate this report and `results-author-logs.json`. Run `node --test experiments/dispatch-audit/author-logs.test.mjs` for the classifier fixture. The script reads frozen inputs and writes only these two audit outputs.

## Counts

Check errors are failed construction/validation/check operations, not rejected dispatch events. Rejected events below are partitioned into authored policy, recognized other error text, and unknown. Other errors include deliberately invalid input probes and bounded-history protections; this audit does not call them implementation bugs.

| Study | Run | Checks | Check errors | Accepted events | Rejected events | Authored policy | Other errors | Unknown |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| v1 | A1 | 8 | 1 | 71 | 47 | 13 | 34 | 0 |
| v1 | A2 | 5 | 1 | 37 | 25 | 6 | 19 | 0 |
| v1 | B1 | 7 | 1 | 54 | 38 | 16 | 22 | 0 |
| v1 | B2 | 6 | 1 | 62 | 29 | 12 | 17 | 0 |
| v1 | C1 | 6 | 0 | 34 | 57 | 29 | 28 | 0 |
| v1 | C2 | 8 | 0 | 44 | 65 | 35 | 30 | 0 |
| v2 | A1 | 5 | 0 | 40 | 33 | 7 | 26 | 0 |
| v2 | A2 | 5 | 0 | 47 | 32 | 9 | 23 | 0 |
| v2 | B1 | 5 | 0 | 44 | 39 | 13 | 26 | 0 |
| v2 | B2 | 8 | 1 | 61 | 50 | 20 | 30 | 0 |
| v1 | **total** | 40 | 4 | 302 | 261 | 111 | 150 | 0 |
| v2 | **total** | 23 | 1 | 192 | 154 | 49 | 105 | 0 |

## Attribution rule and evidence

A policy classification requires the entire prefix `event IDENTIFIER, rule POSITIVE_INTEGER: rejected: `, exact agreement with the logged event name, the snapshot source identity matching the preserved version, and the corresponding top-level `on` rule having the identical `reject` literal. A `rejected: ` substring alone is never sufficient. The corroborator respects quoted strings, comments, and procedure braces; it declines expansion/linking and nested-procedure attribution instead of guessing.

Both recorded source commits have the unique `Effect::Reject` constructor and the event/rule and procedure/step wrappers. The JSON preserves each historical source commit, source SHA-256, constructor line/text, frozen runtime manifest/artifact hashes, runner hash, run-record hash, saved-version hash, every check-log/input hash, and every rejected observation's JSONL line, input line, source version, error, and classification. Historical runtime source is read with `git show` at each packet's recorded commit; current core edits do not alter this evidence. Packet WASM hashes match their manifests. This verifies file consistency, not a cryptographic proof that the historical WASM was compiled from those source bytes.

## Check-level errors

| Run | Check | Version | Saved message |
|---|---:|---:|---|
| v1/A1 | 1 | 1 | line 14, column 1: reactive bounds must be ordered and within +/-1e12 |
| v1/A2 | 1 | 1 | line 13, column 1: reactive bounds must be ordered and within +/-1e12 |
| v1/B1 | 1 | 1 | line 20, column 1: reactive bounds must be ordered and within +/-1e12 |
| v1/B2 | 1 | 1 | line 17, column 1: reactive bounds must be ordered and within +/-1e12 |
| v2/B2 | 1 | 1 | line 1, column 1: cannot parse statement: -- Fixed assertions |

## Other rejected-event templates

| Study | Recognized template | Count |
|---|---|---:|
| v1 | closed_revision_text | 2 |
| v1 | history_limit_text | 9 |
| v1 | numeric_range_text | 44 |
| v1 | parameter_set_text | 47 |
| v1 | parameter_type_text | 15 |
| v1 | payload_decode_text | 25 |
| v1 | undeclared_event_text | 8 |
| v2 | history_limit_text | 9 |
| v2 | numeric_range_text | 23 |
| v2 | parameter_set_text | 21 |
| v2 | parameter_type_text | 8 |
| v2 | payload_decode_text | 39 |
| v2 | undeclared_event_text | 5 |

## Counterfeit text and limits

Authored text or payload strings can contain `rejected: `, including apparently complete contextual prefixes. Unknown-event/payload/parser diagnostics can repeat supplied text; quoted source literals and repetition templates can carry arbitrary text. Such embedded text retains an outer error template and is not a policy rejection. The fixture checks counterfeit markers in payload diagnostics, quoted source text, event-name spoofing, and a source-rule/message mismatch. Saved versions contain no `rejected: ` literal and use no repetition/linking expansion; procedures occur, but no observed policy rejection has a nested procedure frame.

The anchored prefix plus preserved direct-rule corroboration makes the observed policy cases strong historical evidence, but does not turn untyped strings into a runtime guarantee. Attribution assumes trustworthy recorded logs and runtime provenance. A fabricated entry or novel diagnostic reproducing the complete matching frame, source identity, rule, and reject literal cannot be detected by this retrospective classifier; an unrecorded runtime change can likewise invalidate the inference. Unsupported source rewriting and nested procedure attribution remain unknown. The classifier deliberately recognizes only the other-error templates observed here; unrecognized formats remain unknown. Authored policy means the reject effect is inferred to have caused the recorded failure, not that the author's admission policy was correct.
