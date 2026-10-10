# Glowcap round 8: the same beat, played to the registered rules

Status: **registered, stage 1.** The owner (OumuamuaWalt) chose Approve on
the project card "Approve round 8's protocol draft so it can be registered?"
at 21:56 UTC on 2026-10-10, as recorded in the project timeline, before any
change request for the round was written and before the probe was sent. It
was committed with the files named under *Registration*, stage 1, and
`registration.json` records their hashes. The text above *Registration* is
the approved draft, unchanged except for this status paragraph and the
heading of *Choices still open*. Later changes are listed under *Amendments*
with their reason.

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

## Choices still open at registration

1. **Blind requests (row 3).** New requests CR17–CR20 after inherited
   CR1–CR16 (recommended), or round 7's CR13–CR16 again as the blind phases,
   which keeps the round directly comparable and costs four fewer phases per
   author but tests requests the language has been shaped around since.
2. **The probe model.** Any model from another family, run by the owner in
   their own chat with `probe/prompt.txt` pasted unchanged, and its reply
   pasted back. The sessions running this round cannot reach a model outside
   their own family.

## Registration

As round 7 (`round7/PROTOCOL.md` L333–341), with an earlier first stage,
because the runtime pin and the blind requests do not exist yet. No stage
overwrites another (`register.mjs`).

1. **Stage 1, now:** this protocol, `PREDICTIONS.md`, the probe's sources and
   built prompt, `register.mjs`, and the inherited round 7 files the round
   reuses (`BEAT.md`, the phase texts base to CR16, `scenarios-r7.mjs`), as
   `registration.json`.
2. **Stage 2, once rc.17 is published:** the prompts, the packet manifest
   with the rc.17 tarball and document hashes, the runner and the round-8
   comparator, as `registration-2.json`.
3. **Stage 3, before any author starts:** CR17–CR20 (or the owner's other
   choice under *Choices still open*), their scenarios, the round-8 fuzz
   mode and the per-request predictions, as `registration-3.json`.

The probe's prompt hash is recorded when it is sent, before its reply is
read. The owner's answers to the open choices are recorded under
*Amendments* as they arrive.

## Amendments

1. **Blind requests (choice 1, after stage 1).** The owner chose "New CR17
   to CR20" on the project card "Choose the change requests for round 8's
   blind phases." at 21:56 UTC on 2026-10-10. Row 3 stands as drafted: the
   inherited phases are base and CR1–CR16, and the blind phases are CR17–CR20,
   written by a fresh writer and reviewed as in round 7. Each author
   implements 21 phases.
2. **The probe runs now (choice 2, after stage 1).** The owner chose "Now" on
   the project card "Choose when to run round 8's learnability probe." at
   21:56 UTC on 2026-10-10. The owner runs it in a chat with a model of
   another family. The prompt handed over is `probe/prompt.txt`, SHA256
   `55e771f86b268afa5eacb504fbefc0f94f4208f7913a27a23035ac9a129bc7a7`, as
   stage 1 registered it; the reply is committed with the model's name and
   time before it is read for the diff. Guide items it yields may land before
   rc.17 freezes its docs, and the record says which did.
3. **The writer's inputs, registered early (after stage 1).** New requests
   need a writer and a reviewer, and neither needs the runtime, so they can
   run before rc.17 if the owner says so; otherwise they start with the run,
   after the owner's go on its size. Either way their inputs are registered
   before they start as `registration-2.json`: the
   consolidated beat base to CR16 (`build-beat.mjs`: round 7's `BEAT.md`
   plus its registered CR13–CR16 texts, verbatim), both scenario modules, the
   prompts and directory READMEs (round 7's, with the request numbers, the
   scenario file name and the first id changed, and the scratchpad sentence
   of row 6 added to both prompts) and `prepare.mjs`. The stages under
   *Registration* shift by one: what depends on rc.17 is stage 3
   (`registration-3.json`) and the requests, scenarios, fuzz mode and
   per-request predictions are stage 4 (`registration-4.json`).
4. **The requests are registered before the runtime work (after stage 2).**
   The writer, reviewer and one revision ran on the owner's "Now"
   (2026-10-10, 21:59 UTC), and the reviewer found two wording problems in
   CR18 and no wrong expectation (`REVIEW.md`). So the requests' stage comes
   before the runtime's: stage 3 (`registration-3.json`) is `REQUESTS.md`,
   `REVIEW.md`, `scenarios-r8.mjs`, the phase texts CR17–CR20
   (`build-phases.mjs`, verbatim from `REQUESTS.md`) and the per-request
   predictions; stage 4 (`registration-4.json`) is what depends on rc.17 and
   the round-8 fuzz mode, before any author starts.

