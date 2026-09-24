# Notes — run A

## Documentation read

All inside `node_modules/caveat-lang/` after installing the tarball:

- `README.md`, `docs/README.md`, `docs/GETTING_STARTED.md`
- `docs/reference/docs/AI_AUTHORING.md`
- Specs, read in full: `caveat-reactive-0.1` through `0.7` (0.4 and 0.7 read
  more quickly), `caveat-reject-0.1`, `caveat-explanations-0.1` and `0.2`,
  `caveat-decision-journal-0.1`, `caveat-define-0.1`, `caveat-scenarios-0.1`,
  `caveat-dispatch-0.1`, `caveat-save-0.1`, `caveat-text-0.1`,
  `caveat-observation-order-0.1`, `caveat-view-0.1`, and the first part of
  `caveat-state-caveats-0.1`
- `docs/reference/runtime/prelude.cav` and
  `docs/reference/examples/thermostat_history.cav`
- For the rest (late qualification, renewal, elapsed, typed parameters,
  repetition, procedure symbols, caveat-0.1, CAVEAT_ESSENCE) I read only the
  headings. The program doesn't use those features.

## Design

- `wind` / `swell` are reading streams (limit 10) from `anemometer` / `buoy`.
  The caveats `sheltered_mast` and `buoy_drift` qualify those templates, so
  every occurrence carries its caveat. `storm_warning` is revealed once and is
  qualified by `regional_forecast`.
- `crossing` is a decision series with limit 4. `decide` commits
  `fold_history(wind, latest(wind), max)`. Its grounds are exactly every
  `wind@N` plus `sheltered_mast`, because none of the sampling guards read
  anything with provenance (they compare only the event parameter).
- Every refusal is a `reject` placed before the effects: the 11th gust or wave,
  a second warning, and a `decide` that comes too early, while a decision is in
  force, or after four decisions. Malformed input is refused by the runtime
  itself (`input/...`).
- A reopening happens only when `committed and not reopened and latest(crossing) <= 30`,
  so a hold is never reopened and a reopened decision gets no second reopening.
- The HUD uses `bind` defaults with `when` overrides. Only the eight required
  properties are bound.

## Testing

- `tests/explore.mjs` is a script that dispatches a sequence and prints the
  HUD, grounds and journal (`node tests/explore.mjs`).
- `tests/ferry.scenarios.json` has 8 scenarios covering: the initial display;
  unknown events, missing, extra, non-numeric and out-of-range parameters; the
  boundary values 30 kt and 1.5 m (which support) against 30.5 kt and 1.51/1.6 m
  (which oppose); the eleventh gust and wave; the preconditions on `decide`;
  reopening by a gust, a wave or the warning, with exact journal `because` and
  `caveats`; no second reopening; a hold that is never reopened; exact
  `commitment_grounds`; unchanged earlier records (`same_as`); the four-decision
  limit; and `resume` (save and restore) in the middle of most scenarios. The
  runner also checks atomicity of every refusal, that resumed sessions agree,
  and that grounds stay within lineage.
  Run: `npx --no-install caveat test tests/ferry.scenarios.json`. All 8 pass.

## Unsure

- For the eleventh gust or wave and the fifth decision, I refuse with a policy
  `reject`. The declared history limits would also refuse, as
  `limit/history_limit`. The task leaves the form of refusal messages open, so
  I assume a policy rejection is acceptable.
- I read "strongest"/"highest" as the maximum over all accepted readings. They
  are numbers, not text. `decision` is text.
- `warning` before any decision records the evidence but reopens nothing, since
  no decision is in force.

## Departures from the rules

None that I know of. I read only files in the working directory and the
installed package, used no web access, and started no other agents. I used git
only to commit and push. `npm init -y` created `package.json` /
`package-lock.json`; they are committed so the tests can be reinstalled, and
`.gitignore` keeps `node_modules/` out of the repository.
