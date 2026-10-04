# Phase cr5

Applied after every earlier phase; cumulative.

- **CR5 — a taste fades.** Sixty seconds after a taste (counted in ticks since
  that taste), it has faded. The tasted mushroom is labelled
  `Probably a glowcap (taste has faded)` / `Probably a duskcap (taste has
  faded)`, which wins over the dark label, with the same `canAbsorb` and
  `canTaste`. The taste evidence now carries the caveat `taste_faded`, so
  every view citing it includes that caveat. The trust decision keeps the
  caveats its basis carried when it was committed; a later fade does not
  change them.
