# Notes on writing ferry.cav

## Documentation read

All from the installed package, `node_modules/caveat-lang/`:

- `README.md` and `docs/GETTING_STARTED.md` — install, CLI (`caveat test`,
  `caveat explain`), the umbrella example, dispatch outcomes.
- `docs/reference/docs/AI_AUTHORING.md` — the authoring loop, numeric-range
  planning, the session clock, decisions that age.
- `docs/reference/spec/caveat-reactive-0.1.md` through `-0.3.md` and `-0.5.md`,
  `-0.6.md` — state/events/rules, pure functions and bindings, qualified
  numeric values and provenance, reading streams and decision series,
  `history_count`/`history_at`/`fold_history`.
- `docs/reference/spec/caveat-dispatch-0.1.md` — rejection origins/codes
  (`input`, `policy`, `limit`), including the automatic `limit/history_limit`
  refusal for a reading stream or decision series at capacity.
- `docs/reference/spec/caveat-decision-journal-0.1.md` and
  `caveat-explanations-0.1.md` / `-0.2.md` — how `decision_journal` and
  `commitment_grounds` are built, and specifically that `commit ACTION
  because REASON using EXPR` grounds a commitment on the grounds of `EXPR`
  only (never the rule's `when` guard), while `latest`/`history_*` supply
  their full lineage as grounds. This is what let a single `fold_history(wind,
  latest(wind), max)` give both the right value (the strongest gust ever) and
  the right grounds (every `wind` reading, nothing from `swell` or the
  storm warning).
- `docs/reference/spec/caveat-reject-0.1.md` — the `reject "message"` effect
  used for the policy-level refusals (decide too early, decide while a
  decision is in force, a repeated storm warning).
- `docs/reference/spec/caveat-save-0.1.md` and `caveat-view-0.1.md` — save
  and restore are automatic; nothing extra was needed in the source for the
  "save and restore must preserve everything" requirement.
- `docs/reference/examples/thermostat_history.cav` and its scenarios file —
  the closest existing example (a reading stream driving a repeatedly
  revised, reopened decision); its `when V >= T sample ... supports` /
  `when V < T sample ... opposes` pattern is exactly the shape used here for
  gusts and waves.

I did not need `caveat-reactive-0.4.md`, `-0.7.md`, `caveat-define-0.1.md`,
`caveat-repetition-0.1.md`, `caveat-procedure-symbols-0.1.md`,
`caveat-typed-parameters-0.1.md`, `caveat-late-qualification-0.1.md`,
`caveat-renewal-0.1.md`, `caveat-state-caveats-0.1.md`,
`caveat-observation-order-0.1.md`, or `caveat-elapsed-0.1.md`: the program
uses no entities, no procedures, no time/clock, and no caveats learned after
the fact (both `qualifies` relations are declared once, at load time, exactly
like the shipped examples).

## How I tested it

- `npx --no-install caveat explain ferry.cav [events.jsonl]` against several
  hand-written event sequences, to read the decision, evidence and `hud.*`
  state after each step and check grounds/journal by eye. (These scratch
  `.jsonl` files were deleted once I was done with them; they are not part
  of the commit.)
- The kept test is `ferry.scenarios.json`, run with
  `npx --no-install caveat test ferry.scenarios.json`; all three scenarios
  pass:
  - **F01** — two gusts and two waves let the ferry go; a stronger gust
    reopens the decision (`review`), and a second `decide` commits a new,
    frozen `hold` whose grounds now include the third gust; further
    (non-reopening) evidence after a hold changes nothing, and `decide` is
    then refused as "still in force". Uses `checkpoint`/`resume`/`same_as`
    to check the first commitment's basis and grounds are untouched by the
    reopening, and that resuming a saved session behaves identically.
  - **F02** — every kind of automatic input refusal (unknown event, missing/
    extra/out-of-range parameter for `gust` and `wave`), the two `decide`
    policy refusals, filling both reading streams to their limit of 10 and
    getting the eleventh `gust`/`wave` refused as `limit/history_limit`, and
    a repeated `warning` refused by the program's own `reject`. Checks that
    `/sequence` and `/decision_journal` are untouched by the initial run of
    refusals.
  - **F03** — a decision reopened and re-committed four times exhausts the
    series' capacity of 4; a fifth `decide` (after a fifth reopening) is
    refused as `limit/history_limit`, matching "refused once four decisions
    have been made".
  - The scenario runner itself additionally checks, for every step: that a
    rejected event changed nothing, that the `resume` step's session agrees
    with the one it was saved from, and that every decision's grounds stay
    within its lineage.
  - I also checked, with `explain` only (not committed as scenarios), the
    exact boundary values: `kt = 30` supports and `kt = 30.0001` opposes;
    `m = 1.5` supports and `m = 1.50001` opposes; and that `decide`/`warning`
    reject a payload with an unexpected field.

## Things I'm unsure of

- **`consequence material` / `consequence low`.** The shipped docs and
  example only use `material` and `high`; I never found a written enum of
  legal consequence labels. I used `material` for `sheltered_mast` and `low`
  for `buoy_drift` and `regional_forecast` exactly as the task names them,
  and the program loads and runs, so the runtime accepts `low` as well — but
  I don't have a spec citation for the full set of valid levels.
- **Whether `hud.strongest`/`hud.highest` should track the true maximum.**
  My first draft bound them to `latest(wind)`/`latest(swell)` (the most
  recent reading), which is wrong: the task asks for "the strongest gust so
  far" / "the highest wave so far", i.e. a running maximum, independent of
  what the current frozen decision used. I caught this by testing a gust
  sequence with a spike followed by a calmer reading and noticing
  `hud.strongest` dropped; the fix was the same `fold_history(..., max)`
  pattern already used for the decision's own value. I'm fairly confident in
  the corrected version (it's tested in F01/F03), but flagging the reasoning
  since the task wording is close to, but distinct from, the decision's
  "strongest gust recorded so far" phrase.
- I chose "policy" wording for `reject` messages myself, since the task
  leaves that up to the author.

## Departures from the rules

None that I'm aware of: I worked only inside this directory, read only the
package's own shipped docs, ran only the local `caveat` CLI, and used no
network access or other agents/sessions.
