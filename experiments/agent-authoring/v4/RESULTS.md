# Trial v4 results

Eight fresh agents, each in its own cloud session, used only the installed
`caveat-lang` 0.1.0-rc.2 tarball and its documentation:

- Four authors wrote a program for a task nobody had attempted before, deciding
  whether an island ferry may cross.
- Four maintainers each inherited one of those programs and carried out a
  sealed change request, adding a second anemometer.

Every program passed every registered case: 115/115 in phase one and 120/120
in phase two. This is a small feasibility result for one task, with the limits
below.

## Outcomes

| Run | Model | Phase | Given | Registered cases | Program | Change |
| --- | --- | --- | --- | ---: | --- | --- |
| A | Opus 5.5 | 1: author | task | 115/115 | 73 lines, `e5fef462` | |
| B | Opus 5.5 | 1: author | task | 115/115 | 65 lines, `57742c4f` | |
| C | Sonnet 5 | 1: author | task | 115/115 | 60 lines, `4a740d2f` | |
| D | Sonnet 5 | 1: author | task | 115/115 | 61 lines, `360e9bd1` | |
| A2 | Opus 5.5 | 2: maintainer | A's program, task, change | 120/120 | 90 lines, `0fca0377` | +21 −4 |
| B2 | Opus 5.5 | 2: maintainer | B's program, task, change | 120/120 | 80 lines, `af0c7b24` | +19 −4 |
| C2 | Sonnet 5 | 2: maintainer | C's program, task, change | 120/120 | 72 lines, `0d3e93f5` | +15 −3 |
| D2 | Sonnet 5 | 2: maintainer | D's program, task, change | 120/120 | 73 lines, `4bca20d5` | +16 −4 |

- **Program:** the SHA-256 recorded when each program was frozen, in
  `runs/<id>/record.json`.
- **Change:** lines added and removed relative to the inherited program.
- **Case counts:** from [phase 1](results/phase1.json) and
  [phase 2](results/phase2.json).

Each agent finished in 2.5 to 9 minutes, measured from dispatch to its last
push. [`launches.json`](launches.json) has the sessions, prompt hashes, times
and each session's reported cost (US$1.54 to US$2.70).

The phase-two cases include 9 of phase one's targeted cases, unchanged, and
the model decides their outcome under the amended rules. So a maintainer that
broke the original behaviour would have failed there.

## Procedures and full histories

The task has two instruments with parallel rules, gusts and waves. The change
adds a third, pier gusts. rc.2 lets one procedure take a reading stream, so a
single `proc` could have served all three.

- **No agent used a procedure.** All four authors wrote each instrument's
  rules out, and all four maintainers copied the gust rules for the pier.
  Where the notes mention procedures:
  - A read only the headings of the procedure specifications;
  - C decided that the task "needs no `proc`";
  - D and D2 list procedure symbols among the documents they skipped.

  A2, B2 and C2 do not mention procedures.

  The packaged [authoring guide](../../../docs/AI_AUTHORING.md), which every
  agent read first, never mentions procedures. They appear only as two entries
  in the documentation index, "reusable procedures" and "procedures that take
  evidence, other symbols and histories as parameters". The feature is
  documented, but nothing brings it to an author's attention.
- **Full histories.** A and B still guard the ten-reading limit with their
  own `reject`, and their maintainers add the same guard for the pier's six.
  A's notes say the declared limit "would also refuse, as
  `limit/history_limit`", so the guard is a choice there, not a workaround. C,
  D and their maintainers leave the limit to the runtime. In v3, both
  authors guarded their limits, because a full history was fatal in rc.1.

## What they ran into

- **Consequence labels.** D found no written list of legal consequence labels
  (the examples use `material` and `high`) and used `low` as the task asked.
- **Order of grounds.** B2 and D2 found that `commitment_grounds` lists
  evidence sorted by name, and the journal's `because` lists it in
  observation order. D2 notes that only the journal's order is documented.
- **Non-finite payloads.** B could not send NaN or Infinity through the
  session library, which refuses them before the runtime. So that refusal
  path was not tested from the program's side.
- **`hud.warned` type.** C was unsure whether `1`/`0` meant numbers or
  booleans, and bound the numeric state.

## Departures

Every agent wrote a "Departures" section in its notes. These are what they
reported, and what the investigator observed.

- **Outside the working directory.** B ran a command that created, then
  deleted, an empty file under `/tmp`. B2 kept a scratch events file in its
  session's scratchpad directory. Neither read anything outside.
- **Extra git commands.** C2 ran `git status`, `git branch -a`, `git fetch
  origin --prune` and `git log` to find the branch to push to. `git fetch`
  downloads every public branch. At that time those included the other
  authors' frozen programs and the phase-one model and cases, which had been
  committed after phase one froze. The change request's model and the
  references were not yet on the repository. C2 reports using the commands
  only to find its branch, and its program is its inherited program plus the
  change.
- **Branches.** A2, B2 and C2 created their run branch themselves before
  pushing. D2's session reported its packet branch as its branch and pushed
  there. Its program is frozen from `trial/v4-packet-D2`.
- **Installed manifests.** Most agents made a final commit adding
  `package.json`, `package-lock.json` or `.gitignore` after the session asked
  them to leave no untracked files. The investigator froze `package.json` but
  not the lock files or `.gitignore`.

## Limits

- **One task, eight runs.** Eight runs on one task cannot estimate how often
  an agent succeeds, compare the two models, or compare Caveat with another
  language.
- **The investigator's task.** The investigator wrote the task, the model, the
  cases and the change request, and wrote a passing reference program for
  each phase. The task is close in shape to v3's: readings, a warning, and a
  decision that later evidence reopens.
- **Same model family.** Both models are from the investigator's model
  family. The session harness also placed the account owner's standing
  preferences in each agent's instructions. They contain no Caveat syntax.
- **Isolation.** Each agent ran on its own machine, and its checkout held only
  its packet. Sealed files were not on the repository while an agent that
  could use them was running. The rest of the public repository was reachable
  over the network and, as C2 shows, one agent fetched it. Nothing enforced
  the rule against reading it.
- **What passing covers.** A corpus pass is not proof of correctness for
  every history. The seeded sequences mix every event, including malformed
  input and restores, but cannot cover every possible history.

## Registration and integrity

The trial was registered in `3d2474da9850e1838b8adcc2a11d820ce81c05e8` before
any agent started. That commit fixed:

- the SHA-256 of the rc.2 tarball;
- the SHA-256 of every public file;
- the SHA-256 of every sealed file.

Phase one was frozen and scored in `1f0d8a3b538b1b390cc47b3e7b16a4c0c6b932ca`,
which also committed the task model and cases. Both matched their registered
hashes. Phase two's commit adds the change request, its model and both
references, each matching its registered hash.

No case, model or scoring rule changed after registration, and no program was
repaired. `score.mjs` re-checks every file against `registration.json` before
it scores.

## Next

Make history parameters discoverable. The authoring guide should show a
procedure over reading streams when a task has parallel instruments. A later
trial can then check whether authors use it.
