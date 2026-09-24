# Agent authoring trial v4

Fresh agents, each in its own cloud session, write a Caveat program for a new
task using only the installed `caveat-lang` 0.1.0-rc.2 package. Other fresh
agents then carry out a change request on those programs. The design is in
[PROTOCOL.md](PROTOCOL.md); the task is [tasks/ferry.md](tasks/ferry.md).

| Path | What it holds |
| --- | --- |
| `registration.json` | SHA-256 of the release candidate, every public file and every sealed file, recorded before any agent started |
| `tasks/` | The task; the change request once phase two is frozen |
| `prompts/` | The exact instructions each agent received (`{RUN}` filled in) |
| `private/` | The scoring library; the task model and cases once phase one is frozen; the change request's model once phase two is frozen |
| `references/` | The investigator's reference programs, sealed until phase two is frozen |
| `runs/<id>/` | Each agent's frozen program, notes and own tests, with `record.json` |
| `results/` | Scores, written by `score.mjs` after each phase |

## Reproducing the scores

`score.mjs` needs the registered tarball, `caveat-lang-0.1.0-rc.2.tgz`. Its
SHA-256 is in `registration.json`. The script checks every registered file,
installs the tarball offline into `test-results/agent-authoring-v4/`, and
scores the frozen programs through the installed package:

```sh
node experiments/agent-authoring/v4/score.mjs --phase 1 --tarball PATH --output test-results/v4-phase1.json
node experiments/agent-authoring/v4/score.mjs --phase 2 --tarball PATH --output test-results/v4-phase2.json
```

Without `--output`, it writes `results/phase<N>.json` and refuses to overwrite
it. A failing program is a result; the scorer exits with an error only when an
input does not match its registration.
