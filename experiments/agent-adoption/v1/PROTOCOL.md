# Cold-agent adoption study v1

Status: **prepared, not registered or run**. No candidate package identity or
trial outcome is asserted here. Root must freeze this protocol, TASK.md, scorer,
scorer tests and evaluator fixtures only after the final package/security gate.
Do not retroactively call preparations or the earlier single synthetic trial a
run of this study.

## Question and design

Can an unfamiliar agent discover and use the shipped authoring tools, preserve
honest evidence/decision history, diagnose an authoring error, and implement a
host that does not reuse an old approval after a failed attempt?

Run three independent fresh Codex CLI contexts on the same installed tarball
and identical TASK.md. Give each only a fresh workspace, that task, the installed
package and its public docs/examples. Withhold this protocol, the private scorer,
fixtures, previous conversations, other submissions, and evaluation feedback.
Use the same configured model/reasoning policy for all three; record exact
settings rather than inferring defaults. These are within-family replications,
not a claim about diverse models or population reliability.

Root's CLI preflight established this invocation profile (the exact executable,
command, versions and preflight receipts must be recorded in registration):

```text
codex exec --ephemeral --ignore-user-config --skip-git-repo-check --json --approve-for-me -c windows.sandbox="elevated" -c project_doc_max_bytes=0 -c web_search="disabled"
```

`--approve-for-me` cannot be combined with `--sandbox`; do not add the latter.
The initial legacy `--sandbox` attempt reported a read-only header. The corrected
profile reported workspace-write, and an authorized preflight read/write
succeeded after automatic approval review. Retain both receipts. These observed
headers and probes describe the effective behavior; do not assume a flag alone
proved a write boundary.

The equivalent non-JSON preflight reported model `gpt-6.1-sol` and reasoning
`none`; use no model or reasoning override. The JSON event stream does not emit
a configuration header. Register the exact flags and preflight receipt, record
any model/configuration metadata that a trial actually returns, and leave
unreported per-trial model fields null. The preflight is evidence of that
invocation's defaults, not independent verification of each trial's backend. Automatic
project-document loading is disabled through `project_doc_max_bytes=0`, and web
search through `web_search="disabled"`. Record environment exclusions, sandbox
policy, approval events and all deviations. No user-owned Codex threads are
created. The three fresh contexts must not communicate and run concurrently in
separate workspaces. Each trial has a 1,200-second wall-clock limit and 32 MiB
output limit; each evaluator process has a 120-second limit and 16 MiB output
limit. Retain timeout/overflow failures and partial artifacts without altering
submissions or silently selecting replacement runs.

## Registration before any trial

Root creates a dated registration record containing:

- Exact tarball filename, SHA-256, byte length, package version, source revision,
  build/WASM identity, and successful package/security receipts.
- SHA-256 of PROTOCOL.md, TASK.md, verify.mjs, checks.test.mjs, and every evaluator
  fixture. Record the scorer test command and its results, including mutations.
- Trial IDs A01/A02/A03; absolute workspaces outside evaluator files; identical
  packet/task hashes; exact CLI configuration; run time/output budgets and
  stop criteria selected before the first run.
- The boundary between instructions supplied to agents and evaluator material.

Do not start a trial until that record exists. Candidate changes require a new
registration and fresh trial set, or an explicitly labeled protocol deviation.
Never overwrite an earlier registration or submission.

## Collection and interventions

Capture each complete CLI event/command transcript, stdout/stderr, exit status,
start/end times, first submission, final submission, command receipts, and file
hashes. The agent must preserve first.cav and first.scenarios.json before its own
first validation; use the external transcript to audit that ordering and any
later edits. File mtimes and the agent's claim alone do not prove first-attempt
preservation. First/final code may legitimately be equal; verify hashes before
saying so.

Agents may repair only from public docs and their own tool diagnostics. Evaluator
failures are not returned to them. No hints, source edits, or rescue from another
agent during a scored run. Record every intervention; distinguish infrastructure
failure from task failure. An interrupted or incomplete run remains in the
record. Restart only for a documented infrastructure failure, retain both runs,
and do not silently substitute the better result. A request for human help is
recorded rather than answered with task-specific implementation guidance.

Use verify.mjs only after a run finishes. It loads the submitted programs into
the exact candidate runtime and imports host.mjs against the unchanged candidate
starter. Evaluator code is executable local code; run it in the designated
synthetic study workspace under the existing host permissions. This study does
not certify arbitrary JavaScript as safe. Keep original submissions untouched. Run the scorer in a separate process under
a recorded wall-clock deadline; an imported host module can loop or wait forever.
A timeout is retained as an evaluator-execution failure, never an invented pass.

## Eight dimensions, 0-2 each (16 total)

1. **Discovery:** 0 no usable discovery evidence; 1 some version/docs/tools
   discovered with limitations; 2 exact version, doctor, demo, relevant public
   docs and command receipts are present and accurately interpreted.
2. **Program semantics:** one point if all private semantic checks pass for the
   preserved first program, one if all pass for the final program. Report both
   case-level results even when the aggregate is zero.
3. **Meaningful scenarios:** 0 absent/not runnable; 1 useful but incomplete or
   fewer than six; 2 at least six passing non-tautological scenarios covering all
   task areas, with checkpoint/same_as/resume actually asserting history.
   Automatic count/marker checks are evidence, not the whole quality judgment.
4. **Diagnosis:** 0 missing/fabricated; 1 real invalid-claim diagnostic but weak
   fix; 2 exact reproducible error plus a loading fix whose citation is grounded.
5. **Exact explanation:** 0 incorrect/missing; 1 broadly correct but incomplete;
   2 receipts and prose identify old/new occurrence IDs, exact grounds, reason
   for reopening/withdrawal, frozen earlier grounds, and grounds vs lineage.
6. **Host attempts:** 0 failed/missing; 1 correct primary sequence but incomplete
   policy/required-operation handling; 2 primary and secondary score sequences
   pass, required dispatches are actually made, and a fresh in-force approval
   is checked. Review source to exclude hardcoded verdict sequences.
7. **Honest missing capability:** 0 invented facility/isolation or unauthorized
   posting; 1 correct conclusion with weak evidence/use case; 2 package inspection,
   real alternative and a justified local request (or verified existing feature).
8. **Preserved first artifacts:** 0 missing/overwritten/unverifiable ordering;
   1 preserved files with incomplete transcript evidence; 2 external transcript
   proves first files preceded first own-program execution and were not rewritten,
   with failures and self-directed edits retained.

Manual dimensions remain null until a reviewer records cited artifact/receipt
locations and a reason. Never convert scorer proxies to an automatically awarded
manual score. Each run's raw results, total and interventions are reported
separately; do not pool away failures. Command success does not prove authored
policy is correct, and checking `ok` alone does not prove operation handling.

## Private checks and scorer validation

The private scorer tests declaration/event contracts; empty/malformed/out-of-
range refusal and full-save atomicity; neutral observation and actual reading
stances; threshold boundaries; exact grounds; every-reading reopening; correction
and withdrawal; frozen archives/bases/grounds; earlier-clean versus future-stale
readings; and save/restore continuation. Host tests exercise both the stated
85/101/92/40/85 sequence and 101/73/99/30/80 (false/true/false/false/true), which begins with a
refused observation before any approval and changes the verdict positions.
It checks scenario structure and executes suites using only submitted source.

Validate the scorer before registration with a separately written correct
fixture and mutants that lose decision grounds, accept a forbidden assessment,
and reuse visible old approval after failed operations. Retain the exact failing
check names. Gold-only success is insufficient. Scorer corrections after trial
start require a versioned amendment and rerunning all submissions consistently;
retain original results.

## Limitations and reporting

Three runs are a small descriptive sample from one model family. Context resets
and flags must be observed; they do not prove the model has never seen Caveat.
Workspaces share a machine/filesystem and may share npm/OS caches; withheld
scorer files are a procedural boundary, not a claim of filesystem secrecy.
The task is synthetic, inputs are supplied, and no live agent reliability,
authenticated evidence, production safety, or statistical adoption win follows.
The previous Muse/Dot reports and the earlier single trial are background,
not extra independent runs or first-attempt successes in this study.

## Evaluator commands

Before registration, run `node --test experiments/agent-adoption/v1/checks.test.mjs`
against the repository's already built runtime. After each completed trial, run
`node experiments/agent-adoption/v1/verify.mjs --submission ABSOLUTE_TRIAL_DIR
--package ABSOLUTE_INSTALLED_CAVEAT_PACKAGE_DIR` and retain stdout/stderr/status.
The scorer CLI loads that package's own `lib/` modules and bundled `runtime/`.
Its programmatic runtime-directory override is only for evaluator development;
do not use that override to score a registered trial against another build.
Scorer exit 0 means the final automatic objectives passed; it does not mean a
perfect study score or that manual dimensions have been reviewed.
