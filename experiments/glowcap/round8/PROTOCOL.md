# Glowcap round 8: the same beat, played to the registered rules

Status: **draft, not registered.** It becomes the round's protocol when the
owner approves it, as round 7's was approved on a project card before any
change request was written (`round7/PROTOCOL.md` L3–8). Until then nothing
below has started except the learnability probe's preparation.

This is step R1 of the "Round 8" table in
[the path to 1.0](../../../docs/releases/path-to-1.0.md): pre-register;
R2 runs it on the first published candidate carrying View 0.2 and commits the
record (B6), whatever its verdict. Its direction is
[beat-typescript.md](../../../docs/design/beat-typescript.md), "Round 8" and
"What not to do".

## The rule of this document

**Round 7's protocol applies to round 8 word for word, except where the
table below changes it.** Every measure, margin, operator, catching rule,
decision rule, reporting rule and threat in `round7/PROTOCOL.md` L24–331
carries over, and so do the rules set by its amendments 1–3 (L343–396).
Nothing here re-scores round 7, drops a measure, moves a margin or weakens
the suite (beat-typescript.md, "What not to do").

| # | Round 7 rule | Round 8 | Why |
| --- | --- | --- | --- |
| 1 | Runtime frozen at `caveat-lang@0.1.0-rc.11` (L39–45) | The first published candidate carrying View 0.2, expected `0.1.0-rc.17`. Its tarball SHA256 is taken from that version's verified publication record (`docs/releases/v0.1.0-rc.17-npm-publication.json`) and checked before install | Owner card, 2026-10-10 16:13:13 UTC (path-to-1.0.md, "Decisions already made") |
| 2 | Packet: the rc.11 package's docs, with the guide cut at "Evidence from fresh authors" (L46–51) | The same rule applied to the rc.17 package. No round-specific text is added. The packet's statement of save growth is whatever the package ships: rc.16's `docs/HISTORY.md` L100 says windows and collection give no universal save bound | beat-typescript.md asks that the packet state the save's size behaviour; the package docs already do, and adding text for the round would favour one side |
| 3 | Inherited phases base, CR1–CR12; blind CR13–CR16 (L126–131, L148–151) | Inherited phases base, CR1–CR16, using round 7's registered texts and scenarios (CR10 as amended, amendment 1). Blind phases **CR17–CR20**, written for this round by a fresh writer and reviewed as in round 7 | CR13–CR16 have shaped the language since round 7 (windows, departure, ordered grounds); they are no longer blind to the Caveat side. **Pending the owner's choice** (below) |
| 4 | "Why rc.11 and not rc.12 tooling" (L67–77) | Removed. Authors may use anything the rc.17 package ships and documents: `caveat types`, check-first `caveat test` with C006, the starter. Glue they write still counts as code | The tooling round 7 lacked is in the package (rc.15 check-first and C006, `spec/caveat-check-0.1.md` L323–414; starter, `kit/docs/STARTER.md`) |
| 5 | The scenario comparator (`harness.mjs` `compare`, L60–72) sorts a list of journal entries as if it were already sorted, so each entry's `caveats` is compared in order (RESULTS.md L846–857) | Each journal entry's `caveats` is compared as a set, as CR16 defines it (a union, order unstated). Every other comparison is as in round 7. Registered as a round-8 copy of the harness; the shared `harness.mjs` stays as round 7 registered it | Round 7's record names this a registration defect that added four Caveat failing runs. Both sides get the same comparator |
| 6 | Scratchpad sentence added for wave 2 only (amendment 2) | In every author's prompt from the start. Each agent's directory is outside the repository and the session scratchpad | Amendment 2's incident |
| 7 | Replacement author by owner decision (amendment 3) | Standing rule: a run with a disclosed isolation breach is excluded from the primary counts, kept in the record, and replaced by a fresh author under the same prompt. Results are reported with and without it | beat-typescript.md: "the C1-style exclusion rule kept" |
| 8 | Predictions per request (L141–144) | Per-request predictions as before, after CR17–CR20 are registered. In addition, round-level numeric predictions registered now, in [PREDICTIONS.md](PREDICTIONS.md) | beat-typescript.md, "Predictions pre-registered" |
| 9 | (none) | A learnability probe runs before the round (below) | beat-typescript.md |
| 10 | Rule A's reading of a measure group (round 7 verdict, RESULTS.md "Verdicts") | Stated in advance, unchanged: a measure counts as better only when no number in its group is worse. Change cost is lines **and** regressions | Round 7 states its verdict turned on this reading; beat-typescript.md keeps it |
| 11 | Node 22.22.0, `typescript` 7.0.2 (registration notes) | Node and `typescript` versions recorded at registration; `typescript` pinned to one exact version | Same rule, current versions |

Exact model revision, tokens and compute are recorded where the tools expose
them, as in round 7. The authors, writer, reviewer, mutator and drift counter
are one model family, as in round 7 (beat-typescript.md: "same model
family"). Only the probe uses another.

## The learnability probe

**Purpose.** Before any author starts, find where Caveat's surface differs
from what a capable programmer expects, so that each mismatch can become a
guide or packet item. It measures the language's guessability, not a
program's correctness.

**Who.** One model from a family other than the authors', never shown
Caveat's syntax: one fresh chat, no tools, no web.

**Input.** `probe/prompt.txt`, built by `probe/make-prompt.mjs` from three
registered sources and nothing else: `probe/instructions.md`; the package
README's headline paragraph and `docs/CAVEAT_ESSENCE.md`, with every code
block and inline code span removed and every link reduced to its text; and
the consolidated beat `round7/BEAT.md` (base to CR12). Its SHA256 is
recorded when it is sent; `node make-prompt.mjs --check` shows it matches
the sources.

**Reference.** `experiments/glowcap/caveat5/glowcap.cav`, the repository's
CR12 program, which `runtime/tests/check.rs` checks on every runtime run.

**Diff.** The probe's reply is committed unchanged as `probe/reply.md` with
the model's name and the time. Then, and only then:

1. its program is run through `caveat check` on the pinned package, and the
   diagnostics are recorded as they come;
2. each construct the probe used is matched to the reference's construct for
   the same purpose, and classified: **same** (the guess is the language),
   **renamed** (same idea, other spelling or shape), **missing** (the probe
   expected a facility Caveat lacks), **unanticipated** (the reference uses a
   facility the probe did not expect), or **adapter** (the probe put in the
   host what the reference keeps in source, or the reverse);
3. every renamed, missing and unanticipated item becomes one line in
   `probe/DIFF.md` naming the guide or packet section it calls for, or
   saying none is needed and why.

The counts per class are the probe's numbers. The diff goes in the record
whatever it shows.

**Timing.** "First" means before the round's authors. The probe is cheap and
needs nothing from rc.17, so it can run now; any guide item it yields can
then land before rc.17 freezes its docs, and round 8 measures the docs that
include it. If it runs after rc.17 is published, its items wait for a later
candidate and the round runs on rc.17's docs as they are. Either way the
record says which.

## Size of the run

Ten agents, as round 7 had before its replacement: one change-request
writer, one scenario reviewer, three Caveat and three TypeScript authors, one
mutator, one drift counter; plus a replacement author only under rule 7.
Each author implements 21 phases (base, CR1–CR20), four more than round 7's
17. The probe is one chat with the other model and one diff. The run starts
only after rc.17 is published, its size and cost estimate have gone to the
owner, and the owner has said to start.

## Choices for the owner before registration

1. **Blind requests (row 3).** New requests CR17–CR20 after inherited
   CR1–CR16 (recommended), or round 7's CR13–CR16 again as the blind phases,
   which keeps the round directly comparable and costs four fewer phases per
   author but tests requests the language has been shaped around since.
2. **The probe model.** Any model from another family, run by the owner in
   their own chat with `probe/prompt.txt` pasted unchanged, and its reply
   pasted back. The sessions running this round cannot reach a model outside
   their own family.

## Registration

As round 7 (`round7/PROTOCOL.md` L333–341), in two stages: first this
protocol, the prompts, the packet manifest with the rc.17 tarball and
document hashes, the runner and comparator, and `registration.json`; then
CR17–CR20, their scenarios, the round-8 fuzz mode and the per-request
predictions, as `registration-2.json`, before any author starts. The probe's
prompt hash is recorded when it is sent, before its reply is read.

## Amendments

None yet.
