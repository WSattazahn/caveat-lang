# Fresh-agent trial v3: a new task from the packaged kit, then a change

Question: can a fresh agent that has only the installed `caveat-lang` release
candidate and its packaged documentation write a correct program for a task
nobody has attempted before, and can a second fresh agent, given that program,
carry out a change request it has never seen?

v1 and v2 used tasks from the same family and a documentation packet cut from
the repository. v3 changes all three: a new task (whether the village rink may
open), the product itself as the only reference, and a maintenance step. It
remains a small feasibility trial. It cannot estimate a success rate, compare
Caveat with another language, or separate the effect of the documentation from
that of the task.

## Materials

- **The release candidate.** The tagged `caveat-lang-0.1.0-rc.1.tgz` built by CI
  for the tagged commit, with the SHA-256 recorded in `registration.json`. Its
  documentation (`docs/`, the getting-started guide, the authoring guide,
  the language and scenario specifications and one example) is the only reference an author
  receives. Nothing is removed from it for this trial; its authoring guide
  summarizes the earlier studies, and authors see that summary as any user would.
- **The task.** [`tasks/pond.md`](tasks/pond.md), written from scratch for this
  trial. Authors receive it as `TASK.md`.
- **The change request.** Written, modelled and tested before any author
  started, but sealed: only its SHA-256, and those of its model and cases, are
  registered. It is committed, and checked against those hashes, only after
  both phase-one programs are frozen.
- **The scoring.** An independent model of the task
  ([`private/oracle.mjs`](private/oracle.mjs)), registered cases
  ([`private/corpus.mjs`](private/corpus.mjs)) and a scorer
  ([`private/score-lib.mjs`](private/score-lib.mjs), [`score.mjs`](score.mjs))
  that runs a program through the kit's own session library. Phase one has 12
  targeted cases and 100 seeded 40-step sequences; the sealed phase two has 20
  targeted cases (the 12 unchanged ones among them) and 100 sequences.

Before registration, reference programs written by the investigator passed
every case of their phase, and 18 deliberately wrong programs, each with one
plausible mistake, failed. The references stay sealed with the change request
and are committed with it.

## Runs

Phase one: two fresh authors, **A** and **B**, each implement the task
independently. Phase two: two further fresh agents, **A2** and **B2**, each
receive the frozen phase-one program of A or B, the task and the change request,
and change the program. A maintainer never sees the author's conversation or
notes, only the program, so phase two stands in for returning to code months
later or inheriting someone else's.

Every agent starts in its own empty directory outside the repository with the
tarball and its instructions. The prompts are registered
([`prompts/`](prompts)). An agent may read only files in its own directory,
including the installed package, may not use the web, other directories or
other agents, and writes `NOTES.md`: what it read, what it tested, what it is
unsure of, and any departure from these rules. It may use the `caveat` command
and scripts as often as it likes; there is no budget of checks or versions, and
drafts are not observed.

The agents are new contexts of the investigator's own model family, dispatched
by the investigator, who also wrote the task and its model. The directories are
not sandboxed; isolation is by instruction, and unobserved reads cannot be ruled
out. These limits are stated with every result.

## Freeze and scoring

A program is frozen when its agent finishes: the investigator copies it into
`runs/<id>/`, with the agent's notes and any tests it wrote, and records its
SHA-256. Phase-two agents start only after both phase-one programs are frozen
and the change request is unsealed. Scoring happens after each phase's
programs are frozen, never during a run, and no agent receives results.

A program passes a phase if it loads and passes every registered case of that
phase. For each case the scorer checks, at every step: admission; the complete
`hud`; supports/opposes relations to `ice_safe` in order; qualifications on
observed evidence; `commitment_grounds`; the `decision_journal`; that grounds
stay within lineage; the session sequence; that a refused event leaves the save
and view unchanged; and, at every save and restore, an identical view and
snapshot and continued agreement with the original session. A fatal runtime
error fails the case; it is not a refusal.

Report every run, including failures and programs that do not load: the case
counts, the first failing step of each failing case, and, for phase two, the
size of the change (lines added and removed). Do not select among versions, add
cases after registration, or repair a program. If the scorer or model is found
to be wrong after registration, preserve the original results, document the
correction and apply it to every run.
