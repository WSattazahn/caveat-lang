# May the ferry cross?

Write one reactive Caveat program, `ferry.cav`, that decides whether the island
ferry may make its crossing. This is a synthetic policy for an authoring study,
not advice about real weather. All policy belongs in the Caveat source: the
host only sends events, saves and restores sessions, and reads the runtime's
output.

## Inputs

| Event | Payload |
| --- | --- |
| `gust` | `kt`: a number in `0..90`, fractions allowed |
| `wave` | `m`: a number in `0..6`, fractions allowed |
| `warning` | none |
| `decide` | none |

An event with an unknown name, a missing or extra parameter, or a parameter
that is not a finite number in its range is refused. So is every event the
rules below refuse. A refused event changes nothing at all: no reading,
relation, decision, journal entry or display value, and the session's
sequence does not advance. The wording of refusal messages is up to you.

## What the program knows

- **Gusts.** Each accepted `gust` records the strongest gust in the last
  minute, in knots, in the reading stream `wind`, whose evidence is
  `anemometer` (the mast anemometer at the landing). The first reading is
  `wind@1`, the second `wind@2`, and so on. A gust of **30 kt or less supports**
  the claim `crossing_safe`; a stronger gust **opposes** it. The program accepts
  at most **ten** gusts; an eleventh `gust` is refused.
- **Waves.** Each accepted `wave` records a wave height, in metres, in the
  reading stream `swell`, whose evidence is `buoy` (the harbour buoy). A wave of
  **1.5 m or less supports** `crossing_safe`; a higher wave **opposes** it. The
  program accepts at most **ten** waves; an eleventh `wave` is refused. Gusts
  and waves are counted separately.
- **Their caveats.** The mast stands in the lee of the harbour wall. Declare
  the caveat `sheltered_mast` with consequence `material` and qualify
  `anemometer` with it, so that every gust carries `sheltered_mast`. The buoy
  drifts on its mooring: declare `buoy_drift` with consequence `low` and qualify
  `buoy` with it, so that every wave carries `buoy_drift`.
- **A storm warning.** The first accepted `warning` records that the
  coastguard broadcast a storm warning: the evidence `storm_warning` (the
  coastguard radio), which **opposes** `crossing_safe` and carries the caveat
  `regional_forecast` (consequence `low`). A second `warning` is refused.

## The decision

The decision series is `crossing`, with capacity four.

`decide` commits a new `crossing` decision whose value is **the strongest gust
recorded so far**: the maximum over every accepted gust, not just recent ones.
The decision rests on exactly those gusts: its grounds are every `wind` reading
so far and their caveat `sheltered_mast`. Waves and the storm warning are not
part of the grounds.

`decide` is refused unless at least two gusts **and** at least two waves have
been recorded. It is also refused while a decision is in force: after a
decision, the next `decide` is accepted only once that decision has been
reopened. It is refused once four decisions have been made.

A decision whose value is 30 kt or less lets the ferry **go**; a stronger one
**holds** it in harbour. While the ferry may go, new evidence against a safe
crossing reopens the decision:

- a gust stronger than 30 kt reopens it, because of that new `wind` reading;
- a wave higher than 1.5 m reopens it, because of that new `swell` reading;
- the storm warning reopens it, because of `storm_warning`.

A decision that has been reopened stays reopened until the next `decide`; later
evidence against a safe crossing adds no further reopening. A decision that
held the ferry is never reopened, so it stays in force for the rest of the
session.

Use the runtime's own decisions: a committed decision, `commitment_grounds`, and
its `decision_journal` with its committed and reopened entries, not a history
you keep yourself.

## What to show

Bind all and only these properties on `hud`:

| Property | Value |
| --- | --- |
| `gusts` | the number of gusts recorded |
| `waves` | the number of waves recorded |
| `strongest` | the strongest gust so far, or `-1` before any |
| `highest` | the highest wave so far, or `-1` before any |
| `warned` | `1` once a storm warning has been broadcast, otherwise `0` |
| `decision` | `"none"` before the first decision; `"review"` while the current decision is reopened; otherwise `"go"` or `"hold"` |
| `frozen` | the current decision's value, or `-1` before any |
| `revision` | the number of decisions made |

Saving a session and restoring it must preserve everything above, and the
restored session must go on to behave exactly as the original would have.
