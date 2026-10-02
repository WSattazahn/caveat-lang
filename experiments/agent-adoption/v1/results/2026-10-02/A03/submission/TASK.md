# Caveat authoring and integration task

You are evaluating the Caveat package installed in this workspace. Use only this
task and the installed package's public documentation/examples for guidance.
Do not inspect the parent repository, evaluator, other trials, or external
projects. No network, issue posting, release, or publishing is needed.

Use the installed CLI to record its version, doctor result, and agent demo. Find
and read the package's agent quickstart, authoring guide, and relevant language
reference. Save command receipts (command, exit status, stdout/stderr) under
`receipts/`, including failed commands; keep a command index in
`receipts/commands.jsonl`. Record environment or tool limitations honestly.

## Build a tracker

Create a single-file `tracker.cav` with this external contract. You may add
states, definitions, procedures, and bindings as needed, but keep these names:

- Evidence `tool` and `correction`; claim `ready`.
- Caveat `uncertain` is declared to qualify `tool`; caveat `stale` is learned
  later. Both have material consequence.
- Readings `checks` come from `tool`, limit 16. Decisions `answer` have limit 8.
- Events `observe value min 0 max 100`, `assess`, `correct`, and `learn_stale`.

Required behavior:

1. `observe` records that `tool` was consulted without asserting that the
   template supports or opposes a claim, then samples `checks`: values >= 70
   support `ready`, values < 70 oppose it. Observation never commits. Every new
   reading, even an equal or supporting one, reopens an in-force answer.
2. `assess` rejects as authored policy when there is no reading, the latest
   reading is withdrawn, its value is below 70, that reading carries `stale`,
   or an answer is already in force. Otherwise it commits `answer` using the
   latest reading, with exactly that occurrence as grounds and its caveats.
3. `correct` rejects if no reading exists or the latest is already withdrawn.
   Otherwise it neutrally observes `correction`, withdraws the latest reading
   because of that correction, and reopens an answer resting on the withdrawal.
   Preserve the reading archive and the original decision's grounds and basis.
4. `learn_stale` rejects before `tool` is observed; otherwise it qualifies
   `tool` with `stale`. Earlier reading records and committed grounds stay
   frozen; later readings carry `stale`. This event does not reopen an existing
   answer. An earlier clean reading remains eligible for assessment after
   learning stale if no other refusal applies.
5. All refused events leave the complete session unchanged. Save/restore must
   preserve the behavior and history. No fabricated support relation may be
   attached to `tool` or `correction` to satisfy observation bookkeeping.

Before validating, checking, running, or evaluating your own tracker for the
first time, preserve your first complete candidate as **`first.cav`** and
**`first.scenarios.json`**. The first scenario document should name `first.cav`.
Do not rewrite those files after the first validation attempt. Then perform
self-directed repairs in `tracker.cav` and `tracker.scenarios.json`, whose
source is `tracker.cav`. Keep every failed receipt. Do not ask another agent
for a solution. If you cannot complete something, submit the actual result and
explain the limitation.

Register at least six meaningful scenarios before your first tracker run.
Collectively cover observation versus assessment, refusal, correction/revision,
repeated assessments, late qualification, and save/restore. Include actual
`checkpoint`, `same_as`, and `resume` steps. Final scenarios must run with the
installed scenario runner. Additional boundary values may be used in evaluation.

Run validate, check, and test. Use explain and dependents on an event history
that demonstrates a decision, correction, and revision. Save that history as
`events.jsonl` and the unedited command outputs. In `explanation.md`, name the
exact old/new occurrence IDs and decision grounds, the correction reason, and
what remained unchanged. Distinguish grounds from lineage.

In `diagnosis.md`, include a separate minimal program that deliberately cites a
claim in an invalid `because` position, its exact diagnostic from the installed
version, and a corrected program that loads and honestly cites what its value
uses. Keep this diagnostic exercise separate from the preserved tracker attempt.

## Integrate the unchanged official starter

Write `host.mjs` exporting **`async function attempt(session, score)`**, returning
an object with a boolean `ok` plus any useful details. Do not print from this
module. The caller supplies a public Caveat session already loaded from the
installed, unchanged `examples/agent-evidence/assessment.cav`.

Each attempt's required operations are `observe` with `{confidence: score}` and
then `assess`. Both must be accepted and the current assessment must be approved,
in force, and committed during that attempt for `ok` to be true. You may stop
sending after a rejected observation. Do not add a reopening event or change
the starter's source/policy. An old visible approval cannot cover a failed
required operation. Do not close the supplied session.

On one fresh starter session, calls with scores **85, 101, 92, 40, 85** must return
**true, false, false, false, true**. Show that 101 is refused while the old
approval remains visible; 92 cannot assess again without reopening; 40 reopens
but does not permit an answer; the final fresh strong reading can be assessed.
The same rule must work for other scores, not just this example sequence.

## Investigate one missing capability

Determine whether this installed package exposes a public `whatif` or session
fork facility for hypothetical events while preserving a live session. Cite the
actual public commands/APIs and documented alternatives you inspected. If the
needed public facility is absent, write a local **`FEATURE_REQUEST.md`** with a
small use case, exact version, expected behavior, existing alternative and an
acceptance check. If present, demonstrate it and explain why no request is
needed. Do not invent a command/API, claim process isolation without evidence,
or post an issue.

Finish with `RESULTS.md` listing first-attempt status, self-directed corrections,
final commands/outcomes, files, and remaining limitations. The evaluator grades
what you actually built and recorded, not confident wording.
