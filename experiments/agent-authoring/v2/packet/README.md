# Public author packet

Implement only your assigned synthetic task as standalone `candidate.cav` in
your own `experiments/agent-authoring/v2/runs/ID/` directory. The repository is
`C:/Dev/caveat-lang`, explicitly authorized for this work. All policy belongs
in Caveat. No imports, host policy code, runtime changes or dependencies.

Read `reference/docs/AI_AUTHORING.md` first. You may read this README, the
public runner, any file under `reference/`, your assigned task and your own
run directory. The guide's final historical-study section was omitted from
the packet. Do not follow links to material outside this packet. Do not read
the runtime implementation, private verification, study protocol/results,
v1 artifacts, other runs, repository game/example programs or the web. Do not
contact or spawn other agents. This is instructed isolation on a shared
filesystem, not an operating-system security boundary.

Use the supplied runner instead of native CLI commands in the guide:

```sh
node experiments/agent-authoring/v2/packet/runner.mjs A1 validate
node experiments/agent-authoring/v2/packet/runner.mjs A1 replay events.jsonl
node experiments/agent-authoring/v2/packet/runner.mjs A1 finish
```

Replace A1 with your ID. The runner reads `runs/ID/candidate.cav`. Replay
input is relative to your own directory. Each JSONL line is either
`{"event":"name","payload":{...}}` or `{"resume":true}`. Omitted payloads
default to `{}`; explicit `null`, arrays and scalars are preserved. The runner
parses/stringifies JSON, so duplicate keys/nonfinite spellings are not a raw
transport test. Replays continue after rejected events and report the snapshot.
Inspect acceptance/rejection and output yourself: this tool knows no task
policy. Full snapshots are retained in `checks/*.jsonl`; console output shows
bindings, grounds, journal and relations.

Use at most **three distinct submitted source versions** and **twelve
validation/replay commands**. The runner preserves each new source before
execution. Self-tests and repairs are permitted within those bounds, including
returning to a previously preserved source. `finish` does not consume a check.
Do not use another execution path, edit records/logs/preserved versions, or
modify the packet. There is no private-verifier feedback during authoring.

Write `notes.md` listing references read, self-checks, uncertainties, access-rule
deviations and why the final source meets the contract. Run `finish` to freeze
`final.cav`, then report completion. Do not edit after finishing. If unsolved,
finish your best available source and state its limitations honestly.
