# CAVEAT Reactive Profile 0.6 — computation over retained history

This profile extends [Reactive 0.5](caveat-reactive-0.5.md) with read-only,
bounded history queries and source-defined reductions. Programs can compare
observations, summarize recorded ranges, and compute control policies without
putting those algorithms in the Rust interpreter or browser.

```caveat
fn add_reading(total, reading) = total + reading;
state recorded_mean = 0;
event summarize;
on summarize when history_count(temperature) > 0 set recorded_mean =
    fold_history(temperature, 0, add_reading) / history_count(temperature);
bind trend.text = "Need two readings";
bind trend.text = if(latest(temperature) > history_at(temperature,
    history_count(temperature) - 2), "Rising", "Not rising")
    when history_count(temperature) >= 2;
```

Here `temperature` must already be a declared reading stream. Every operation
also accepts a declared decision series; its numeric records are the frozen
commitment bases. A history name is a literal identifier, not a numeric state
alias or a dynamically constructed string.

## Count, index, and fold

| Expression | Result |
| --- | --- |
| `history_count(HISTORY)` | Number of reached observations or committed revisions, including zero. |
| `history_at(HISTORY, INDEX)` | Numeric value at a zero-based index, oldest first. |
| `fold_history(HISTORY, INITIAL, REDUCER)` | Left fold in archive order using a pure Caveat function. |

An index must evaluate to a finite integer in `0..256` and identify an existing
record. A negative, fractional, or missing index errors; no wrapping, clamping,
invented zero, or implicit latest-value fallback occurs. The index may be a
qualified expression. `latest` retains its existing behavior.

The reducer is the name of an ordinary source function with exactly two numeric
parameters and a numeric result. Its parameters receive the accumulator and
the next record's value. It may call other pure source functions, including
`min`, `max`, and `clamp`. It cannot capture mutable state, query history or the
graph, recurse, or perform effects. Reducer type and purity checks apply even
when the history is empty or the containing branch is not evaluated.

The initial expression is evaluated once. An empty fold returns it without
executing the reducer. Otherwise the reducer is applied once per reached
record, in oldest-to-newest order. Every record is read eagerly, including when
the reducer ignores its second parameter or returns a constant. For example:

```caveat
fn accumulate(total, reading) = total + reading;
fn sum_squares(total, reading) = total + reading * reading;
// Guard these expressions with has_sample(temperature) where appropriate.
// fold_history(temperature, 0, accumulate) computes a recorded sum.
// fold_history(temperature, latest(temperature), min) computes a recorded minimum.
// fold_history(temperature, latest(temperature), max) computes a recorded maximum.
```

These calculations are author-selected policies. The interpreter supplies
iteration and qualification tracking, not a built-in mean, trend, confidence
score, or rule for what old evidence should imply about the present.

## Qualifications and immutable archives

- A count carries the union of the reached records' provenance and the history's
  current selection qualifications. This conservatively records why those
  observations or decisions are members of the current history.
- An indexed read carries the chosen record's stored provenance, the index
  expression's provenance, and the current selection qualifications. A literal
  index does not automatically read every other record. An index derived from
  `history_count` does inherit that count's broader dependencies.
- A fold retains its initial expression, the qualified count, and every record
  actually traversed. Ignored reducer inputs cannot remove these dependencies.
  The same metadata survives subsequent arithmetic, conditions, text, and
  commitment bases.
- A skipped qualified sample or revision can explain an unchanged history or an
  empty count. Queries retain that selection metadata without marking the
  source template observed or manufacturing an occurrence.

Querying does not rewrite archived numbers, sources, caveats, predecessor links,
or earlier commitment bases. Queries neither reopen a decision nor create a
revision. A decision without a numeric `using` value can be counted, but selecting
or folding over that record errors rather than treating it as zero.

Historical calculations describe the selected evidence. Averaging does not
remove calibration uncertainty, make old readings current, or establish truth.
Contradictory graph relations remain separate even if their numeric values
cancel in a sum. Programs choose how to use a summary and retain its caveats.

## Work bounds and atomic failure

The existing maximum of 256 records per history and 1,024 total declared record
capacity remains. The existing parse, nesting, function expansion, finite-value,
text, and provenance bounds also remain. There is no unbounded loop or arbitrary
collection allocation.

Each expression evaluation permits at most 65,536 evaluated AST nodes, shared
across its nested expressions and all reducer iterations. The reducer body is
expanded once during compilation and reused; a fold does not generate one AST
copy per record. This work limit applies to every expression, including those
without folds. The snapshot schema remains `caveat-reactive/0.1`; these queries
add no mutable snapshot fields.

Invalid indexing, reducer requirements, nonfinite results, missing numeric
decision bases, work limits, or provenance overflow reject the complete event.
Earlier effects in that event—including samples, revisions, graph edges, state,
attention, and cues—are rolled back. Failed events consume no published IDs.
Initializers and binding expressions use the same rules; a failed binding also
prevents the event from publishing. No history is evicted to make a query pass.

## Applications and boundary

The [thermostat example](../examples/thermostat_history.cav) computes recorded
mean, range, and change with Caveat functions. Its default heating policy still
uses the latest observation. A test changes only the source policy to use the
recorded mean and verifies a different command with identical sensor archives.

Light the Way compares two archived current measurements to show a reversal
message. A coarse weather warning alone cannot supply that measured conclusion.
The comparison's display text retains both readings and their caveats; the
physical current remains independent of the navigator's observations.

This extends source expressiveness. It does not claim a net reduction in Rust
or JavaScript lines, general-purpose collections, modules, or self-hosting.
Rust remains the generic interpreter and graph runtime; the browser remains
the input, audio, DOM, and graphics adapter.
