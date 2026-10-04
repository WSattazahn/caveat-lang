# Glowcap round 7: fresh authors, blind change requests

Status: **registered.** The owner (OumuamuaWalt) chose Start on the round's
project card at 03:13:55 UTC and Approve on the card "Approve the Glowcap
round 7 protocol draft for rc.11?" at 03:17:31 UTC on 2026-10-04, as recorded
in the project timeline, before any change request was written. It was
committed with the files named under *Registration*, and `registration.json`
records their hashes. Changes after that commit are listed under *Amendments*
with their reason, as in the earlier rounds (`experiments/glowcap/PROTOCOL.md`
L265–286). The text above *Registration notes* is the approved draft,
unchanged except for this status paragraph and the title.

Each rule below says where it comes from:

- **[overview]** the owner's rc.11 plan, section "After rc.11: round 7 against
  TypeScript", items 1–5 and the paragraphs after them;
- **[glowcap]** `experiments/glowcap/PROTOCOL.md` or `RESULTS.md` at `main`
  4500dec, carried over unchanged;
- **[v2]** the fresh-agent study method in
  `experiments/agent-authoring/v1` and `v2` (`PROTOCOL.md`, `README.md`);
- **[default]** the overview and the repository are silent; this is the
  drafter's proposal, and the owner can change it before approval.

## Question

On the Glowcap beat, when the authors are fresh agents working from frozen
documentation and the change requests are written by someone who has seen
neither implementation, does Caveat produce fewer wrong decisions and fewer
wrong explanations under change, at lower change cost, under a fixed cost
ceiling? [overview]

Microseconds and kilobytes are not the claim; they enter only as the ceiling.
[overview] The round also tests learnability: if fresh agents break more on the
Caveat side than on TypeScript, the record says so, and the indicated fix is
syntax and tooling rather than runtime. [overview]

## What is frozen

- **Runtime.** `caveat-lang@0.1.0-rc.11` from npm, tarball SHA256
  `13fd6e298731c46e024f10788e1f68834a1c02625fd74bb4715730b4a4887a8e`,
  built from tag `v0.1.0-rc.11` at `8e7805a`. The published tarball is
  installed and its hash checked before use; nothing in `runtime/` or `kit/`
  changes for this round until it is scored. [overview: "the language frozen
  at the round's start" and "it must run against a tagged runtime"; rc.11
  is the owner's choice, below]
- **Caveat documentation.** The reference set packed into that tarball
  (`docs/reference/…`, listed in `kit/pack-docs.json` at `8e7805a`) plus the
  tarball's `README.md` and kit docs. As in v2, the one transformation removes
  `docs/AI_AUTHORING.md` from its line `## Evidence from fresh authors` to the
  end, so authors do not see earlier study results; the original and
  transformed hashes are recorded. [v2; packet contents are a default]
- **TypeScript side.** Node 22.x running `.ts` through its built-in type
  stripping, as the existing harness loads `ts/glowcap.ts`; `typescript` pinned
  to one exact version for `tsc --noEmit --strict`, which TypeScript authors may
  run. The exact Node and `typescript` versions are recorded at registration.
  [glowcap for the loader; the `tsc` allowance is a default, chosen so that each
  side has its static checker: `caveat check` on one side, `tsc` on the other]
- **The beat.** A consolidated behavioural specification: `PROTOCOL.md`
  sections "The beat" through CR12 (L13–192), copied verbatim, with the round
  narratives, predictions, measurements and results removed, and with the one
  change under *Bounded capacity* below. [default]
- **Scenarios, harness and comparator.** `scenarios.mjs` (base through CR12),
  `harness.mjs` and `compare-views.mjs` as at `main` 4500dec, extended only by
  the files registered with the new change requests. The differential fuzz
  uses `--ordered-decisions`. [glowcap]

### Why rc.11 and not rc.12 tooling

The overview also says the round starts "only once the tooling that lost
rounds 3 and 6 exists": `caveat types`, the delta view and skipping unchanged
tick-rule guards, all planned for rc.12. rc.11 has none of them (its CLI at
`8e7805a` has no `types` command). The owner chose on 2026-10-04 at 03:13 UTC
to start the round now, on rc.11. So Caveat authors write the adapter glue by
hand, which is where two of round 6's three failed runs came from (RESULTS.md
L399–400), and the round measures rc.11 as the baseline for first-run
correctness, not the tooling the overview expects to win it. A later round on
rc.12 tooling would change only the runtime pin and the packet.

## Bounded capacity

"Unlimited" is out of scope for this round. Bounded, declared capacity is a
contract; a request that needs unbounded memory is one TypeScript meets by
having no cost model. [overview]

So, before the round: CR10's words "and so on, with no limit" are read, for
round 7 only, as "up to a capacity the implementation declares, of at least
64 lives per mushroom; exceeding it rejects the event and leaves the view
unchanged." Both sides are held to the same reading. Earlier rounds' records
are not rescored. [the reading follows the overview; the 64-life floor is a
default, well above the six lives the scenarios reach and below the 1,024 the
round 6 replay declared]

Change requests written for this round must state a bound for anything that
grows. [default]

## Who does what

Every agent below is a fresh context with no fork of any earlier
conversation, given only its registered prompt and the files named for it.
All share one model family; they are separate contexts, not independent
models. Exact model revision, tokens and compute are recorded where the tools
expose them and are not invented. [v2]

| Role | Count | Receives | Must not read |
| --- | --- | --- | --- |
| Change-request writer | 1 | the consolidated beat, the scenario step format, the bounded-capacity rule | either language's documentation, any implementation, `RESULTS.md`, `PROTOCOL.md` history, this protocol's measures and decision rule, the web |
| Scenario reviewer | 1 | the consolidated beat, the new requests and their scenarios | the same as the writer |
| Caveat authors | 3 (C1–C3) | the beat, the scenarios, the rc.11 packet, the logged runner | any implementation, `RESULTS.md`, other authors, runtime source, examples and games, the web |
| TypeScript authors | 3 (T1–T3) | the beat, the scenarios, the logged runner, `tsc` | the same, and the Caveat packet |
| Mutator | 1 | the registered operators, the six frozen programs | scores, `RESULTS.md` |

Sources: authors who have seen neither implementation write the requests
[overview item 1]; both sides are written by fresh agents from frozen
documentation, using the v1/v2 study method, which removes the one-author
threat (RESULTS.md L128) [overview item 2]. Three authors per side, the
scenario reviewer and the mutator are defaults: one author per side would
leave one data point per language, which the earlier threats list already
names (PROTOCOL.md L262–263).

The writer is not told which languages are compared. Authors work in a
directory outside the repository checkout containing only their packet and
run directory. The filesystem is not a security sandbox: isolation is
instructed, disclosed breaches stay in the report, and a run with a breach is
excluded from the primary counts but kept. [v2]

## The new change requests

The writer produces **four requests, CR13–CR16**, each with scenarios
continuing the numbering after S38 in the existing step format, applied
cumulatively after CR12. [glowcap: four per blind round; count is a default]

The reviewer checks the scenarios against the requests for internal
consistency and reachable expectations, and reports defects to the writer once;
after that exchange the requests and scenarios are registered. Neither sees an
implementation. [default]

The round-7 fuzz mode (event generators for whatever CR13–CR16 add) is written
and registered with the requests, before any implementation exists. [glowcap
round 6 committed its fuzz mode with the requests, PROTOCOL.md L208–212]

**Predictions.** After the requests are registered and before any author
starts, the owner (or Claude for the owner, labelled as such) records a
prediction per request, as round 6 did. A prediction that fails is reported as
failed. [glowcap PROTOCOL.md L193–206]

## How authors work

Each author implements the whole beat from scratch in phase order: base,
CR1…CR12 (the **inherited phases**), then CR13…CR16 (the **blind phases**).
Each phase ends in one commit to the author's own run directory. [one commit
per phase is glowcap; building from scratch is a default, because the
overview's item 2 has both sides implemented by fresh agents]

- A phase's request text and scenarios are released to the author only after
  the previous phase is committed, so no earlier code is shaped by a later
  request. [default; earlier rounds released all four at once to one author
  who already knew the beat]
- Every execution of the candidate goes through the logged runner, which
  snapshots the source and appends to the round's `runs.jsonl`: phase, author,
  content hash, pass/fail per scenario. The first run of each phase counts for
  correctness. [glowcap harness; the snapshot-before-run rule is v2]
- No cap on runs per phase; every run is in the log. [default; glowcap had no
  cap, v2 capped at twelve checks, and a cap here would turn correctness into
  a budget measure]
- Each author keeps `notes.md`: references read, self-checks, uncertainties,
  access deviations. No hints, feedback or edits from anyone while authoring.
  A runtime or tooling obstruction is recorded as such, never as an author
  error. [v2]
- Each implementation exports `createPolicy(saved?)` returning
  `{ dispatch(event), view(), save() }`. Glue a side needs to produce the view
  shape counts as its code. [glowcap]

Authors run in two waves (C1, T1, C2, T2, then C3, T3), with no stopping
because results look good or bad. [v2 waves; the split is a default]

## Measurements

The original five, unchanged in definition [glowcap PROTOCOL.md L226–242],
plus two [overview item 3]. Unless stated, each is reported per author and as
the median of the three authors per side.

1. **Correctness.** Phases green on the first complete run, and runs to reach
   all-green, reported separately for the inherited phases (learnability) and
   the blind phases (change). The blind phases decide the rule.
2. **Size.** Non-blank, non-comment lines and bytes of policy plus glue,
   measured by `harness.mjs --measure`'s counting rule, at the end of CR16.
3. **Change cost.** Lines added + removed per blind request (`git diff
   --numstat` of the author's files), and regressions: previously passing
   scenarios that failed during the request.
4. **Explanation.** Failures on `because`, `supportedBy`, `contradictedBy`,
   `basis`, `reopenedBy`, `caveats`, `why` and history assertions, counted
   separately, and whether explanations came from the language's own model or
   from hand-written bookkeeping.
5. **Cost to run and ship.** Median and p95 µs per dispatch + view over the
   existing 10k-event replay, and gzipped bytes a host ships.
6. **Mutation score.** Specified below.
7. **Explanation drift.** After each blind request, the number of
   explanation-producing units the diff had to touch to keep the explanations
   correct. A unit is a Caveat binding, rule, state, decision or adapter
   function, or a TypeScript function or object literal, whose output feeds any
   field listed in measure 4. Lower is better. [definition is a default; the
   overview names the measure only]

### Mutation score

Applied to each of the six programs as frozen at the end of CR16. [overview
item 3]

**Operators** [overview, the five listed]:

| Operator | Caveat site | TypeScript site |
| --- | --- | --- |
| M1 drop a caveat from a citation | remove one `qualify`, `qualifies`, `carries` or caveat-bearing argument | remove one caveat from a caveat map entry or union |
| M2 forget to freeze at commit | make the committed basis or its caveats follow live state | read basis caveats live instead of the copy taken at commit |
| M3 skip a reopen | remove one reopening trigger or reopen rule | remove one reopen transition or `reopenedBy` push |
| M4 reorder a basis | change the order evidence enters a commitment | reverse or sort one basis array |
| M5 cite evidence never read | add to one `because` evidence the value never reads | add one evidence id to one explanation list |

The site mapping above is a default. The mutator first lists every
applicable site per program per operator, and that list is committed before
any mutant runs. Each mutant changes one site. [default]

**Catching.** A mutant is caught if, without further edits:

- the **static** check rejects it (`caveat check` or `tsc --noEmit --strict`);
- the **runtime** rejects it at load or refuses an event the unmutated
  program accepts; or
- the **suite** fails: any registered scenario through S-last fails.

The score is caught ÷ total, per operator and overall, with the three columns
reported separately so a runtime catch is visible as such. Survivors are then
run through the round-7 differential fuzz; a survivor the fuzz cannot tell from
its original is labelled possibly equivalent and reported, not removed.
[catch definition "by the runtime or the suite" is the overview's; the static
column and the equivalence label are defaults]

## Decision rule

**Margins.** A difference smaller than 10 percent is a tie; on a tie the
incumbent wins. [the 10 percent figure is a default, chosen to match how the
record already reads: 55 against 57 was a tie (RESULTS.md L65) and 92 against
103, 11 percent, was "Caveat, −11%" (L358)]

**Rule A, adoption, unchanged** [glowcap PROTOCOL.md L244–250]. Caveat is
adopted for this kind of gameplay logic only if it is better on at least two of
measures 1–4 and no more than 2× worse on any, and its median dispatch stays
under 1 ms. Otherwise TypeScript is the default.

**Rule B, the stated claim** [overview's claim; the operational form is a
default]. The claim "fewer wrong decisions and fewer wrong explanations under
change, at lower change cost, under a fixed cost ceiling" holds only if all of:

- the overall mutation score (static + runtime + suite) is higher for Caveat;
- explanation drift summed over CR13–CR16 is lower for Caveat;
- blind-phase change cost is lower for Caveat;
- measure 1 (blind phases) and measure 4 are no worse for Caveat; and
- median dispatch is under 1 ms.

Both verdicts are reported. Neither can be reached by dropping a measure,
author or request after the fact.

## Reporting

- Results go in `experiments/glowcap/RESULTS.md` under a round 7 section, with
  the summary line at the top as for every earlier round, including losses and
  failed predictions. [glowcap; overview: "publish it with its losses"]
- The record states what the round can and cannot show; no claim that Caveat
  is better than TypeScript goes into README, AGENTS, the site or other docs
  beyond this record. [project instruction]
- Retained: prompts, packet manifest, every source snapshot, `runs.jsonl`,
  notes, the mutant list and outcomes, fuzz seeds and outputs, the runtime
  hashes and host. [v2, glowcap evidence files]
- Learnability: if the Caveat side's inherited-phase first-run correctness is
  below TypeScript's, the report names which failures were glue, which were
  language, and which were documentation. [overview "The risk, stated"]

## Conditions and threats to validity

- All agents share one model family, and the drafter of this protocol is of
  the same family. The authors are not independent models. [v2]
- The beat and its first twelve requests were designed by the project's
  author while the language was shaped around them; CR1–CR12 favour neither
  side by construction only to the extent earlier rounds showed. The blind
  phases carry the weight for that reason.
- TypeScript is the language these agents know far better. Fresh Caveat
  authors learn it from about the packet alone; that is the point of the
  round, not a correction to make.
- On rc.11, Caveat glue is hand-written (see *Why rc.11 and not rc.12 tooling*).
- One beat is one data point. The result decides this adoption question, not
  whether Caveat's ideas are sound. [glowcap PROTOCOL.md L262–263]

## Registration notes

How the files registered with this protocol carry it out. None changes a rule
above; each fixes a detail the rules leave to the implementation.

- **Versions.** Node 22.22.0; `typescript` 7.0.2 with `strict`,
  `erasableSyntaxOnly` and `noEmit` (`prepare.mjs`). The rc.11 tarball's
  SHA256 is checked before it is installed; `packet/manifest.json` hashes
  every Caveat document an author may read and records the guide
  transformation.
- **Directories.** Every agent works in its own directory under
  `/home/claude/glowcap-r7`, outside the repository. Logs, snapshots, phase
  commits and unreleased phases are in `private/`. The runner
  (`workspace/run.mjs`) releases a phase's text and scenarios only when the
  previous phase is committed, and logs every `test`, `check`, `try` and
  `caveat` use; only `test` is measured.
- **What a Caveat author may read.** The package's `README.md`, everything
  under its `docs/`, and its type declarations `lib/*.d.mts`, which are the
  session API's reference. Not its runtime, `lib/*.mjs`, `examples/` or
  `templates/`.
- **Where the logic lives.** On the Caveat side the game rules, text and
  explanations are in `.cav` source and the adapter only translates events and
  views, as every earlier Caveat adapter did ("rules, text and explanations
  stay in the source", `caveat5/adapter.mjs` L1–3). The adapter counts as
  code either way. The TypeScript side uses no npm package.
- **The writer-reviewer exchange.** The reviewer's `REVIEW.md` is given to the
  writer once; the writer's revision is what is registered.
- **Disclosed before the round: Glowcap in the Caveat documentation.** Ten of
  the specifications in the frozen packet motivate their feature with
  fragments of earlier Glowcap Caveat programs (for example
  `caveat-procedure-symbols-0.1.md`'s `proc learn` and
  `caveat-renewal-0.1.md`'s regrowth and fade), and some link to
  `RESULTS.md`, which authors may not open. The protocol freezes the packet
  with one transformation only, so these stay. They favour the Caveat side on
  CR1–CR12; the TypeScript side has no equivalent. CR13–CR16 are new to both.
  The report weighs inherited-phase correctness with this in mind.
- **Infrastructure check.** Before registration, two root-only runs in a
  separate directory took the existing `ts/` program and `caveat5/` (with an
  adapter rewritten for the rc.11 session API) through the runner: both
  passed CR12's 38 scenarios. They are not results.

## Registration

Before any agent of the round starts, commit: this protocol; the consolidated
beat; the writer's, reviewer's, authors' and mutator's prompts; the packet
manifest with tarball and document hashes; the runner; and a
`registration.json` of their hashes. Requests CR13–CR16, their scenarios and
the round-7 fuzz mode are committed and hashed as a second registration before
any author starts. Dispatch times and agent identities go in `launches.json`.
[v2 `register.mjs` / `launches.json` method]

## Amendments

1. **CR10's bound at the limit (before any author started).** The scenario
   reviewer found that "an event that would exceed it is rejected" does not
   say which event: the regrowth is caused by a tick, and rejecting that tick
   would reject every later tick and freeze the timers. `phases/cr10.md` now
   reads: a mushroom that has used its last life stays consumed and does not
   regrow; no tick is rejected for this, and an event on that mushroom is
   rejected as on any consumed mushroom. It applies to both sides alike, no
   scenario reaches the limit, and the 64-life floor is unchanged. The
   stage-1 hash of `phases/cr10.md` in `registration.json` is the original
   wording; `registration-2.json` records the amended file.

2. **Isolation incident in wave 1, and the scratchpad (during wave 1).** The
   agents' environment offers a session scratchpad directory, which every
   agent of the round shares. At CR13, C1 kept a backup there named
   `adapter.bak`, which C2 also used; C1 restored it, saw C2's adapter source
   and ran one `test` with it (snapshot `8cdfd394b1d2`, 10/41), then rewrote
   its own adapter. C1 disclosed this in its notes and report. Under *Who does
   what*, C1 is excluded from the primary counts and kept in the record. C2's
   snapshots contain no C1 adapter; C2 was sent an infrastructure notice
   (keep every file in its own directory, read nothing from the scratchpad,
   record anything it restored) and stays in the primary counts unless its
   notes show otherwise. The scratchpad also held the root's infrastructure
   files until about 04:15 UTC, including the smoke-run copies of `ts/` and
   `caveat5/`. No author reports reading them; access times cannot show it
   either way. They were then moved out, and the files C1 and C2 wrote there
   were preserved. Wave 2 authors receive one added sentence at the end of
   their prompt: "Keep every file you write, scratch files and backups
   included, inside that directory; the session's scratchpad directory is
   shared with other agents and is outside it."
3. **A replacement Caveat author, C4 (during wave 1, pending the owner).**
   With C1 excluded, the Caveat side would have two primary authors to
   TypeScript's three. C4 runs in wave 2 under the same prompt as C3. If the
   owner declines the replacement, C4 is reported as supplementary and the
   primary Caveat counts use C2 and C3 only.
