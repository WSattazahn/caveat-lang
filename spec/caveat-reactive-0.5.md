# CAVEAT Reactive Profile 0.5 — observations and decision histories

This profile extends [Reactive Profile 0.4](caveat-reactive-0.4.md) with bounded
reading streams and decision series. Each reached sample receives its own
evidence identity. Each deliberate decision revision receives its own commitment
identity and frozen basis. Programs can repeat an observation and revision
workflow without inventing separate first, second, and third variables.

The runtime remains a generic Rust interpreter. These features do not introduce
self-hosting, arbitrary collections, historical indexing, or rule blocks. Rules
still execute in source order, evaluating each rule's guard against the state
left by earlier rules in the same atomic event.

## Declaring and using histories

```caveat
claim flow_is_mild;
evidence forecast from "morning forecast";
evidence foam_sensor from "beam-observed foam drift";
caveat reading_may_age consequence high;
forecast supports flow_is_mild;
reading_may_age qualifies foam_sensor;

readings flow from foam_sensor limit 128;
decisions navigation limit 128;

fn correction(reading) = -reading * 0.65;
event start;
event inspect speed min -4 max 4;
on start commit navigation because enough using qualified(0, forecast);
on inspect sample flow = speed opposes flow_is_mild;
on inspect reopen navigation because latest(flow);
on inspect commit navigation because enough using correction(latest(flow));

bind reading.text = if(has_sample(flow), number_text(latest(flow)), "Unread");
bind command.text = if(committed(navigation), number_text(latest(navigation)), "No decision");
```

The syntax is:

```text
readings STREAM from EVIDENCE_TEMPLATE limit CAPACITY;
decisions SERIES limit CAPACITY;
on EVENT [when CONDITION] sample STREAM = NUMERIC_EXPRESSION supports CLAIM;
on EVENT [when CONDITION] sample STREAM = NUMERIC_EXPRESSION opposes CLAIM;
latest(HISTORY)
has_sample(STREAM)
on EVENT [when CONDITION] reopen ACTION_OR_SERIES because latest(STREAM);
```

Templates and claims must already be declared with their respective kinds.
History names cannot collide with numeric state, graph symbols, or another
history. Their generated `NAME@...` graph-symbol namespace is reserved too.
Histories are registered before numeric state initialization, so an initializer
can use a guarded default such as `if(has_sample(flow), latest(flow), 0)`.

## A sample is a new observation

Every successful `sample` effect evaluates its numeric expression and creates a
fresh evidence node, even when its number equals a previous sample. Names are
deterministic within a session: `flow@1`, `flow@2`, and so on. The new node copies
the template's canonical source string and receives the source-authored
`supports` or `opposes` relation to the named claim. Template qualifications are
linked to the new node, and transitive qualification of its template caveats and
the claim it bears on is retained.

The recorded value combines the expression's provenance, the sampling guard's
provenance, the new occurrence's evidence identity, and its inherited caveats.
The value, event sequence, source event name, identity, relation, claim, and
provenance are archived together. Taking another sample does not rewrite them.
Contrary readings can coexist in the graph with separate identities and numbers.

Creating an occurrence does not itself mark the evidence template as observed.
`qualified(value, foam_sensor)` and `observed(foam_sensor)` keep their existing
meanings; neither is a hidden alias for the latest reading. Evidence from an
unreached sample is never manufactured. Sampling does not examine a caveat,
spend attention, remove qualifications, or prove the sampled number is true.
An application explicitly authors attention costs or time spent acquiring a
reading, and how that reading produces an estimate.

`latest(flow)` returns the current archived numeric reading with its occurrence
provenance and the stream's current selection dependencies. It errors before
the first reached sample. `has_sample(flow)` is a boolean query for whether a
reading exists, allowing a lazy default without inventing a zero measurement.
It accepts a reading stream, not a decision series. Both reads preserve
qualification tracking through functions, arithmetic, conditions, and text.
Pure functions cannot capture a history; callers pass a `latest(...)` result as
a numeric argument.

## A revision is a new commitment

The first `commit navigation ...` creates `navigation@1`. Further commits to the
declared series require its current commitment to have been explicitly reopened.
A successful revision creates a new commitment node, freezes its numeric
`using` value and provenance, and records the previous revision's identity.
Its inherited caveats receive `retains` edges; its evidence dependencies receive
`relies_on` edges. Earlier nodes, bases, retained caveats, and reopening reasons
remain inspectable.

`reopen navigation because latest(flow)` resolves both the current decision and
the current reading at execution time. Its graph edge names those concrete
identities. The selector accepts only a reading stream. Existing
`reopen ACTION because NAMED_EVIDENCE` remains available. Duplicate reopening
edges remain idempotent; they do not invent a new observation or revision.

For a declared decision series, `committed(navigation)` asks whether a current
revision exists, and `reopened(navigation)` asks whether that current revision is
open. A new revision begins closed even though earlier revisions remain open in
history. These predicates carry the selected revision's basis and applicable
selection/reopening dependencies. Existing singleton commitments retain their
previous behavior; repeating a singleton commit still errors.

`latest(navigation)` returns the current revision's frozen numeric basis, plus
current head-selection dependencies. It errors if no revision exists or its
commit omitted a numeric `using` value. It does not reevaluate the expression
used by an earlier commitment against today's state. This lets navigation use
the exact plan selected at sampling time instead of renormalizing an old sample
against a later clock value.

Appending a revision also depends on the mandatory check that its predecessor
was reopened. The new basis therefore includes that check's provenance: the old
decision basis, reopening witness, and any relevant head-selection dependency.
These are decision-history control dependencies. They do not claim that the new
measurement equals an older measurement or that earlier evidence is now true.
An independently acquired reading can have only its own occurrence basis while
the resulting decision has this broader history. The predecessor's stored basis
is never modified by that union.

Historical commitment identity, numeric basis, provenance, and predecessor links
are immutable. A commitment's openness may still change through an explicit
reopening, with the actual cause retained in the graph.

## Selecting a current value without rewriting history

A qualified guard can govern a decision to leave a stream or series unchanged.
A skipped sample or skipped series commit records that guard in a separate
`selection_qualifications` value. Later `latest`, `has_sample`, or current-series
predicates inherit it. This prevents a qualified decision to keep an older value
from silently losing its basis.

The archived occurrence and commitment base are not changed. A first absent
stream can therefore explain why no sample was taken without claiming an
observation happened. An independent successful replacement clears earlier
selection dependencies, while qualifications actually read by its expression,
guard, or required predecessor check remain in the new value. Skipped reopening
guards use the existing predicate-dependency mechanism for the selected concrete
commitment.

## Snapshot and identity contract

The additive snapshot schema remains `caveat-reactive/0.1`. Existing numeric
values, qualified values, graph symbols, relations, and commitment bases remain.
Generated evidence and commitment names appear in those ordinary graph fields;
`commitment_bases` is keyed by the concrete commitment identity.

New maps have these shapes:

```json
{
  "reading_streams": {
    "flow": {
      "template": "foam_sensor",
      "limit": 128,
      "current": "flow@1",
      "occurrences": [
        {
          "id": "flow@1",
          "ordinal": 1,
          "sequence": 2,
          "event": "inspect",
          "value": 1.2,
          "provenance": {
            "evidence": ["flow@1"],
            "caveats": ["reading_may_age"]
          },
          "relation": "opposes",
          "claim": "flow_is_mild"
        }
      ],
      "selection_qualifications": {"evidence": [], "caveats": []}
    }
  },
  "decision_series": {
    "navigation": {
      "limit": 128,
      "current": "navigation@2",
      "revisions": [
        {"id": "navigation@1", "previous": null, "ordinal": 1, "sequence": 1, "event": "start"},
        {"id": "navigation@2", "previous": "navigation@1", "ordinal": 2, "sequence": 2, "event": "inspect"}
      ],
      "selection_qualifications": {"evidence": [], "caveats": []}
    }
  }
}
```

`sequence` counts successful dispatched events, not wall-clock time. Applications
author their own elapsed-time values when needed. Occurrence names are local to
the session and reproducible by replaying the same source and events; they are
not globally unique identifiers across unrelated runs. Reading a snapshot or
rendering a frame cannot append a history record.

## Bounds and failure

Each declared capacity must be an integer in `1..256`. The sum of all reading
and decision capacities may not exceed 1,024. This bounds additional history
records; static graph declarations continue to use the existing profile rules.
There is no eviction, silent history truncation, or published identity reuse.

The existing finite-number, expression, text, and per-value provenance bounds
remain. A long decision chain may reach its provenance bound before its declared
record capacity; that is an error rather than permission to discard its basis.
Any failure rejects the complete event, including samples, revisions, reopening
edges, numeric updates, attention spending, cues, and head-selection metadata.
An aborted event consumes neither a published sequence nor an occurrence ID.

The standard library additionally provides the ordinary Caveat function
`wrap(value, period) = require(period > 0, value - floor(value / period) * period)`.
It rejects nonpositive periods and preserves its numeric argument provenance.
This brings the library to ten reserved source functions, leaving 118
application functions under the existing combined 128-definition limit.

The non-game [thermostat history example](../examples/thermostat_history.cav)
uses the same sampling, reopening, and revision semantics to revise a control
decision from repeated instrument readings.
