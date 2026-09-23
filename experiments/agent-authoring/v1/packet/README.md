# Public author packet

Implement only the assigned task as a standalone `candidate.cav` in your own
`runs/ID/` directory. The repository root is `C:/Dev/caveat-lang`, explicitly
authorized for this work. Your run ID and task path are given in your prompt.
No imports, host gameplay code, runtime changes, or third-party dependencies.

Read `reference/docs/AI_AUTHORING.md` first. You may read any file under
`reference/`, this README, this public runner, your assigned task, and your own
run directory. Links in the guide to examples, tests or game programs are not
part of this packet: do not follow them. Do not read the private verifier,
other runs, repository runtime source, existing game/example implementations,
study results, or the web. Do not contact other agents. This is instructed
isolation, not an operating-system security boundary.

Use the provided runner instead of the native CLI commands in the guide:

```sh
node experiments/agent-authoring/v1/packet/runner.mjs A1 validate
node experiments/agent-authoring/v1/packet/runner.mjs A1 replay events.jsonl
node experiments/agent-authoring/v1/packet/runner.mjs A1 finish
```

Replace A1 with your run ID. The runner always reads `runs/ID/candidate.cav`.
The replay file is relative to your run directory. Each line is either
`{"event":"name","payload":{...}}` or `{"resume":true}`. Replays continue
after rejected events, reporting each rejection and the unchanged snapshot.
You must inspect acceptance/rejection and output yourself; the runner does
not know the task policy. Full snapshots are saved in `checks/*.jsonl`; console
output selects bindings, grounds, journal and relations for readability.

You may submit **at most three distinct source versions** and use **at most
twelve validation/replay commands**. The first source is preserved before
the first runtime check; later changed sources are also preserved. Self-tests
and local repairs are allowed within those bounds. The private verifier will
not give feedback during authoring. Do not use another execution path to
avoid the runner's submission/check accounting, and do not edit its records,
versions or check logs. You may return to a previously preserved source.

When ready, write `notes.md` describing references read, tests tried, any
uncertainty or access-rule deviation, and why the final candidate meets the
contract. Then run `finish`, which freezes `final.cav`, and report completion.
Do not edit after finishing. If you cannot finish correctly, preserve and
finish your best available candidate, explaining the limitation honestly.
