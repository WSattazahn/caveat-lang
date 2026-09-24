# CAVEAT Serve 0.1

`caveat serve PROGRAM` keeps one session of a program open and answers
requests, so that a hook, a test harness or a program in another language can
send events and ask why without writing JavaScript. It speaks JSON lines: one
request per line on standard input, one response per line on standard output.
The session follows [Dispatch outcomes 0.1](caveat-dispatch-0.1.md).

## Starting

The first line the server writes says whether the program loaded:

```json
{"schema":"caveat-serve/0.1","ready":true,"program":"umbrella.cav","runtime":{"reactiveWasmSha256":"…"}}
```

If it does not load, the line is
`{"schema":"caveat-serve/0.1","ready":false,"error":{"kind":"load","message":"…"}}`
and the server exits with status 2.

## Requests

A request is a JSON object with an `op`, an optional `id` of any JSON value,
and the fields its operation takes. Any other field is an error. Blank lines
are skipped.

| `op` | Fields | Result fields |
| --- | --- | --- |
| `dispatch` | `event`, optional `payload` object, optional `snapshot: true` | `outcome` (`accepted` or `rejected`), `origin`, `code` and `message` when rejected, `sequence`, and `snapshot` when asked for |
| `snapshot` | | `snapshot` |
| `explain` | | `report`: the `caveat-explain/0.1` report of the session, listing the events sent since it opened or was restored |
| `dependents` | `of`: evidence, a reading stream or a caveat | `report`: the `caveat-dependents/0.1` report of what rests on it |
| `save` | | `save`: the text of the session's save |
| `restore` | `save`: text from `save` | `sequence`. The restored session replaces the current one. |
| `close` | | none. The server then exits with status 0. |

## Responses

Every response echoes the request's `id`, or `null` when the request could not
be read.

```json
{"id":1,"ok":true,"outcome":"accepted","sequence":1}
{"id":2,"ok":true,"outcome":"rejected","origin":"input","code":"bound_exceeded","message":"chance must be finite and in 0..100","sequence":1}
{"id":3,"ok":false,"error":{"kind":"request","message":"dispatch has no field extra"}}
```

A refused event is a successful request: `ok` is true and the outcome says
what refused it. The session is unchanged, as for every rejection.

`ok` is false in two cases:

- `kind: "request"`: the line was not a valid request. Examples are text that
  is not JSON, an unknown `op`, a missing or extra field, a payload that is
  not an object or holds a number JSON cannot carry, a name `dependents` does
  not know, or a save `restore` cannot use. Nothing changes, and the server
  keeps serving.
- any other kind, usually `fatal`: the session failed and cannot be used. The
  server writes the response and exits with status 1. A host that wants to go
  on starts a new server and can `restore` its last save.

When standard input ends without `close`, the server closes the session and
exits with status 0.

## Limits

One server holds one session of one program. Requests are answered in order,
one at a time. The `explain` report's event list covers only this server's
requests since the session opened or was last restored; the snapshot itself
holds the full history.
