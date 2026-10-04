# Counting explanation drift

Seven independent programs implement the same game feature (`BEAT.md`, then
the change requests in `REQUESTS.md`): `C1`–`C4` in Caveat (a `.cav` program
plus `adapter.mjs`), `T1`–`T3` in TypeScript. For each program and each of the
four last change requests, CR13–CR16, this directory holds the diff that
change made (`<program>/<phase>.diff`) and the program's files after it
(`<program>/<phase>/`).

**Explanation drift** is, for one program and one change request, the number of
*explanation-producing units* the diff touched.

- A **unit** is one Caveat binding (`bind`), rule (`on …`), state, decision,
  `define`, `proc` or `fn` declaration, or one function in an `adapter.mjs`;
  or one TypeScript function, or one object literal or class that is not inside
  a function.
- A unit is **explanation-producing** if its output feeds, directly or through
  other units, any of these view fields: `because`, `supportedBy`,
  `contradictedBy`, `basis`, `reopenedBy`, `caveats`, `why`, `history`, or a
  journal entry's `because` or `caveats`.
- A unit is **touched** if the diff adds, removes or changes at least one
  non-blank, non-comment line inside it. A unit the change adds counts.
  Units in the `C`/`T` repetition or loop bodies count once per declaration in
  the source, not once per mushroom.

Apply the definition the same way to every program. Do not judge which
language is better and do not count anything else.

Write `DRIFT.json`: for each program and phase, the list of touched
explanation-producing units, as
`{ "program": "C2", "phase": "cr13", "units": [{ "file": "glowcap.cav", "name": "…", "lines": "120-131", "feeds": "belief.supportedBy" }], "count": N }`,
and `DRIFT.md`: a table of counts (programs × phases, with a total per program)
and a short note on any unit you were unsure about.

Your final answer: the table.
