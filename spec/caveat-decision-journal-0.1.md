# CAVEAT Decision Journal 0.1

Caveat decisions already keep their frozen bases, their grounds and their
reopening evidence. A host that wanted to show a decision's history ("trusted
because of the cave mushroom; stopped trusting because of the pool; trusted
again because of the ruin") still had to rebuild it:
- joining decision-series revisions with commitment records;
- ordering evidence by scanning relations, because grounds are sets.

The runtime already knows that history as it happens. This profile publishes
it.

## The journal

`decision_journal` is a list in the snapshot and in the
[view](caveat-view-0.1.md). Each accepted event appends one entry per decision
change it made, in the order it made them:

| Field | Meaning |
| --- | --- |
| `decision` | the declared commitment or decision series (`trust`) |
| `commitment` | the concrete commitment (`trust`, or `trust@2` for a series revision) |
| `change` | `committed` or `reopened` |
| `sequence`, `event` | the accepted event that made the change |
| `elapsed` | source-controlled elapsed seconds at the change; absent in older saved entries |
| `value` | the concrete commitment's frozen numeric `using` value; absent when none was supplied or in older saved entries |
| `because` | for `committed`, the commitment's [grounds](caveat-explanations-0.2.md) evidence; for `reopened`, the evidence that reopened it |
| `caveats` | the caveats of that evidence at that moment |
| `permitted_by` | for a `committed` entry made with `permitted by`, the concrete grant ([Permission 0.1](caveat-permission-0.1.md)); absent otherwise and in older saves |

`because` lists evidence in the order it was **first observed**, not in
alphabetical order. A revision's entry shows what that revision was made on,
not what its predecessor was.

`elapsed` and `value` are additive optional fields. New entries always have
`elapsed`; a reopening retains the commitment's original `value` even when
the live state used to make it has changed. This lets a host display when and
what was decided without maintaining another history. `elapsed` uses the
program's clock, never wall time. Older saved entries remain readable and
keep these fields absent; the runtime does not invent historical values.

The [state-caveat selector](caveat-state-caveats-0.1.md) can reopen a decision
because several grounded observations aged together. One selector execution
adds one entry containing all its newly added witnesses in observation order.

## Guarantees

- **Append-only.** Entries are never edited. A [late
  qualification](caveat-late-qualification-0.1.md) does not rewrite them.
- **Transactional.** A failed event appends nothing.
- **No duplicates.** A reopening that repeats an existing reopening edge is
  idempotent and adds no entry.

## Example

```json
[
  {"decision": "trust", "commitment": "trust@1", "change": "committed", "because": ["absorb_cave"], "caveats": []},
  {"decision": "trust", "commitment": "trust@1", "change": "reopened", "because": ["absorb_pool"], "caveats": []},
  {"decision": "trust", "commitment": "trust@2", "change": "committed", "because": ["taste_ruin", "absorb_ruin"], "caveats": ["tasted_in_dark"]}
]
```
