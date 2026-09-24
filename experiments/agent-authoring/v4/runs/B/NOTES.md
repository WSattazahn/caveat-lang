# Notes: ferry.cav (run B)

## Documentation read

All from `node_modules/caveat-lang/` after installing the tarball:

- `README.md`, `docs/README.md`, `docs/GETTING_STARTED.md`
- `docs/reference/docs/AI_AUTHORING.md`
- `docs/reference/examples/thermostat_history.cav`
- `docs/reference/runtime/prelude.cav`
- Specs in `docs/reference/spec/`: `caveat-reactive-0.1`, `-0.2`, `-0.5`, `-0.6`,
  `caveat-decision-journal-0.1`, `caveat-reject-0.1`, `caveat-define-0.1`,
  `caveat-explanations-0.1`, `caveat-explanations-0.2`, `caveat-dispatch-0.1`,
  `caveat-save-0.1`, `caveat-scenarios-0.1` (first ~200 lines) and the start of
  `caveat-state-caveats-0.1`.

## Design

- `wind` (from `anemometer`, qualified by `sheltered_mast`, material) and
  `swell` (from `buoy`, qualified by `buoy_drift`, low) are reading streams
  with limit 10; `crossing` is a decision series with limit 4.
- Explicit `reject` rules refuse the 11th gust/wave, a second `warning`, and
  `decide` when four decisions exist, a decision is in force (committed and
  not reopened), or there are fewer than two gusts or two waves. Range, missing,
  extra and unknown-event checks come from the event declarations.
- `decide` commits `using fold_history(wind, 0, highest_of)`, so the frozen
  value is the maximum gust, and its grounds are every `wind@N` plus
  `sheltered_mast` (checked in the tests; waves and the warning are absent).
- A `define may_go` (committed, not reopened, value <= 30) guards the three
  reopen rules: `because latest(wind)`, `because latest(swell)`, and
  `because storm_warning`. The warning is revealed *before* the reopen rule,
  since reopening needs observed evidence (my first draft had the order the
  other way round and the runtime refused it as fatal).
- HUD `strongest`/`highest` use `fold_history(..., -1, highest_of)`, which
  gives -1 for an empty history without a guard.

## Testing

- `tests/ferry.scenarios.json` (9 scenarios): run with
  `npx --no-install caveat test tests/ferry.scenarios.json`. All pass. They
  cover the initial HUD, boundaries (30 kt, 1.5 m go; 30.5 kt, 1.6 m reopen),
  exact `commitment_grounds` and full `decision_journal` entries, reopening by
  gust, wave and warning, no second reopening, hold never reopening, the
  four-decision cap, ten-reading caps counted separately, input refusals
  (out of range, missing, extra, wrong type, null, unknown event), and several
  `resume` steps (the runner also checks refusals change nothing and that
  resumed sessions agree).
- `tests/fuzz.mjs`: a randomized check against a plain JavaScript model of
  TASK.md, comparing every outcome, the whole `hud` and the journal
  (change + because), checking refused events leave the save unchanged, and
  saving/restoring at random points. `node tests/fuzz.mjs SEED RUNS`; seeds 1,
  42 and 7 with 300 runs each all agreed.
- `tests/sample.events.jsonl`: a sample event list for
  `npx --no-install caveat explain ferry.cav tests/sample.events.jsonl`.

## Unsure

- Non-finite numbers (NaN, Infinity) cannot be expressed in JSON; the session
  library refuses them with a `payload` error before the runtime, so I could
  not test that path through the program itself.
- The ten-reading and four-decision caps use my own `reject` rules (policy
  refusals). The runtime would also refuse them as `limit/history_limit`; I
  assumed either counts as "refused".
- Refusal wording is my own, as TASK.md allows.

## Departures from the rules

- One command I ran contained a stray `cat > /tmp/claude-0/x.jsonl`, which
  created an empty file outside the working directory (and hung waiting for
  input until I stopped it). I deleted that file immediately. Nothing was read
  from outside the working directory.
- Otherwise none: no web, no other agents, git used only to commit and push.
  I created and pushed the branch `trial/v4-run-B` as the session instructed.
