# CAVEAT Save 0.1

A game has to save. Round 6 of the
[glowcap benchmark](../experiments/glowcap/RESULTS.md) asked for save and
resume, and the runtime had no way to restore a session, so the Caveat side
saved the events it had accepted and replayed them. That took 9 lines, and 8
seconds to resume ten minutes of play, growing with every minute played. A
session can now be saved and restored directly:

```js
const saved = session.save();                        // JSON a host can store
const resumed = WebReactiveSession.restore(source, saved);
```

```rust
let save = session.save()?;                          // ReactiveSave, serde
let resumed = ReactiveSession::restore(source, &save)?;
```

Restoring costs what loading the program costs plus the size of the save, not
the length of the game. With the same source and runtime contract, restoring
the intact save text produced by the runtime preserves the saved session's
full snapshot and the outcomes and effects of later events. Acceptance of an
edited save does not promise equivalence to the session before the edit.

## What a save holds

Schema `caveat-reactive-save/0.1`, as JSON. Some empty or defaulted fields are
left out; required fields, including the `states` map, must remain even when
empty. A runtime-produced save holds everything an event can change, named as
the program names it:

- the program's `source_id`, the event `sequence`, the last event, and
  `elapsed` time;
- every state whose value, lineage or grounds differ from what the program gave
  it when it loaded, with its grounds written only when they differ from its
  lineage. A value differs when its bits do, so `-0` and `0` differ: the sign
  of a zero is observable, for example through `atan2`. A state the save
  leaves out is recomputed from the source. Lineage leaves out an empty list,
  so `{"value": 3}` is a state with no evidence;
- what events did to the graph since the program loaded: the nodes they
  created (reading and renewal occurrences, rebuilt from their names, and
  commitments with their reason), the relations they added in order (their
  order is observation order), caveats' attention and open commitments;
- commitment bases and grounds, reading streams, decision series, renewals,
  scheduled qualifications, observation, examination, reopening and predicate
  records, and the decision journal;
- the attention budget;
- the identifiers received for `id` parameters, in handle order
  ([Identifiers 0.1](caveat-identifiers-0.1.md)). A save without them, such as
  one made before that profile, holds none;
- the withdrawn observations ([Withdrawal 0.1](caveat-withdrawal-0.1.md)),
  checked against the graph's `withdrawn` relations when restored;
- each permitted commitment's frozen permission record
  ([Permission 0.1](caveat-permission-0.1.md)), checked against the journal's
  `permitted_by` when restored;
- the last event's effects, and its cues by id.

Bindings are not saved. The source's binding expressions are evaluated against
the restored state and graph, like after any event. Changing those inputs can
change the displayed result; recomputing a binding does not establish that its
inputs arose from the program's event history.

Every number is written exactly, `-0.0` included. Store the text `save()`
returns: JavaScript's `JSON.stringify` writes `-0` as `0`, so a save parsed and
encoded again can lose a zero's sign. The
[save-text host example](../kit/examples/agent-evidence/save-text.mjs) writes and
reads that string intact, then restores it and uses its signed zero in a decision.

## Restore validation

A save is data a host stored and may hand back changed. Restoring loads the
source again, which supplies every declaration, and then applies the save. The
save is refused with an error, and never crashes the runtime, when:

- it is for a different program or schema, or has a field this schema lacks;
- a required schema field is missing, including the `states` map, or a present
  state entry lacks its required `value`;
- a present state entry names an undeclared state, or its value is outside the
  declared range or is not a finite number. Individual state entries may be
  omitted: they keep the value, lineage and grounds initialized from source;
- a name in any lineage, relation, record or effect is not declared or created
  evidence, caveat, claim or commitment of the kind its place needs;
- a relation connects kinds of node no event relates. Events add `supports`
  and `opposes` from evidence to a claim, `qualifies` from a caveat to
  evidence, `retains` from a commitment to a caveat, `relies_on` from a
  commitment to evidence and `reopens` from evidence to a commitment, and
  never add `in_context`;
- evidence the save cites is not observed, that is, it has neither a restored
  supporting/opposing relation nor a valid [neutral observation record](caveat-neutral-reveal-0.1.md): evidence a `relies_on` relation relies on
  or a `reopens` relation names as its cause, evidence in any lineage,
  grounds, basis, guard or other provenance record, and withdrawn evidence
  and a withdrawal's reason. Evidence enters these only once observed, and
  the next commitment, qualification or withdrawal that reads it requires
  that;
- a state's grounds cite evidence or caveats outside its saved lineage, or a
  commitment's grounds cite dependencies outside its frozen basis. Grounds
  may be a strict subset; an omitted state grounds field still defaults to
  that state's saved lineage;
- a commitment retains a caveat or relies on evidence that is not in its
  basis. A commitment retains exactly its basis's caveats and relies on
  exactly its evidence, and reading `committed(...)` or `reopened(...)` adds
  what it retains and relies on to that basis;
- a created node's name is not an occurrence of a declared reading stream or
  renewable evidence, or a commitment this program's rules make;
- the graph and the renewals disagree: the graph holds an occurrence
  `EVIDENCE@N` that is not entry N of that evidence's renewals, or the
  renewals list an occurrence the graph does not hold, out of order, or more
  occurrences than the declared limit;
- the graph and a reading stream disagree: the graph holds a reading
  `STREAM@N` that is not entry N of that stream's occurrences, or the
  occurrences list a reading the graph does not hold, or out of order;
- a history's template or limit differs from the program's, holds more than its
  limit, has a revision without a basis, or a current entry that is not its
  latest;
- the attention budget does not add up to the program's;
- the decision journal disagrees with the graph's commitment/reopening order,
  the frozen grounds or numeric basis, the revision chain, or the observed
  evidence and declared caveats it cites. A committed entry's `because` and
  `caveats` must equal the commitment's frozen grounds exactly: the strict
  subset allowed above relates grounds to their lineage or basis, never a
  journal entry to the grounds it records;
- a journal entry has an impossible event/sequence relationship, unreachable
  decision effect, or inconsistent optional elapsed time. Clocks whose source
  admits negative `dt` are not incorrectly treated as monotonic;
- its `sequence` is past 2^53 - 1 (9007199254740991), or a reading, journal
  entry or withdrawal is dated 0 or past the `sequence`. The sequence counts
  accepted events, and at ten million a second 2^53 of them take over 28
  years. Records bound the sequence only from below, since an event can leave
  none (a `tick`), so the bound above is this constant, the largest integer a
  JSON host reads exactly. A restored session numbers its next events like
  any other, past the bound too: a session resumed from an edited save at or
  near the bound plays on, but its saves past the bound are refused.

Journal `elapsed` and `value` fields are optional for saves written before
those fields were introduced. Historical caveats may be a strict subset of
current caveats: later qualification does not rewrite history.

The runtime tests alter a saved game at random, 3,000 times on every run and
30,000 when asked, and remove each of its records in turn. Each altered save
must be refused or accepted, and an accepted one must then run events,
snapshots, views and saves again without a crash or a fatal outcome. The tests
can list a fatal outcome that is a known runtime bug with its own fix under
way, by its event and exact message with the fix it waits for, and skip only
what they list; they list none now. Each entry also names its witness, the
altered save that reproduces it, and a test fails when a listed fatal outcome
no longer happens on its witness, so an entry is removed once its fix is in.

## Host trust boundary

A save is not signed. Restore checks the names, kinds, ranges and cross-record
relationships listed above; it does not authenticate the historical inputs,
guards or source effects that produced the supplied data. Acceptance means
those checks passed, not that the save is the original account of a session.

Even one edited relation can pass. Given declared evidence `sensor` and a
declared caveat `phantom`, a save can accept this added `graph.relations` entry
without any source operation attaching that caveat to that evidence:

```json
["phantom", "qualifies", "sensor"]
```

The relation path checks that its endpoints exist and have the required kinds,
then inserts the edge. It does not establish source-effect reachability for
that qualification. Other save records must still pass their own checks.
After restoration, `carries(sensor, phantom)` can read the edge, and a later
decision using observed `sensor` can retain `phantom` in its grounds. Snapshot
`origin: "live"` identifies a current graph edge, including a restored edge;
it does not authenticate how the edge arose. Unknown endpoints and invalid
endpoint kinds are refused.

Sparse state entries have a similar boundary. Deleting a changed state's
entry can be accepted and restore that state from its source initializer,
while earlier commitments keep their frozen bases and grounds. This differs
from deleting the required `states` map or a present entry's `value`, which
is malformed. Restore does not prove that an omitted entry was also omitted
by the runtime when it wrote the save.

The [restore boundary fixtures](../kit/test/restore-contract.test.mjs) exercise
the single-edge and omitted-state cases with malformed controls, later decisions
and another save/restore cycle. They also contrast the host example's intact
save text with JavaScript normalization that changes a signed-zero decision.

Hosts that rely on an unchanged history must establish the saved text's
integrity and trusted origin outside this format. Preserve the text returned
by `save()` intact, including numeric representations. Mutation tests exercise
whether edited saves are refused or remain playable without crashing; they
do not establish that accepted histories really occurred. The
[restore trust design review](../docs/RESTORE_TRUST_BOUNDARY.md) considers further
validation and host trust mechanisms separately from this contract.

## Changes

- 2026-10-02: restore checks that state and commitment grounds are subsets of
  their saved lineage and frozen basis, for both evidence and caveats. Edited
  saves could add unrelated observed evidence or declared caveats to grounds;
  even matching edits to a commitment's journal passed without this check.
  Such saves are now refused. Narrowed or omitted state grounds, neutral
  observations, and historical grounds frozen before later caveats keep their
  existing meaning.

- 2026-10-01: [Neutral Reveal 0.1](caveat-neutral-reveal-0.1.md) adds an
  optional ordered `observations` record without fabricating graph relations.
  Stance-only saves retain their existing format; new runtimes accept old saves.

- 2026-09-27: restore refuses a save whose graph holds an occurrence of
  renewable evidence that its renewals do not list at its position
  ([#43](https://github.com/WSattazahn/caveat-lang/issues/43)). Such a save
  used to be accepted. Without the entry, the evidence's first occurrence
  stayed current, so its next `renew` generated a name the graph already
  held, and that event was fatal `unclassified`. A renamed occurrence could
  also sit past the declared limit. `save()` lists every occurrence, so only
  an edited save holds one, and saves the runtime writes restore as before.
  Renewals that name an occurrence the graph does not hold were already
  refused.
- 2026-09-27: restore refuses a save whose graph holds a reading that its
  stream does not list at its position, or whose stream lists its readings
  out of order: the gap above, for reading streams, found in the review of
  [#50](https://github.com/WSattazahn/caveat-lang/pull/50). Such a save used
  to be accepted. Without the entry, the stream's next `sample` generated a
  name the graph already held, and that event was fatal `unclassified`. A
  renamed reading could also sit past the declared limit. And an entry had
  only to name evidence, so a stream could list its template, other evidence,
  a reading twice or another stream's reading; the next event that qualified
  or withdrew its latest reading could then be fatal. Entry N of a stream's
  occurrences must now be `STREAM@N`, as entry N of renewals must be
  `EVIDENCE@N`. `sample` always adds the next reading and `save()` lists every
  one, so only an edited save is refused, and saves the runtime writes
  restore as before. Occurrences that name evidence the graph does not hold
  were already refused. Decision series had no such gap: every commitment the
  graph holds needs a journal entry, and each entry must find the revision at
  its position. The save fuzz now also removes records, and an accepted save
  fails it when a later event is fatal, as it does when one crashes, unless
  the tests list that event and exact message as a known bug with its own fix
  under way. Each listed outcome names a save that reproduces it, and a test
  fails when it no longer happens.
- 2026-09-28: a save keeps a state that holds `-0` where the program loads
  `0`, or `0` where it loads `-0`. It compared the two with `==`, which takes
  them for the same number, so it left the state out, and the restored
  session held the loaded zero: its snapshot, and a binding that shows the
  sign, such as `atan2(x, -1)`, differed from the session that was saved. A
  program makes `-0` itself, from `-v`, `w * 0` with `w` negative, `v / -2` or
  `round(-0.4)`, and a host can send it as `-0`, `-0.0` or a negative number
  too small for a double, such as `-1e-400`. The live session had the same
  blind spot, described in
  [Incremental Evaluation 0.1](caveat-incremental-evaluation-0.1.md#changes):
  a binding kept showing the old sign until a restore evaluated it in full.
  Both compare the bits now. A save written before this leaves such a state
  out and restores as it did, to the loaded zero. Every other number a save
  holds, from readings to elapsed time, was already written exactly.
- 2026-09-27: restore refuses a save whose `sequence` is past 2^53 - 1, or
  that holds a reading dated 0 or past its `sequence`. A save at 2^64 - 1 used
  to be accepted, and its next event was fatal `unclassified` ("reactive event
  sequence exhausted"); one a few below it was fatal a few events later. A
  session resumed at 2^53 - 1 still has 2^64 - 2^53 events to number, so
  exhaustion stays unreachable and dispatch is unchanged. A reading's
  sequence was not checked, unlike a journal entry's or a withdrawal's, and
  now bounds the save's from below as theirs do. Saves the runtime writes
  restore as before, except from a session resumed from an edited save at or
  near the bound: it numbers its events past 2^53 - 1, and its saves from
  then on are refused.
- 2026-09-27: restore refuses a save with a relation between kinds of node no
  event relates, a commitment retaining or relying on more than its basis,
  or evidence nothing observes where the runtime relies on it being
  observed, found by the save fuzz. Such a save used to be accepted as long
  as every name was declared or created. Read back as what events write, a
  revision relying on a claim made the decision's next revision fatal
  `unclassified` ("safe must name a declared evidence"), and a retained
  evidence did the same ("bite must name a declared caveat"). Relations
  beyond a commitment's basis changed what its next revision was made on,
  and long enough names took the read of `committed(...)` past the
  provenance limit, which was fatal. Evidence nothing observed, whether
  relied on, in a commitment's basis, a state's lineage or a selection
  guard, made the next commitment on it fatal ("commitment basis includes
  unobserved evidence"), a withdrawal's unobserved reason the next read of
  `withdrawn(...)`, and unobserved withdrawn evidence the next `withdraw` of
  it. Events add only the relations listed above, a commitment's `retains`
  and `relies_on` relations are its basis, and evidence enters a `relies_on`
  or `reopens` relation, a withdrawal or a provenance record only once
  observed, so saves the runtime writes restore as before. A reopening's
  cause had to be observed already, through the decision journal it must
  match.
