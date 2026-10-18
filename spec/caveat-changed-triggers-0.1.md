# CAVEAT Changed Triggers 0.1 (draft) — reopen only when a reading changes

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.1 ([#223](https://github.com/WSattazahn/caveat-lang/issues/223)): implemented on `main`, not yet in a release. It becomes stable when a 1.x release ships it. It adds to
[Reopening Triggers 0.1](caveat-reopening-triggers-0.1.md); it changes no
program that loads today.

A `reopened by STREAM` trigger fires on every new reading, including one that
repeats the last
([Reopening Triggers 0.1](caveat-reopening-triggers-0.1.md#meaning), "New
observation, not new value"). That is right when every observation is news.
It is wrong for a routine re-check that re-reads everything: in agent pilot 3
the first repeated reading reopened the decision, and the journal named a
reading that had not changed as the cause, ahead of the event that did change
things:

```text
install_n24@1 = -434  reopened
  #13 assess: committed because n20_days@1, n21_days@1, n22_cves@1, n24_days@1, n24_cves@1; ...
  #14 observe_present: reopened because n20_present@2 (caveats: probe_is_point_in_time)
```

(`caveat-lang@0.1.0-rc.19`; `n20_present@2` repeated `n20_present@1`'s value
of 1.) The workaround moves the comparison into the host, which then appends
only readings that changed. This profile lets the declaration say it:

```caveat
decisions install_n24 limit 4 reopened by changed n20_present;
```

## Syntax

```text
decisions NAME limit N reopened by TRIGGER, TRIGGER, …;
TRIGGER := [changed] STREAM [supporting CLAIM | opposing CLAIM]
```

`changed` comes first. A trigger without it keeps today's meaning.
`reopened by changed n20_present` refuses to load today ("reopened by takes
STREAM or STREAM supporting|opposing CLAIM"), so no program that loads
changes. `changed` leads only before a stream: a stream named `changed`
keeps both of its forms, `changed` and `changed supporting CLAIM`.

## Meaning

A `changed` trigger matches a reading when the trigger without `changed`
would, and the reading's value differs from the stream's previous reading:
the reading `latest(STREAM)` named just before this sample.

- **No previous reading.** A stream's first reading is a change: it matches.
- **Comparison.** Values are numbers, compared exactly, as `!=` compares them.
- **Stance.** The previous reading is the stream's, whatever its stance or
  its lack of one ([Neutral Sample 0.1](caveat-neutral-sample-0.1.md)); a
  `supporting|opposing CLAIM` filter applies to the new reading only.
- **Windows.** The previous reading is the one before this sample, even when
  taking this one retires it.
- **Withdrawal.** A withdrawn previous reading still counts as the previous
  value. Whether a withdrawal should reopen a decision stays with
  `rests_on_withdrawn` and authored rules.

Everything else is Reopening Triggers 0.1: the step runs right after the
reading, inherits the sample's guard, reopens only a revision in force, names
the triggering reading in the journal, and runs once per series however many
of its triggers match. A reading that does not match leaves the decision
exactly as it was, with no record.

No new refusal code is added.

## What stays authored

- A tolerance ("changed by more than 5"). Write a rule with a guard that
  compares `latest(STREAM)` with `history_at(STREAM, …)`.
- A comparison with the reading the decision rests on, rather than the
  stream's previous one. The two are the same for the re-check in #223; they
  differ when several readings arrive between decisions, and a rule can say
  which it means.

## Snapshot, save, explain and views

Triggers come from the source, so snapshots, saves, explanations and views
hold nothing new. A reopening a `changed` trigger causes is recorded like any
other. Restore accepts it as it accepts any trigger's reopening: the event
samples, directly or through a procedure, into a stream the series watches,
with a matching relation and claim. As with every restore check, restore
never authenticates history.

## Compatibility with other 1.1 additions

- [Neutral Sample 0.1](caveat-neutral-sample-0.1.md): `changed STREAM`
  watches neutral readings as an unfiltered trigger does.

## Not in this profile

- `changed` on an authored `reopen` rule.
- Values that are not numbers ([#220](https://github.com/WSattazahn/caveat-lang/issues/220)).

## Proofs

`proofs/lean/Caveat` does not model reopening triggers; their reopenings are
ordinary reopenings, which the frozen-basis theorems cover. A `changed`
trigger only fires less often. The implementation adds runtime tests.

## Corpus sweep

`node experiments/changed-trigger-sweep/sweep.mjs` reads every tracked Caveat
program and writes
[`RESULTS.json`](../experiments/changed-trigger-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 191 programs declare 21 triggers, in 14 programs; none writes `changed`, so
  no program reaches the new form;
- no authored reopening rule compares a stream's latest reading with an
  earlier one: the corpus has no hand-written form of this trigger to replace.

The pilot 3 program is not in the repository. No accepted program changes.

## Implementation

- The runtime parses `changed` into the trigger, notes the stream's previous
  value before a sample retires anything, and runs a `changed` trigger only
  when the new value differs. Restore is unchanged.
- `runtime/tests/changed_triggers.rs` covers #223's re-check (a repeated
  value does not reopen; a plain trigger beside it still does), a changed
  reading named in the journal, a first reading, the claim filter with the
  stream's previous value, a window of one, a save and restore round trip,
  and the load rules, including a stream named `changed`.
- The module linker keeps `changed` where it leads a trigger, so a module
  may still name a stream `changed`; `runtime/tests/modules.rs` covers it.
- The language reference, the authoring guide and the package describe the
  form.

## Changes

- 2026-10-17: specified for 1.1 (#223).
- 2026-10-17: implemented.
