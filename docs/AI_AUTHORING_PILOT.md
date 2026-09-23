# Documentation-only authoring pilot — 2026-09-19

Question: can an agent starting without this task's conversation history write
a small useful Caveat program from the language documentation and use the new
headless runner to check it?

This was one constrained authoring task. It was not a comparison against
another language, a test of model training changes, or a measure of adoption.

A later [six-context, three-task study](../experiments/agent-authoring/v1/RESULTS.md)
repeats this question with preregistered tasks, preserved revisions and withheld
verification. Two first submissions and all six final submissions pass its
corpus, with known clock-horizon contract violations in two final sources.

## Task and access

The agent was asked to implement a ventilation controller with two inputs:
`read(value)` bounded from 0 to 40, and `unavailable`. Readings at or below 26
support a comfort claim; hotter readings oppose it. Each reading creates a
distinct record and a fan decision, 1 above 26 and 0 otherwise. Decisions
retain calibration uncertainty and require reopening before revision.

An outage must record opposing evidence and reopen any current decision. It
must not invent a temperature reading or replacement command. Later readings
can revise the command while preserving outage evidence and earlier bases.
The task also required fan and status bindings derived from actual state.

Allowed references were the authoring guide, essence document, reactive
profiles 0.1–0.7, and standard prelude. The agent was instructed not to read
the implementation, tests, existing examples, other agents' work, or the web.
It used the CLI to validate/replay its candidate and inspect JSON output.

## Recorded result

- One complete candidate, preserved before testing.
- Zero source revisions.
- Four CLI runs: validation and three event histories.
- No unexpected validation or replay failure.
- The ninth reading correctly failed at the declared capacity of eight.

The candidate is preserved unchanged in
[`examples/ventilation_controller.cav`](../examples/ventilation_controller.cav).
The saved first attempt and copied example were byte-identical before commit.

The agent's event histories were:

1. `read(26), read(27), read(27), unavailable, unavailable, read(20)`.
   Commands were 0, 1, 1, retained 1 through both outages, then 0. The two equal
   readings had different occurrence identities.
2. `unavailable, unavailable, read(40), unavailable, read(0)`.
   Initial outages produced neither a reading nor a decision. Later input
   exercised both numeric boundaries and recovery after a selected decision.
3. Nine consecutive readings. Replay rejected the ninth, reported committed
   sequence eight, and emitted no ninth event snapshot.

The author inspected snapshots; its first large replay output was partially
truncated in the tool display. Later runs projected relevant JSON fields.
Manual inspection alone was not treated as complete verification.

## Independent verification

After receiving the candidate, the root task added separate regression tests
without editing the source. Four tests pass, including every one of the 256
four-event histories formed from `read(0)`, `read(26)`, `read(40)`, and
`unavailable`. They check command/status behavior, exact reading and revision
counts, and immutable prior readings and decision bases. Other checks cover
outage-first behavior, retained outage/calibration qualifications after recovery,
coexisting supporting/opposing evidence, and atomic capacity failure.

Run `cargo test --manifest-path runtime/Cargo.toml --test ventilation_controller`.

## Interpretation and limits

The observed result supports a narrow conclusion: the supplied documentation
and CLI were sufficient for this agent to author this specified program on its
first candidate. The prompt supplied a detailed policy and invariants. It did
not ask the agent to discover the correct physical policy or verify a sensor.

Recovery also exposed an explanation issue worth investigating. A fresh
decision retains outage reasons and predecessor dependencies even after a new
reading arrives. That records history, but readers need to distinguish a past
reason for revision from a claim about present sensor availability. Flat
qualification lists alone may be insufficient for clear explanations.

Token cost and wall-time advantage were not measured. There was no TypeScript
baseline, repeated model sample, independent human user, live sensor, or
deployment. The candidate's capacity error occurs before later effects; the
separate runtime suite covers late-effect rollback. Repeat across tasks and
compare equivalent provenance implementations before claiming reliability or
efficiency advantages.
