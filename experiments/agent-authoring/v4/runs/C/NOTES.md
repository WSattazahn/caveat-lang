# Notes on writing `ferry.cav`

## Documentation read

Everything under `node_modules/caveat-lang/`, in this order:

- `README.md` and `docs/GETTING_STARTED.md` — install, the `caveat` CLI
  (`test`, `explain`), the dispatch outcome contract, session API.
- `docs/reference/spec/caveat-reactive-0.1.md` through `-0.7.md`, in order —
  state/events/rules, pure functions/bindings/cues, qualified computation,
  the source standard library and text, reading streams and decision series
  (0.5, the profile this program leans on most), history queries (`history_count`,
  `history_at`, `fold_history`), and procedures (not used here).
- `docs/reference/spec/caveat-decision-journal-0.1.md` — the journal's shape
  and its append-only/idempotent guarantees.
- `docs/reference/spec/caveat-state-caveats-0.1.md` and
  `caveat-late-qualification-0.1.md` — read for background on reopening by
  caveat and `qualify`; neither feature is used in `ferry.cav`, since every
  caveat here is a static `qualifies` declaration on the evidence template
  (`sheltered_mast`, `buoy_drift`, `regional_forecast`), not something learned
  after the fact.
- `docs/reference/spec/caveat-dispatch-0.1.md`, `caveat-save-0.1.md`,
  `caveat-view-0.1.md` — the accepted/rejected/fatal contract, what a save
  holds and how restore is validated, and the compact per-event view.
- `docs/reference/spec/caveat-explanations-0.1.md` and `-0.2.md` — lineage
  vs. grounds. This is what let me be sure that `commit crossing because
  enough using fold_history(wind, -1, max);`, with no explicit `retaining`
  or `because` citation, grounds the decision on exactly the wind readings
  and `sheltered_mast`, and nothing from `swell` or `storm_warning`: grounds
  of a `history_*` read are "as lineage" (Explanations 0.2's table), and a
  fold's lineage is the union of every record it actually traversed, which
  here is only the `wind` stream.
- `docs/reference/spec/caveat-observation-order-0.1.md` — checked that
  `reopen crossing because storm_warning;`, which cites a plain evidence
  name rather than `latest(stream)`, is safe because the `reveal` that
  observes `storm_warning` runs as an earlier rule of the same `warning`
  event (source order matters here).
- `docs/reference/spec/caveat-reject-0.1.md`, `caveat-text-0.1.md` —
  `reject "…";` syntax, and comment/quoting rules.
- `docs/reference/spec/caveat-scenarios-0.1.md` — the scenario file format
  used for `ferry.scenarios.json`: `send`/`expect`/`checkpoint`/`same_as`/
  `resume`, matchers (`$includes` etc.), and what the runner checks
  automatically on every step (atomicity of rejections, resume agreement,
  a final save/restore, grounds-within-lineage).
- `docs/reference/docs/AI_AUTHORING.md` — the authoring loop and the
  worked example under "Reconsider a decision when its evidence ages".
- `docs/reference/examples/thermostat_history.cav` and its
  `.scenarios.json` — the closest worked example to this task (one
  `readings` stream, one `decisions` series, `fold_history` for a summary,
  `commit … because enough using …`, a `proc`-free version of the same
  read/reopen/commit shape). `ferry.cav` follows its style directly instead
  of the procedure-based Light the Way / Trail Rescue games, since the task
  needs no `proc` (each event has its own small, fixed set of rules).
- `docs/reference/spec/caveat-0.1.md` — background on the six primitives,
  read for vocabulary (claim/evidence/caveat/commitment/reopening), not used
  for syntax.

Not read in full, since nothing in the task needed them: `caveat-renewal-0.1.md`,
`caveat-repetition-0.1.md`, `caveat-define-0.1.md`, `caveat-typed-parameters-0.1.md`,
`caveat-procedure-symbols-0.1.md`, `caveat-elapsed-0.1.md` (no clock event in
this program), `CAVEAT_ESSENCE.md`.

## How I tested

- `npx --no-install caveat explain ferry.cav <events.jsonl>` while writing the
  program, to read `qualified_values`, `commitment_bases`/`commitment_grounds`,
  `reading_streams`, `decision_series` and the decision journal after small
  event sequences, and to check the exact refusal wording and origin/code.
- `ferry.scenarios.json` (kept in this directory) is the real test suite, run
  with `npx --no-install caveat test ferry.scenarios.json`. All 8 scenarios
  pass:
  - **F01** — two calm gusts and waves; `decide` accepts and grounds the
    decision on exactly the two `wind` readings and `sheltered_mast`, not the
    waves.
  - **F02** — a gust over 30kt reopens a `go` decision; checks the reopened
    decision's frozen basis and grounds are unchanged (`same_as` against a
    checkpoint), then a `resume`, then a second `decide` producing `crossing@2`
    grounded on all three gusts.
  - **F03** — once a decision holds (value over 30kt), a later strong gust,
    high wave and storm warning do none of them reopen it — it stays `hold`
    for the rest of the session, per the task's explicit requirement.
  - **F04** — a wave over 1.5m and a storm warning each independently reopen
    a `go` decision, each citing only its own evidence in the journal; a
    second `warning` is refused.
  - **F05** — `decide` refused for too few readings and while a decision is
    in force; out-of-range/unknown/malformed events refused by the core;
    nothing changes (`/sequence` stays `0`, journal stays empty).
  - **F06** — an 11th gust and 11th wave are each refused
    (`limit/history_limit`) once ten of that kind are recorded, counted
    separately per stream.
  - **F07** — exactly 30kt and exactly 1.5m count as safe (inclusive
    thresholds), checked both via the HUD and via the raw `supports` relation.
  - **F08** — a `go` decision can be reopened and re-committed up to the
    four-decision cap (using repeated wave-triggered reopenings that don't
    raise the recorded strongest gust, since a decision that has ever seen a
    gust over 30kt can never return to `go`); the fifth `decide` at the cap is
    refused `limit/history_limit`.
- Besides what the scenario file states, the `caveat test` runner itself
  checks on every step, without being asked: that every rejected event left
  the save/view/snapshot unchanged, that each `resume` restores to an
  identical session that then agrees with the original on every later event,
  that a final save/restore after the whole scenario reproduces the snapshot,
  and that grounds stay within lineage. That covers the task's save/restore
  requirement more thoroughly than I could by hand.
- A few one-off `caveat explain` probes not kept as scenarios: repeated
  adverse evidence after a decision is already reopened adds no further
  reopening entry (checked the journal has exactly one `reopened` entry
  despite a second wave, a gust, and a warning all arriving afterward); the
  four-decision-cap sequence, read in full to confirm `commitment_grounds`
  for each revision only ever lists `wind` occurrences.
- Two throwaway Node scripts (`probe2.mjs`, `probe3.mjs`) inspected the raw
  `commitments` and `relations` snapshot shapes so the scenario file's
  `$includes`/partial-object assertions matched the actual field names
  (`from`/`relation`/`to`/`origin`, and `commitments` as an array keyed by
  `action`, not an object keyed by decision name). Both were deleted after use
  and are not part of the committed test set.

## What I'm unsure of

- **`hud.warned` type.** The task table gives `1`/`0` as the values, which I
  read as the numeric state `warned` (0 or 1) bound directly, rather than a
  boolean `true`/`false`. If a boolean was intended instead, that's a
  one-line change (`bind hud.warned = warned == 1;`), but the literal `1`/`0`
  in the spec table reads to me as numbers.
- **Explicit `history_limit` guards.** I rely on the runtime's own
  `limit/history_limit` refusal for the 11th gust/wave and the 5th decision at
  capacity, rather than writing my own `reject` checks ahead of them (e.g.
  `when history_count(wind) >= 10 reject …`). The task never says what
  refusal *origin* is expected, only that the event must be refused and
  change nothing, which the built-in limit already guarantees atomically.
  I preferred this over duplicating the capacity logic, but a study reader
  expecting a policy-level `reject` message for these specific cases would
  see `limit/history_limit` with the runtime's own diagnostic text instead.
- **Whether "the strongest gust recorded so far" can ever fall back below
  30kt after being reopened.** I convinced myself (and F08 exercises) that
  because the decision's value is `fold_history(wind, -1, max)` — a running
  maximum over *all* gusts ever recorded — a decision can only move from
  `go` to `hold`, never back, and a `hold` decision, once made, is therefore
  never reopened by a gust (matching "a decision that held the ferry is
  never reopened"). I did not find anything in the task text that says this
  monotonicity is intended as opposed to incidental, but it falls directly
  out of "the maximum over every accepted gust, not just recent ones" and I
  did not see a way to read the spec that avoids it.

## Departures from the rules

None that I'm aware of. I worked only inside this directory, read only
files under it (including the installed package), did not use the web or
contact any other agent/session, and used `git` only to commit and push this
work to the session's branch.
