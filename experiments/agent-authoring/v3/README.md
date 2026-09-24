# Agent authoring trial v3

Fresh agents write a Caveat program for a new task using only the installed
release candidate and its documentation, and other fresh agents then carry
out a change request on those programs. The design is in
[PROTOCOL.md](PROTOCOL.md); the task is [tasks/pond.md](tasks/pond.md).

| Path | What it holds |
| --- | --- |
| `registration.json` | SHA-256 of the release candidate, every public file and every sealed file, recorded before any agent started |
| `tasks/` | The task; the change request once unsealed |
| `prompts/` | The exact instructions each agent received (`{RUN}` and `{DIR}` filled in) |
| `private/` | The task model, the registered cases and the scoring library; the change request's model once unsealed |
| `references/` | The investigator's reference programs, sealed until phase two |
| `runs/<id>/` | Each agent's frozen program, notes and own tests, with `record.json` |
| `results/` | Scores, written by `score.mjs` after each phase |

## Reproducing the scores

`score.mjs` needs the registered tarball, `caveat-lang-0.1.0-rc.1.tgz` from
the `kit-package-candidate` artifact of the tagged commit's CI run (its SHA-256
is in `registration.json`). It checks every registered file, installs the
tarball offline into `test-results/agent-authoring-v3/`, and scores the frozen
programs through the installed package:

```sh
node experiments/agent-authoring/v3/score.mjs --phase 1 --tarball PATH --output test-results/v3-phase1.json
node experiments/agent-authoring/v3/score.mjs --phase 2 --tarball PATH --output test-results/v3-phase2.json
```

Without `--output`, it writes `results/phase<N>.json` and refuses to overwrite
it. A failing program is a result; the scorer exits with an error only when an
input does not match its registration.
