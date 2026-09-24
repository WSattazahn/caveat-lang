# Change request: add the sonar gauge

The village has bought a sonar gauge. It measures ice thickness without
drilling, and the warden wants its scans to count alongside the auger's
measurements. Change `pond.cav` so that it follows the original task, as
amended below. Everything this change does not mention stays as it was.

## New input

| Event | Payload |
| --- | --- |
| `scan` | `cm`: a number in `0..60`, fractions allowed |

Each accepted `scan` records its thickness in a new reading stream,
`sonar_thickness`, whose evidence is `sonar` (a sonar gauge on the ice). The
first scan is `sonar_thickness@1`, and so on. Like a measurement, a scan of
10 cm or more **supports** `ice_safe` and a thinner scan **opposes** it. The
program accepts at most **eight** scans; a ninth `scan` is refused. Scans and
measurements are counted separately.

The gauge has not been calibrated against the auger. Declare the caveat
`uncalibrated_sonar` with consequence `material`, and qualify `sonar` with it,
so that every scan carries `uncalibrated_sonar`.

## Amended decision

- `decide` is refused unless at least three readings have been taken in total,
  counting measurements and scans together.
- A decision's value is the thinnest reading from **either** instrument, and
  its grounds are every reading from both instruments taken so far, with their
  caveats.
- While the rink is open, a scan thinner than 10 cm reopens the decision,
  because of that scan, just as a thin measurement does.

## Amended display

- `readings` still counts auger measurements only.
- Add `scans`: the number of scans taken.
- `thinnest` becomes the thinnest reading from either instrument, or `-1`
  before any reading.

`hud` then has exactly these properties: `readings`, `scans`, `thinnest`,
`cracked`, `decision`, `frozen` and `revision`.
