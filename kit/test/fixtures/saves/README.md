# Save fixtures, one set per published release

[COMPATIBILITY.md](../../../../docs/COMPATIBILITY.md) promises that a save an
earlier release wrote restores under the same source in a later runtime, and
that the events after it give the outcomes the earlier release gave. These
fixtures are that promise's per-release check, starting with rc.17.

- `programs/` holds the sources. A save binds its exact source, so these are
  copies and never change: `readiness.cav` is `kit/examples/readiness`,
  `frost.cav` is the worked example's program, and `ordered.cav` was written
  for these fixtures (evidence observed out of name order, a withdrawal, a
  late qualification and departures).
- `events/` holds, for each program, the events played before the save and
  the events played after restoring it.
- `VERSION/` holds what that published release gave: the save, the archive it
  drained, each later outcome and View 0.1, and the final View 0.2 and
  explanation. `scripts/save-fixture.mjs --package=DIR` writes it from the
  release installed at DIR; nothing edits it by hand.

`kit/test/save-fixtures.test.mjs` restores every save with the runtime under
test and requires the same outcomes (origin and code, not message text), the
same View 0.1 and View 0.2 bytes, and an explanation that contains the
recorded one. With the drained archive, `dependents` of every evidence name
and departed record must give the report the same events give when played
live. One exception is View 0.2 in releases before rc.18, which sorted
the evidence in binding explanations. rc.18 lists that evidence in
first-observed order instead. For those releases, View 0.2 is compared with
the evidence lists sorted, so every other byte must still match that release.
Its exact bytes, order included, must also match the same events played live
by the runtime under test, so a restore that loses the order fails. Each release adds its directory once it is published.
