# CAVEAT Explanations 0.2 — grounds

[Explanations 0.1](caveat-explanations-0.1.md) let a binding cite part of its
lineage. The [glowcap blind round](../experiments/glowcap/RESULTS.md#round-3-blind-change-requests)
showed the same problem one level down, in state and commitments. A request
to restore trust "based on those two glowcaps" cost 71 lines, because:

- a rule's guard enters the lineage of whatever it sets, even when it is skipped;
- `qualified(v, e)` inherits the guard that revealed `e`;
- a new decision revision's basis includes its predecessor's basis and the
  reopening witness.

All three are correct lineage: they answer *what could have influenced this?*
None of them is what the value is *based on*. To keep a designer's "based on
those two" intact, the program needed plain shadow copies of every guard's
state, and the host had to subtract inherited lineage.

This profile adds a second channel. **Grounds** answer *what is this based
on?* Lineage is unchanged.

## Grounds of a state

When `set NAME = EXPR` succeeds, the state's grounds become the grounds of
`EXPR`:

- The rule's `when` guard is **control**, not content. It stays in the
  lineage and never enters the grounds.
- A **skipped** rule changes the target's lineage, as before. It never changes
  its grounds.
- Within `EXPR`, every read supplies grounds rather than lineage:

| Read | Lineage (unchanged) | Grounds |
| --- | --- | --- |
| a state | its lineage | its grounds |
| `qualified(v, e, extra...)` | `e`, its qualifications, the reveal guard, absence dependencies | `e`, the caveats that qualify it or the claims it bears on, and `extra` |
| `observed(e)` | as `qualified` | as `qualified` |
| `examined(c)` | `c`, its qualifications, the examination guard | `c` and its qualifications |
| `committed(x)` / `reopened(x)` | stored basis, reopening, selection and absence dependencies | `x`'s grounds; `reopened` adds the reopening evidence |
| `latest`, `history_*`, `has_sample` | as before | as lineage in this profile |

An initializer's grounds are computed the same way.

### `because` on `set`

```caveat
set trust_basis = support + recovery because recovery;
set note = based_on(count) because nothing;
```

`because CITATIONS` replaces a state's grounds with the grounds of the
citations. As on a binding, every cited evidence and caveat must be in the
**new lineage** of the state. Otherwise the event is rejected atomically:
`state trust_basis cites evidence absorb_cave that its value and conditions never read`.

## Grounds of a commitment

`commit ACTION because REASON using EXPR retaining CAVEATS` records, beside its
existing basis, the grounds of `EXPR` together with the retained caveats. It
does not include the guard, and it does not include a series predecessor's
basis or reopening witness. For a decision series each revision has its own
grounds: `trust@2` is grounded on what `trust@2` was committed with.

## Citations read grounds

`because` citations, on bindings and on `set`, are evaluated for their
grounds. `bind label = "…" when … because recovery` therefore cites what
`recovery` is based on, not every guard that ever touched it. The grounding
check still compares against the full lineage.

## The contract

For every state and commitment, **grounds ⊆ lineage**, both for evidence and
for caveats. By induction, each grounds read above is a subset of the matching
lineage read, and an authored `because` is checked. Grounds therefore never
introduce a dependency. They choose, from the recorded dependencies, the ones
that are content. Nothing is removed from lineage, and audit keeps the
complete record.

## Snapshot

Two additive fields; the schema stays `caveat-reactive/0.1`:

| Field | Shape |
| --- | --- |
| `value_grounds` | state → `{evidence, caveats}` |
| `commitment_grounds` | commitment id (including `name@N`) → `{evidence, caveats}` |

## Limits of this profile

- Within an expression, an `if` or `require` condition is still read as
  content. Put control in a rule's `when` guard, or narrow with `because`.
- Reading streams and decision-series *values* (`latest`, `history_at`,
  `fold_history`) supply their lineage.
- `reopen` has no grounds of its own. Its evidence is already exact:
  `reopened_by` lists it.
