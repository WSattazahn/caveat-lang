# Explanation drift, CR13–CR16

Counts of touched explanation-producing units per program and change request
(definition in `README.md`; the unit lists with lines and what each feeds are
in `DRIFT.json`).

| Program | CR13 | CR14 | CR15 | CR16 | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| C1 (Caveat) | 24 | 8 | 3 | 35 | 70 |
| C2 (Caveat) | 11 | 8 | 3 | 12 | 34 |
| C3 (Caveat) | 22 | 7 | 2 | 22 | 53 |
| C4 (Caveat) | 18 | 8 | 3 | 22 | 51 |
| T1 (TypeScript) | 10 | 7 | 3 | 7 | 27 |
| T2 (TypeScript) | 10 | 8 | 3 | 7 | 28 |
| T3 (TypeScript) | 24 | 6 | 4 | 6 | 40 |

## How the definition was applied (the same way for every program)

- **What counts as a unit.** In Caveat, each `bind`, `on`, `state`, `decisions`,
  `define`, `proc` and `fn` statement is one unit, including two `state`
  statements on one line. Statements in a `for` body count once. `entity`,
  `event`, `evidence`, `renewable`, `caveat`, `claim` and `qualifies` lines are
  not units. In `adapter.mjs` and TypeScript, a unit is a top-level function
  (including a top-level arrow `const` or IIFE) or a top-level object literal or
  class. Nested helpers, closures and the methods of the object returned by
  `createPolicy` belong to the function that encloses them. Top-level scalars and
  arrays (`SHOWN`, `KEEP`, `MEMORY`, `LEVEL`, `SAVE_VERSION`, `OUT_OF_REACH`),
  interfaces and types are not units.
- **What "feeds" means.** A unit feeds an explanation field if, for a fixed
  sequence of accepted events, its output reaches the value of an explanation
  field through data flow or by selecting which binding or branch applies.
  `why.reason` counts because `why` is on the list. `label`, `text`, `note`,
  `state`, `canAbsorb`, `canTaste`, `marked` and the timers do not.
- **Excluded as non-feeding:**
  - Guards that only reject an event (`on … when … reject`, `fail(...)` checks).
  - `canAbsorb`/`canTaste` defines and bindings.
  - `belief.state/text/note` bindings, because the adapters do not read their
    explanations.
  - Functions and literals that only produce journal text: C1 `entry_prefix`,
    `entry_suffix` and `trust_text`; C2/C4 `mushroom_name`; C4
    `observation_text`; C3 `$j_name`; T1 `OBSERVATION_TEXT`, `observationText`
    and `TRUST_TEXT`; T2/T3 `OBSERVATION_TEXT`/`JOURNAL_TEXT` and `journalText`.
  - C2 `journal_who_$index`.
- **Counted, but open to argument:**
  - **Save and restore code.** This covers `pack`/`unpack`, `createPolicy`'s
    restore path, `encode`/`decode`, `encodeSave`/`decodeSave` and their
    helpers. It counts as feeding because after `['resume']` every `because`,
    `caveats`, `basis` and `history` comes from the restored data. The
    decision accounts for much of CR13 (all programs) and part of CR14/CR16 in
    the TypeScript programs. Excluding it would give: C1 cr13 21; C2 cr13 8;
    C3 cr13 19; T1 cr13 6, cr14 5, cr16 5; T2 cr13 8, cr14 6, cr16 5; T3 cr13
    14, cr14 4, cr15 3, cr16 4.
  - **Input translation and validation that return the id or kind.** This
    covers the adapters' `translate` (CR14 added `mark`/`unmark`; marks feed
    `why`) and T3 `target`. They pass the id and kind on to evidence ids, so
    they count as feeding.
  - **Journal-text functions whose empty result drops an entry.** C2 and C4
    `fn journal_text` return `""` for an empty slot, and the adapter then omits
    the entry and its `because`.
  - **Selectors.** These are C1 `entry_trusted`, `entry_untrusted` and
    `$m_entry_low`, C3 `$j_what` and `bind $j.shown`. Each decides whether a
    slot shows an entry that cites evidence, or an empty one. C1's six
    `bind journal.eN = "" because nothing` defaults and C3's `bind $j.text = ""`
    are counted for the same reason.
- **Removed and renamed units.** A unit that is replaced under the same name
  counts once, for example C1/C3/C4 `support` changing from `state` to `define`,
  or T3 `evidenceId` moving. A unit that the diff only removes counts, for
  example C1 `recovery`, T3 `consume` and T3 `VERBS`. T3 `VERBS` and
  `VERB_CODES` have different names and count as two units.
- **Unsure.**
  - C4 CR13 removed `define belief_none`. Its former uses are not visible in
    the supplied files, and none of C4's later explanation-producing bindings
    reads it. I judged it non-feeding (counting it would make C4 cr13 19).
  - C2 CR13 `state supported`/`contradicted`/`recovery` changed only their
    `max` bound. They still count as touched.
  - TypeScript units are whole functions, so any line change in, for example,
    `apply` or `mushroomView` touches an explanation-producing unit even when
    the changed line itself is about `canAbsorb` or rejection.
