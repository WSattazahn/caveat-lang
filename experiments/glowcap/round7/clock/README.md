# Round 7 on an integer clock

A worked example for rc.15's integer clock
([Elapsed 0.1, "Integer clocks"](../../../../spec/caveat-elapsed-0.1.md#integer-clocks)).
Each file is a round 7 author's final Caveat program
(`../runs/C1`–`C4/impl/glowcap.cav`) converted by the authoring guide's
recipe; the round's records, scores and verdicts are not changed or re-read.

The conversion, the same in all four:

- `event tick dt min 0 max 0.1;` becomes `event step dt min 0 max 100;` with
  `clock step every 16 integer;`, and every `on tick` becomes `on step`. Time
  is in milliseconds.
- `after 60` becomes `after 60000`; the 45-second regrowth becomes 45000.
- The effect timers `glow` and `heavy` count milliseconds (30000, 20000), and
  a binding that shows whole seconds divides by 1000.

The runtime test
`round_7_programs_on_an_integer_clock_fade_a_taste_at_exactly_sixty_seconds`
(`runtime/tests/integer_clock.rs`) tastes a mushroom in each program and
steps 62 and 63 ms at a time: the fade applies on the step at which 60,000 ms
have passed, never one later (Version Lab finding F267 was one tick late on
the binary64 clock). The adapters are not converted; a host would send whole
milliseconds and carry the remainder of a 16.67 ms frame, as the guide says.
