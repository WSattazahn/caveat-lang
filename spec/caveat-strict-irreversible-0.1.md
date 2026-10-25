# CAVEAT Strict Irreversible 0.1 (draft) — an irreversible commit that reads its whole basis

**Status:** draft for 1.4, not yet under the [compatibility commitment](../docs/COMPATIBILITY.md). It adds to [Foreclosure 0.1](caveat-foreclosure-0.1.md) and
changes no program that loads today: the new word only appears in a form
that does not load in 1.3.

[Foreclosure 0.1](caveat-foreclosure-0.1.md) refuses an irreversible commit
whose **grounds** carry an unretained caveat of consequence `material` or
worse. It reads only the grounds, by design: "The guard, a `permitted by`
grant and other lineage are not grounds and are not read." A commit can
still depend on caveated evidence that is not in its grounds. The clearest
case is a guard:

```caveat
caveat self_reported consequence material;
self_reported qualifies customer_email;
decisions payout limit 1 irreversible;
on read_email set support = support + qualified(3, customer_email);
on pay when support >= 3 commit payout because enough using 40;
```

The payout cannot happen without the customer's unchecked email, yet its
grounds are only `40`, so it is accepted. The record shows the email in the
payout's lineage, and the [outcome review](caveat-review-0.1.md) reports it as
decisive, but nothing stopped the payment. For an agent this is where a
tainted input slips through: evidence that only opens a gate is never asked
about.

Reading the whole lineage by default would refuse commits that 1.x accepts,
which the [1.x promise](../docs/COMPATIBILITY.md#versioning-policy) does not
allow. This profile adds it as a word an author opts into:

```caveat
decisions payout limit 1 irreversible strict;
```

## Syntax

```text
decisions NAME limit 1 irreversible [strict] [forecloses ACTION, …] [reopened by TRIGGER, …];
```

`strict` comes straight after `irreversible`. It is read only in that
position, so it reserves nothing: a series, stream, evidence or state named
`strict` keeps loading, and so does a module of a draft 0.5 bundle that
declares it ([Caveat 0.5](caveat-0.5-draft.md)); its rewriter protects the
word by position, as it does `changed` and `hold`.

These do not load:

| Source | Message |
| --- | --- |
| `strict` without `irreversible` straight before it, such as `limit 4 strict` or `limit 1 strict irreversible` | `strict follows irreversible: decisions NAME limit 1 irreversible strict` |
| `strict` after `forecloses` | as for any other word there: `forecloses takes decision names separated by commas, not …` |

A declaration without `strict` keeps every message byte for byte.

## Meaning

An executed `commit` of a strict irreversible series is refused as
`evaluation/caveated_irreversible` when its **lineage** carries a caveat of
consequence `material`, `high` or `catastrophic` that its `retaining` clause
does not name. The lineage is the commitment's recorded basis: its grounds,
and its **control**, everything else the commit read. That covers the rule's
`when` guard, a `permitted by` grant and the reveal guards of what it read
([Explanations 0.2](caveat-explanations-0.2.md)). An irreversible series has
one revision, so no predecessor's basis is involved.

Everything else is as in Foreclosure 0.1:

- the code, origin and message form, which names each caveat in grounds
  order, then each lineage-only caveat:
  `irreversible payout rests on self_reported (material), which its commit does not retain`;
- `negligible` and `low` caveats are not read;
- `retaining C` decides with `C` attached, on purpose, and `C` joins the
  grounds;
- the order of checks: `foreclosed` first, then the `permitted by` check, then
  this one;
- foreclosure, reopening and the one-revision limit.

`strict` adds no refusal code and no field. The snapshot's `irreversible`
map lists a strict series exactly as a plain one, and saves, views and
`explain` are unchanged. A save binds its source's SHA-256, so a save made
without `strict` does not restore onto a source that adds it.

## Questions settled

1. **Opt-in, not a default.** Refusing a commit that 1.3 accepts is a 2.0
   change ([Compatibility](../docs/COMPATIBILITY.md#versioning-policy)), and
   Foreclosure 0.1 documents the guard as not read, so the default is not a
   broken promise that a patch could fix. A program opts in per series. Making
   `strict` the default is a candidate for 2.0.
2. **Lineage, not only the guard.** A `permitted by` grant carries caveats as
   well: an approval sampled from a rushed reviewer is as much a basis as a
   guard. Lineage is the set the runtime already records and `explain` shows,
   so the rule names no new set.
3. **Per series, not per commit.** Irreversibility is declared on the series,
   so its strictness is too. A rule cannot opt one commit out.
4. **No new code.** The refusal is the same fact (an irreversible commit
   resting on a caveat it does not name) about a wider basis, so it keeps
   `caveated_irreversible`.

## Not in this profile

- Reading lineage for reversible decisions. Their caveats travel with them
  and a withdrawal reopens them, so a refusal adds less. The retractions
  work's static checks in `caveat check` look at that side.
- A `caveat check` warning for an irreversible series without `strict`
  whose guard reads caveated evidence.

## Corpus sweep

`node experiments/strict-irreversible-sweep/sweep.mjs` reads the 200 tracked
programs at 063f072 ([RESULTS.json](../experiments/strict-irreversible-sweep/RESULTS.json)):

- 2 irreversible series, both in `kit/examples/handoff/handoff.cav`
  (`notify_customers`, `ship`); none is written `strict`;
- no declared name is spelled `strict`;
- 3 rules commit an irreversible series, and none has a `when` guard or a
  `permitted by` grant, so declaring either series `strict` would read nothing
  more than its grounds there.

No tracked program changes meaning, and none could load the new form before.

**Programs without `strict` behave byte for byte as in 1.3.** The sweep shows
that no tracked program writes the word, and no 1.3 program can, since the
form does not load there. Without the word, the commit check reads exactly
the caveats it read before, in the same order with the same message, and the
snapshot, save, view and `explain` gain no field. The runtime's existing
tests, which pin these outputs, pass unchanged.

## Implementation

- `runtime/src/reactive.rs`: the declaration parser reads `strict`; the
  session keeps the strict series apart from the `irreversible` map, so the
  snapshot is unchanged; the commit check adds the lineage's caveats for a
  strict series.
- `runtime/src/link.rs`: `tag_offsets` protects `strict` straight after
  `irreversible`; `runtime/tests/modules.rs` lists it as positional and tests
  a module that declares `strict`.
- `runtime/tests/strict_irreversible.rs`: a guard's caveat is not read without
  `strict` and is with it; retaining it commits; a `low` caveat is not read; a
  permission grant's caveat is read; no snapshot field; foreclosure and
  triggers still parse; a save restores with the rule in force; load errors;
  the word reserves nothing.
- `editors/vscode`: `strict` is highlighted as a clause word.

## Changes

- 2026-10-25: drafted for 1.4 and implemented. It came from the outcome
  review demo, which showed a payout that hinged on an unchecked email read
  only by a guard ([Review 0.1](caveat-review-0.1.md)).
