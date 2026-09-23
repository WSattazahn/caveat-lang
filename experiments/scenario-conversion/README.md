# Converted Trail Rescue and Glowcap scenarios

Acceptance check for [Scenarios 0.1](../../spec/caveat-scenarios-0.1.md) and
the [kit runner](../../kit/README.md): Trail Rescue's 24 registered scenarios
and Glowcap's 38 `cr12` scenarios, rewritten as scenario files and run by the
kit. The originals are unchanged and remain the record:
`experiments/trail-rescue/scenarios.json` with its harness, and
`experiments/glowcap/scenarios.mjs` with its harness and `runs.jsonl`
scoreboard. These files run alongside them; they do not rescore any round.

| File | What it is |
| --- | --- |
| `convert.mjs` | Generates both files. `--check` regenerates in memory, fails on drift, and runs them. |
| `trail-rescue.scenarios.json` | 24 scenarios against `game/trail_rescue.cav`. |
| `glowcap-cr12.scenarios.json` | 38 scenarios against `experiments/glowcap/caveat5/glowcap.cav`. |
| `conversion-report.json` | Counts, adapter-only steps, runtime identity and input hashes. |
| `mutation-check.mjs` | Alters every original expectation and outcome and requires each alteration to be caught. |
| `results-mutations.json` | Its results. |

```sh
npm run build
npm run test:scenario-conversion   # regenerate, compare, run
npm run check:scenario-mutations   # about three minutes
```

## Method

The originals assert on each page adapter's projected view. The kit asserts on
the raw snapshot. Each original expectation therefore becomes assertions on
the raw fields that the adapter projects:

- **Derived** assertions take their expected value from the original, through
  the adapter's naming: `report_stone` becomes `seen_report_stone`, and
  `absorb_cave_2` becomes `absorb_cave@2`, using the program's declared
  evidence names. Lists that the adapter or harness treats as sets become
  `$set`. Journal `because` lists keep their order.
- **Located** assertions record where the adapter looks something up: the
  current decision, which journal entries exist, the observation acquired at
  a position, and which relations bear on a claim. They come from a reference
  run of the same history on the current runtime and are asserted as well, so
  a converted scenario checks at least what the original did.
- **Expected rejections** are classified from the program's declared event
  signatures, not from the reference outcome. A payload outside a declared
  signature is an `input` refusal with its code; anything else is a policy
  rejection, which a bare `rejected: true` requires.
- **Adapter-only steps**: Trail Rescue's page adapter refuses 11 malformed
  events in TR12 before calling Caveat. Those are host-input checks, listed in
  `conversion-report.json` and not converted.

Two adapter presentation choices are not carried into the raw assertions: the
Trail Rescue adapter orders evidence lists by acquisition, and it filters
caveats to `secondhand` and `stale`. The converter checks that the program
declares no other caveats, so the filter changes nothing. Evidence order is a
host display concern.

## Results

On 2026-09-23, against the clean compiled reactive build of `541e09d`:

| | Trail Rescue | Glowcap `cr12` |
| --- | ---: | ---: |
| Scenarios passed | 24 of 24 | 38 of 38 |
| Events sent | 140 | 24,336 |
| Expected rejections | 20 policy, 2 `input/bound_exceeded` | 15 policy, 2 `input/bound_exceeded`, 4 `input/payload_invalid` |
| Resumes | 11 | 4 |
| Derived assertions | 868 | 501 |
| Located assertions | 274 | 155 |
| Adapter-only steps not converted | 11 | 0 |

The runner also made its automatic checks on every step. For Glowcap these go
beyond the original harness, which compared views only on rejection and had no
resumed sessions running alongside. All passed.

The mutation check altered each original expected value (a wrong string,
number, boolean or name; a list with an element dropped or a valid name added)
and each expected acceptance or rejection, one at a time:

| | Alterations | Failed in the runner | Refused by the converter | Not detected |
| --- | ---: | ---: | ---: | ---: |
| Trail Rescue | 506 | 473 | 33 | 0 |
| Glowcap | 854 | 737 | 117 | 0 |

The converter refuses an alteration when it contradicts a located fact in the
same step, when it names no relation or journal entry that exists, or when it
would make an adapter-only step accepted. Glowcap's relation sets are checked
by the converter: a support or contradiction list with a name added or
dropped cannot be placed on the reference relations.

## Limits

Located facts are pinned from the reference run. A runtime change that, for
example, adds or reorders relations will fail these files even if the original
harness still passes; `--check` then reports the drift for review. The
conversion depends on the two adapters' projection code as of these commits:
`convert.mjs` refuses to run if Glowcap's payload mapping or naming lines
change, and Trail Rescue's adapter is called directly to record the exact
events it sends.
