# CAVEAT Review 0.1 (draft) — which evidence a decision hinged on

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.3, beside [Outcomes 0.1](caveat-outcomes-0.1.md). It is a report of the kit, as [Dependents 0.1](caveat-dependents-0.1.md) is: it adds no syntax, changes no program and no session, and a program means what it meant before.

[Outcomes 0.1](caveat-outcomes-0.1.md) records how a decision turned out
next to the basis it was made on. That tells a review what the decision
rested on and what happened. It does not say which of those grounds the
decision actually turned on: a revision based on four pieces of evidence may
have needed only one of them, or two together, while the rest only moved its
value.

A review answers that by replaying. Caveat sessions are deterministic: the
same source and the same events make the same session
([A3](../docs/RESTORE_TRUST_BOUNDARY.md)). So "would this have been decided
without that evidence?" has an exact answer: send the events again, leaving
out the event that observed it, and see whether the event that decided still
commits. The review does this for each part of the grounds and, when there
are few enough, for every combination.

It does not give blame scores or weights. Caveat's decisions are rules over
evidence, not weighted sums, so any percentage would be a number the review
made up. It sorts the grounds into kinds instead, and says how each replay
failed.

## Example

An agent chooses a vendor when support reaches 3: two references count 1,
the vendor's own uptime report counts 2 and carries `self_reported`, a short
trial counts 1 and carries `short_trial`. An analyst note is observed and
read by no rule. Later the vendor fails, and the outcome names the risk that
came true:

```caveat
claim vendor_reliable;
evidence references from "two customer references";
evidence uptime_report from "vendor's own uptime report";
evidence trial from "two-week trial";
evidence analyst from "analyst note";
evidence incident from "incident report";
caveat self_reported consequence material;
caveat short_trial consequence low;
self_reported qualifies uptime_report;
short_trial qualifies trial;
decisions choose limit 4;
state support = 0 min 0 max 10;
event refs; event uptime; event run_trial; event analyst_note; event decide; event failed;
on refs reveal references supports vendor_reliable;
on refs set support = support + qualified(1, references);
on uptime reveal uptime_report supports vendor_reliable;
on uptime set support = support + qualified(2, uptime_report);
on run_trial reveal trial supports vendor_reliable;
on run_trial set support = support + qualified(1, trial);
on analyst_note reveal analyst supports vendor_reliable;
on decide when support >= 3 commit choose because enough using support
  retaining self_reported, short_trial;
on decide when support < 3 reject "Not enough support to choose.";
on failed reveal incident opposes vendor_reliable;
on failed resolve choose = 0 because incident materialized self_reported;
```

After `refs`, `uptime`, `run_trial`, `analyst_note`, `decide` and `failed`,
`caveat review vendor.cav choose@1 events.jsonl` prints:

```text
vendor.cav: review of choose@1 = 4  in force
  decided at #5 decide
  based on references, trial, uptime_report (caveats: self_reported, short_trial)
  resolved at #6 failed = 0 because incident; materialized: self_reported

Decisive (without these, it is not decided)
  uptime_report (caveats: self_reported): decide is refused (policy): Not enough support to choose.
  references + trial (caveats: short_trial): decide is refused (policy): Not enough support to choose.

Value only (without these, it is decided with another value)
  references: choose@1 would be 3
  trial: choose@1 would be 3

Not decisive: nothing
Observed before the decision, outside its grounds: analyst
Came true: self_reported (carried by uptime_report, decisive); the revision is still in force

4 replays searched every combination of 3 observing events.
```

Read as a review: the choice hinged on the one piece of evidence carrying a
material caveat the agent chose to keep, and that caveat is the one that came
true. The references and the trial mattered only together.

## 1. Command and function

```text
caveat review [--runtime <dir>] [--json] <program.cav> <revision> <events.jsonl>
```

In the kit, `review(open, events, revision, { limit })` in
`kit/lib/review.mjs` makes the report; it is not a package export while this
profile is a draft. `open()` returns a fresh session of the program on each
call; `events` are `{ event, payload }` in order; `revision` names a revision
of a decision series, such as `choose@1`. `formatReview(report, title)` gives
the text above. A name that is not a revision of a declared series throws,
and the command exits 2.

## 2. Meaning

The review sends all the events to a fresh session, as `explain` does, and
reads the revision from that session. The **deciding event** is the event
whose journal entry `committed` made the revision.

The revision's grounds evidence is grouped by the accepted event whose
`reveal` or `sample` first observed it, the same event `explain` reports as
`observed_by`. Each group is a **unit**: leaving out the event leaves out
everything it did, so grounds observed by one event can only be judged
together. Grounds observed by the deciding event itself are **entangled**
and are not dropped, since that would drop the decision. Grounds no event
observed are listed as **unobserved**.

For each set of units the review replays every event except the units'
events, in their original order, in a fresh session, and asks whether the
deciding event is accepted and commits a revision of the same series:

- If it does not, the set is **decisive**. The report says why, from the
  deciding event's own outcome in the replay: refused, with its origin, code
  and message (an authored `reject`, or a refusal such as
  `evaluation/unobserved_evidence` when the program cannot decide without
  that evidence); accepted with no rule committing; or failed.
- If it commits another value, the set is **value only**.
- Otherwise the set changes nothing.

Only smallest sets are reported. A set that contains a decisive set is not
replayed. A value-only set is reported when it contains no smaller value-only
or decisive set. Evidence in no reported set is **not decisive**.

With at most `limit` units (8 by default), every combination is searched,
smallest first: at most 255 replays, fewer when decisive sets are found
early. With more, each unit is replayed alone, and the report says that
combinations were not searched. `search.replays` counts the replays run.

When the revision has an outcome, the review lists each `materialized`
caveat as **came true**, with the grounds evidence that carries it now and
whether any carrier is decisive. `withdrawn` is the revision's grounds
withdrawn since, as `explain` reports them. **Outside grounds** lists
evidence observed before the deciding event that is not in the grounds.

What the review is not:

- It does not say a decision was right or wrong. It keeps the basis, the
  outcome and what the decision turned on side by side, as Outcomes 0.1
  says, for a person or an agent to judge.
- It is not a cause. "Decisive" means the program, given these events
  without that one, does not decide: a fact about the rules and the record,
  not about the world.
- It changes nothing. Replays run in fresh sessions; the session the events
  describe is never touched, and a save is never written.

## 3. Report

`caveat-review/0.1`, as `--json` prints it (with `program`):

| Field | Meaning |
| --- | --- |
| `revision`, `value`, `status` | as `explain` reports the revision |
| `decided_by` | `{ sequence, event }` of the deciding event |
| `grounds` | `{ evidence, caveats }` for each grounds evidence, caveats as they are now |
| `retained` | the revision's retained caveats |
| `outcome` | the revision's outcome (Outcomes 0.1), or null |
| `decisive` | `{ evidence, without, because, caveats }`; `without` lists `{ sequence, event }` left out |
| `value_only` | `{ evidence, without, value, caveats }`; `value` is what the deciding event commits instead |
| `not_decisive`, `entangled`, `unobserved`, `outside_grounds` | evidence names |
| `came_true` | `{ caveat, carried_by, decisive }` for each materialized caveat |
| `withdrawn` | as on `explain`'s revision |
| `fatal` | the message of a fatal event that ended the run, or null |
| `search` | `{ mode: "exhaustive" or "single", units, replays, limit }` |

Arrays keep grounds order, or smallest set first. The command exits 0 after
a review, 1 when an event failed fatally (the review is of the session
before it), 2 when the program, the events or the revision cannot be used.

## Limits

- The review drops events, not observations. An event that observes several
  grounds makes them one unit. A way to replay without one observation would
  need the runtime, and is left for when a program needs it.
- Combinations grow as 2^n. Past the limit only single units are tried, so a
  pair that is only decisive together can go unreported; the report says so.
- Each replay re-runs the whole record, so a long record with many units is
  slow. The review is for looking back at a decision, not for a hot path.

## Prototype

This profile follows a prototype run on caveat-lang 1.2.0, before outcomes
existed, over the vendor program above and
[`kit/examples/reviewer-decision/rely.cav`](../kit/examples/reviewer-decision/rely.cav).
Its four findings shaped section 2: telling "not decided" from "decided with
another value" (the first run called every input decisive because each
moved the value), searching combinations (leave-one-out alone missed that the
references and the trial were decisive together), saying how a replay failed
(in `rely.cav` all three grounds are decisive because the program cannot
qualify unobserved evidence, not because a threshold was missed), and
grouping grounds by the event that observed them.

## Implementation

- `kit/lib/review.mjs` runs the replays through the kit's sessions and reads
  each with `explain`; `kit/lib/review.d.mts` types it.
- `kit/bin/caveat.mjs` adds the `review` command; `kit/README.md` lists it.
- `kit/test/review.test.mjs` covers the example above, a program that cannot
  decide without its evidence, entangled and grouped grounds, the single
  mode past the limit, an unknown revision, and that the reviewed session's
  replay leaves the full run unchanged.

## Changes

- 2026-10-24: drafted and implemented for 1.3.
