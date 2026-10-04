# CAVEAT Interface 0.1

A program's interface is what a host can send it and what it can show: its
events and their payload fields, its states, its displayed values and their
types, its cues, its histories, and the names of its evidence, caveats and
claims. It is read from the loaded program. Nothing is evaluated beyond what
loading the program already evaluates, and the interface is the same before
and after any events and after a restore.

It exists so that a host does not spell these names and shapes by hand. Glowcap
round 7's adapters were 95 to 113 lines per program, and most of their blind
failures were there (`experiments/glowcap/RESULTS.md`). The interface is the
input for generated host declarations; it changes nothing a program means or
does, and nothing in the [view](caveat-view-0.1.md) or the snapshot.

## Getting it

| Where | Call |
| --- | --- |
| WASM | `WebReactiveSession.interface(source)` returns the JSON text. |
| Rust | `reactive::interface_source(source)`, or `session.interface()` on a loaded session. |
| Native CLI | `caveat --interface PROGRAM` prints the JSON text. |

A program that does not load is that load error, exactly as `new` and
`ReactiveSession::from_source` report it, and never a partial interface. The
CLI prints it as `load error: MESSAGE` and exits 1. A bundle, or an entry file
whose imports the CLI resolves, gives the interface of the linked program.

The text is pretty-printed JSON with two-space indentation. For a given
program it is the same bytes from the native runtime and from both WASM builds.

## Shape

```json
{
  "schema": "caveat-interface/0.1",
  "events": [
    {"name": "pick", "parameters": [
      {"name": "bed", "type": "entity", "kind": "bed", "members": ["north", "south"]},
      {"name": "size", "type": "member", "members": ["small", "large"]},
      {"name": "who", "type": "id"}
    ]},
    {"name": "read", "parameters": [
      {"name": "celsius", "type": "number", "min": -40.0, "max": 60.0}
    ]}
  ],
  "states": [{"name": "level", "min": -40.0, "max": 60.0}],
  "bindings": [
    {"target": "gauge", "property": "label", "type": "text", "always": false},
    {"target": "gauge", "property": "value", "type": "number", "always": true}
  ],
  "cues": [{"name": "ping", "kind": "sound"}],
  "decisions": [{"name": "cover", "limit": 4}],
  "readings": [{"name": "soil", "evidence": "probe", "limit": 12}],
  "evidence": ["grant", "probe"],
  "caveats": ["uncalibrated"],
  "claims": ["frost_risk"]
}
```

Fields appear in this order. Every list is in name order (bindings by target,
then property), except an event's parameters, which keep their declared
order.

| Field | What it lists |
| --- | --- |
| `events` | Every declared event, with its parameters: the fields of its JSON payload. |
| `states` | Every state, with the bounds a value must stay within. A state declared without bounds has `-1e12` and `1e12`. |
| `bindings` | Every bound `TARGET.PROPERTY`, after `for` blocks expand, with its value type. |
| `cues` | Every declared cue and its kind: `sound`, `toast`, `flash` or `ring`. |
| `decisions` | Every decision series and how many revisions it keeps. |
| `readings` | Every reading stream, the evidence its readings come from, and how many it keeps. |
| `evidence`, `caveats`, `claims` | The names the program declares, including a `withdrawn` caveat the runtime declares for a program that withdraws. Occurrences that events create later (`soil@1`, `NAME@2`) are not listed. |

### Parameters

A parameter's `type` says what its payload field takes
([typed parameters](caveat-typed-parameters-0.1.md),
[identifiers](caveat-identifiers-0.1.md)):

| `type` | Payload | Other fields |
| --- | --- | --- |
| `number` | A number within the bounds. | `min`, `max` |
| `entity` | The name of an entity of that kind, or its position from 1. | `kind`, `members` in position order |
| `member` | One of the names, or its position from 1. | `members` in position order |
| `id` | Text, learned while the program runs. | none |

### Bindings

`type` is `number`, `boolean` or `text`, the type every declaration of the
property shares; a program whose declarations disagree does not load.
`always` is true when some declaration of the property has no `when`
condition (or `when true`). Such a property is in every view. Otherwise a view
shows it only while one of its conditions holds.

## Not in 0.1

The interface does not list rules, procedures, functions, `define`s,
constants, controls, the clock, commitments made without a decision series,
initial values or anything an event sets. It does not describe the view's
shape, which [View 0.1](caveat-view-0.1.md) fixes. A new field is a new
version of this profile.

## Changes

- 2026-10-04: first version (rc.13).
