# CAVEAT Neutral Reveal 0.1

Consulting a source does not by itself support a claim. This profile adds an
observation without a stance:

```caveat
evidence memory from "the memory lookup";
caveat stale consequence material;
event consult;
event learn_stale;
on consult reveal memory;
on learn_stale qualify memory with stale;
```

The evidence starts unobserved. `consult` makes `observed(memory)` true without
creating a `supports` or `opposes` relation. The later qualification is valid
because the source was actually observed. No fabricated trustworthiness claim
is needed.

## Syntax and meaning

`reveal EVIDENCE` is an event-rule effect or a procedure step, including a typed
`evidence` procedure parameter. The existing `reveal EVIDENCE supports CLAIM`
and `reveal EVIDENCE opposes CLAIM` forms keep their meaning. This profile does
not add neutral `sample`, neutral declaration relations, or a new decision state.

- A successful first reveal records the current evidence occurrence as observed.
  It adds no claim, stance relation, reading value, or new occurrence ID.
- Revealing already observed evidence neutrally is a no-op. Adding a stance
  later adds its real relation, without changing the first-observation order.
  A neutral reveal after a stance also adds nothing.
- Observation enables the existing `qualified`, `qualify`, `withdraw`,
  `carries`, and explicit `reopen ... because` rules for that occurrence. It
  does not automatically reopen a decision or make an assessment succeed.
- Revealing guards and inherited procedure-call dependencies join observation
  lineage, not the grounds of values subsequently qualified by that evidence.
  Skipped guarded effects retain control dependencies using the existing
  `observed` predicate machinery.
- Declaring evidence, taking a reading from its template, and renewing it still
  do not observe the template/current occurrence. A renewal starts unobserved;
  earlier occurrences remain observed with their original identities.
- Late and scheduled qualification, withdrawal, frozen decision grounds and
  archived readings follow their existing profiles. Neutrality removes no
  caveats and does not change which identity a value depends on.
- A neutral source can be a grant only when the author explicitly names it in
  `permitted by E`, whose existing rule requires observed, unwithdrawn evidence.
  Observation itself neither grants authority nor authenticates permission.
- Observation is transactional: a rejected event publishes no observation,
  relation, provenance, binding change, or consumed ID.

The observation-order check recognizes both reveal forms. Uses that certainly
precede any possible observation still fail at load time. A neutral reveal
provides an honest on-ramp; it does not relax observed-before-qualify.

## Snapshot, effect, and save representation

After a session first records a neutral observation, snapshots and saves include
`observations`, an ordered list of evidence occurrence names. It contains all
observed evidence, including previously observed declarations and readings,
exactly once in first-observation order. Subsequent samples and first stance
reveals append their occurrences. Renewal alone appends nothing.

Until a neutral observation is needed, this field is omitted and observation
membership/order is derived from the existing supporting/opposing relations.
Thus stance-only programs and their old saves retain their representation.
The separate `observation_qualifications` map holds control provenance; its
membership is not proof of observation, because renewal may record a guard
before the new occurrence is observed.

A first neutral reveal reports:

```json
{"kind":"reveal","evidence":"memory"}
```

A stance reveal retains `relation` and `target`. Both fields are absent for a
neutral reveal; exactly one present is invalid. An idempotent reveal reports
no new effect.

The save schema remains `caveat-reactive-save/0.1` with an optional observation
field. New runtimes accept old saves; old runtimes need not accept saves using
this extension. An explicit record must have unique known evidence names,
include every stance-observed occurrence, and preserve the initial declared
observations as its prefix. Reading/renewal identities and journal/ground
references retain their existing consistency checks. Neutral-only records must
be consistent with the source's possible neutral effects. Missing observation
proof cannot authorize otherwise unobserved evidence cited by a saved record.

A neutral observation can precede its stance: `[B, A]` is valid for neutral B,
then supporting A, then supporting B, even though the stance edges are ordered
A then B. Restoration must use observation order for journal evidence ordering,
not reorder the ledger by when the later stance was asserted.

These are consistency checks on unsigned saves, not authentication of the
reported historical events or guards.

## Explanations and dependents

`explain` includes observed evidence without a stance. Its structured entry has
`relation: null` and `claim: null`; text says `observed (no stance)`. It includes
current direct caveats and withdrawal status. A later real stance is shown as
that relation, without an extra synthetic neutral relation. Neutral observations
have no invented numeric value, event timestamp, or reading sequence.

`dependents` follows the actual grounds and lineage, so values and decisions
based on neutral evidence are discoverable like any other evidence. Neither
report infers support, opposition, confidence, or external authority.
