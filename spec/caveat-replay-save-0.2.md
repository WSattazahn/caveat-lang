# CAVEAT Replay Save 0.2

**Status:** stable from 1.2.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.2. It adds to [Replay Save 0.1](caveat-replay-save-0.1.md) and changes nothing that 0.1 writes or restores.

A [replay save](caveat-replay-save-0.1.md) writes each event as its name and
a payload object that names every parameter. For a program whose events carry
several parameters, the names take most of the bytes:

```json
["withdraw",{"check":1,"component":1,"named_check":1,"named_component":1,"named_number":0}]
```

At 4,096 bytes, such a save holds about 45 of those events. A host that keeps
more than that in a small store was back to keeping its own event log and
replaying it.

A positional replay save writes the same event as its position among the
program's events, then its parameters' values in declared order, all events in
one list:

```text
4,1,1,1,1,0
```

When every value in the save is a whole number below 36, as when every
parameter is a typed member or a small count, the list is written as a text of
base-36 digits, one per value:

```json
"411110"
```

It is the same replay save in every other respect: the same events, restored
the same way, with the same bound.

```js
const saved = session.save({ compact: true, positional: true });
const resumed = runtime.restore(source, saved);   // takes any save
```

`save()` and `save({ compact: true })` are unchanged, byte for byte. Asking for
a positional save changes nothing either of them writes.

## What a positional compact save is

`save({ compact: true, positional: true })` returns the positional replay save
when the session holds one and it is smaller than the state save, and
otherwise the state save that `save()` returns. A session holds a positional
replay save when:

- it was opened from source, or restored from either kind of replay save; and
- the events it accepted since its program loaded fit in a positional replay
  save of at most 4,096 bytes written as a list.

Since a positional entry is never longer than the named one, a session holds a
positional replay save whenever it holds a named one, and for longer. The
bound is counted on the list, so a save written as digits stays well under it. Once the
named form no longer fits, `save({ compact: true })` writes state saves as 0.1
says. The positional form keeps the events until they no longer fit in it.

## Format

Schema `caveat-replay-save/0.2`, as JSON:

```json
{"schema":"caveat-replay-save/0.2","sha256":"WSmafxHrAu0aXvQwrI2Te8MCBhKos5hPXXamgSiV_Zw",
 "events":[0,"602bdbec…",2,1]}
```

- `sha256` is the SHA-256 digest of the program's source, the digest a state
  save writes as `source_sha256` in hex, here in base64url without padding (43
  characters). It alone binds the save to its exact source, so the save has no
  `source_id`.
- `events` holds each accepted event in order, as `E, VALUE, …`, with nothing
  between one event and the next:
  - `E` is the event's position, from 0, among the program's events in name
    order. That is the order of `events` in the program's
    [interface](caveat-interface-0.1.md).
  - Then comes one value per parameter, in the order the event declares them.
    Each is written as 0.1 writes it in a payload: a typed parameter as its
    position, an identifier as its text, and a number as a whole number when
    it is one within 2^53, otherwise as the runtime writes an `f64`.
  - Each event's position says how many values follow it, so the list needs
    no brackets per event.
- When every value in `events` is a whole number from 0 to 35, `events` is a
  text instead, one lowercase base-36 digit (`0`–`9`, `a`–`z`) per value.
  Otherwise it is a JSON list.

Every field is required, and none other is allowed.

## Restore

Restore is 0.1's: the source is loaded and each event is sent to it in order,
through the same path as `dispatch`. The restored session holds both forms of
replay save for the events it replayed, as far as each form fits.

A positional replay save is refused with an error, and never crashes the
runtime, for each reason 0.1 gives, and when:

- `sha256` is not this source's digest;
- an entry does not begin with the position of one of the program's events;
- the list ends before an event's last parameter;
- a value is not a number or a text;
- `events` is a text with a character that is not a lowercase base-36 digit,
  or neither a list nor a text.

A text no longer than 4,096 bytes whose `schema` is `caveat-replay-save/0.2`
is read as a positional replay save. A longer one is refused as too long when
it begins `{"schema":"caveat-replay-save/0.2"`.

## What it does not do

Everything 0.1 says holds here too. A positional replay save does not
authenticate itself. Restoring it costs what dispatching its events costs, and
it can hold more of them.

A positional save is tied to the program more tightly than a named one.
Adding an event that sorts before an existing one, or reordering an event's
parameters, changes what each position means. A save is restored only with the
exact source that wrote it, so this cannot reach a restore: a changed program
has a different digest, and its save is refused.

A runtime that predates positional replay saves refuses one as a state save of
an unknown schema.

## Checks

- `runtime/tests/replay_save.rs`:
  - a positional save restores the session that wrote it, with an identifier,
    a typed parameter and a signed zero;
  - a save of small whole numbers is a text of digits, restores, and turns
    back into a list once a value passes 35;
  - it outlasts the named save, which stays byte for byte as it was;
  - each refusal above.
- `kit/test/session.test.mjs`: the kit's
  `save({ compact: true, positional: true })` and `restore`, and a runtime
  build without it.
