# CAVEAT Neutral Sample 0.1 (draft) — a reading without a stance

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.1 ([#218](https://github.com/WSattazahn/caveat-lang/issues/218)): implemented on `main`, not yet in a release. It becomes stable when a 1.x release ships it. It adds to
[Reactive 0.3](caveat-reactive-0.3.md),
[Neutral Reveal 0.1](caveat-neutral-reveal-0.1.md) and
[Reopening Triggers 0.1](caveat-reopening-triggers-0.1.md); it changes no
program that loads today.

Many readings bear on no claim. An agent that records the local Node version
before a toolchain upgrade has a number, not support for or against anything.
`reveal` already observes evidence without a stance; `sample` does not:

```text
on go sample r = v;
```

`caveat validate` prints `does not load: line 5, column 1: invalid reactive
effect: sample r = v` (`caveat-lang@0.1.0-rc.19` in agent pilot 2, and the
same on `1.0.0`). The workaround declares a claim only so the reading has
something to support; the pilot's agent invented `toolchain_inventory`. The
record then says the reading supports a claim nobody made. This profile lets a
reading be only a reading:

```caveat
evidence node_version from "node --version on the build machine";
readings node from node_version limit 8;
event inventory major min 0 max 99;
on inventory sample node = major;
```

`explain` prints `node@1 = 22 observed (no stance)  (#1 inventory)`.

## Syntax

```text
sample STREAM = EXPRESSION [supports CLAIM | opposes CLAIM]
```

Without `supports` or `opposes`, the whole remainder is the value. A `sample`
with `supports` or `opposes` anywhere but before a final claim name does not
load, and its error names the form:
`sample expects STREAM = EXPRESSION [supports|opposes CLAIM]`. That message
also replaces today's "invalid reactive effect" for a sample with a missing
or misplaced stance.

## Meaning

A neutral sample is a sample with nothing related to a claim. Everything the
stance form does, it does, apart from the stance:

- It takes the stream's next occurrence, `STREAM@N`, with the value, the
  event and the sequence, inherits the template's caveats, counts against the
  stream's limit, and retires and departs exactly as a stance reading does.
- It observes that occurrence, as a neutral `reveal` observes evidence: the
  occurrence joins `observations` (see Snapshot below), and `observed`,
  `qualify`, `withdraw`, `carries`, `latest`, `has_sample`, the history
  functions and `qualified` treat it as any reading.
- It adds no `supports` or `opposes` relation, and so no claim gains or loses
  support. A value computed from the reading has the same grounds and lineage
  as one computed from a stance reading.
- An unfiltered trigger, `reopened by STREAM`, fires on it. A filtered trigger,
  `reopened by STREAM supporting|opposing CLAIM`, does not: the reading bears
  on no claim.

No new refusal code is added. A neutral sample that fails refuses as the stance
form would (a non-numeric value, a full history, a failed expression).

## Snapshot, save, explain and views

- A reading occurrence in the snapshot has `relation` and `claim` only when it
  took a stance; both are absent for a neutral reading. Exactly one present
  is invalid.
- A neutral sample's effect report is `{"kind":"sample","stream":…,"id":…,"value":…}`,
  without `relation` and `target`, as a neutral reveal's is.
- The first neutral sample records `observations`, the ordered observation
  ledger [Neutral Reveal 0.1](caveat-neutral-reveal-0.1.md#snapshot-effect-and-save-representation)
  defines; a program with no neutral observation keeps today's snapshot and
  save, byte for byte.
- `caveat-reactive-save/0.1` holds neutral readings the same way, so every 1.0
  save restores as before. A 1.0 runtime need not restore a save holding one.
  Restoring refuses a reading with only one of `relation` and `claim`, a
  neutral reading that has a stance relation in the graph or is missing from
  `observations`, and a neutral reading of a stream no neutral `sample` in the
  program writes. As with every restore check, a save that passes holds
  records the program could hold; restore never authenticates history.
- An archived reading leaves out `relation` and `claim` in the same way.
- `caveat-explain/0.1` lists a neutral reading with `relation: null` and
  `claim: null`, and its text says `observed (no stance)`, as for a neutral
  reveal. `dependents` follows grounds and lineage, unchanged.
- `caveat-reactive-view/0.1` and `/0.2` are unchanged: they list relations,
  and a neutral reading adds none.

## Compatibility with other 1.1 additions

- [Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md):
  `occurrence(STREAM, n)` names a neutral reading as any other.
- [Citing 0.1](caveat-citing-0.1.md) and
  [Retention Reasons 0.1](caveat-retention-reasons-0.1.md): a neutral reading
  can be cited or be a reason; neither reads stances.

## Not in this profile

- A neutral relation in declarations (`e supports c` stays the only form).
- Values that are not numbers ([#220](https://github.com/WSattazahn/caveat-lang/issues/220)).
- A stance added to a reading after it was taken.

## Proofs

`proofs/lean/Caveat` models evidence, grounds and frozen bases; stance
relations are not part of that model. A neutral reading is a reading whose
claim edge is absent, so the theorems cover it without change. The
implementation adds runtime tests.

## Corpus sweep

`node experiments/neutral-sample-sweep/sweep.mjs` reads every tracked Caveat
program and writes
[`RESULTS.json`](../experiments/neutral-sample-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 191 programs make 202 `sample` effects, in 63 programs; none leaves out
  the stance, so no program reaches the new form;
- samples point at 69 claims, and for 30 of them, in 27 programs, nothing but
  a sample relates evidence to the claim. Several are named `seen`,
  `checks_pass` or `ready` and look made for their readings, as #218's
  `toolchain_inventory` was. Source text cannot say why a claim exists, so
  these are candidates for the neutral form, not mistakes;
- 4 reopening triggers filter by claim. A neutral reading never matches one,
  and none of those programs would change, since none writes a neutral sample.

No accepted program changes.

## Implementation

- The runtime parses the stance as optional, records a neutral reading as
  observed without a stance relation, and runs only unfiltered reopening
  triggers for it. Its snapshot, save, archive and effect records leave out
  `relation` and `claim` (or `target`). Restore checks a neutral reading
  against the observation ledger, which already requires a reachable neutral
  effect, now a neutral reveal or a neutral sample of the reading's stream.
- `runtime/tests/neutral_sample.rs` covers #218's inventory, the effect
  report, triggers with and without a claim filter, a save and restore round
  trip with restore's checks, a stance sample's unchanged record, the load
  rules and a procedure's `readings` parameter.
- The kit's types make `relation` and `claim` optional on a reading and
  `relation` and `target` optional on a sample effect; its archive reader
  accepts a reading with neither. The language reference, the authoring
  guide and the package describe the form.

## Changes

- 2026-10-17: specified for 1.1 (#218).
- 2026-10-17: implemented.
