# Explanation drift, CR17–CR20

This table counts the explanation-producing units each change request's diff touched. The definition is in `README.md`. `programs.txt` is empty, so there are no replacement programs. The per-unit lists are in `DRIFT.json`.

| program | cr17 | cr18 | cr19 | cr20 | total |
| --- | ---: | ---: | ---: | ---: | ---: |
| C1 | 3 | 38 | 15 | 50 | 106 |
| C2 | 9 | 30 | 26 | 123 | 188 |
| C3 | 1 | 80 | 27 | 206 | 314 |
| T1 | 3 | 8 | 16 | 1 | 28 |
| T2 | 3 | 5 | 13 | 4 | 25 |
| T3 | 2 | 9 | 15 | 5 | 31 |

## How the count was made

- The pre-CR17 files were rebuilt by reverse-applying `cr17.diff` to `cr17/`. Each later diff was checked to turn the previous phase's files into the next phase's files exactly. Changed lines were then mapped to units in the old file (removed lines) and in the new file (added lines) (`_work/units.py`, `_work/build.py`).
- A line counts as changed only if its code changed. Comment-only, blank-line and whitespace-only edits were removed before counting. This was done by cancelling identical comment-stripped, whitespace-collapsed lines within each hunk.
- **How units were identified:**
  - **Caveat:** each `state`, `define`, `fn`, `proc`, `decisions`, `on` and `bind` statement is one unit, counted once per declaration. Declarations inside `for … as $m`/`$j`/`$e` blocks also count once each, not once per mushroom.
  - **adapter.mjs:** functions only. Top-level `const` arrow functions and nested named helpers, such as C2's `entry` inside `project`, are units. Anonymous callbacks belong to the function around them.
  - **TypeScript:** named functions (declarations, `const` arrows and object methods), plus top-level object literals.
- A removed rule or binding and an added one count as one edited unit only when they handle the same event with the same action on the same target, or bind the same field in the same case. A renamed `define`/`proc`/`fn` counts as one removed unit plus one added unit, e.g. C1 `proc undoubt` → `set_doubt`, C3 `shift_journal` → `shift_memory`.
- **When a unit counts as explanation-producing:** a unit qualifies if its value, or its effect on whether an event is accepted, reaches a listed field through any chain of units. The listed fields are `because`, `supportedBy`, `contradictedBy`, `basis`, `reopenedBy`, `caveats`, `why`, `history` and journal `because`/`caveats`.
  - **Included:** event translation in adapters, state that rules write, rejection guards (an accepted event adds evidence; a rejected one adds none), save/restore codecs (what is restored feeds every view after `resume`), and whole-view projections (`project`, `render`, `viewOf`). A projection counts even when the diff only added a countdown field to it.
  - **Excluded:** units whose output reaches only countdowns (`glowSeconds`, `regrowsIn`, `fadesIn`), `can*` flags, label or journal *text*, or nothing at all (write-only state). In Caveat, a `bind` qualifies when its field's explanation is read into a listed field (label, why, journal cites/entries). This includes `because nothing` defaults for those fields.

## Units I was unsure about

1. **Rejection guards and acceptance predicates count as explanation-producing.** Examples: C1/C2 `$m_can_retaste`, the doubt `*_found`/`*_named`/`open_count` defines, T `canRetaste`, `rememberedWitness`. These counts would be lower under a narrower reading that follows values only (no accept/reject control). The CR18–CR20 Caveat counts would shrink the most.
2. **Save codecs count as explanation-producing:** T1 `encode*/decode*`, `CAVEAT_CODE`, `PREFIX_CODE`; T2 `encode`, `decode`, `VERBS`, `parseId`; T3 `fullToken`, `readToken`, `restore`, `JOURNAL_PREFIX`. The exceptions are T2 `JOURNAL_CODES`/`JOURNAL_LETTERS`, which only carry the journal entry's kind (text), so I left them out.
3. **Some edits fall outside any unit as defined and were not counted:** TS `interface`/`type` declarations, scalar or array constants and regex constants (`SAVE_VERSION`, T3 `TOKEN`, T3 `ACTIONS`), and Caveat `event`/`evidence`/`renewable`/`caveat`/`entity` declarations. Object literals in `adapter.mjs` were also left out (C2 `ACT`, `EVIDENCE`, `CAVEAT`), because the README counts only functions there. Some of these do feed caveats or evidence names.
4. **C1's `$m_tasted_at` and its `on taste`/`on retaste … set $m_tasted_at` rules were not counted.** In C1 they feed only `fadesIn`, because the fade itself is a runtime `qualify … after`. C3's `$m_tasted_at` does drive the `taste_faded` qualification, so there it was counted.
5. **A few units did not count because they feed only journal text.** C1 `code_who` feeds only `entry_text`. C2 `$j_name`, `$e_name`, `$e_glow` and `$e_who` feed only journal text. C3 `j0` is write-only.
6. **C2 cr18–cr19 journal text bindings (`journal.$j_seen`, `$j_trusted`) count, but C2 cr20 `journal.$e_text` and C3 cr20 `diary.rN` bindings do not.** In cr18–cr19 those bindings carried the entry's `because` clause, and the adapter read their explanation. In cr20 the adapter reads `because` and `caveats` from separate `cites`/`cN`/`dN` bindings.
7. **Some units were touched only by a type annotation or an unrelated new field, and still counted.** Examples: T1 `mushroomRule` in cr17 and cr19, and the projections in cr17.
