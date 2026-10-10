# Integration starter

`caveat-lang/starter` puts one program's session behind a host that an
application can call, persist and restart. It is for Node and browsers.
`caveat-lang/starter/node` adds a store in a directory, and the browser store
uses IndexedDB. The starter is new in this candidate, and its interface may
change before 1.0.

It does four things the session alone leaves to you:

- answers each send with separate facts, so a refused or unpersisted event never
  reads as success;
- checkpoints every accepted event: the save, the SHA-256 of the exact source
  text, and the runtime's identity;
- keeps every archive item the session drains until the store has written it
  (see [History integration](HISTORY.md));
- reads the current assessment of a decision from the current view.

Taking an external action is still up to the application. The
[readiness example](../examples/readiness/README.md) runs a whole approval
lifecycle through a starter host.

## A host in Node

```js
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { openHost } from 'caveat-lang/starter';
import { fileStore } from 'caveat-lang/starter/node';

const runtime = await loadRuntimeFromDirectory();
const source = await readFile('assessment.cav', 'utf8');
const host = await openHost({ runtime, source, store: await fileStore('assessment-store') });

const sent = await host.send('observe', { confidence: 80 });
if (!sent.handled) console.error('not handled:', sent.error.kind, sent.error.message);
else if (!sent.accepted) console.log('refused:', sent.rejection.origin, sent.rejection.code, sent.rejection.message);
else if (!sent.durable) console.error('accepted but not stored:', sent.storeError);

if (host.permits('assessment')) {
  // The application decides what to do with an approval in force.
}
await host.close();
```

`openHost` opens the program fresh when the store is empty. Otherwise it
resumes from the store's checkpoint. With [`caveat types`](REFERENCE.md), write
`openHost<Program>(...)`, and `send` takes only the program's events and
payloads.

In a browser, load the runtime with `loadRuntime` from `caveat-lang/session`
and pass `store: await indexedDbStore('assessment')`.

## Reading a send

A send answers each of these questions separately:

| Field | Meaning |
| --- | --- |
| `handled` | The request reached the session. False: the payload cannot be sent as JSON, or the session failed (`error.kind` is `payload` or `fatal`). Nothing changed. |
| `accepted` | The program accepted the event. False: `rejection` holds the origin, code and message ([dispatch outcomes](reference/spec/caveat-dispatch-0.1.md)), and the session is unchanged. |
| `durable` | The store holds a checkpoint at the session's current sequence. Accepted but not durable means a restart would resume from the event before; `flush()` retries. |
| `archived` | No drained archive item is still waiting for the store. An item that waits is in the checkpoint, so it is not lost, and it is retried. |

A decision is a separate question. `host.assessment(name)` reads the current
view. Its `status` is `none`, `in_force` or `reopened`, and it also gives the
current revision, the journal's last change to it, with grounds and caveats,
and `source_sha256`. `host.permits(name)` is true only while the decision is in
force. An approval made earlier never turns a later refused send into a
success. Read `accepted` for the operation and `permits` for the decision.

## What the stores guarantee

Each accepted event is checkpointed before its archive items are appended. A
chunk is appended only after a checkpoint that holds it is written, so the
archive never runs ahead of the checkpoint.

- **A failed checkpoint write** returns `durable: false` and keeps every
  drained item in memory. The store still holds the previous checkpoint.
- **A failed append** returns `archived: false`. The items stay in the
  checkpoint and are appended on the next send, `flush()` or restart. An append
  that wrote and then reported failure can leave a chunk in the store twice.
  The host reads it once, and refuses two different chunks with one number.
- **A restart** resumes at the last checkpoint written. Any items it still
  holds are appended. An event accepted after that checkpoint is not in the
  session, and its send said `durable: false`, or the process stopped before
  the send returned.
- **A store whose archive is ahead of its checkpoint**, such as an older
  checkpoint copied over a newer one, is refused rather than mixed.
- **Other source text** is refused. The runtime restores a save only under the
  source it was made under, and the checkpoint names that source by SHA-256.

`fileStore(directory)` writes `checkpoint.json` through a temporary file. It
syncs that file, renames it over the old one and syncs the directory, except on
Windows, which cannot sync a directory. `archive.jsonl` gets one synced line per
chunk. Opening the store cuts off a torn last line. The kit's tests kill a
process at random points during sends, then check that the reopened store
matches a run that never stopped.

`indexedDbStore(name)` puts the checkpoint and the chunks in IndexedDB
transactions, and asks for strict durability. How durable a completed
transaction is depends on the browser. `memoryStore()` keeps nothing past the
process.

The starter does not claim more than this. A store is as durable as the disk
or browser under it. The SHA-256 values identify the source and archive chunks;
they authenticate nothing. A checkpoint after every event costs one save per
event, which grows with the session (see [History integration](HISTORY.md)).

## Resynchronizing a view

A client that keeps a copy of the view calls `host.resync(sequence)` with the
sequence it holds. The result is `{ resync: false }` while the copy is current,
and `{ resync: true, view }` when it is not. View 0.1 has no delta, so a resync
sends the whole view.

## Reading history

`await host.archive()` returns every archive item the store holds, in order,
then any items still waiting. Pass it to `explain` and `dependents` as
`{ archive }`, as [History integration](HISTORY.md) shows.
