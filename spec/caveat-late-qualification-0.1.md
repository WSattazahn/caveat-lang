# CAVEAT Late Qualification 0.1

Some caveats arrive after the fact. A taste fades from memory. An instrument
turns out to have been miscalibrated. A witness is discovered to have been
elsewhere. When that happens, everything built on that evidence should now be
qualified, except decisions already made: they were made on what was known
then.

```caveat
caveat taste_faded consequence material;
on tick when taste_age >= 60 qualify taste_pool with taste_faded;
```

To record that an observation is no longer stood behind, rather than add a
caveat to it, see [Withdrawal 0.1](caveat-withdrawal-0.1.md).

## Syntax

```text
on EVENT [when CONDITION] qualify EVIDENCE with CAVEAT;
[when CONDITION] qualify EVIDENCE with CAVEAT;        -- as a procedure step
```

`EVIDENCE` must be declared evidence and `CAVEAT` a declared caveat. `qualify`
and `with` are reserved. [Renewal 0.1](caveat-renewal-0.1.md) adds
`qualify EVIDENCE with CAVEAT after SECONDS`, which applies the same effect
once that much time has passed, to the occurrence current when it was
scheduled.

## Meaning

When the effect runs, `EVIDENCE` must already be observed; otherwise the event
fails. Then:

1. The graph gains `CAVEAT qualifies EVIDENCE`, once. From now on every
   `qualified(v, EVIDENCE)` and `observed(EVIDENCE)` inherits the caveat, and
   any caveats that qualify it, as it would a declared qualification.
2. Every **current value** whose evidence includes `EVIDENCE` gains the caveat
   and its qualifications, in its lineage and in its grounds. The rule's guard
   joins the lineage only, as for any effect. Bindings are recomputed after the
   event, so an explanation that cites such a value shows the caveat
   immediately.
3. **What was already recorded is left alone:**
   - commitment bases and grounds;
   - the decision journal;
   - reading archives.

   A decision keeps the caveats its basis carried when it was made.

The effect is idempotent, is reported in `effects` as
`{"kind": "qualify", "evidence", "caveat"}`, and is part of the event's
transaction: a failed event qualifies nothing.

## Why a language needs this

Without it, a late caveat has to be pushed by hand into every value that
counted the evidence. Each of those values also needs a way to know whether it
still counts it. In the [glowcap benchmark](../experiments/glowcap/RESULTS.md)
that cost a plain `epoch` counter per mushroom, four fade rules per mushroom,
and a window test on the recovery accumulator. With `qualify`, the program
states the one fact, "this taste has faded", and the runtime finds every value
that fact touches, because provenance already records the dependency.
