# Change request: add the pier anemometer

The harbour has fitted a second anemometer at the end of the pier, clear of the
harbour wall, and the skipper wants its gusts to count alongside the mast's.
Change `ferry.cav` so that it follows the original task, as amended below.
Everything this change does not mention stays as it was.

## New input

| Event | Payload |
| --- | --- |
| `pier_gust` | `kt`: a number in `0..90`, fractions allowed |

Each accepted `pier_gust` records the strongest gust in the last minute, in
knots, in a new reading stream, `pier_wind`, whose evidence is
`pier_anemometer` (the anemometer at the end of the pier). The first is
`pier_wind@1`, and so on. Like a mast gust, a pier gust of 30 kt or less
**supports** `crossing_safe` and a stronger one **opposes** it. The program
accepts at most **six** pier gusts; a seventh `pier_gust` is refused. Pier gusts
and mast gusts are counted separately.

The pier anemometer has not been checked against the mast's. Declare the caveat
`unverified_pier` with consequence `material`, and qualify `pier_anemometer`
with it, so that every pier gust carries `unverified_pier`.

## Amended decision

- `decide` is refused unless at least two gusts have been recorded in total,
  counting mast and pier gusts together, and at least two waves.
- A decision's value is the strongest gust from **either** anemometer, and its
  grounds are every gust from both anemometers recorded so far, with their
  caveats. Waves and the storm warning are still not part of the grounds.
- While the ferry may go, a pier gust stronger than 30 kt reopens the decision,
  because of that pier gust, just as a strong mast gust does.

## Amended display

- `gusts` still counts mast gusts only.
- Add `pier_gusts`: the number of pier gusts recorded.
- `strongest` becomes the strongest gust from either anemometer, or `-1`
  before any gust.

`hud` then has exactly these properties: `gusts`, `pier_gusts`, `waves`,
`strongest`, `highest`, `warned`, `decision`, `frozen` and `revision`.
