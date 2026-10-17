# CAVEAT Replay Save 0.1

**Status:** stable from 1.1.0 under the [compatibility commitment](../docs/COMPATIBILITY.md).

A [state save](caveat-save-0.1.md) holds everything an event can change, and
for a small program that is more than its events. A program with three
components, nine renewable evidences and three decision series saves 8,471
bytes after 24 events: the frozen bases, grounds, journal and graph name the
same evidence many times over. The 24 events themselves take under 800 bytes.
A host that keeps saves in a small store (a 4 KB row, a cookie, an agent's
scratch file) could not keep that session, and its fallback was to save its
own event log and replay it.

A replay save is that fallback, made by the runtime. It holds the events a
session accepted since its program loaded, and restoring it dispatches them
again:

```js
const saved = session.save({ compact: true });   // at most 4,096 bytes when it is a replay save
const resumed = runtime.restore(source, saved);  // takes a state save or a replay save
```

`save()` with no options is unchanged: it writes the state save, byte for
byte as before. `restore` takes either form.

## What a compact save is

`save({ compact: true })` returns the replay save when the session holds one
and it is smaller than the state save, and otherwise the state save that
`save()` returns. A session holds a replay save when:

- it was opened from source, or restored from a replay save; and
- the events it accepted since its program loaded fit in a replay save of at
  most 4,096 bytes.

A session restored from a state save holds none, and its compact save is its
state save. Once a session's accepted events no longer fit, it stops keeping
them, and its compact saves are state saves from then on. Rejected events
change nothing and are not kept.

The choice depends only on the session, so the same session gives the same
compact save every time.

## Format

Schema `caveat-replay-save/0.1`, as JSON:

```json
{"schema":"caveat-replay-save/0.1","source_id":"…","source_sha256":"…",
 "events":[["checks",{"commit":"602bdbec…","result":1}],["decide"]]}
```

- `source_id` and `source_sha256` are the program's, as in a state save
  ([Source digest](caveat-save-0.1.md#source-digest)).
- `events` lists each accepted event in order, as `[event]` when it has no
  parameters and `[event, payload]` otherwise. The payload is one that
  `dispatch` accepts: a typed parameter is written as its position, an
  identifier parameter as its text, and a number as a whole number when it is
  one within 2^53 and otherwise as the runtime writes an `f64`, `-0.0`
  included.

Every field is required, and none other is allowed.

## Restore

Restoring loads the source and sends each event to it in order, through the
same path as `dispatch`. The restored session is the session those events
make: its snapshot, view, state save and compact save are the ones the saved
session gives, and so are the outcomes of later events. Its archive is empty,
as after restoring a state save: the records those events departed were the
saved session's to drain ([Departure 0.1](caveat-departure-0.1.md)).

A replay save is refused with an error, and never crashes the runtime, when:

- it is longer than 4,096 bytes;
- it is not this schema, has a field the schema lacks, or lacks one it has;
- its `source_id` or `source_sha256` is not the loaded source's;
- an entry is not `[event]` or `[event, payload]` with a non-empty payload
  object;
- the program does not accept one of its events, for any reason a dispatch
  would reject it. The message names the event's position.

A text no longer than 4,096 bytes whose `schema` is `caveat-replay-save/0.1`
is read as a replay save. A longer text is read as a state save, unless it
begins `{"schema":"caveat-replay-save/0.1"`, which is refused as too long.

## What it does not do

A replay save is data a host stored and may hand back changed, like a state
save. Restoring it establishes only that the program accepts those events in
that order. It does not establish that the events happened, who sent them, or
that the save came from the session it claims to: restore does not
authenticate a save, and a matching digest is not proof that a save is
genuine ([Host trust boundary](caveat-save-0.1.md#host-trust-boundary)).

Restoring costs what dispatching the events costs, which grows with their
number. The 4,096-byte bound keeps it to at most a few hundred events.

A runtime that predates compact saves restores no replay save; it refuses one
as a state save of an unknown schema. A host that may move a save back to such
a runtime keeps using `save()`.

## Checks

- `runtime/tests/replay_save.rs`: a compact save restores the session that
  wrote it, with identifiers and a signed zero; refused events are not kept;
  a session restored from a state save writes state saves; a compact save is
  never larger than the state save or the bound; each refusal above.
- `runtime/examples/replay_save_sweep.rs`, run by hand rather than in CI,
  drives every tracked program with seeded events and restores every compact
  save that is a replay save, requiring the same snapshot, state save and
  compact save, and the same outcome for the next event. On the change that
  added replay saves it checked 33,384 of them over 161 programs, with no
  difference.
- `kit/test/session.test.mjs`: the kit's `save({ compact: true })` and
  `restore`, and a runtime build without compact saves.
