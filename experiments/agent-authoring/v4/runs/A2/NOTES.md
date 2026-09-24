# Notes — run A2

## Documentation read

All from `node_modules/caveat-lang/`:

- `README.md`, `docs/README.md`, `docs/GETTING_STARTED.md`, `docs/reference/docs/AI_AUTHORING.md`
- Specs in `docs/reference/spec/`: `caveat-reactive-0.1.md`, `caveat-reactive-0.5.md` (readings, decision series),
  `caveat-reactive-0.6.md` (`history_count`, `fold_history`), `caveat-reject-0.1.md`, `caveat-define-0.1.md`,
  `caveat-explanations-0.1.md` and `-0.2.md` (grounds vs lineage), `caveat-decision-journal-0.1.md`,
  `caveat-scenarios-0.1.md`, `caveat-dispatch-0.1.md`
- `docs/reference/runtime/prelude.cav` (the `max` function used as a fold reducer)

## What I changed and why

`ferry.cav`, following `CHANGE.md`:

- New evidence `pier_anemometer`, caveat `unverified_pier` (consequence `material`) that qualifies it,
  reading stream `pier_wind` (limit 6), and event `pier_gust kt min 0 max 90`.
- `pier_gust` rules mirror `gust`: a policy `reject` once six pier gusts are recorded; a sample that
  supports `crossing_safe` at 30 kt or less and opposes it above that; and a reopen `because latest(pier_wind)`
  when the gust is above 30 kt and the ferry may go (`may_go` already covers "go and not yet reopened").
- New `define strongest_gust = max(fold_history(wind, -1, max), fold_history(pier_wind, -1, max))`.
  I used `-1` as the fold's starting value instead of `latest(...)`, because either stream can now be empty
  when `decide` runs, and `latest` fails on an empty stream. Gusts are never below 0, so `-1` never wins.
  The decision commits `using strongest_gust`, so its grounds are every gust from both anemometers and
  their caveats. Waves and the warning are only in guards, which don't count as grounds.
- `decide` needs `history_count(wind) + history_count(pier_wind) >= 2` (plus two waves, as before).
- HUD: added `pier_gusts`; `strongest` now uses `strongest_gust`, and is still `-1` before any gust.
  `gusts` still counts mast gusts only.

Did the program meet `TASK.md` before the change? I checked every rule against it (limits, refusals, the
four-decision cap, when a decision reopens and that it reopens only once, a held decision never reopening,
grounds limited to wind readings, the HUD values) and ran the old program with `caveat explain`. I found
no defect, so I made no other fixes.

## How I tested

- `tests/ferry.scenarios.json`, ten scenarios, run with
  `npx --no-install caveat test tests/ferry.scenarios.json`. All 10 pass. They cover:
  the exact HUD property set and initial values; the pier-only, mixed and mast-only two-gust minimum;
  the decision's value and exact `commitment_grounds` (both anemometers, no waves or warning); reopening by a
  pier gust, a mast gust, a wave and the warning, each citing its own evidence and caveat in `decision_journal`;
  no second reopening; a held decision never reopening and blocking `decide`; the four-decision cap (including
  a fourth decision that is reopened); limits of 6 pier gusts, 10 gusts, 10 waves and one warning;
  30 kt / 1.5 m boundaries; refused `pier_gust` payloads (out of range, missing, extra, non-number, unknown
  event); and save/restore (`resume`) partway through. The runner also checks for itself that refused events
  change nothing, that resumed sessions agree, and that grounds stay within lineage.
- `tests/orig.jsonl`: an event list I used with `npx --no-install caveat explain ferry.cav tests/orig.jsonl`
  on the program before and after the change. The two runs gave the same result.

## Things I'm unsure of

- Refusing the 7th pier gust, the 11th gust and the 11th wave is done with a policy `reject`. The runtime would
  refuse those events anyway (`limit/history_limit`); either way they are refused and change nothing.
- The rejection messages are my own wording, which `TASK.md` allows.
- `hud.strongest` and `hud.decision` have wider lineage, visible as "could also have been influenced by" in
  `explain`, e.g. a wave that reopened an earlier decision. The task only constrains the decision's grounds,
  and those are exact.

## Departures from the rules

None that I know of. I used git only to create the session branch `trial/v4-run-A2` (the clone was on
`trial/v4-packet-A2`) and to commit and push. `node_modules/` is not committed (it is in `.gitignore`). `package.json` and
`package-lock.json` from the install step are committed so the install can be repeated. The session's
git check asked for no untracked files to be left.
