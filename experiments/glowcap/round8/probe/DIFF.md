# Learnability probe: the diff

This compares the counted reply (`reply.md`, Gemini, 2026-10-11 03:03 UTC)
with the reference program `experiments/glowcap/caveat5/glowcap.cav` (CR12)
and its adapter `caveat5/adapter.mjs`, following the steps in PROTOCOL.md,
"The learnability probe". The reply was committed before this diff was
written. Its items come after rc.17 and so go to a later candidate
(amendment 6). Section names below are those in the rc.17 package docs.

## 1. `caveat check` on the pinned package

The program is the reply's first code block, unchanged. The check used
`caveat-lang@0.1.0-rc.17` (tarball SHA256 `03bc86cf…91b6`) on Node 22.22.0.

```text
$ caveat check probe.cav
probe.cav does not load: line 47, column 5: for block has text after its body: return cavs.to_list(); end the block with `};`
exit 2
```

`check --json` and `validate` report the same single error. Line 1
(`caveat tasted_in_dark;`) is not valid either: checked on its own it gives
"cannot parse statement". The loader reports a `for` block's shape before it
parses any statement, so the first error shown is not the first line in error.

## 2. Constructs, classified

Each construct the probe used is matched with the reference construct that
serves the same purpose.

| # | Probe | Reference | Class |
| --- | --- | --- | --- |
| S1 | `max`, `min`, `ceil` | the same functions | same |
| S2 | `+` joins text | the same | same |
| S3 | `//` comments | the same | same |
| R1 | `caveat tasted_in_dark;` | `caveat tasted_in_dark consequence material;` | renamed |
| R2 | `evidence absorb_ev { id, kind }`, one typed evidence per kind of observation | `evidence absorb_$m from "…"`, one named evidence per mushroom, with a source | renamed |
| R3 | `observe absorb_ev(name, payload)` | `reveal E supports\|opposes CLAIM`, or a value `qualified(…, E)` | renamed |
| R4 | `archive Support; archive Contradict;` with `.append(ev)` | `claim glowing_is_safe;` and the stance in `reveal` | renamed |
| R5 | `decision Trust { … }` with `Trust.commit([…])` and `Trust.reopen([…])` | `decisions trust limit 16;`, `commit trust because enough using X`, `reopen trust because E` | renamed |
| R6 | `state glow_timer: number = 0;` | `state glow = 0 min 0 max 30;` | renamed |
| R7 | `event tick(payload: any)`, then `require dt >= 0 && dt <= 0.1` | `event tick dt min 0 max 0.1;` and `sort in glowcap duskcap` (typed parameters) | renamed |
| R8 | `require …` to refuse an event | `on E when C reject "message";` | renamed |
| R9 | imperative event bodies | `on EVENT when CONDITION EFFECT;` rules in source order | renamed |
| R10 | `ev.add_caveat(c)` | `C qualifies E;`, `qualify E with C`, `qualified(V, E, C)` | renamed |
| R11 | `ev.has_caveat(c)` | `carries(E, C)` | renamed |
| R12 | `fn handle_observation(kind, ev)` sharing logic between events | `proc learn(e evidence, sort) { … };` and `call learn(…)` | renamed |
| R13 | `fn get_belief()` computing the state | `define probably_safe = …;` | renamed |
| R14 | `query get_view()` returning one object | `bind TARGET.PROPERTY = X when C because …;` | renamed |
| R15 | `&&`, `\|\|`, `!` | `and`, `or`, `not` | renamed |
| R16 | `import { Runtime } from 'caveat-lang'`, with `.dispatch`, `.query`, `.save`, `.load` | the package's session runtime (the reference uses the repository's `WebReactiveSession`) | renamed |
| M1 | `struct Mushroom`, `list<Mushroom>`, `any`, `null` | none: state holds numbers | missing |
| M2 | `let`, `for (… in …)` loops, mutable sets and `return` in function bodies | none: `fn` is one pure expression | missing |
| M3 | text-valued state (`tasted_kind: string`) | none: the reference stores the tasted sort as a number | missing |
| M4 | an evidence value carrying a payload `{ id, kind }` | none: the evidence's name and the rule say what it is about | missing |
| U1 | — | `entity … kind mushroom` with `for mushroom as $m { … };` declares each mushroom's rules once | unanticipated |
| U2 | — (the probe kept lives as a counter and built names by hand; its assumptions say it chose not to guess the keyword) | `renewable absorb_$m limit 1024;` and `renew` | unanticipated |
| U3 | — (the probe counted the fade on its own timer in `tick`) | `qualify taste_$m with taste_faded after 60` | unanticipated |
| U4 | — (the probe wrote `because` lists, caveat unions and decision history by hand, and read the basis's caveats live) | values carry their grounds and caveats; a commit freezes its grounds; the decision journal is kept by the runtime | unanticipated |
| U5 | — | `display C "text";` | unanticipated |
| U6 | — | `scene`, `place garden`, `entity` | unanticipated |
| A1 | the source's `get_view` returns the finished view; the adapter only forwards it | the adapter builds `because`, `caveats`, `supportedBy`, `basis` and `history` from the runtime's explanation output, and renames `absorb_cave@2` to `absorb_cave_2` | adapter |

| Class | Count |
| --- | ---: |
| same | 3 |
| renamed | 16 |
| missing | 4 |
| unanticipated | 6 |
| adapter | 1 |
| **total** | **30** |

## 3. Items

Each renamed, missing and unanticipated item gets one line here. All of these
are for rc.18 or later, because rc.17's docs are frozen.

- **R1:** REFERENCE.md, "Common mistakes". Add "a caveat needs `consequence LEVEL`". The load error ("cannot parse statement: caveat tasted_in_dark") does not name the missing clause, and a runtime diagnostic would help more than a doc line.
- **R2, R3, R4:** none needed. REFERENCE.md "Declarations" and "Rules" state the forms, and the probe never saw them.
- **R5:** none needed, for the same reason ("Declarations", `decisions`; "Rules", `commit` and `reopen`).
- **R6, R7, R8, R9, R15:** none needed. These are the ordinary syntax that REFERENCE.md "Syntax" gives on one page.
- **R10, R11:** none needed. They are covered in "Rules" (`qualify`) and "Expressions" (`carries`).
- **R12, R13:** none needed. "Declarations" lists `proc` and `define`, and "Common mistakes" already points to `proc`.
- **R14:** none needed ("Declarations", `bind`).
- **R16:** none needed. AGENT_START.md and STARTER.md name the package's host entry points.
- **M1, M2, M3, M4:** AI_AUTHORING.md, "What the language refuses to do for you". Add one item. The probe reached for records, lists, loops and text state, which is the general-purpose habit that section is written for, but the section names none of them. The line would say: state is numbers, functions are single expressions, and a collection of similar things is a `for` block over a kind (U1).
- **U1:** REFERENCE.md, "More". It says `for` blocks are "not covered here", yet a program about several similar things needs them first. Add a row to "Declarations" pointing to `caveat-repetition-0.1.md`. This item is the most likely to change what an author writes.
- **U2:** none needed. "Declarations" lists `renewable`, and "Common mistakes" covers `renew`.
- **U3:** none needed. "What the language refuses to do for you" already shows `qualify E with C after N`.
- **U4:** AI_AUTHORING.md, "Citations and named expressions". Open it with one sentence saying the runtime carries grounds and caveats, so `because` lists and caveat unions are not written by hand. The probe's hand-written versions are the round 7 glue habit in the source, and its live reading of the basis's caveats would be wrong after a late fade.
- **U5, U6:** none needed. They are presentation and world declarations, which the beat does not require.
- **A1:** none needed in the docs. This is the reverse of the "adapter" prediction. Note it for round 8's record: the probe expected the source to produce the finished view, while the CR12 reference shapes the view in its host.

## 4. Against the predictions

PREDICTIONS.md, "The learnability probe", registered before the reply was
read:

| Prediction | Outcome |
| --- | --- |
| Fails `caveat check` on the first run | **held**: it does not load (section 1) |
| Fewer than half of its constructs are **same** | **held**: 3 of 30 |
| Puts the explanation bookkeeping (`because` lists, caveat unions) in the adapter rather than the source | **failed**: it put that bookkeeping in the source, by hand (U4), and kept its adapter to forwarding calls (A1) |
