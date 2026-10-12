# Reviewer study v1

Can a fresh reviewer establish what each decision in a program it did not
write rested on, what a withdrawal would touch, and which statements in the
author's handoff note are false? [PROTOCOL.md](PROTOCOL.md) is the design;
its Amendment 1 records what changed before registration.

| File | What it is |
| --- | --- |
| `TASK.md` | The release gate the two authors write, from the protocol's task section |
| `histories/`, `probes.json`, `notes/` | The three registered histories, the Q2 probes and the handoff notes |
| `cases/` | Twenty-one further histories' inputs; their expected views are sealed |
| `QUESTIONS.md`, `answers.schema.json` | What reviewers answer, and the answer file's shape |
| `prompts/` | The author and reviewer prompts, filled per run at stage 2 |
| `run-cases.mjs` | Runs a program of either arm through every history and case |
| `score.mjs` | Scores programs against the expected views and answers against the keys |
| `register.mjs` | Writes `registration.json` (stage 1) and `registration-2.json` (stage 2) |

The sealed files (`model.mjs`, `expected.json`, `keys.json` and the
investigator's two reference programs) are committed after phase two is
frozen, and checked against the hashes in `registration.json`.

Nothing here is a claim that Caveat beats TypeScript
(`experiments/glowcap/RESULTS.md` L1338–1339).
