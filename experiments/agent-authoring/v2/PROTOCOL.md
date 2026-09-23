# Fresh-agent clock-task follow-up, v2

Question: can four fresh author contexts implement the two previously studied
clock tasks with the updated authoring guide and a runtime that exposes its
source clock through `elapsed()`?

This is a repeated feasibility study on the same A/B tasks and test corpus.
It evaluates the documentation and runtime improvement together. It cannot
isolate their causal effects, establish a comparative reliability advantage,
or measure performance on new tasks. Every v1 artifact remains unchanged.

## Freeze before dispatch

Commit the runtime and documentation first. `study.json` records that full
commit ID; build it with the pinned toolchain and a clean tracked checkout.
`prepare-packet.mjs` requires matching compiled build metadata and rejects
runtime/build inputs that differ from the pinned revision. The generated
manifest freezes the actual JS/WASM hashes, build host and compiler provenance.
Generated packet files are ignored in Git and regenerated against those hashes.
A byte-identical rebuild may require the registered build-host platform;
the repository's current build does not promise Windows/Linux byte equality.

The reference packet comes from `git show` at that commit. The sole content
transformation removes the guide's final section beginning exactly with
`## Evidence from fresh authors`, preserving every preceding byte. This avoids
exposing previous study results to authors. `packet-transform.mjs` defines the
operation; the manifest hashes both the original guide and transformed guide.
Every other reference, including the new elapsed specification, is copied
byte-for-byte. Links outside the packet are not author permissions.

Before any fresh author starts, review and test this protocol, tasks, oracle,
corpus, public tool, scorer, metadata-corruption checks and packet. Run
`register.mjs`, then commit all files named by `registration.json`. Record
that registration commit in `launches.json` together with dispatch timestamps,
agent identities and `fork_turns: none`. No task, oracle, case, scoring rule or
reference change may be hidden after registration. If a tooling defect is
found, preserve its evidence, document the correction, and apply it equally.

## Four fresh authors and isolation

Run A1/B1 in wave one, then A2/B2 in wave two. All four use new contexts with
`fork_turns: none`, none is reused from v1 or the implementation/verification
work, and none receives the parent conversation. Authors inherit the same
session model/settings; exact model revision, tokens and compute budget are
not exposed by the collaboration API and will not be invented. These are
independent contexts, not independent model families.

Each author receives only its exact registered prompt, assigned task and
public packet. Permitted reads are the assigned task, public README/runner,
packet references and own run directory. Runtime implementation, private
verification, v1 artifacts/results, other authors' files, existing examples or
games, web access, agent communication and spawning are prohibited. The
filesystem is shared; this is instructed isolation, not a security sandbox.
Disclosed or observed breaches remain in the report and are excluded from any
docs-only success count. Unobserved reads cannot be ruled out.

The sample is fixed at four, two per task, with no outcome-based early stopping.
Each author may submit at most **three distinct source versions** and use at
most **twelve validation/replay checks** through the public runner. Source is
snapshotted before execution. Draft edits before first submission are not
observed, so first submission does not mean first internal idea. Authors may
design their own histories and repair from their own runtime outputs. No
supervisor candidate edits, task hints or private feedback are allowed.
All four must freeze `final.cav` before private candidate scoring begins.

There is no hard token or wall-clock cutoff. Dispatch, submission, checks and
completion are descriptive timestamps, not a speed/cost benchmark. Authors
retain references read, self-checks, uncertainties and access deviations in
`notes.md`. A runtime or tooling obstruction is reported separately, never
silently repaired away or relabelled as an author error. Root-only `SMOKE`
artifacts, if created, are infrastructure checks excluded from the denominator.

## Unchanged policy and corpus; stronger primary verification

The A/B public contracts and independent domain-oracle module are byte-identical
to v1. The module retains its unused C definitions, but v2 scores only A/B.
There are **114 cases for A** (14 targeted scenarios plus 100 seeded sequences)
and **116 for B** (16 plus 100). Each seeded sequence has 40 steps, with the
same seeds A=`0xa17001`, B=`0xb17001` and the exact same event histories as v1.
`register.mjs` asserts their fixed case digests and records counts and coverage
in `corpus.json`. No new clock-horizon cases enter the primary corpus.

Primary outcomes are exact-corpus pass/fail for every first submitted and final
source, including parse/load failures. A pass requires every registered case
and invariant. Report all four denominators and both tasks separately, with
versions, checks, validation errors and failure categories. Do not select a
best intermediate version or omit a failed/obstructed run from the record.

At initialization, after each event/resume and after final restore, compare
the independent oracle with all required HUD values, observed relations,
qualifications, frozen commitment grounds and ordered decision journal. Require
real runtime provenance; invented text does not replace grounds or a journal.
Unlike v1's original primary scorer, v2 also checks actual `snapshot().elapsed`
and `snapshot().sequence` at each of those points. The compact view has no
elapsed field. Deliberate independent elapsed/sequence corruptions must fail
before registration.

Every event compares exact admission. Rejection must preserve full saved state
and full view. Each registered resume and final resume must preserve full
snapshots/views, and subsequent behavior is compared with an uninterrupted
shadow session. The oracle remains an ordinary domain transition model, not
a second interpreter. It does not implement candidate policy for the host.

The public runner preserves explicit null payloads, correcting v1's public
wrapper defect before this registration. Omitted payloads default to `{}`;
explicit arrays/scalars/null are forwarded as such. The primary scorer forwards
each registered payload unchanged. Both still parse/stringify JSON: raw
duplicate keys and nonfinite numeric spellings are not claimed as covered by
this study. Runtime feature-regression tests are separate evidence.

## Interpretation and reporting

Compare v2 descriptively with v1's two A and two B runs, keeping the original
outcomes intact and disclosing the stronger primary metadata check and public
transport correction. Repeated tasks, the small cohort, shared model family,
updated documentation/runtime together and prior investigator knowledge prevent
causal attribution or general claims about reliability on unseen work.

The corpus remains short in elapsed time. A corpus pass is not proof of every
valid long history. After all authors freeze, report use of `elapsed()` and
any remaining capped/manual clock representation as a labelled source-review
observation; disclose known contract violations separately from corpus scores.
Do not turn source review into new hidden primary cases or retroactive success
criteria. Any additional exploratory verification is labelled separately.

Retain all prompts, submissions, checks, author notes, dispatch metadata,
scored cases, corrections and missing measurements. Reproduction verifies
frozen inputs and source hashes before executing candidates. Candidate failures
are experimental results, so completed scoring may exit successfully with
failed candidates; infrastructure/integrity failures are errors.
