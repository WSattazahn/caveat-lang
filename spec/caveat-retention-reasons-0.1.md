# CAVEAT Retention Reasons 0.1 — a caveat kept on purpose, with its reason

**Status:** stable from 1.1.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.1 (finding D006 in [agent decisions](../experiments/agent-decisions/README.md)). It adds to
[Reactive 0.3](caveat-reactive-0.3.md) and
[Explanations 0.2](caveat-explanations-0.2.md); it changes no program that
loads today.

A decision often goes ahead with a caveat its author chose not to pursue:
"the presentation pass can wait, because the game does not gate 1.0". The
`retaining` clause keeps the caveat on the decision, but it takes names only.
The reason has nowhere to go:

```text
probes/retaining-because.cav does not load: line 6, column 1: invalid reactive identifier k because e
```

(`caveat-lang@0.1.0-rc.19`, `caveat validate`, kept in
[`probes/retaining-because.txt`](../experiments/agent-decisions/003-presentation-pass-deferral/probes/retaining-because.txt);
the same on `1.0.0`.) The workaround puts the reason into the decision's
grounds through `using`, and with two caveats left for two reasons the
pairing is lost, or it takes one decision per caveat
([guide](../docs/AI_AUTHORING.md#record-a-caveat-you-chose-not-to-pursue)).
In the record, a caveat kept on purpose then looks the same as one kept while
it waits. This profile lets each retained caveat name its reason:

```caveat
on triage commit defer_pass because enough
  retaining presentation_gaps because scope_note, pr179_drift because d1_only;
```

`explain` prints, for `defer_pass@1`, `based on d1_only, scope_note` and
`retaining: pr179_drift because d1_only, presentation_gaps because scope_note`.

## Syntax

```text
commit D because REASON [using V | citing X, …] [permitted by G [for X]]
    retaining CAVEAT [because EVIDENCE] {, CAVEAT [because EVIDENCE]}
```

Each retained caveat may name one reason, declared evidence, by name. A
retained caveat without `because` keeps today's meaning. The same evidence
may be the reason for several caveats. Inside `retaining`, `because` is
recognised only after a caveat name; `retaining k because e` refuses to load
today ("invalid reactive identifier"), so no program that loads changes.

The reason must be declared evidence. A reading stream, `latest(…)`,
`occurrence(…)` and an expression do not load. A procedure's `evidence`
parameter is allowed, as for any evidence position.

## Meaning

When the commit runs, each reason resolves to its evidence's current
occurrence, as `qualified(1, EVIDENCE)` would, and:

- **It must be observed.** Otherwise the event refuses as
  `evaluation/unobserved_evidence`, as for a `using` value that cites it.
- **It joins the decision's grounds**, with the caveats it carries, as a
  `using` value that read it would add. A caveat kept on purpose is part of
  what the decision was made on, so its reason is too. It joins the lineage
  too, as grounds always do.
- **The commit records the pairing**: each retained caveat that named a
  reason, with the concrete occurrence of that reason. The record is frozen
  with the commitment, as its grounds are.

Everything else follows from the reason being in the grounds. Withdrawing the
reason makes `rests_on_withdrawn(D)` true for the revision, and a rule may
reopen it. A reopening and the next revision behave as for any grounds. The
retained caveat itself is retained exactly as without `because`.

No new refusal code is added. A reason that names undeclared evidence, a
reading stream or anything but a name does not load.

## Snapshot, save and views

- The snapshot gains `retention_reasons`: by commitment, a map from each
  retained caveat that named a reason to the occurrence of its reason, such
  as `{"defer_pass@1": {"presentation_gaps": "scope_note"}}`. It is left out
  when empty, so a program that names no reason has the snapshot it has
  today, byte for byte.
- `caveat-reactive-save/0.1` holds the same map under the same name, and
  leaves it out when empty, so every 1.0 save restores as before. Restoring
  refuses a record for an unknown commitment, a caveat the commitment's
  grounds do not retain, a reason that is not known evidence, was never
  observed, or is not in the commitment's grounds. As with every restore
  check, a save that passes holds records the program could hold; restore
  never authenticates history.
- `caveat-explain/0.1` gives each decision revision `retention_reasons`, the
  same map for that revision, left out when empty. Its text prints a reason
  after its caveat on the `retaining:` line.
- `caveat-reactive-view/0.1` and `/0.2` are unchanged. A host that needs the
  reasons reads the snapshot or `explain`.

The reason is pinned as grounds already are, so departure and the withdrawal
collector see no new kind of record.

## Compatibility with other 1.1 additions

- [Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md): a reason names
  the current occurrence. Naming an earlier one is not in this profile.
- [Citing 0.1](caveat-citing-0.1.md): `retaining` follows `citing` as it
  follows `using`; a citing decision's grounds gain its reasons.

## Not in this profile

- A reason given as text, or as an expression. A reason is evidence, so that
  it can be withdrawn, cited and asked about like any other.
- A reason for a caveat the grounds carried rather than the author retained.
- A predicate that asks for a caveat's reason. `explain` and `dependents`
  answer "what did we choose not to look at, and why?".
- The reasons in `caveat-reactive-view/0.2` and its delta. They can be added
  there later as a member present only when a program names a reason.
- A reason on a caveat's declaration. It would hold for every decision, and
  the reason belongs to one choice.

## Proofs

`proofs/lean/Caveat` models a commitment's frozen grounds as a provenance.
A reason is one more piece of evidence in them, gathered when the commit
runs; the theorems about frozen bases cover it without change. The pairing
record is bookkeeping beside the grounds, which the model leaves out. The
implementation adds runtime tests.

## Corpus sweep

`node experiments/retention-reasons-sweep/sweep.mjs` reads every tracked
Caveat program and writes
[`RESULTS.json`](../experiments/retention-reasons-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 191 programs make 204 commits; 36 of them, in 27 programs, retain
  caveats, and none writes `because` inside `retaining`, so no program
  reaches the new form;
- 12 of those commits retain more than one caveat, where one reason per
  decision cannot say which caveat it is for;
- 24 retaining commits cite evidence with `qualified(…)` in `using`, the
  workaround that puts a reason into the grounds by hand. Among them is
  D006's own `defer_pass`.

No accepted program changes.

## Implementation

- The runtime parses each `CAVEAT because EVIDENCE` into the commit effect
  with a `qualified(1, EVIDENCE)` reading, evaluates it where `using` is
  evaluated, and records the pairing. The reason is pinned as a permission's
  grant is, so it never departs while the record holds it.
- `runtime/tests/retention_reasons.rs` covers D006's two caveats with two
  reasons, a program with no reason (no new field in its snapshot or save),
  withdrawal of a reason reopening the decision, an unobserved reason, a save
  and restore round trip with restore's checks, the load rules and a
  procedure's evidence parameter.
- `explain` reports `retention_reasons` per revision and prints each reason
  after its caveat. The kit's types, the language reference and the authoring
  guide describe the clause, and the package ships this profile.

## Changes

- 2026-10-17: specified for 1.1 (D006).
- 2026-10-17: implemented.
