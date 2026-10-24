# Outcomes sweep and prototypes

Evidence for [Outcomes 0.1](../../spec/caveat-outcomes-0.1.md), the 1.3
draft that records how a decision turned out on the revision it judges. The
spec decides; this folder keeps what it was drawn from.

## The design note

A design note of 2026-10-13 merged two earlier findings: forecast tracking
(experiment C of a review of Wissner-Gross and Freer, Physical Review
Letters 110, 168702, 2013) and idea 2 of a decision-science pass (Annie
Duke's warning against "resulting", judging a decision by how it turned
out). The owner approved it on 2026-10-13 as post-1.0 work, with foreclosure;
foreclosure shipped in 1.2.0.

## The prototypes

Both ran on `caveat-lang@0.1.0-rc.19`. They are kept here as text, not as
`.cav` files, so that the corpus the sweep reads stays the repository's own
programs.

**Outcome by convention.** A pilot's premortem named thin demand. The
outcome is a reading of its own, `outcomes@1`, which the host pairs with
`pilot@1`. To say the premortem's risk came true, the program qualifies the
plan's evidence with a second caveat and reopens citing it. Its one scenario
passed.

```caveat
claim pilot_working;
evidence plan_review from "planning review";
evidence usage_feed from "weekly active users";
caveat thin_demand consequence material;
caveat thin_demand_happened consequence high;
thin_demand qualifies plan_review;

readings usage from usage_feed limit 52;
readings outcomes from usage_feed limit 8;
decisions pilot limit 4;

state basis = 0 min 0 max 10;

event plan score min 0 max 10;
event weekly users min 0 max 100000;
event close users min 0 max 100000;

on plan reveal plan_review supports pilot_working;
on plan set basis = qualified(score, plan_review);
on plan commit pilot because enough using basis;
on weekly when users >= 20 sample usage = users supports pilot_working;
on weekly when users < 20 sample usage = users opposes pilot_working;
on weekly when users < 20 qualify plan_review with thin_demand_happened;
on weekly when users < 20 and committed(pilot) and not reopened(pilot)
    reopen pilot because caveated(basis, thin_demand_happened);
on close sample outcomes = users supports pilot_working;
```

It showed three gaps: the reopening cites `plan_review`, not the reading
that triggered it; the outcome reading had to claim a stance; and nothing
pairs an outcome with its revision.

**Forecast tracking.** A release agent forecasts each canary; each resolved
forecast gets a Brier score, and the record sets the next bar. Its four
scenarios passed. It paired forecasts with outcomes by a `pending` state
that refused a second forecast before the first resolved, and scored at
resolve time, because `fold_history` folds one history.

Neutral `sample` (1.1.0) has since removed the stance gap. The pairing gap
is what Outcomes 0.1 closes.

## Questions settled

1. **Late outcomes.** An outcome may attach to a superseded revision, by
   index: `resolve revision(pilot, 1) = …`. Real outcomes often arrive after
   the next decision.
2. **Numbers or named results.** Numbers only, as readings and `using`
   values are.
3. **A declaration clause.** The note sketched `decisions pilot limit 4
   resolved by usage`. The spec leaves it out: the effect's `because`
   already names the evidence, and a clause would add a word to every
   declaration form for no new meaning.
4. **Correcting an outcome.** By withdrawal, not overwrite, as for any
   evidence. A revision whose outcome rests on withdrawn evidence may be
   resolved again; the journal keeps both entries.
5. **A fold over outcomes.** Left out. A program scores each outcome as it
   arrives and folds the scores with `fold_history`.

## The sweep

`node experiments/outcomes-sweep/sweep.mjs --write` regenerates
[RESULTS.json](RESULTS.json). The spec's "Corpus sweep" section reads it.
