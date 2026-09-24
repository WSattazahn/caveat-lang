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
the length of the game. A restored session is the session that was saved: its
full snapshot is equal, and every later event is accepted or rejected, and
changes it, exactly as it would have in the original.

## What a save holds

Schema `caveat-reactive-save/0.1`, as JSON. Empty parts are left out. It holds
everything an event can change, named as the program names it:

- the program's `source_id`, the event `sequence`, the last event, and
  `elapsed` time;
- every state whose value, lineage or grounds differ from what the program gave
  it when it loaded, with its grounds written only when they differ from its
  lineage. A state the save leaves out is recomputed from the source. Lineage
  leaves out an empty list, so `{"value": 3}` is a state with no evidence;
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
- the last event's effects, and its cues by id.

Bindings are not saved. They are computed from the state when a session is
restored, like after any event, so a save cannot make a program show something
its rules do not.

## Restore validation

A save is data a host stored and may hand back changed. Restoring loads the
source again, which supplies every declaration, and then applies the save. The
save is refused with an error, and never crashes the runtime, when:

- it is for a different program or schema, or has a field this schema lacks;
- a state is missing, extra, out of its declared range or not a finite number;
- a name in any lineage, relation, record or effect is not declared or created
  evidence, caveat, claim or commitment of the kind its place needs;
- a created node's name is not an occurrence of a declared reading stream or
  renewable evidence, or a commitment this program's rules make;
- a history's template or limit differs from the program's, holds more than its
  limit, has a revision without a basis, or a current entry that is not its
  latest;
- the attention budget does not add up to the program's;
- the decision journal disagrees with the graph's commitment/reopening order,
  the frozen grounds or numeric basis, the revision chain, or the observed
  evidence and declared caveats it cites;
- a journal entry has an impossible event/sequence relationship, unreachable
  decision effect, or inconsistent optional elapsed time. Clocks whose source
  admits negative `dt` are not incorrectly treated as monotonic.

Journal `elapsed` and `value` fields are optional for saves written before
those fields were introduced. Historical caveats may be a strict subset of
current caveats: later qualification does not rewrite history.

The runtime tests alter a saved game at random, 3,000 times on every run and
30,000 when asked. Each altered save must be refused or accepted, and an
accepted one must then run events, snapshots, views and saves again without a
crash.

A save is not signed. These checks establish internal consistency, not proof
that historical inputs or guards really occurred. Coordinated edits to mutually
consistent records can still be accepted; authentication would require a
different trust mechanism. Mutation tests exercise the requirement that an
edited save is refused or remains playable without crashing.
