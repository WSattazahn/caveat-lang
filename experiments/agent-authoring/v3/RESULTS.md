# Fresh-agent trial v3 — 2026-09-24 UTC

**Both fresh authors wrote a program for a task nobody had attempted before,
using only the installed `caveat-lang` release candidate, and both programs
passed every registered case (112/112). Two further fresh agents each
inherited one of those programs and carried out a change request they had
never seen, and both changed programs passed every case (120/120).**

The task, whether a village rink may open on pond ice, is new. It is not one
of the clock tasks of [v1](../v1/RESULTS.md) and [v2](../v2/RESULTS.md), and it
is not one of the repository's games. Each agent had only the tagged
`v0.1.0-rc.1` tarball and its packaged documentation. This is a small
feasibility result for one task, with the limits below.

## Outcomes

| Run | Phase | Given | Registered cases | Program |
| --- | --- | --- | ---: | --- |
| A | 1: author | task | 112/112 | 69 lines, `9099a48c` |
| B | 1: author | task | 112/112 | 72 lines, `5286c80c` |
| A2 | 2: maintainer | A's program, task, change request | 120/120 | 91 lines, `6ee3cdf5` |
| B2 | 2: maintainer | B's program, task, change request | 120/120 | 107 lines, `31c85e8c` |

Program hashes are the SHA-256 recorded when each program was frozen, and the
case counts come from [phase 1](results/phase1.json) and
[phase 2](results/phase2.json).

**What phase one checked.** Phase one has 12 targeted scenarios and 100 seeded
40-step sequences.

**What phase two checked.** Phase two has 20 targeted scenarios, 12 of them the
phase-one scenarios unchanged, plus 100 sequences.

**What every step checked.**

- admission;
- the complete `hud`;
- relations and qualifications;
- `commitment_grounds` and the `decision_journal`;
- that grounds stay within lineage;
- the sequence;
- that a refused event changes nothing;
- exact save and restore;
- continued agreement with an uninterrupted session.

**The change.** The change request added a second instrument, a sonar gauge.
It has its own event, reading stream, caveat and cap. It enters the decision's
value and grounds and can reopen an open rink, and it adds a display property.
A2 changed 27 lines added and 5 removed; B2 changed 42 added and 7 removed.
Both maintainers first checked the inherited program against the original task
and found nothing to fix.

## How the agents worked

| Run | Duration | Tool uses | Tokens |
| --- | --- | ---: | ---: |
| A | 5 min 30 s | 31 | 177,536 |
| B | 5 min 10 s | 24 | 178,074 |
| A2 | 7 min 14 s after resuming | 16 after resuming | 210,899 after resuming |
| B2 | 5 min 36 s after resuming | 14 after resuming | 202,134 after resuming |

[launches.json](launches.json) records each prompt's SHA-256 and the dispatch
and completion times.

**What they read.** All four agents read the getting-started guide, then most
of the packaged specifications, about thirty documents in all. None asked for
anything outside the package.

**How they tested.** Each wrote its own scenario file and ran it with
`caveat test`. Each also built an independent JavaScript model of the task and
compared it with its program on random event sequences, with saves and
restores:

| Run | Random testing |
| --- | --- |
| A | 2,400 runs |
| B | about 56,000 events |
| A2 | 7,500 runs, plus 1,500 against the unchanged original |
| B2 | about 150,000 events |

B and A2 also ran deliberately broken copies of their program against their
own tests. Their tests are kept in `runs/<id>/`.

## What they ran into

- **A full history was fatal, not a refusal.** Both authors guarded their
  reading stream with an explicit `reject` before its declared limit. B found
  through its own mutation test that without the guard the ninth measurement
  ends the session with a fatal `unclassified` error. The dispatch
  specification said so, but only in passing. The runtime has since been
  changed to refuse such an event as `limit/history_limit` (see
  [the dispatch specification](../../../spec/caveat-dispatch-0.1.md#changes)).
  This result is for the release candidate as tested.
- **Lineage versus grounds.** A, B and A2 each noted that a later decision's
  lineage includes the evidence that reopened its predecessor, even though its
  grounds do not. They checked the specification and kept the grounds correct.
  `caveat explain`, added after the candidate, shows the two side by side.
- **Repeated rules for a second instrument.** A2 notes that a reading stream
  cannot be passed to a procedure, so the sonar rules repeat the auger rules.
  B2 wrote them out twice as well.
- **Interpretation.** A was unsure whether a crack reported before any decision
  should affect a later one. The task says reopening applies to an open rink.
  Both programs pass the targeted case for exactly this history, so they treat
  it as the registered model does.

## Limits

- **One task, four runs.** Four runs on one task cannot estimate how often an
  agent succeeds, or compare Caveat with another language.
- **Same model family.** The agents are new contexts of the investigator's own
  model family, dispatched by the investigator, who also wrote the task, its
  model and the cases. The harness also placed the investigator's unrelated
  project instructions and memory index in each agent's session. A and both
  maintainers noticed this and report not using it; that material contains no
  Caveat syntax.
- **Isolation by instruction.** The trial directories sat in the
  investigator's scratch area, next to the sealed materials. Isolation was
  instructed, not enforced, so unobserved reads cannot be ruled out. Every
  agent reports none.
- **One network access.** The instructed `npm install` ran npm's default audit,
  which probably contacted the registry. B2 disclosed this. The package itself
  has no dependencies.
- **An interrupted phase two.** An account usage limit stopped both maintainers
  early. Neither had yet changed its program; both still matched the inherited
  program byte for byte. Each was resumed, with its context, by one neutral
  message recorded in `launches.json`. Their durations above cover only the
  work after resuming.
- **What passing covers.** A corpus pass is not proof of correctness for every
  history. The seeded sequences mix every event, with malformed input and
  restores, but they cannot cover every possible history.

## Registration and integrity

The trial was registered in `d8eb296` before any agent started. That commit
fixed:

- the tarball's SHA-256 (`be0dac79…ec32`, the CI-built `v0.1.0-rc.1` package
  of `aa5f4fc`);
- every public study file;
- the SHA-256 of each sealed file: the change request, its model and cases, and
  the investigator's reference programs.

The sealed files were committed in `bcc688a`, after both phase-one programs were
frozen and scored, and each matched its registered hash. The scorer refuses any
input that differs from its registration.

Before registration, through the installed release candidate, the reference
programs passed every case of their phase, and 18 deliberately wrong programs
failed ([validation](references/)). No case, model or scoring rule changed after
registration, and no program was repaired.
