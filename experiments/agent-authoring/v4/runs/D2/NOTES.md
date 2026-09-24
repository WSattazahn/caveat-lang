# Notes on the pier-anemometer change

## Documentation read

Everything under `node_modules/caveat-lang/`, in this order:

- `README.md` (package root) — the `caveat` CLI, `test` and `explain`,
  dispatch outcomes, and the session API.
- `docs/README.md` — the documentation index.
- `docs/GETTING_STARTED.md` — walked the umbrella example end to end.
- `docs/reference/spec/caveat-reactive-0.1.md` — state, events, rules,
  claims/evidence/caveats, the graph predicates (`observed`, `committed`,
  `reopened`), atomicity.
- `docs/reference/spec/caveat-reactive-0.5.md` — `readings`/`decisions`
  declarations, `sample`, `latest`, `has_sample`, decision-series revisions,
  and the `limit`/`history_limit` refusal on a full history.
- `docs/reference/spec/caveat-reactive-0.6.md` — `history_count`,
  `history_at`, `fold_history` and the reducer rules (a pure two-argument
  source function; an empty fold returns its initial value without invoking
  the reducer).
- `docs/reference/spec/caveat-explanations-0.1.md` and `-0.2.md` — the
  lineage/grounds distinction, and specifically that `commit ACTION because
  REASON using EXPR` grounds the commitment on `EXPR`'s grounds, and that
  `fold_history`/`latest` pass through their full traversed lineage as
  grounds.
- `docs/reference/spec/caveat-decision-journal-0.1.md` — `decision_journal`
  entries and evidence ordering (first-observed order, not declaration
  order).
- `docs/reference/spec/caveat-reject-0.1.md` — `reject` semantics.
- `docs/reference/spec/caveat-save-0.1.md` — what a save preserves and what
  makes a restore refused.
- `docs/reference/spec/caveat-dispatch-0.1.md` — rejection `origin`/`code`
  catalog (used to write precise scenario assertions, e.g. `limit` /
  `history_limit` for a full reading stream, not `policy`).
- `docs/reference/spec/caveat-scenarios-0.1.md` (`$set` matcher, only the
  parts needed to fix an ordering assumption in my own test).
- `docs/reference/runtime/prelude.cav` — confirmed `max(a, b)` is an
  ordinary pure source function (`fn max(left, right) = if(left > right,
  left, right);`), so it is legal both as an inline call and as a
  `fold_history` reducer.

I did not read the background specs (`caveat-0.1.md`, `CAVEAT_ESSENCE.md`)
or the profiles/features not exercised by this program (0.2–0.4, 0.7,
`define`, `repetition`, procedure symbols, typed parameters, text, late
qualification, renewal, state caveats, observation order, `elapsed`,
`view`) — nothing in `TASK.md`/`CHANGE.md` needed them, and `ferry.cav`
does not use them.

## What I found before making the change

Before touching anything, I checked the existing `ferry.cav` against
`TASK.md` (the rule to "fix that too" if it didn't already meet the
original task). It matched: gusts/waves sampled with the right thresholds,
capacities enforced by the runtime's own `history_limit`, the storm warning
declared and guarded against a second broadcast, `decide`'s two guards
(`< 2` readings, decision still in force) present, the four-decision cap
relying on the runtime's own `history_limit` on `decisions crossing limit
4` (no extra rule needed), the commit's value and grounds computed with
`fold_history(wind, latest(wind), max)`, reopening rules for gust/wave/
warning all guarded on `latest(crossing) <= 30` (so a `hold` never
reopens) and `not reopened(crossing)` (so a reopened decision doesn't
reopen again), and the `hud` bindings matching the display table exactly.
I found no pre-existing bug, so the diff below is only the change request.

## What I changed, and why

Added, per `CHANGE.md`:

- `evidence pier_anemometer from "the anemometer at the end of the pier";`
  and `caveat unverified_pier consequence material;` qualifying it.
- `readings pier_wind from pier_anemometer limit 6;` and
  `event pier_gust kt min 0 max 90;`.
- Two `sample`/`supports`/`opposes` rules for `pier_gust`, mirroring the
  mast ones, and a reopening rule `reopen crossing because latest(pier_wind)`
  guarded exactly like the mast one (`committed(crossing) and not
  reopened(crossing) and latest(crossing) <= 30 and kt > 30`).
- `decide`'s "at least two gusts" guard now counts
  `history_count(wind) + history_count(pier_wind)`, so two pier gusts alone
  (with two waves) are enough.
- `hud.pier_gusts = history_count(pier_wind)`. `hud.gusts` is unchanged
  (mast only, as the change request says).

The one change that needed more than copy-and-adjust: the decision's value
and `hud.strongest` used `fold_history(wind, latest(wind), max)`. That
initial-value expression, `latest(wind)`, throws if `wind` is empty —
harmless before, because `decide`'s old guard already required
`history_count(wind) >= 2`. After the change, `decide` can fire with, say,
two pier gusts and zero mast gusts, so `wind` can be empty while `decide`
still runs, and likewise `pier_wind` can be empty while a mast-only
decision runs. I replaced the fold's initial value with the literal `-1`
(below any real gust, which is `>= 0`) on both streams:

```
max(fold_history(wind, -1, max), fold_history(pier_wind, -1, max))
```

An empty fold returns its initial value without calling the reducer or
touching `latest` (Reactive 0.6), so this is safe when either stream (but
not both, since `decide` still requires two gusts total) is empty, and it
folds the *entire* history of both streams (not just recent readings), per
`TASK.md`'s "the maximum over every accepted gust, not just recent ones."
Grounds: `fold_history`/`latest` pass through their full traversed lineage
as grounds (Explanations 0.2), and a plain numeric literal carries no
provenance, so the commit's grounds are exactly the union of every reached
`wind` and `pier_wind` reading (with `sheltered_mast`/`unverified_pier`)
and nothing from `swell` or `storm_warning` — verified with `caveat
explain` (see below).

Everything CHANGE.md doesn't mention — wave/storm-warning handling, the
`hold` decision never reopening, the reopened-decision "stays reopened
until the next `decide`" rule, save/restore — is untouched.

## How I tested it

- `npx --no-install caveat test ferry.scenarios.json`: 10 scenarios I
  wrote, covering: the initial HUD; refusal of an unknown event, a
  missing/extra parameter, a non-finite payload, and an out-of-range
  payload, each leaving state unchanged; mast and pier gusts counted
  separately in `gusts`/`pier_gusts` but combined for `strongest` and for
  `decide`'s two-gust gate; a decision reached from pier gusts alone, with
  `commitment_grounds` containing only the pier evidence and
  `unverified_pier`; a pier gust reopening a `go` decision; a `hold`
  decision never reopening even after a strong pier gust, a wave and a
  warning; the reading-stream capacity refusal at the 7th pier gust and the
  11th mast gust (`limit`/`history_limit`); the storm warning refusing a
  second broadcast; save/restore (`resume`) preserving the HUD and
  behaving identically afterwards, checked with `same_as`; and `decide`
  refused with `limit`/`history_limit` once a fifth commit is attempted
  after four decisions were made (each of the first four reopened by a
  wave so the chain of commits could continue).
- `npx --no-install caveat explain ferry.cav events.jsonl` on a few
  hand-picked event sequences (mixed mast/pier gusts, pier-only readings)
  to read the actual `commitment_grounds`, `decision_journal` and
  `binding_explanations` and confirm by eye that only wind/pier_wind
  evidence and their own caveats ground the decision, never swell or
  storm_warning.
- Ran the full scenario file after each edit; all 10 scenarios pass.

## Things I'm unsure of

- `commitment_grounds`' evidence array order is not documented (only
  `decision_journal`'s `because` is documented as first-observed order). I
  initially wrote a test asserting a specific order in
  `commitment_grounds` and it failed; I switched that assertion to
  `$set` (order-independent) and kept an order-sensitive assertion only on
  `decision_journal`, which is documented. I did not find a normative
  statement about `commitment_grounds` ordering either way.
- I used a `-1` sentinel for the fold's initial value on both gust streams.
  Gust `kt` is declared `0..90`, so `-1` can never collide with a real
  reading, but the safety of the sentinel technique depends on the
  declared event range not including negative numbers — worth a comment
  if this pattern is reused elsewhere with a range that includes negatives.

## Departures from the rules

None. All reading, editing and command execution stayed inside this
working directory; the `caveat` CLI was the only "other software" run; no
web access or other sessions were used.
