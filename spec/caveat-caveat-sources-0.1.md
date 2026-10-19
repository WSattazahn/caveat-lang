# CAVEAT Caveat Sources 0.1: where each caveat came from

**Status:** stable from 1.1.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.1. It changes how the runtime keeps
[Take-back 0.1](caveat-take-back-0.1.md) and
[Late Qualification 0.1](caveat-late-qualification-0.1.md) consistent; it adds
no syntax.

A value's lineage lists the evidence it rests on and the caveats it carries.
Before this profile the caveats were names only. Whether a caveat was still
live was worked out afterwards, by looking for a live relation on any evidence
the lineage, grounds or stored record listed. That test went wrong in four cases
(D012 to D015 in [the decision log](../experiments/agent-decisions/README.md)):
it froze a caveat a guard had read, it judged grounds by the lineage's
evidence, a late `qualify` missed records, and a value read through a
decision's basis gained a late caveat or not depending on when it was
computed. Three were fixes, made one by one on `main` and in 1.0.1, each by
filtering the name test. The fourth was an open case, and this profile
settles it. This profile replaces the
test: every caveat a lineage carries names the routes it arrived by, and
`unqualify` and `qualify` act on routes.

## Routes

A caveat `C` on a lineage, grounds or stored qualification has one or more
**routes**:

- **through `O`**: it arrived through occurrence `O`'s `C qualifies O`
  relation, declared or added by `qualify`. A caveat that qualifies `C` (a
  `D qualifies C` declaration) arrives through the same route as `C`.
- **kept**: it arrived from a record: a commitment's frozen basis or grounds,
  a reading archived with it, a stored reopening or selection qualification,
  or a record that has departed. These are the records Take-back 0.1 lists
  under "Kept caveats".

A value carries `C` while at least one route remains. Computation joins
routes as it joins caveats: a value computed from two inputs carries each
caveat with the routes of both.

A stored observation or predicate qualification is not a record in this
sense. It is what the guard of a `renew`, `reveal`, `observe` or `sample`
rule read, or the guard of a `commit`, `reopen`, `withdraw` or `examine`
rule, stored with the predicate the rule made true. It keeps
each route it read. A value read through the occurrence it made takes those
routes, so a take-back or a `qualify` reaches it the same way it reaches a
current value.

## Taking back and qualifying

- `unqualify E with C` marks each live `C` relation of a held occurrence of
  `E`, as Take-back 0.1 says, and removes every route through a marked
  occurrence from every current value, its grounds and every stored
  observation or predicate qualification. A caveat with no route left is
  dropped. Kept routes are never removed.
- `qualify E with C` adds `C` with the route through `E`'s current occurrence
  to every current value, grounds and stored observation or predicate
  qualification whose evidence includes that occurrence, as Late
  Qualification 0.1 step 2 says for current values.

`qualify` reaches evidence a value holds through a record too: a value set
under `when committed(D)` holds the evidence of D's frozen basis in its
lineage, so a later `qualify` of that evidence adds a route through it, whenever
the value was computed (D015). The basis itself is a record and stays as
recorded; a caveat it carried reaches the value as a kept route. So a value
read through a basis composes the two: kept routes for what the basis
recorded, live routes through the evidence it names.

The rule for every record is the same: **a record's own caveats are kept;
evidence a record lists is read live.** It runs one way. A later `qualify`
reaches through a basis, because the basis lists the evidence; a take-back
never removes what a basis recorded, because that caveat is the basis's own.
A decision made while its evidence was flagged keeps the flag after the flag
is taken back (`d012_a_decision_made_while_flagged_keeps_the_flag` in
`runtime/tests/take_back.rs`). An implementation must not let a take-back
reach through a basis.

Commitment bases and grounds, reading archives and the decision journal keep
what they recorded, as before.

## The invariant

For every current value, its grounds and every stored observation or
predicate qualification: a caveat is carried exactly when it has a kept
route, or a route through an occurrence whose relation is live. The
randomized check in `runtime/tests/take_back.rs` asserts it after every event
and after every save and restore, for every value it reads, including one
set through a decision's basis. Its events include every guard that stores
a record: `renew`, `reveal`, `sample`, `commit`, `reopen`, `withdraw` and a
guarded `examine`.

The runtime keeps no route list. Since every listed occurrence is read live,
the invariant above is the test the runtime already applies: a caveat is
live on a lineage when it is kept, or some occurrence the lineage lists has
a live relation for it. The three fixes (D012 to D014) are that test applied
to stored guard records and to grounds over their own evidence; they stay as
the implementation, with their regression tests. The runtime change for
D015 is the live read: a value read through a record gains each caveat that
a `qualify` effect names and that now qualifies evidence the record lists,
with whatever qualifies that caveat. Only those relations can be added after
a record is made, so only they can be missing from it. A decision's `relies
on` citations join `committed(…)` as names without their declared caveats,
as before; a later `qualify` of one reaches the value like any other listed
evidence. This profile's runtime change is
complete when that check covers every reader, with none left unchecked.

## Snapshot and save

The snapshot is unchanged: `qualified_values`, `value_grounds` and the
qualification tables still list caveat names, and `kept` as before.

The save gains no field. Every occurrence a lineage lists is read live, so
a caveat's live routes are exactly the listed occurrences whose `C
qualifies` relation is live, which the save already holds (the relations and
the take-backs). The one route that is not derived is `kept`, and the save
already stores it. Every route is therefore at its **default**, and a save is
byte for byte the save an earlier 1.x runtime writes for the same session.
A save a 1.0.x runtime wrote restores as before. A save written by 1.0.0
after D012 or D013 fired holds a frozen doubt as `kept`; it restores that
way, as it does today.

Repair on load is not done: a restore could drop a doubt that 1.0.0 froze as
kept. Whether restore should do that is a separate decision; this profile
keeps restore as Save 0.1 states. Replaying the session's events repairs it.

## Reading a record as recorded

The frozen view stays readable: `history_at(SERIES, INDEX)` reads a
revision's frozen basis with its stored provenance
([Reactive 0.6](caveat-reactive-0.6.md)), and `explain` shows each
decision's basis and grounds as recorded. A value read under `committed(D)`
is the live view; the record is the frozen one.

## Compatibility

This is a 1.x fix, under [COMPATIBILITY.md](../docs/COMPATIBILITY.md)'s
clause "unless a fix restores a documented promise". Late Qualification 0.1
step 2 says every current value whose evidence includes the qualified
evidence gains the caveat. Before this profile a value set under
`committed(…)` after the `qualify` did not, though its evidence includes it;
it now does. The change only adds caveats: no value a 1.0 program computes
loses one. The 1.1 release note names it as a behavior change.

The corpus sweep (`experiments/caveat-sources-sweep/`, results in its
`RESULTS.json`) drives every tracked program with the save sweep's seeded
events, 8 runs of 150, on the runtime before and after this change, and
compares every state's caveats after every event, in its value and its
grounds, and every view binding's. It counts each direction separately:

- **Gains** (a later `qualify` reaching a value through a record): 66 of
  194 tracked programs have a rule conditioned on `committed(…)` or
  `reopened(…)`, or read `history_at` or `basis_of`, and also `qualify`;
  that is the population that can gain. Under the seeded events, 16
  programs gained a caveat, at 8,805 readings; 12 of them are versions of
  four authoring-trial runs whose view reads a decision's record.
  `RESULTS.json` lists each. The departure gate's save sweep covers 14 of
  them and lists them as reviewed save changes
  (`experiments/departure-gate/reviewed-save-changes.mjs`); their
  refusals and fatal outcomes are unchanged.
- **Losses** (a take-back removing a caveat a value kept): none are
  possible, since a take-back never removes a kept route, and none were
  found. 2 programs both `unqualify` and read a decision
  (`experiments/stabilization-trial/trial.cav` and `round8.cav`).

Every event's outcome was the same on both runtimes. The sweep fails on any
loss or any changed outcome.

Two of the kit's save fixtures (`kit/test/fixtures/saves/`, `ordered.cav`
and `readiness.cav`) continue, after a restore, into views that gain such a
caveat. `kit/test/save-fixtures.test.mjs` holds their views to every
release's except for caveats added, and byte for byte to the same events
played live without a save.

1.1 ships a detector, asked for like `--unused`
([Check 0.1](caveat-check-0.1.md), C008 and C009, from 1.1.0).
`caveat check --records PROGRAM` reports each rule, procedure or binding
that reads a decision's record in a program that qualifies evidence (C008
`record-reads-late-caveat`). `caveat check --records --save SAVE PROGRAM`
also restores the save and reports each state or shown binding it holds
without a caveat that evidence it lists now carries through a live
`qualify` relation (C009 `save-lacks-late-caveat`): a value a 1.0.x runtime
saved that 1.1 would compute with the caveat. Restore does not repair such a
value; C009 says where one is. A save 1.1 wrote has none.

## Changes

- 2026-10-18: drafted.
- 2026-10-18: the guard records also cover `commit`, `reopen`, `withdraw`
  and `examine`; the one rule for records (a record's own caveats are kept,
  evidence it lists is read live) and that a take-back never reaches through
  a basis; the corpus count split by direction.
- 2026-10-18: implemented. A value read through a record reads the evidence
  the record lists live; the save gains no field, since every route is at
  its default. Tests: the D015 case and the randomized check in
  `runtime/tests/take_back.rs`; corpus sweep
  `experiments/caveat-sources-sweep/`.
- 2026-10-18: the detector is Check 0.1's C008 and C009 (`--records`,
  `--save`).
