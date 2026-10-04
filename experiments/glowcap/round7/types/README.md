# Round 7's adapters against generated declarations

rc.13 PR 9 adds `caveat types PROGRAM`, which writes TypeScript declarations
from a program's [interface](../../../../spec/caveat-interface-0.1.md). The plan sets
the bar as a measurement to record, not a claim to assert. Round 7's four
Caveat adapters are plain JavaScript. Each one gets `// @ts-check` and the
declarations generated from its author's own program, and
`tsc --checkJs --noEmit` runs over it. Three things are recorded: (a) what the
adapter spells by hand that the types now carry, (b) every mismatch the check
finds between the adapter and the program, and (c) whether any of the round's
recorded blind-phase failures would have been caught at type-check time.

The finding is small numbers. This record makes no claim for the README
either way.

## Reproduce

```sh
npm run build
node experiments/glowcap/round7/types/measure.mjs        # or --json
```

[`measure.mjs`](measure.mjs) copies each author's `runs/<ID>/impl/glowcap.cav`
and the annotated adapter in `<ID>/adapter.mjs` into a scratch directory. It
writes `glowcap.d.ts` with `caveat types`, then runs TypeScript 5.9.3 with
`--checkJs --noEmit --module nodenext --target es2022` against
`@types/node` 22.18.6. The round's own files are unchanged. Recorded on
2026-10-04 with the rc.13 PR 9 branch built locally.

## Which line counts

The plan quotes 115–136 lines per adapter, and the round's work list quotes
95–113. Both describe the same files. 115, 136, 129 and 127 are whole-file
lines (`wc -l`) for C1–C4. 95, 110, 106 and 113 are code lines, excluding
blank and comment-only lines, by the round's `harness.mjs --measure` rule
(`experiments/glowcap/RESULTS.md`, "Code lines, end of CR16"). This record uses
the whole files, because the annotation adds lines to them.

## Annotation cost

Each adapter gained the same six header lines: `// @ts-check`, the `typed`
import and four `@typedef`s. Its session creation is wrapped in `typed(…)`
under a `@type` line. Its mushroom list, `translate` and `project` (and C2's
`apply`) each gained one JSDoc line.

| | C1 | C2 | C3 | C4 |
| --- | ---: | ---: | ---: | ---: |
| Adapter, whole-file lines | 115 | 136 | 129 | 127 |
| Lines added, rewritten ones included | 11 | 12 | 11 | 12 |
| Lines of the round's adapter rewritten | 1 | 1 | 1 | 2 |
| Generated `glowcap.d.ts`, lines | 51 | 56 | 56 | 51 |

## (a) What the types now carry

Counted in the round's final adapters (`runs/<ID>/impl/adapter.mjs`).
"Checked" means a misspelling there becomes a type error under the
annotation above.

| | C1 | C2 | C3 | C4 |
| --- | ---: | ---: | ---: | ---: |
| Event names the adapter writes as literals, checked | 6 | 1 (`tick`) | 1 (`tick`) | 1 (`tick`) |
| Event names forwarded from the host's `type`, not checked | 0 | 5 | 5 | 5 |
| Payload fields written as literals (`target`, `sort`, `dt`), checked | 3 | 2 | 2 | 3 |
| Displayed values read as `b.TARGET.PROPERTY` or `b[mushroom].PROPERTY`, checked | 12 | 12 | 14 | 14 |
| Displayed values read through a computed name (journal slots, `why.${action}`), not checked | 2 | 1 | 2 | 1 |

Some names the types carry but these adapters do not use in checked form:

- **The mushroom names.** Every adapter derives the list at run time. C1 and
  C3 read the `absorb` event's signature, and C2 and C4 filter the snapshot's
  world entities. The declarations hold the same five names as a type, but a
  type cannot be iterated, so the run-time list stays. `runtime.interface(source)`,
  which this PR adds to the kit, is a shorter way to read it.
- **Binding names passed as strings.** Each adapter passes them to its own
  explanation helper (`cites(id, 'label')`, `caveatsOf(view, m, 'why_absorb')`
  and so on), and the helpers take a `string`. `TypedView` types
  `binding_explanations` by target and property, so a typed helper would be
  checked, but none of the four helpers is.
- **Not covered by the declarations.** The decision series name (`trust`),
  journal and grounds records, `relations` and the `@N` occurrence names keep
  their `string` keys. The view's shape is View 0.1's contract, and this PR
  does not change it.

## (b) Mismatches the check finds

None between any adapter and its program. Every event name, payload field and
displayed value that is checked matches the program.

The check reports five diagnostics, and none of them is a mismatch:

| Author | Diagnostic | Cause |
| --- | --- | --- |
| C1, C3 | `Property 'entity' does not exist` on `domain` | Reading `domain.entity.members` needs narrowing under the kit's existing `Snapshot` type. It is the adapter's run-time mushroom list. |
| C2, C4 | `string[]` is not assignable to the mushroom names | The run-time list is `string[]`, and the annotation asks for the five names. |
| C2 | `Property 'dt' does not exist` on the payload union | The tick-merging line in `apply` reads `payload.dt` after comparing names, which does not narrow a separate payload variable. |

Four of the five follow from typing the run-time mushroom list. That is the
one place where a value the host computes has to meet a type the program
declares.

### Control: does the check catch a misspelling?

`measure.mjs` makes one misspelling at a time in each annotated adapter and
counts it caught when a new diagnostic appears on that line.

| Misspelling | C1 | C2 | C3 | C4 |
| --- | --- | --- | --- | --- |
| A bound property (`.present` → `.presnt`) | caught | caught | caught | caught |
| A bound target (`b.slime.` → `b.slim.`) | caught | caught | caught | caught |
| A payload field (`target:` → `targte:`) | caught | caught | caught | caught |
| An event name (`['tick'` → `['tock'`) | caught | caught | caught | caught |
| The decision series (`.trust` → `.trusts`) | not caught | (not written so) | (not written so) | not caught |

## (c) The round's recorded blind-phase failures

None of them would have been caught at type-check time. The round's
learnability table (`experiments/glowcap/RESULTS.md`, "why each Caveat run
failed") lists every failing Caveat run:

- **The session save over 4,096 bytes** (C1, C2, C3, six runs) is a size,
  not a type.
- **C4's JSON wrapper** pushed a 3,764-byte save over the bound. It is the
  table's one glue failure, and it is also a size.
- **`ungrounded_citation`** (C2, C3, three runs) is a load-time language rule.
- **The renewed-evidence lineage** in C4's CR16 run is a language failure,
  recorded as F268 and fixed in #115.
- **Caveat order inside journal entries** (S48, S50) comes from a scenario's
  comparison. The view lists caveats as sets either way.
- **C1's CR13 incident runs** used another author's adapter.
- **C1's CR16 load error** is in the program. `caveat types` refuses to
  generate for a program that does not load, as `validate` and `test` do.

The round's work list says that "most blind-phase failures were there" (in
the glue). Its own table, which this record follows, classifies one failure
as glue. The adapters' larger cost was in size, change cost and drift, and
that is where the run-time names and shapes counted in (a) live.
