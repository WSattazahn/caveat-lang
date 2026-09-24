# Notes for run A — `pond.cav`

## Documentation read

All from `node_modules/caveat-lang/` after installing the tarball:

- `README.md`, `docs/README.md`, `docs/GETTING_STARTED.md`
- `docs/reference/docs/AI_AUTHORING.md`
- Specs in `docs/reference/spec/`: `caveat-0.1`, `caveat-reactive-0.1` through
  `caveat-reactive-0.7`, `caveat-define-0.1`, `caveat-reject-0.1`,
  `caveat-decision-journal-0.1`, `caveat-explanations-0.1` and `0.2`,
  `caveat-text-0.1`, `caveat-save-0.1`, `caveat-view-0.1`,
  `caveat-scenarios-0.1`, `caveat-dispatch-0.1`,
  `caveat-observation-order-0.1`, `caveat-late-qualification-0.1`,
  `caveat-state-caveats-0.1`, `caveat-renewal-0.1`,
  `caveat-typed-parameters-0.1`
- `docs/reference/runtime/prelude.cav`
- `docs/reference/examples/thermostat_history.cav` and its scenarios file

Not read: `CAVEAT_ESSENCE.md`, and the repetition, procedure-symbols and
elapsed specs, because the task does not use those features.

## How the program works

- `readings thickness from auger limit 8` holds the measurements. `single_hole`
  (material) qualifies `auger`, so every `thickness@N` carries it. A reading of
  10 cm or more `supports ice_safe`; a thinner one `opposes` it.
- `crack_report` (from "a skater") is qualified by `secondhand` (low). The first
  `crack` reveals it as opposing `ice_safe`.
- `decisions rink limit 3`. `decide` runs
  `commit rink because enough using fold_history(thickness, latest(thickness), min)`.
  That makes the decision's value the minimum over every reading. Its
  `commitment_grounds` are every `thickness@N` plus `single_hole`, and nothing
  else. The crack report is not in them.
- Refusals are `reject` rules placed before any effect: a ninth `measure`, a
  second `crack`, a `decide` with fewer than three readings, and a `decide`
  while a decision is in force (committed and not reopened). Unknown events,
  missing or extra parameters, and values that are not numbers in 0..60 are
  refused by the runtime itself, with origin `input`.
- Reopening happens only while the rink is open, meaning
  `committed(rink) and not reopened(rink) and latest(rink) >= 10`. A thin
  reading reopens `because latest(thickness)`, which is the reading just taken.
  The crack reopens `because crack_report`. Because of the
  `not reopened(rink)` test, an already reopened decision gets no further
  reopening. Because of the `>= 10` test, a closing decision is never reopened.
- The HUD reads only runtime state: `history_count`, `fold_history`,
  `observed(crack_report)`, `committed`/`reopened`/`latest(rink)`. The program
  declares no numeric state at all, so the save holds only the runtime's graph,
  histories and journal.

## How I tested it

1. `pond.scenarios.json`, run with `npx --no-install caveat test pond.scenarios.json`.
   All 8 scenarios pass. They cover:
   - the initial HUD (`$exact` on `/bindings`, which shows only the six `hud`
     properties exist);
   - open, then reopened by a thin reading, then closed. The scenario checks
     exact grounds, the journal (order, `because`, `caveats`, `value`) and
     that a 10 cm reading does not reopen;
   - a crack reopening, and no second reopening from a later thin reading;
   - three revisions: crack reopen, re-decide open, thin reopen, re-decide
     closed. The crack is absent from `rink@2`'s grounds;
   - a crack before any decision, which reopens nothing;
   - a closing decision that is never reopened;
   - every kind of input refusal (out of range, missing, extra, string, null,
     parameters on `crack`/`decide`, unknown event), each leaving
     `sequence` and state unchanged;
   - the minimum over eight readings, including the oldest one.
   Several scenarios include `resume` steps and `checkpoint`/`same_as`
   comparisons. The runner also checks atomicity, resume agreement and
   grounds ⊆ lineage on every step.
2. `fuzz.mjs`: a randomised differential test against an independent
   JavaScript model of TASK.md. It runs `node fuzz.mjs SEED RUNS`, with an
   optional `THICK=0.7` environment variable to bias toward thick readings.
   Each run sends a random sequence of valid and invalid events. At random
   points it saves and restores the session and keeps every restored session
   running alongside. On every step it checks:
   - the outcome class (accepted / policy / input);
   - that a refusal leaves `save()` and `snapshot()` byte-identical;
   - that every restored session's snapshot equals the original's;
   - that the six HUD values match the model;
   - the full decision journal (commitment, change, value, because, caveats);
   - every commitment's grounds.
   Five seeds and 2,400 runs in total, with over 5,000 restores and
   revision 3 reached, all agree with the model.
3. `explore.mjs` and `inspect.mjs` are small scripts that print the HUD, the
   journal, relations and a save after a chosen event list. I used them for
   manual inspection.

## Things I am unsure of

- **When a crack counts as "new" evidence.** A crack reported before any
  decision reopens nothing. A later decision can then open the rink even though
  a crack was reported. I read "new evidence against safe ice reopens the
  decision" as evidence arriving while the rink is open. `hud.cracked` still
  shows 1.
- **Refusal precedence.** A `decide` with fewer than three readings is refused
  with the "three measurements" message before the "in force" check. Both
  conditions refuse the event, and the task leaves wording to me.
- **Decision capacity 3.** The policy allows at most three revisions: open,
  reopened by the crack; open again, reopened by a thin reading; closed.
  After that no further `decide` is accepted. So the series never overflows,
  and I added no separate capacity refusal. If it did overflow, the runtime
  would report an unclassified error rather than a `reject`, but I believe
  that cannot happen.
- **Lineage beyond grounds.** For `rink@2` and later, `commitment_bases`
  lineage includes the predecessor's basis and its reopening witness, for
  example `crack_report`. That is the runtime's documented behaviour for a
  decision series. The task constrains grounds, and those contain only the
  readings and `single_hole`.
- The code for a string or `null` `cm` is not pinned in my scenarios; I only
  check that the origin is `input`.

## Departures from the rules

- I read and listed nothing outside this directory, used no web access, started
  no other agents and did not use git. I edited my own test files with `sed`
  and a shell heredoc.
- For transparency: the agent harness put some context about the user's own
  project into my session automatically (its instructions and memory index).
  I did not open those files myself, and none of it was used in this work.
