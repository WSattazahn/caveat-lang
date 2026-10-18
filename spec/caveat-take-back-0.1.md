# CAVEAT Take-back 0.1 — taking back a late qualification

**Status:** stable from 1.0.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); until then a prerelease may still change it. Specified for rc.18 ([rc.18 plan](../docs/releases/rc18-development.md),
P3; the owner's decision Q1 B, 2026-10-11 18:24 UTC) and implemented in P4:
the runtime, save, View 0.2, kit types and the Late.lean take-back step. This
profile adds to
[Late Qualification 0.1](caveat-late-qualification-0.1.md); it changes no
program that does not take back.

A caveat learned late can turn out to be wrong. A player tells the slime
that a creature's reaction was misleading, and later takes that back. An
agent marks a test result as run against the wrong configuration, then finds
the configuration was right. `qualify` records the first fact. Before this
profile nothing could record the second: the `qualifies` relation a
`qualify` adds is permanent, and
[withdrawal](caveat-withdrawal-0.1.md) adds its own caveat rather than removing
another one.

```caveat
caveat doubted consequence material;
evidence witness_pool from "the slime saw a creature eat it";
renewable witness_pool limit 64;
state memory = 0;
event see;
event doubt;
event regrow;
event undoubt;
on see reveal witness_pool;
on see set memory = qualified(1, witness_pool);
on doubt qualify witness_pool with doubted;
on regrow renew witness_pool;
on undoubt unqualify witness_pool with doubted;
```

A take-back records that a qualification no longer stands. It removes
nothing: the relation, the event that added it, every decision made while it
stood and every record that copied it stay as they were.

## Why the take-back names a qualification, not an occurrence

In [Glowcap round 8](../experiments/glowcap/RESULTS.md#round-8-fresh-authors-blind-requests-rc17)
a witness is doubted (CR18), its mushroom regrows (CR19, a `renew`), and the
doubt is taken back (CR20). After the `renew`, `witness_pool` names the new
occurrence, `witness_pool@2`, and a new occurrence does not carry a caveat a
`qualify` added to an earlier one ([Renewal 0.1](caveat-renewal-0.1.md)). The
doubt sits on the first occurrence. A take-back that reached only the
current occurrence would find nothing to take back.

So a take-back is aimed at the **qualification record**: the
`CAVEAT qualifies EVIDENCE@k` relations that a `qualify` added. Source names
the evidence and the caveat, never an occurrence. Naming a past occurrence
in source, which `qualify` would need to doubt a remembered witness from an
earlier life, is a separate design question; it is planned as a 1.1 addition
(round 8 item 2) and is not part of this profile.

## Syntax

```text
on EVENT [when CONDITION] unqualify EVIDENCE with CAVEAT;
[when CONDITION] unqualify EVIDENCE with CAVEAT;        -- as a procedure step
```

`unqualify` is reserved. `EVIDENCE` must be declared evidence, renewable or
not; it may not be a reading stream's template (below). `CAVEAT` must be a
declared caveat. A caveat that some `unqualify` names is a **take-back
caveat**.

### What a take-back caveat may not do

A program is refused when it loads if a take-back caveat:

- is `withdrawn`, which only `withdraw` applies and which nothing undoes
  ([Withdrawal 0.1](caveat-withdrawal-0.1.md));
- is declared to qualify anything (`CAVEAT qualifies NAME;`), or is qualified
  by a declared caveat (`OTHER qualifies CAVEAT;`);
- is named as an extra caveat in `qualified(VALUE, EVIDENCE, CAVEAT)`, in
  `commit … retaining CAVEAT`, or by `examine CAVEAT` or `examined(CAVEAT)`.

It may be applied with `qualify` and `qualify … after`, and read with
`carries(…)` and `has_caveat(…)`.

These rules are what make a take-back exact. Under them a take-back caveat
reaches a value only through a `qualifies` relation that a `qualify` added to
evidence the value holds, or through a record that already holds it (below),
so the runtime can tell which values a take-back reaches without guessing.
The rules restrict only a program that takes back; they mirror the ones
[Withdrawal 0.1](caveat-withdrawal-0.1.md) places on `withdrawn`.

`EVIDENCE` may not be a reading stream's template. A `sample` gives each new
reading the template's caveats as relations of its own, which a `qualify`
did not add; a take-back of the template's relation would leave those copies
on every reading.

## Meaning

A qualification is **live** while no take-back has marked it. When the effect
runs:

1. **The record.** For each **held** occurrence of `EVIDENCE` that has a live
   `CAVEAT qualifies` relation a `qualify` added, immediate or scheduled, the
   session's `taken_back` list gains `{evidence, caveat, sequence, event}`,
   with `evidence` the concrete occurrence. The occurrences of plain evidence
   are the evidence itself. Those of renewable evidence are its declared first
   occurrence and each one `renew` made, first to current. An occurrence is held
   until it retires ([Windows 0.1](caveat-windows-0.1.md)); a retired record is
   history, and no new effect names it. The relation stays in the graph,
   marked by its record.
2. **Nothing to take back is not a failure.** When no held occurrence has
   such a relation, the effect adds no record and reports nothing. A
   take-back never needs `EVIDENCE` to be observed, since the current
   occurrence may be a fresh, unobserved one while an earlier one holds the
   relation. It adds no refusal code.
3. **Later reads.** `qualified(…, EVIDENCE)`, `observed(…)` and
   `carries(…)` no longer take the caveat through a marked relation. They
   still take it through any live one.
4. **Current values.** Every state whose lineage holds a marked occurrence
   is checked. It drops `CAVEAT` from its lineage unless the caveat is still
   live for it, and from its grounds on the same test over the evidence the
   grounds hold. The caveat is still live for a lineage or grounds when
   evidence it holds has a live `CAVEAT qualifies` relation, or when it
   **keeps** the caveat. A lineage can hold evidence a guard read that the
   grounds do not, so the grounds can lose a caveat the lineage keeps
   ([D013](../experiments/agent-decisions/README.md)). The state's value, its other
   caveats and its evidence do not change. The take-back's guard joins no
   lineage: the guard of a take-back usually reads `carries(EVIDENCE,
   CAVEAT)`, which would put back the caveat the effect removes. The
   record names the event instead.
5. **What was already recorded is left alone:**
   - commitment bases and grounds, and what an in-force commitment retains;
   - the decision journal;
   - reading archives and decision series revisions;
   - the graph's relations, each withdrawal record and the observation order;
   - pending scheduled qualifications.

   A decision keeps the caveats its basis carried when it was made.

### Kept caveats

A value **keeps** a take-back caveat when the caveat reached it through
something other than a live relation: a record that holds it. That is a
frozen basis or grounds read through `committed(…)`, `reopened(…)` or a
decision series, a reading archived with it, a stored reopening or selection
qualification, or a record that has since departed (below). A take-back never
removes a kept caveat, so a value computed before a take-back and one
computed after it from the same records carry the same caveats. Kept caveats
travel with a value through computation, as its caveats do, and the runtime
keeps them in the provenance (see "Snapshot and save").

A stored observation or predicate qualification is what the guard of a
`renew`, `reveal`, `observe` or `sample` rule read. It keeps a take-back
caveat only when it did not take that caveat through a live relation of
evidence it lists. A caveat it took that way stays live in the record: the
take-back removes it from the record on the test in step 4, as it does from
current values, so a later read of the occurrence does not bring it back
(step 3). Without this, a renewal guarded on `observed(EVIDENCE)` would copy
a doubt into the new occurrence's record for good: after the take-back
neither occurrence would carry it, while every value read from the new one
would ([D012](../experiments/agent-decisions/README.md)).

### Qualifying again

`qualify` adds a relation only when the occurrence has no live relation for
that caveat, whether declared or added. After a take-back, a `qualify` of the
same occurrence and caveat adds a new relation and applies as
[Late Qualification 0.1](caveat-late-qualification-0.1.md) says. A doubt can
be taken back and raised again any number of times. Each relation has at
most one take-back record.

A scheduled `qualify … after` waiting when a take-back runs still applies
when it falls due, to the occurrence it was scheduled against, and adds a new
live relation. A take-back does not cancel what time will do.

### Effects

The effect is part of its event's transaction, so a refused or failed event
takes nothing back. It is reported in `effects` as

```json
{"kind": "unqualify", "evidence": "witness_pool", "caveat": "doubted",
 "occurrences": ["witness_pool"]}
```

with the marked occurrences in occurrence order. An effect that marked
nothing is not reported.

## The round 8 sequence

With the program above:

1. `witness_pool` is observed, and a state `memory` copies a value
   qualified by it.
2. `doubt`: the graph gains `doubted qualifies witness_pool`. `memory` gains
   `doubted`, as Late Qualification 0.1 says.
3. `regrow`: `witness_pool` now names `witness_pool@2`, unobserved, which
   does not carry `doubted`. `memory` still holds `witness_pool` and
   `doubted`.
4. `undoubt`: `witness_pool`, a held occurrence, has a live `doubted`
   relation, so `taken_back` gains `{evidence: "witness_pool", caveat:
   "doubted", …}`. `memory` holds no other evidence with a live `doubted`
   relation and keeps nothing, so it drops `doubted`. Its value is
   unchanged.

A commitment made between steps 2 and 4 keeps `doubted` in its basis and
grounds, and the journal entry that recorded it keeps it too.

### The limit this leaves

A take-back reaches every held occurrence of the evidence. Suppose the
witness's first life was doubted before the regrow and its second life after
it. Taking back the first life's doubt then takes back both. Choosing one
occurrence needs the same past-occurrence naming as doubting one, so both go
to 1.1. A program can still tell the occurrences apart by its own state,
as the round 8 programs did.

## Snapshot and save

- The snapshot lists `taken_back`, in the order the records were made.
- A save holds the same list, in `caveat-reactive-save/0.1` as an additive
  field, and leaves it out when it is empty.
- A provenance's kept caveats are written as `kept`, a list of caveat names
  in name order, beside `evidence` and `caveats`, and left out when empty.
  Only a program that takes back writes it.
- A save without these fields, including every save made before this
  profile, restores with no take-backs and nothing kept. A runtime without
  this profile refuses a save that has them, as it refuses any field its
  schema lacks; only a program that takes back writes them.

Restoring refuses a save when:

- it has `taken_back` and the program has no `unqualify`;
- a record names evidence that is neither declared evidence nor an
  occurrence of renewable evidence, a caveat that no `unqualify` names with
  that evidence, or an event that reaches no `unqualify` of that evidence and
  caveat in a rule or a procedure a rule calls. An occurrence counts as its
  renewable evidence, and conditions are not evaluated: the event could have
  taken it back, not shown to have;
- a record is dated 0 or after the save's `sequence`, or the records are not
  in sequence order;
- the records disagree with the graph's relations. For each pair of a caveat
  and an occurrence, the `qualifies` relations events added are matched to
  its records in order: every relation but the last needs a record, and the
  last may have one. A record without its relation, or a pair with two live
  relations, is refused;
- a provenance's `kept` names a caveat that is not a take-back caveat or that
  the provenance does not carry, or a grounds keeps a caveat its lineage does
  not keep.

The existing rules apply unchanged. A `qualifies` relation must still pair a
caveat and evidence some `qualify` of the source can pair, and a take-back
caveat on a provenance must still be one the source could attach
([Save 0.1](caveat-save-0.1.md)). As with every restore check, a save that
passes holds records the program could hold, not records events are shown to
have produced. Restore never authenticates history.

## Views

`caveat-reactive-view/0.1` is unchanged. Its `relations` list the graph's
relations, and a marked relation is still one of them. Bindings, their
explanations and `effects` show a take-back as they show any other change. A
host that needs the records reads the snapshot or View 0.2.

`caveat-reactive-view/0.2` gains a member `taken_back`, the snapshot's list,
present only in a program that has an `unqualify`. Its delta gains a member
`taken_back` with `removed` and `appended`, applied like `decision_journal`:
the runtime only appends records, and departure drops some (below). It too
is present only in such a program, so no existing program's 0.2 view or
delta changes. The delta gate's named cases gain a take-back, a qualification
raised again after one, and the departure of a taken-back occurrence.

## Departure and collection

These rules apply only to a program with a window.

- A take-back never marks a retired occurrence's relation, so a relation
  that retires live stays live.
- When an occurrence departs, its take-back records depart with it into the
  archive, beside its relations. A take-back record pins nothing: it has no
  reason to keep exact. Restore refuses a record that names a departed
  occurrence, as it refuses a relation that does.
- When a record departs, every current value that held it keeps each
  take-back caveat that the record's live relations gave it. Those relations
  can never be marked, so the value carries exactly what it would have carried
  had the record stayed. Ordinary departure still preserves outcomes
  ([Departure 0.1](caveat-departure-0.1.md)).
- [Withdrawal Collection 0.1](caveat-withdrawal-collection-0.1.md) is
  unchanged. A take-back record is metadata owned by its occurrence (rule 5),
  and it roots nothing.

## Withdrawal, decisions and permission

- A take-back of a caveat on withdrawn evidence marks its relation and leaves
  the withdrawal, its `withdrawn` caveat and `withdrawn(…)` as they were.
  `rests_on_withdrawn(…)` reads frozen grounds and does not change.
- A commitment's frozen basis and grounds, what it retains and relies on, and
  its journal entries are left as they are. Reading `committed(…)` or
  `reopened(…)` after a take-back carries what the commitment retains,
  as a kept caveat.
- A take-back reopens nothing and commits nothing. A program states what it
  means for its decisions, as it does for a withdrawal.
- Permission records are frozen records and do not change.

## Not in this profile

This profile does not include:

- taking back the qualification of one chosen occurrence, or qualifying an
  earlier occurrence (both planned for 1.1);
- taking back a declared qualification, or `withdrawn`;
- cancelling a scheduled qualification;
- a reason for a take-back. The record names the event; a program that
  wants a reason it can cite reveals evidence in the same event;
- a predicate over take-back records. `carries(EVIDENCE, CAVEAT)` answers
  for the current occurrence.

## Proofs

`proofs/lean/Caveat/Late.lean` gains a take-back step in P4, beside
`qualifyStep`. Its theorems are the frozen-record half of this profile: an
accepted take-back leaves every commitment's basis and grounds equal, a
value keeps its number, it loses at most the take-back caveat, its grounds
stay within its lineage, and a value that holds no marked occurrence is
unchanged. Which values lose the caveat (the live and kept test above) is
supplied as input to the step, as the caveat's qualifiers are to
`qualifyStep`, because the model has no graph; that test is checked by the
runtime tests in P4. Renewal, retirement and departure stay unmodeled, as the
[proofs README](../proofs/lean/README.md) says for late qualification.

## Corpus sweep

`node experiments/take-back-sweep/sweep.mjs` reads every tracked Caveat
program (180 at this profile's revision) and writes
[`RESULTS.json`](../experiments/take-back-sweep/RESULTS.json):

- no program uses the word `unqualify`, so reserving it refuses no
  declaration;
- no program has an `unqualify`, so the load rules above, the new effect, the
  save fields and the view member apply to none of them: no accepted program
  changes;
- 66 programs use `qualify`, over 195 pairs of program, caveat and evidence.
  For 187 of them a take-back of that caveat on that evidence would meet the
  load rules. The 8 that would not each qualify a reading stream's template
  with `stale` (the agent-adoption trackers and
  `kit/examples/agent-evidence/qualification.cav`). No pair breaks a rule on
  the caveat itself.

The sweep reads source text and runs no program. P4 reruns the save sweep,
the departure gate and the View 0.2 delta gate over the whole corpus with
the implementation.

## Changes

- 2026-10-11: specified (rc.18 P3).
- 2026-10-11: implemented (rc.18 P4). Tests: `runtime/tests/take_back.rs`
  (round 8 item 1, including CR20's doubt, renew and take-back) and the
  take-back named cases in `runtime/tests/view_v2.rs`. A declared first
  occurrence never departs, so departure reaches only renewed occurrences.
- 2026-10-18: a stored observation or predicate qualification keeps only what
  it did not take through a live relation (D012). A doubt raised, carried
  into a renewal by its guard and taken back no longer stays on values read
  from the renewed occurrence. Tests: the D012 cases in
  `runtime/tests/take_back.rs`, including a randomized check that every
  caveat a value carries is live on evidence it holds.
- 2026-10-18: a take-back tests a state's grounds over the evidence the
  grounds hold, not the lineage's (D013): when a guard read other evidence
  that carries a caveat of the same name, `has_caveat` no longer keeps a
  taken-back flag. Tests: the D013 cases in `runtime/tests/take_back.rs`.
