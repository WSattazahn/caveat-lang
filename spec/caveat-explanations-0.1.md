# CAVEAT Explanations 0.1 — grounded citations

A binding already carries its **lineage**: `binding_qualifications` holds every
evidence and caveat its value and its candidate conditions read (see
[Reactive 0.3](caveat-reactive-0.3.md)). Lineage is conservative on purpose.
Nothing a presentation depended on can disappear from it.

Lineage is not an explanation. A label that says "Could be a duskcap" depends
on the glowcaps that made the slime trust mushrooms *and* on the duskcap that
broke that trust. The designer wants the label to cite the duskcap. Before this
profile, the only way to get that was to steer lineage: order guards so `and`
short-circuits before reading unrelated state, write validation to a dummy
state, or override the citation in host code. The
[glowcap experiment](../experiments/glowcap/RESULTS.md) measured that cost.

This profile lets the author say what a binding cites, and has the runtime
check that the citation is true.

```caveat
bind pool.label = "Could be a duskcap — taste first"
    when support > 0 and contradiction > 0
    because contradiction;
bind pool.label = "" when consumed == 1 because nothing;
```

## Syntax

```text
bind TARGET.PROPERTY = VALUE [when CONDITION] [because CITATIONS];
CITATIONS := nothing | EXPRESSION {, EXPRESSION}
```

`because` is the last clause. It is recognised only outside quoted text and
parentheses, so `"because of the rain"` stays text. Each citation is an
ordinary expression of any type; only its provenance is used. `nothing` cites
nothing and is reserved. A missing citation list, an empty item between commas,
or an unknown name is rejected before execution.

## Meaning

When a declaration supplies a shown value (the last matching one, as before),
its citations are evaluated against the same state. Their union is the
binding's **explanation**, published per target and property in
`binding_explanations`.

- **No `because` clause:** the explanation is the whole lineage. Existing
  programs cite exactly what they did before.
- **`because nothing`:** the explanation is empty.
- **Grounding:** every evidence and caveat in the explanation must also appear
  in the binding's lineage. Otherwise the event is rejected atomically, like
  any other runtime error:
  `binding pool.label cites evidence taste_cave that its value and conditions never read`.
- **Cited values carry their caveats:** citing a value brings the caveats it
  carries. An explanation cannot cite evidence and drop the qualifications that
  travel with it. It also cannot add a qualification the binding never carried.
  For example, `because qualified(1, chart, manual)` is rejected if `manual`
  was not in the lineage.
- **Only the winner is evaluated.** Losing and false candidates never evaluate
  their citations. A binding that no declaration supplies has no explanation.

An explanation may therefore leave dependencies out. It may never introduce
one. That is the honesty contract: **selective, never fabricated.** The full
lineage remains in `binding_qualifications` for audit. Citing less changes
what is shown to the reader, not what the runtime records.

## What this does not claim

The runtime checks that a citation is *grounded*: the binding really read it.
It does not check that the citation is *sufficient*, i.e. that it is the
reason a person would give. The author decides relevance and the runtime
refuses fabrication. Explanations of commitments already exist:
`commitment_bases` holds a decision's frozen basis. Cues keep their existing
`cue_qualifications` in this profile.

## Snapshot

`binding_explanations` has the same shape as `binding_qualifications`:
target → property → `{evidence, caveats}`. It appears only for bindings that
are shown. The schema remains `caveat-reactive/0.1`; this is an additive field.
