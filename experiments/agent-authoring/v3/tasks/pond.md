# Skating on the village pond

Write one reactive Caveat program, `pond.cav`, that decides whether the village
rink may open. This is a synthetic policy for an authoring study, not advice
about real ice. All policy belongs in the Caveat source: the host only sends
events, saves and restores sessions, and reads the runtime's output.

## Inputs

| Event | Payload |
| --- | --- |
| `measure` | `cm`: a number in `0..60`, fractions allowed |
| `crack` | none |
| `decide` | none |

An event with an unknown name, a missing or extra parameter, or a parameter
that is not a finite number in its range is refused. So is every event the
rules below refuse. A refused event changes nothing at all: no reading,
relation, decision, journal entry or display value, and the session's
sequence does not advance. The wording of refusal messages is up to you.

## What the program knows

- **Measurements.** Each accepted `measure` drills one hole and records its
  thickness in the reading stream `thickness`, whose evidence is `auger` (a hand
  auger drilled through the ice). The first reading is `thickness@1`, the
  second `thickness@2`, and so on. A reading of 10 cm or more **supports** the
  claim `ice_safe`; a thinner reading **opposes** it. The program accepts at most
  **eight** measurements; a ninth `measure` is refused.
- **The auger's caveat.** Each hole shows the ice at one spot only. Declare the
  caveat `single_hole` with consequence `material`, and qualify `auger` with it,
  so that every thickness reading carries `single_hole`.
- **A crack report.** The first accepted `crack` records that a skater reported
  a crack: the evidence `crack_report` (a skater), which **opposes** `ice_safe`
  and carries the caveat `secondhand` (consequence `low`). A second `crack` is
  refused.

## The decision

The decision series is `rink`, with capacity three.

`decide` commits a new `rink` decision whose value is **the thinnest
measurement taken so far**: the minimum over every accepted measurement, not
just recent ones. The decision rests on exactly those measurements: its grounds
are every thickness reading so far and their caveat `single_hole`. The crack
report is not part of the grounds.

`decide` is refused unless at least three measurements have been taken. It is
also refused while a decision is in force: after a decision, the next `decide`
is accepted only once that decision has been reopened.

A decision whose value is 10 cm or more **opens** the rink; a thinner one
**closes** it. While the rink is open, new evidence against safe ice reopens
the decision:

- a measurement thinner than 10 cm reopens it, because of that new reading;
- the crack report reopens it, because of `crack_report`.

A decision that has been reopened stays reopened until the next `decide`; later
evidence against safe ice adds no further reopening. A decision that closed the
rink is never reopened, so it stays in force for the rest of the session.

Use the runtime's own decisions: a committed decision, `commitment_grounds`, and
its `decision_journal` with its committed and reopened entries, not a history
you keep yourself.

## What to show

Bind all and only these properties on `hud`:

| Property | Value |
| --- | --- |
| `readings` | the number of measurements taken |
| `thinnest` | the thinnest measurement so far, or `-1` before any |
| `cracked` | `1` once a crack has been reported, otherwise `0` |
| `decision` | `"none"` before the first decision; `"review"` while the current decision is reopened; otherwise `"open"` or `"closed"` |
| `frozen` | the current decision's value, or `-1` before any |
| `revision` | the number of decisions made |

Saving a session and restoring it must preserve everything above, and the
restored session must go on to behave exactly as the original would have.
