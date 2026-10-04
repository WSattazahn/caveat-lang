# CAVEAT Renewal 0.1

Evidence is declared, so a program has a fixed set of things it can learn
from. Games do not work that way. A mushroom is eaten and another grows in its
place; an enemy respawns; a sensor is replaced. The new one is a new source of
evidence: what the slime learned from the last mushroom is still true of that
mushroom, and says nothing certain about this one.

Round 6 of the [glowcap benchmark](../experiments/glowcap/RESULTS.md) asked for
mushrooms that regrow, without limit. The language had no way to say it. The
only route was to declare every life as its own entity, `cave_2` to `grove_8`,
which capped regrowth at eight and made every event about seven times slower.
This profile adds the missing pieces:

```caveat
for mushroom as $m {
    evidence taste_$m from "the slime tasted it";
    renewable taste_$m limit 256;
    on taste when $m_here reveal taste_$m supports $m_is_glowcap;
    on taste when $m_here qualify taste_$m with taste_faded after 60;
    on tick when $m_regrows renew taste_$m;

    bind $m.label = "Probably a glowcap (taste has faded)"
        when $m_known == sort.glowcap and carries(taste_$m, taste_faded) because $m_known;
};
```

## Renewable evidence

```text
renewable EVIDENCE limit N;          -- N in 1..1024
on EVENT [when CONDITION] renew EVIDENCE;
```

A renewable evidence name means its **current occurrence**. The first occurrence
is the declared evidence itself. `renew` makes a new one, `EVIDENCE@2`, then
`EVIDENCE@3`, and from then on every rule and binding that names `EVIDENCE`
means that one: `reveal`, `qualified(…, EVIDENCE)`, `observed(EVIDENCE)`,
`qualify EVIDENCE with …`, `reopen … because EVIDENCE` and
`carries(EVIDENCE, …)`.

- A new occurrence is **unobserved**, and inherits the caveats *declared* on
  the evidence (`secondhand qualifies witness_cave;`). It does not inherit
  caveats a `qualify` added later: those were about an earlier occurrence.
- Earlier occurrences keep their names, relations and caveats. A belief that
  counted the first taste still counts `taste_cave`, and a later taste adds
  `taste_cave@2`. Nothing merges them, so provenance stays true.
- The rule's guard joins the new occurrence's observation lineage, as a
  sample's does.
- `renew` past the declared limit refuses the event as `limit/renewal_limit`
  ([dispatch outcomes](caveat-dispatch-0.1.md)). Nothing the event did is
  kept, and the session continues. A refused clock event advances no time,
  as with any refusal, so a rule that renews on every clock event stops time
  once the limit is reached. The limit bounds how much graph a program can
  grow, like a reading stream's.
- `effects` reports `{"kind": "renew", "evidence": "taste_cave",
  "occurrence": "taste_cave@2"}`, and `renewals` in the snapshot lists every
  occurrence of each renewable evidence, first to current.

An occurrence's `@N` suffix is generated, so a host that shows evidence to a
player can display `taste_cave@2` as `taste_cave`'s source.

## Scheduled qualification

```text
on EVENT [when CONDITION] qualify EVIDENCE with CAVEAT after SECONDS;
```

[Late qualification](caveat-late-qualification-0.1.md) with a delay. It is
scheduled against the occurrence current **now**, which must be observed, and
it applies once `SECONDS` of time have passed, however often that evidence has
been renewed in between. That is what makes an old taste fade on its own clock
after its mushroom has regrown.

Time is the sum of the `dt` of every accepted event of the program's clock (see
`clock EVENT every STEP`), or of `tick` when there is no clock. The engine still
reads no wall clock: the source decides what time is. A program that schedules a
qualification and has neither is rejected when it loads. On each time event,
time advances first. Every qualification now due then applies, in the order it
was scheduled and exactly as an immediate `qualify` would, before the event's
rules run. It is reported in that event's `effects`. The rule's guard and the
delay's lineage join the lineage of what it qualifies. A decision made before it
applies keeps what it was made on.

A scheduled qualification stores the time reading current when it was
scheduled (`scheduled_at`, a save field). It applies on the first time event
at which the current reading minus `scheduled_at` is at least `SECONDS`,
computed in binary64. Each time event's addition rounds the reading to the
binary64 spacing at its magnitude, by up to half that spacing, so the
difference can fall short of, or exceed, the sum of the `dt` values supplied
since scheduling. The qualification can then apply an event later or earlier
than exact arithmetic over those `dt` values would:

- Scheduled at the reading 28 × 0.1 (`2.800000000000001`), a delay of 60 is
  not yet due after 960 steps of `0.0625`, whose sum is exactly 60. The
  difference is `59.99999999999999`, and the qualification applies on the
  961st step (Version Lab finding F267).
- The spacing grows with the reading: about `1.5e-5` near `1e11`. Scheduled
  at a reading of `1e11`, a delay of 1 with steps of `0.001` applies on the
  993rd step.

While the rounding accumulated since scheduling stays below one step, a
qualification applies at most one event away from exact arithmetic. With
steps that are powers of two and a reading small enough that every addition
is exact, it applies on the event exact arithmetic gives.

`elapsed` and `scheduled_qualifications` appear in the snapshot. At most 4,096
qualifications can wait at once; scheduling another refuses the event as
`limit/scheduled_limit` ([Dispatch 0.1](caveat-dispatch-0.1.md)).

## Asking whether evidence carries a caveat

```text
carries(EVIDENCE, CAVEAT)
```

True when `EVIDENCE` (its current occurrence) is observed and `CAVEAT`
qualifies it, directly, through another caveat, or through the claim it bears
on: exactly when `qualified(v, EVIDENCE)` would carry `CAVEAT`. Its lineage is
the evidence's qualification, whether the answer is true or false. Its grounds
are the evidence and its caveats when it is observed. A label can therefore say
"taste has faded" because the taste carries the caveat, with no second timer in
state that could disagree with the evidence.

## Changes

- 2026-10-04: states when a scheduled qualification applies: on the first
  time event at which the current reading minus `scheduled_at` reaches the
  delay in binary64, which can be an event later or earlier than exact
  arithmetic over the supplied `dt` values (Version Lab finding F267, from
  Glowcap round 7). The runtime is unchanged; rc.10 and rc.11 already behave
  this way.

- 2026-09-26: `renew` past the declared limit is refused as
  `limit/renewal_limit`, and the session continues. Before this it was a fatal
  `unclassified` error, so the host had to discard the session although the
  runtime had already rolled the event back. In a program that keeps one
  renewable evidence per subject, such as the agent ledger's push per pull
  request, one subject's exhausted renewals ended the session for every
  subject. The rollback and the diagnostic text are unchanged.
