# Fresh-agent Caveat authoring study, v1

Question: can fresh agents author several specified evidence-aware programs
using Caveat's documentation and runtime tools, without implementation access
or coaching from the language's authors?

This is a repeated feasibility study of Caveat alone. It does not measure a
comparative reliability advantage, user adoption, or another model's ability.

## Registration and isolation

Before any author starts, commit this protocol, all three public task contracts,
the public runner/reference manifest, the private oracle and cases, and the
verification harness. Record their hashes in `registration.json`. The frozen
language implementation is commit `04f72dc` (runtime change `c5c0183`). The
reference packet is copied byte-for-byte from that commit; its generated
reference copies and compiled runtime are ignored in Git and reproducible
using `prepare-packet.mjs` and a build of the pinned runtime.

The task and oracle author is a separate agent which may read language
documentation but not runtime/game/example source or any candidate. The root
reviews the contracts and oracle before freezing them. Candidate authors start
with `fork_turns: none`, so they receive none of this conversation or previous
work. All inherit the same session model/settings; they are separate contexts,
not independent model families. Exact model version, token use and compute
budget are not exposed by the collaboration API and will not be invented.

Authors receive only their run prompt, assigned public task and the common
packet. They are instructed not to read private verification, other candidates,
runtime implementations, existing game/example programs, or the web, and not
to communicate with each other. The shared filesystem is not a security
sandbox. Any observed or disclosed isolation breach is reported and its run
excluded from a docs-only success count; artifacts are still retained.

## Fixed sample and budget

Six fresh author runs: A1 and A2 on cold-storage dispatch, B1 and B2 on access
review, C1 and C2 on repair diagnosis. These are synthetic policies specified
for the test, not operational recommendations for real physical systems.
Run A1/B1/C1 in one wave and A2/B2/C2 in another. There is no early stopping
because results look good or bad. Authors receive no verifier feedback until
all six have finished.

Each author may use up to **three distinct submitted source versions** and
**twelve runtime validation/replay checks**. The wrapper snapshots source
before execution. Draft edits before the first submission are not observable;
"first submission" never means first internal idea or first keystroke. Authors
may design their own input histories and repair from their own runtime output.
One file, `final.cav`, is frozen on completion. No supervisor edits or hints
to candidates are allowed. A runtime/tooling obstruction is recorded separately;
it is not silently counted as an author error or repaired out of the record.

There is no hard wall-clock cutoff or token cap. Record dispatch, first
submission, checks and completion timestamps as descriptive data. Parallel run
times are not a fair timing benchmark. No claim of faster or cheaper authoring
will be made from these measurements.

## Outcomes fixed before execution

Primary outcomes: exact-contract pass/fail of each first submitted source and
each final source, including parse/load failures. A full pass requires every
held-out scenario and invariant; no best-of-three cherry-picking. Report all
six denominators and results separately by task. Also report unique submitted
versions, runtime check count, validation errors and failure categories.

The private oracle uses ordinary domain state and pure transitions, not a
second Caveat interpreter. It supplies expected admission, hud values, frozen
decision history/grounds, and relevant observed graph relationships. Its actual
projection only reads runtime output; it cannot implement candidate gameplay.
All gameplay must live in the submitted Caveat source.

Verification executes every registered held-out sequence and **100 seeded
random sequences of 40 steps per task**, with fixed seeds A=`0xa17001`,
B=`0xb17001`, C=`0xc17001`. Steps may dispatch an event or resume. Every event compares acceptance/rejection and the
contract projection against the oracle. Rejects must preserve complete saved
state and full views, not merely displayed fields. After each held-out scenario
and at explicit or seeded restore points, save/restore must preserve the full
runtime view and subsequent behavior against an uninterrupted shadow. Grounded
explanations and decisions must cite real runtime evidence; fake hud strings
cannot substitute for observed evidence, grounds or a journal.

The root checks the oracle for internal consistency, explicit branch/boundary
cases, and rejection atomicity before authors start. Harness tests deliberately
alter a correct expected view/acceptance to demonstrate that mismatches fail.
If a verifier bug is discovered later, preserve the original run, explain the
correction and re-evaluate all candidates equally without author feedback.
Never change policy requirements after seeing candidates to make them pass.

## Interpretation

Six full passes would support documentation sufficiency across these three
bounded tasks for this model/setup. Fewer passes identify concrete authoring or
documentation gaps. Even six passes do not establish high reliability across
unseen tasks, other models, real users, or production environments. Failures,
missing measurements and deviations stay in the report. Any later documentation
or language repairs are a separate stage with the original packet preserved.
