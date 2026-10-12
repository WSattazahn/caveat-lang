# Reviewer study v1: what a reader can establish from the record

Status: **prepared, not registered or run.** No package identity, program,
history, note, key or outcome is asserted here. Running it needs the owner's
approval, as C6 does (`docs/releases/path-to-1.0.md` L258). It changes
nothing in `spec/`, `runtime/`, `kit/` or the skill, runs against a published
package, and is outside the compatibility promise (`docs/COMPATIBILITY.md`
L8–10). It is not Glowcap round 9 and re-scores no Glowcap round. The
probe programs named below were written while preparing this document, to
check that the task is expressible; they are not reference programs and are
not registered.

## Question

Given a program it did not write, one event history, and the author's
handoff note, can a fresh reviewer establish, correctly and with receipts,
what each decision rested on, what a withdrawal would touch, and which
statements in the note are false? The same questions are put to reviewers of
a Caveat program and of a TypeScript program that implement one task. The
study measures what each artifact affords a reader; it does not measure
authoring.

## Why this and not the material that exists

- The Glowcap beat requires `basis`, `because`, `reopenedBy` and `history` in
  the view (`experiments/glowcap/round8/BEAT.md` L47–56, L72–80, L139–141),
  so on that material "what does trust rest on?" is answered by running
  either program. Round 8 measures the cost of provenance when the
  specification demands it. This study's task demands none: the required
  view is status, head, revision count and a fixed reason text. It measures
  what the artifact affords when nobody asked.
- Round 9 (`docs/design/beat-typescript.md` L43–44) measures authoring under
  correction, with explanations scored by scenarios. No reader is separate
  from the author there, and it is deferred past 1.0 (`path-to-1.0.md` L316).
- Glowcap's M5 operator plants a citation of unread evidence in source
  (`experiments/glowcap/round7/PROTOCOL.md` L217). Here the planted error is
  in the author's prose about the record, which no runtime checks.
- Agent adoption v1 scored an author's explanation of its own program
  (`experiments/agent-adoption/v1/PROTOCOL.md` L107–111). No reader was
  separate from the author.
- `docs/DEVELOPMENT_INSIGHTS.md` L208 lists as not established "that its
  complete provenance graphs stay understandable", and L212–213 names
  explanation accuracy as a next measurement.
- One instance in the repository motivates the note check. The comment at
  `kit/examples/reviewer-decision/rely.cav` L38–39 says the screenshot "stays
  in lineage only". On the published `caveat-lang@0.1.0-rc.19`,
  `explain --json rely.cav rely.events.jsonl` lists `rely@1`'s lineage as
  `npm_view`, `publication_record`, `registry_api`: the same three as its
  grounds, and `socket_page` in neither. No rule reads it after it is
  revealed. The prose and the record disagree, and only a reader who checks
  finds out. (Observed 2026-10-12 on Node 22.22.0; a documentation
  correction, not a runtime defect.)

## Design

Two arms, one task, two phases.

**Phase one, authoring.** Two fresh agents, one per arm, each write the
release gate below from `TASK.md`, told only that a reviewer who has not seen
their work will later have to establish what each decision rested on. Each
submits once. A program passes when, on every registered case, it accepts and
refuses exactly the lines the model of the task does and its view after every
line equals the model's. A program that fails is kept in the record and is
replaced for phase two by the investigator's reference program for that arm;
no program is repaired. For the Caveat program, the record is checked too:
`explain --json` must show, for every decision in the three histories, the
grounds, the permitting grant and the journal the key states. A Caveat
program whose view passes but whose record does not is a finding, reported
with the program, and is likewise replaced for phase two.

**Phase two, review.** Six further fresh agents, three per arm. Each receives
one arm's packet: the frozen program, the task text, the three registered
histories, the three handoff notes, `probes.json`, `QUESTIONS.md` and the
answer schema, plus the pinned package (Caveat) or the pinned `typescript`
package (TypeScript). Each writes `ANSWERS.json` and `NOTES.md`. Nothing is
scored until all six are frozen.

The answer keys come from the model of the task, not from either program. The
model, the expected views of every case, the keys and the notes' verdicts are
written before registration and sealed until phase two is frozen.

## The task: the release gate

The authors receive this section verbatim as `TASK.md`, with the sentence
"A reviewer who has not seen your work will later have to establish, from
what you hand over, what each decision rested on." appended. It is a
synthetic policy for a study, not advice about releasing software. All policy
belongs in the program; the host only sends events and reads the view.

### Inputs

| Event | Payload | Meaning |
| --- | --- | --- |
| `propose` | `head`: integer in `1..999` | The head now under consideration |
| `check` | `run`: integer in `1..999`; `passed`: `0` or `1` | A CI run's result |
| `review` | `approved`: `0` or `1`; `relayed`: `0` or `1` | A reviewer's verdict. `relayed` means a bot posted it on the reviewer's behalf |
| `scan` | `findings`: integer in `0..99` | The security scanner's result |
| `approve` | `head`: integer in `1..999` | The release manager's grant to merge that head |
| `decide` | none | Make a merge decision |
| `recheck` | none | The latest check's run was re-queued as flaky: its result is withdrawn |
| `db_aged` | none | The scanner's database is found stale |
| `db_refreshed` | none | The scanner's database was refreshed |

An event with an unknown name, a missing or extra parameter, or a parameter
that is not an integer in its range (a fraction is refused) is refused. So is
every event the rules below refuse. A refused event changes nothing: no
observation, caveat, decision or view value, and the session's sequence does
not advance. Refusal wording is up to the author.

### What the program knows

- Every accepted `check`, `review`, `scan` and `approve` is a distinct
  observation, identified in this study by the 1-based line of the event in
  the history file that recorded it (`#3` is the observation recorded by line
  3). Earlier observations are retained; a later one never overwrites an
  earlier one.
- A check with `passed` 1 supports the claim `ready`; with `passed` 0 it
  opposes it. The program accepts at most **eight** checks; a ninth is refused.
- A review with `approved` 1 supports `ready`; with `approved` 0 it opposes it.
  A review with `relayed` 1 carries the caveat `secondhand` (consequence
  `material`) from the moment it is recorded; a first-hand review carries no
  caveat. At most **four** reviews; a fifth is refused.
- A scan with `findings` 0 supports `ready`; otherwise it opposes it. At most
  **four** scans; a fifth is refused.
- An `approve` records a grant for its `head`. At most **four**; a fifth is
  refused. A grant is an observation too, identified by its line, but it is
  never something a decision rests on.
- `propose` sets the current head. It is accepted whenever its payload is
  valid; it changes no decision and no observation.
- The caveat `deps_unreviewed` (consequence `low`) is declared and is carried
  by no observation: the team has chosen to merge without a dependency review.

### The decision

The decision series is `merge`, with capacity four. "The latest check" is the
most recent check that is not withdrawn; "the latest review", "the latest
scan" and "the latest grant" are the most recent of each. `decide`:

- is refused unless at least one check, one review and one scan have been
  recorded, the latest check exists, and the latest of each supports `ready`;
- is refused while a merge decision is in force; after a decision, the next
  `decide` is accepted only once that decision has been reopened;
- is refused when no grant has been recorded, or when the latest grant is not
  for the current head; the grant is what **permitted** the decision, and it
  is not part of what the decision rests on;
- is refused once four decisions have been made;
- otherwise commits a decision whose **value is the current head** and which
  **rests on exactly three observations: the latest check, the latest review
  and the latest scan** at that moment, with whatever caveats those three
  carry at that moment, and which **retains `deps_unreviewed`** by choice.
  What a decision rests on, and the caveats it was made with, are frozen when
  it is made; later observations, caveats and take-backs never change them.

### Corrections

- A later check with `passed` 0, or a later scan with `findings` above 0,
  reopens the decision in force, if any, citing that observation. The reason
  text (below) becomes `check failed` or `scan found issues`.
- **Withdrawal.** If an observation that the decision in force rests on is
  withdrawn, that decision is reopened; reason text `check withdrawn`.
  Withdrawing an observation that no decision in force rests on, or a grant,
  reopens nothing. Nothing is erased: a withdrawn observation stays in the
  record as withdrawn. The only event that withdraws anything is `recheck`:
  it withdraws the most recently recorded check, with the re-queue as its
  reason, and is refused when no check has been recorded or when that check
  is already withdrawn.
- `db_aged` qualifies the latest scan with the caveat `db_stale` (consequence
  `material`). It is refused when no scan has been recorded or when the
  latest scan already carries `db_stale`. A decision already made keeps the
  caveats it was made with; if the decision in force rests on that scan, the
  same event reopens it; reason text `scan database stale`.
- `db_refreshed` takes `db_stale` back from every scan that carries it. It is
  refused unless the latest scan carries `db_stale`. It reopens nothing,
  decides nothing and changes no view value; a new `decide` may then rest on
  the same scan without the caveat.
- While the latest decision is reopened, a further failing check, scan with
  findings, `recheck` or `db_aged` records its observation or caveat as above
  but reopens nothing and leaves `reason` as it was. `propose` with a new head
  and a new grant reopen nothing.

### The view

After every event, the host reads:

```js
{ status, head, revisions, reason }
```

- `status`: `undecided` before any decision; `merge` while the latest decision
  is in force; `reopened` while it is reopened.
- `head`: the current head, `0` before the first `propose`.
- `revisions`: the number of merge decisions made so far.
- `reason`: `""` before any decision; `ready` when a decision is in force; on
  reopening, one of `check failed`, `scan found issues`, `check withdrawn`,
  `scan database stale`, unchanged until the next decision.

Nothing else is required. Whatever the program keeps beyond this is the
author's choice.

### Interface

A Caveat program is `gate.cav`. The host is the package's own session; a
history line `{"event": "check", "payload": {"run": 7, "passed": 1}}` is sent
as that event, and the view is the displayed values `gate.status`,
`gate.head`, `gate.revisions` and `gate.reason`. A line is refused when the
runtime returns a rejected outcome of any origin (policy, input, evaluation or
limit); a fatal outcome fails the case. A TypeScript program is `gate.ts`,
exporting `createPolicy(): { dispatch(line), view() }`, where `dispatch`
receives the parsed history line, returns normally when the event is accepted
and throws when it is refused, leaving the view unchanged.

## Histories and probes

Three histories, written before registration. Each is a JSONL file of seven
to fourteen lines in the event vocabulary above; the lines are the only
identifiers the questions use. Every history ends with a decision in force,
so that Q2 has something to ask about, and **no history has a refused line
before its last line**: the runtime numbers observations by the session
sequence, which a refused line does not advance, so `explain`'s `observed at
#N` equals the file line only under that constraint. Their shapes:

- **H1, a clean decision** (one decision). One head, one grant, one check,
  one relayed review, one scan, `decide`, then a further passing check that
  no decision rests on. Seven lines.
- **H2, two reopenings** (three decisions). A decision; a scan with findings
  reopens it; a clean scan; a second decision resting on the same check and
  review and the new scan; `recheck` withdraws the check both decisions rest
  on and reopens the second; a passing check; a third decision; a further
  passing check that no decision rests on. Thirteen lines.
- **H3, a late caveat and its take-back** (two decisions). A decision on a
  first-hand review; `db_aged` reopens it; `db_refreshed`; a second decision
  resting on the same scan, now without the caveat; a passing check no
  decision rests on; a new head is proposed and granted; a third `decide` is
  refused. Thirteen lines; the refusal is the last line.

Six decisions in all. Each history has four Q2 probes in `probes.json`, each
naming an observation by line: one the decision in force rests on; one later
observation no decision rests on; one grant; and one further item chosen for
that history. The grant probes are the lineage items: a grant permitted the
decision, so `explain` prints it beside the decision and under "could also
have been influenced by", and the task says withdrawing it reopens nothing.

**Cases.** `cases/` holds the three histories and twenty further short
histories that exercise the refusals and the corrections: an unknown event; a
parameter out of range; a fractional parameter; `decide` with nothing
recorded; `decide` while a decision is in force; `decide` with the latest
review opposing; `decide` with the latest scan opposing; a grant for another
head; no grant; a fifth review; a ninth check; a fifth scan; a fifth grant;
`recheck` twice; `recheck` when the decision in force does not rest on the
latest check; `db_aged` twice; `db_refreshed` before `db_aged`; a fifth
`decide` after four decisions; `decide` after every check is withdrawn; and a
second correction while already reopened. Their inputs are public at stage 1;
the expected view after every line is part of the sealed model.

## The handoff notes

For each history the investigator writes `notes/H<n>.md`, in the voice of the
program's author, as the handoff a reviewer would receive: six numbered
statements about the decisions in that history, four true and two false.
The same note serves both arms, since it speaks of observations by line and
of decisions by number. A false statement is of one of these kinds:

| Kind | The statement |
| --- | --- |
| F1 | names, among what a decision rests on, an observation it does not rest on |
| F2 | says a caveat was absent, or resolved, when the decision was made with it; or says the decision was made with one it was not |
| F3 | says a withdrawn observation was never part of any decision when a decision rested on it |
| F4 | attributes a reopening to the wrong observation, event or cause |
| F5 | names the wrong grant as what permitted a decision |

Each kind is used at least once across the three notes. Which statements are
false, and their kinds, are sealed with the keys. The reviewer prompt says
that a note may contain errors and that every statement is to be judged; it
does not say how many.

## The questions

`QUESTIONS.md` is identical for both arms. For each history, after all its
lines:

1. **What each decision rests on.** For every merge decision made (the first,
   the second, …): (a) its value; (b) the observations it rests on, by line;
   (c) the caveats those observations carried when the decision was made;
   (d) the caveat it retains by choice; (e) the grant that permitted it, by
   line; (f) its current state: `in_force`, `reopened`, or `superseded` (a
   later decision has been made), and, if it was reopened, the line of the
   event that reopened it and the reason text that reopening produced.
2. **What a withdrawal would touch.** The task says that withdrawing an
   observation the decision in force rests on reopens it, and that
   withdrawing anything else reopens nothing. For each probe in
   `probes.json`: if that observation were withdrawn now, would the decision
   in force be reopened? `yes`, `no`, `none_in_force`, or
   `cannot_determine`. No event in the task withdraws a review, a scan or a
   grant, so this is a question about the record, not a run.
3. **The note.** For each statement in the note: `true`, `false` or
   `cannot_determine`.

An **answer** is one Q1 sub-item for one decision, one Q2 probe, or one Q3
statement. Every answer carries a receipt: `{"kind": "record" | "code" |
"run" | "none", "text": "…"}`. `record` is output of a command the packet
shipped (`explain`, `dependents`, `replay`, `test`, or a dump of the
program's own view or logs); `code` is a file and line of the program; `run`
is a script or scenario the reviewer wrote and its output; `none` is an
answer with nothing behind it. The schema is registered as
`answers.schema.json`; an answer file that does not validate is scored as it
is, with each missing item counted as unanswered.

## Known properties of the pinned package

Checked on 2026-10-12 on the published `caveat-lang@0.1.0-rc.19` with a
probe program that implements the task, written while preparing this
document and not in the repository. They are registered here so that the
Caveat arm's results are read against what the package shows, not against
what it was hoped to show.
Each is also a candidate kit or guide finding, filed the way dogfooding
findings are (`experiments/agent-decisions/README.md` L17–18), separately
from this study.

1. **A decision made after a reopening inherits the reopening's caveats into
   its `retaining:` line.** On H3, `explain` prints for the second decision
   `retaining: deps_unreviewed, db_stale`, although the take-back ran before
   it was made and its own journal entry reads `retaining: deps_unreviewed`;
   `explain --json` gives `grounds.caveats: ["deps_unreviewed"]` and
   `retained: ["deps_unreviewed", "db_stale"]`. On H2, the third decision
   shows `withdrawn` the same way. This is the series basis carrying the
   predecessor's reopening witness (`spec/caveat-reactive-0.5.md` L127–134)
   while grounds exclude it (`spec/caveat-explanations-0.2.md` L62–66). The
   key follows the task: Q1(c) is the grounds' caveats, Q1(d) is
   `deps_unreviewed`. The record is correct about what it records; the
   question is whether a reader tells the two apart.
2. **`explain` does not show a take-back.** After `db_refreshed`, `explain`'s
   Evidence line still reads `scan_result … caveats: db_stale`, and
   `explain --json` has no `taken_back`. `replay`'s snapshot records it
   (`taken_back`, and an `unqualify` effect). A `record` receipt for a
   take-back therefore comes from `replay`.
3. **`observed at #N` is the sequence number.** See the history constraint
   above.
4. **The grant appears under "could also have been influenced by".** As the
   grant probes assume.
5. **Carried and authored caveats share the `retaining:` label in the text
   output** (the kit on main separates them, `kit/lib/explain.mjs`
   L301–311; the published rc.19 does not). Q1(c)/(d) are answerable from
   `explain --json`, where `grounds.caveats` and `retained` are separate
   fields, subject to item 1.

## Runs

**Authors.** `A-C` writes the Caveat program; `A-T` writes the TypeScript
program. Each is a new cloud session with no parent context, on its own
machine, in a packet branch `trial/rs1-packet-<id>` holding only its packet:
`TASK.md`, `histories/` and `cases/` (inputs only), and one tarball: the
pinned `caveat-lang` package (Caveat) or the pinned `typescript` package
(TypeScript). An author may read only its working directory, may not use the
web, other directories or other agents, tests as it likes, writes `NOTES.md`
(what it read, how it tested, what it is unsure of, any departure from the
rules), and commits and pushes to its branch. Budget 40 minutes and 32 MiB of
output; a run that exceeds either is kept as it stands. Authors receive no
results.

**Reviewers.** `R-C1`, `R-C2`, `R-C3` review the Caveat program; `R-T1`,
`R-T2`, `R-T3` the TypeScript program. Same mechanism and rules. A reviewer
may run anything in its directory, write scripts, scenarios or
instrumentation on a copy, and use the Caveat package's commands and its
`mcp` tools; the frozen program's hash must be unchanged at the end, and a
changed program voids the run (kept in the record, not scored). A reviewer
receives no result and no hint; a request for help is recorded, not answered.
Same budget as authors; a run that hits the budget is scored as it stands and
its time is recorded as the budget.

**One model family.** Authors and reviewers are one model family, as in
rounds 7 and 8 and authoring v4. Every reviewer runs on the same model and
effort, registered at stage 2 with whatever revision and token counts the
tools expose. A human reviewer, if one volunteers, takes the same packet and
is reported separately, never pooled.

**Pins.** Caveat: the published candidate current at stage 2, expected
`caveat-lang@0.1.0-rc.19`, SHA-256
`d7cae7dc653a4c53de8d3f8e5b945cc324a14e931adcff004b805577d080672f` from
`docs/releases/v0.1.0-rc.19.md`, installed from the tarball with
`--offline --no-audit --no-fund`. TypeScript: Node 22.22.0 and one exact
`typescript` version, shipped as a tarball and installed the same way; both
registered at stage 2 with their SHA-256. The investigator's machine scores
with the same pins. If a later candidate is published before stage 2, it is
the pin, and the *Known properties* section is re-checked against it and
amended.

**Collection.** For every run: the session transcript, start and end times,
every command and tool call with its exit status, the files pushed and their
SHA-256, and the branch head. Interventions are recorded; a restart is
allowed only for a documented infrastructure failure, with both runs kept.
The three reviewers of an arm start together and do not communicate.

## Scoring

`score.mjs` reads each `ANSWERS.json` against the sealed key and reports, per
run, never pooled:

| Measure | Definition |
| --- | --- |
| Q1 | Per decision (six across the three histories) and per sub-item. **Q1 core** is (a), (b), (e) and (f) all correct for a decision; **Q1 caveats** is (c) and (d) both correct; **Q1 strict** is all six |
| Q2 | Correct probes of twelve, with the three grant probes and the three fourth items reported separately |
| Q3 | Hits: false statements marked `false`, of six. False alarms: true statements marked `false`, of twelve. `cannot_determine` is neither |
| Receipts | Count of answers by receipt kind; `grounded` is a judgment by the investigator, per answer, that the receipt supports the answer, null until the cited location and a reason are recorded |
| Cost | Wall clock from session start to final push; number of commands and tool calls; which packet files were read and which commands were run, from the transcript |

A `cannot_determine` is unanswered, not wrong. An answer with a receipt that
does not support it is reported as such; it is the outcome the study is for.

There is no decision rule and no winner. Glowcap's rules A and B measure
other things and are not applied. The standing rule holds: nothing in this
study may claim that Caveat beats TypeScript
(`experiments/glowcap/RESULTS.md` L1338–1339;
`docs/releases/rc18-development.md` L38–39).

## Predictions

Registered before any run; each is a number the record keeps whether or not
it holds. Denominators are per arm: three reviewers, six decisions, twelve
probes, six false and twelve true statements each.

| # | Prediction |
| --- | --- |
| P1 | Q1 core: Caveat reviewers at least 16 of 18 decisions; TypeScript reviewers at most 12 of 18 |
| P2 | Q2: Caveat at least 33 of 36; TypeScript at most 30 of 36; and at least one grant-probe error in each arm, where "could also have been influenced by" is read as grounds, or the code's permission check is read as a dependency |
| P3 | Q3: Caveat at least 16 of 18 hits with at most 3 false alarms of 36; TypeScript at most 12 of 18 hits |
| P4 | Median wall clock of the Caveat arm at most half of the TypeScript arm's, with runs that hit the budget counted at the budget |
| P5 | At least 90 percent of Caveat answers carry a `record` receipt; fewer than half of TypeScript answers do |
| P6 | Q1 caveats: on the two decisions made after a reopening (H2's third, H3's second), at least one Caveat reviewer reports the inherited caveat (`withdrawn`, `db_stale`) under (c) or (d), reading `retaining:` or `retained` as the task's (c) or (d); on the other four decisions, Caveat reviewers at least 11 of 12. TypeScript reviewers at most 12 of 18 on all six |
| P7 | At least one of the two authors' programs fails at least one registered case |

## What a result licenses

If P1, P3 and P5 hold, the README may gain one sentence, with the numbers:
that in one six-reviewer study on one task, fresh reviewers of the Caveat
program established what each decision rested on, and caught the note's
false statements, from the record, where reviewers of the TypeScript program
did not. If any of the three fails, the README's language about
reviewability does not grow, and the failure table says which measure.
Whatever the outcome, every Caveat-arm error is a candidate kit or guide
item, filed the way dogfooding findings are, for 1.1 or for a documentation
correction during the freeze.

## Limits

- **One task, six reviewers.** This cannot estimate how often a reviewer
  succeeds, and two authors cannot show what either language's programs look
  like in general.
- **The investigator's task, model, histories and notes.** The task adopts
  the distinction between what a decision rests on, what permitted it and
  what merely gated it; the questions define it, so a reviewer is scored on
  the task's definition, not on a theory of dependence. The required view
  carries no provenance by design: that choice is the hypothesis, stated
  plainly, and round 8 is the measurement of the other condition.
- **Authors know a reviewer will come.** Both do, in the same sentence. A
  TypeScript author who wants TypeScript to win, round 9's rule, is not this
  study's; the TypeScript author is a fresh agent of the same family.
- **What phase one proves.** For either arm, passing the cases shows the view
  and the admissions; for the Caveat program the record is checked as well.
  Nothing checks what a TypeScript program keeps inside, so a TypeScript
  reviewer answers from the task, the history and the code.
- **Same model family.** Authors and reviewers are from the investigator's
  model family. The session harness adds the account owner's standing
  preferences to each agent's instructions; they contain no Caveat syntax.
- **Isolation by instruction.** Each agent's machine holds only its packet,
  and the sealed files are not on the repository while a reviewer runs. The
  public repository, including this protocol and the count of false
  statements per note, is reachable over the network; nothing enforces the
  rule against reading it, and a transcript that shows it voids the run.
- **What the record proves.** A `record` receipt shows what the runtime
  recorded; it does not show the evidence was true (`README.md` L171–173).
  The study measures whether a reader can recover the record, not whether
  the record is right about the world.

## What is public when

| When | Committed |
| --- | --- |
| Stage 1, registration | This protocol; `TASK.md`; `QUESTIONS.md`; `answers.schema.json`; `prompts/author.md` and `prompts/reviewer.md` with `{PACKAGE}` unfilled; the three histories and the twenty cases' inputs; `probes.json`; the three notes; `score.mjs`; `register.mjs`; `registration.json` with the SHA-256 of every sealed file: the model of the task, the expected views of every case, the keys and the notes' verdicts |
| Stage 2, before any author starts | The pins: the `caveat-lang` and `typescript` tarballs' SHA-256 and versions, Node's version, the model and effort, and the prompts with `{PACKAGE}` filled, as `registration-2.json` |
| After phase one is frozen | The two programs, their notes, and each program's pass or fail per case |
| After phase two is frozen | The six answer files, notes and transcripts; then the sealed files, unsealed and checked against their registered hashes; then the Caveat program's record check, the scores and the report |

`register.mjs` writes with `wx`; no stage overwrites another. The pattern is
`experiments/agent-authoring/v4/register.mjs`, with these stated differences:
the cases' inputs are public from stage 1 so that authors can run them (their
expectations stay sealed), the package pin is registered at stage 2 rather
than at stage 1 because the published candidate may change before the run,
and a failed program is replaced by the reference rather than reported alone,
because phase two needs a program in each arm. As in v4, no program is
repaired after submission; if the model or the task is found wrong after
registration, the original results are kept, the correction is a versioned
amendment, and it is applied to every run.

## Prompts

`prompts/author.md`, with `{RUN}`, `{ARM}` and `{PACKAGE}` filled at stage 2
(`{PACKAGE}` is `caveat-lang-<version>.tgz` or `typescript-<version>.tgz`):

> You are taking part in a programming study. Your run ID is {RUN}. Your
> working directory is the directory this session started in. It contains
> `TASK.md`, the program to write; `histories/`, three event histories the
> program will be run through; `cases/`, further histories it will be run
> through; and `{PACKAGE}`. In your working directory, run `npm init -y`,
> then `npm install --offline --no-audit --no-fund ./{PACKAGE}`. {Caveat
> only: You have never used Caveat before, and everything you need is in the
> package; its documentation is in `node_modules/caveat-lang/`, starting with
> `README.md` and `docs/`.}
>
> Rules. Work only inside your working directory. Read only files inside it.
> Do not read or list anything else on this computer, do not use the web, and
> do not start or contact other agents or sessions. Do not use git except to
> commit and push your finished work.
>
> Write the program `TASK.md` asks for as `{gate.cav | gate.ts}`. Test it as
> much as you like. A reviewer who has not seen your work will later have to
> establish, from what you hand over, what each decision rested on. When you
> have finished, write `NOTES.md`: what you read, how you tested, anything you
> are unsure of, and any departure from the rules above. Commit the program,
> `NOTES.md` and your tests (not `node_modules/`), push to this session's
> branch, and reply with a short summary. If you cannot finish, push your best
> version and say what is missing.

`prompts/reviewer.md`:

> You are taking part in a study of code review. Your run ID is {RUN}. Your
> working directory is the directory this session started in. It contains a
> program you did not write (`{gate.cav | gate.ts}`), the task it implements
> (`TASK.md`), three event histories (`histories/`), three handoff notes from
> its author (`notes/`), `probes.json`, `QUESTIONS.md`, `answers.schema.json`
> and `{PACKAGE}`. Install the package with `npm init -y` and `npm install
> --offline --no-audit --no-fund ./{PACKAGE}`. {Caveat only: Its
> documentation is in `node_modules/caveat-lang/`.}
>
> Rules. Work only inside your working directory and read only files inside
> it. Do not use the web, and do not start or contact other agents or
> sessions. Do not use git except to commit and push your finished work. Do
> not change the program; work on a copy if you want to instrument it.
>
> Answer every question in `QUESTIONS.md` for every history, as
> `ANSWERS.json` in the registered schema, with a receipt for each answer.
> The handoff notes may contain errors; judge every statement. Answer
> `cannot_determine` rather than guess. Then write `NOTES.md`: what you read,
> what you ran, what you are unsure of, and any departure from the rules.
> Commit `ANSWERS.json`, `NOTES.md` and anything you wrote, push to this
> session's branch, and reply with a short summary.

## Amendments

None at preparation.
