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
| `because` | for `committed`, the commitment's [grounds](caveat-explanations-0.2.md) evidence; for `reopened`, the evidence that reopened it |
| `caveats` | the caveats of that evidence at that moment |

`because` lists evidence in the order it was **first observed**, not in
alphabetical order. A revision's entry shows what that revision was made on,
not what its predecessor was.

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
