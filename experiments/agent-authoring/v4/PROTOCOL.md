# Fresh-agent trial v4: separate machines, two models, the second candidate

Question: using only the installed `caveat-lang` 0.1.0-rc.2 package and its
documentation, can a fresh agent write a correct program for a new task? Can a
second fresh agent, given that program, then carry out a change request it has
never seen?

v4 repeats [v3](../v3/PROTOCOL.md) with a new task. It also tightens the four
limits v3 reported:

| v3 limit | v4 |
| --- | --- |
| Four runs on one task | Eight runs: four authors, four maintainers |
| One model | Two models: A, B and their maintainers on Claude Opus 5.5; C, D and theirs on Claude Sonnet 5 |
| Isolation by instruction, in the investigator's scratch area next to the sealed files | Each agent runs in its own cloud session, on its own machine. That machine holds only the agent's packet. The scoring model and cases stay off the repository until the phase they score is frozen. The change request and the references stay off it until phase two is frozen. |
| The instructed `npm install` contacted the registry | The prompts install with `--offline --no-audit --no-fund` |

It remains a small feasibility trial on one task. It cannot estimate a success
rate, compare Caveat with another language, or compare the two models. Two runs
per model are too few for that.

## Materials

- **The release candidate.** `caveat-lang-0.1.0-rc.2.tgz`, the tarball CI built
  and tested for the tagged commit. Its SHA-256 is in `registration.json`. It
  is the only reference an agent receives.
- **The task.** [`tasks/ferry.md`](tasks/ferry.md), written from scratch for
  this trial. It has two instruments with parallel rules, gusts and waves, so a
  procedure over reading streams (new in rc.2) could serve both. Authors
  receive it as `TASK.md`.
- **The change request.** It was written, modelled and tested before any agent
  started. Only its SHA-256 is registered. Maintainers receive it as
  `CHANGE.md`.
- **The scoring.** An independent model of the task (`private/oracle.mjs`),
  registered cases (`private/corpus.mjs`) and the change request's model and
  cases (`private/change-oracle.mjs`). The scoring library
  ([`private/score-lib.mjs`](private/score-lib.mjs), v3's unchanged) and
  [`score.mjs`](score.mjs) run a program through the installed package's own
  session library. The model also checks the consequence of each caveat the
  task names, which v3 did not. Phase one has 15 targeted cases and 100 seeded
  40-step sequences. Phase two has 20 targeted cases (9 of phase one's among
  them) and 100 sequences.

Before registration, the investigator's reference programs passed every case of
their phase. The phase-two reference uses a procedure over reading streams.
Deliberately wrong programs, each with one plausible mistake, all failed: 19
for phase one and 9 for phase two.

## What is public when

| When | Committed |
| --- | --- |
| Registration, before any agent starts | This protocol, the task, both prompts, `score-lib.mjs`, `score.mjs`, `register.mjs`, and `registration.json` with the SHA-256 of every sealed file |
| After phase one is frozen | `private/oracle.mjs`, `private/corpus.mjs`, the phase-one programs and their score |
| After phase two is frozen | The change request, `private/change-oracle.mjs`, the references, the phase-two programs and their score, and the report |

Each sealed file is checked against its registered hash when it is committed.

## Runs

Phase one: four fresh authors, **A**, **B**, **C** and **D**, each implement
the task independently. Phase two: four further fresh agents, **A2**, **B2**,
**C2** and **D2**. Each receives one frozen phase-one program (A's, B's, C's or
D's), the task and the change request, and changes the program. A maintainer
uses its author's model and sees only the program, never the author's notes.

Each agent is a new cloud session with no parent context. Its checkout is a
packet branch, `trial/v4-packet-<id>`, which holds only its packet: the tarball
and `TASK.md` for an author, plus `ferry.cav` and `CHANGE.md` for a
maintainer. The prompts are registered ([`prompts/`](prompts)). An agent may
read only its working directory, including the installed package. It may not
use the web, other directories or other agents. It writes `NOTES.md`: what it
read, what it tested, what it is unsure of, and any departure from the rules.
It then commits and pushes its work to its own branch. It may use the `caveat`
command and scripts as often as it likes.

These limits remain and are stated with every result:

- The investigator wrote the task, its model and the cases.
- Both models are from the investigator's own model family.
- The session harness adds the account owner's standing preferences to each
  agent's instructions. They contain no Caveat syntax.
- An agent's machine can still reach the public repository over the network.
  Isolation from the sealed files is enforced, because they are not there
  yet. Isolation from the rest of the repository, including earlier trials'
  programs for a different task, is by instruction only.

## Freeze and scoring

A program is frozen when its agent's branch has its final push. The
investigator copies the program, notes and tests into `runs/<id>/` and records
their SHA-256 in `record.json`. Phase-two agents start only after all four
phase-one programs are frozen. Scoring happens after each phase is frozen,
never during a run, and no agent receives results.

A program passes a phase if it loads and passes every registered case of that
phase. The checks are v3's, plus caveat consequences. At every step the scorer
checks:

- admission;
- the complete `hud`;
- supports/opposes relations to `crossing_safe`, in order;
- qualifications on observed evidence, and the consequences of the task's caveats;
- `commitment_grounds` and the `decision_journal`;
- that grounds stay within lineage;
- the session sequence;
- that a refused event leaves the save and the view unchanged.

At every save and restore it checks that the view and snapshot are identical,
and that the restored session keeps agreeing with the original. A fatal
runtime error fails the case; it is not a refusal.

Also report, as observations rather than pass criteria:

- whether each program uses a procedure for the parallel instruments;
- whether it refuses a full history itself or leaves that to the runtime's
  `limit/history_limit`;
- for phase two, the size of the change: lines added and removed.

Report every run, including failures and programs that do not load, with the
first failing step of each failing case. Do not select among versions, add
cases after registration, or repair a program. If the scorer or model is found
to be wrong after registration, keep the original results, document the
correction, and apply it to every run.
