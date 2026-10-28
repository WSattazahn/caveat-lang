# Authoring card

One program, the commands that check it and the mistakes that cost the most,
on one page. It is for an agent writing a program for a task: read this first,
then look up only what it lacks in [Caveat on one page](REFERENCE.md). The
[authoring guide](reference/docs/AI_AUTHORING.md) and the
[documentation index](README.md) are for questions this card and the
reference leave open.

## The loop

1. Write the program and its scenarios (below).
2. `npx --no-install caveat check --unused --names PROGRAM`: does it load, is
   anything declared that nothing uses, and does a `commit` misspell a
   decision series?
3. `npx --no-install caveat test SCENARIOS`: does it do what the task says?
4. `npx --no-install caveat explain PROGRAM EVENTS` when a test fails: what
   each decision rests on, and which event observed it.

## One program with the usual parts

A swimming site opens only while the latest result of every assay is clean.
Results can be withdrawn or flagged, and an inspection can pause the decision.

<!-- file: water.cav -->
```caveat
# Open a swimming site only while every assay's latest result is clean.
claim safe;
place lab kind site;
entity lake kind site at lab;
entity river kind site at lab;
entity bacteria kind assay at lab;
entity algae kind assay at lab;
entity mislabeled kind doubt at lab;
entity expired_kit kind doubt at lab;

caveat mislabeled consequence material;
caveat expired_kit consequence material;

event result site kind site, assay kind assay, outcome in unsafe clean;
event withdraw site kind site, assay kind assay;
event flag site kind site, assay kind assay, reason kind doubt;
event unflag site kind site, assay kind assay, reason kind doubt;
event decide site kind site;
event pause site kind site;
event resume site kind site;

for site as $s routed by site {
    decisions $s_open limit 8 reopened by $s_bacteria, $s_algae;
    evidence $s_inspection from "an inspection that pauses changes at $s";
    on pause when not committed($s_open) or reopened($s_open) reject "Nothing in force to pause.";
    on pause reveal $s_inspection;
    on pause hold $s_open because $s_inspection;
    on resume when not on_hold($s_open) reject "Not paused.";
    on resume release $s_open because $s_inspection;

    for assay as $a routed by assay {
        evidence $s_$a_lab from "lab results for $a at $s";
        evidence $s_$a_recheck from "a finding that a $a result at $s was wrong";
        readings $s_$a from $s_$a_lab limit 8;
        on result when outcome == outcome.clean sample $s_$a = 1 supports safe;
        on result when outcome == outcome.unsafe sample $s_$a = 0 opposes safe;

        on withdraw when not has_sample($s_$a) reject "No result to withdraw.";
        on withdraw when withdrawn(latest($s_$a)) reject "Already withdrawn.";
        on withdraw reveal $s_$a_recheck;
        on withdraw withdraw latest($s_$a) because $s_$a_recheck;
        on withdraw when committed($s_open) and not reopened($s_open) and in_basis($s_open, latest($s_$a))
            reopen $s_open because latest($s_$a);

        on flag when not has_sample($s_$a) reject "No result to flag.";
        for doubt as $d routed by reason {
            on flag when carries(latest($s_$a), $d) reject "Already flagged.";
            on flag qualify latest($s_$a) with $d;
            on unflag when not carries(latest($s_$a), $d) reject "Not flagged.";
            on unflag unqualify latest($s_$a) with $d;
        };
        on flag when committed($s_open) and not reopened($s_open) and in_basis($s_open, latest($s_$a))
            reopen $s_open because latest($s_$a);

        # Every caveat is named here: there is no "carries any caveat" form.
        define $s_$a_doubted = withdrawn(latest($s_$a)) or carries(latest($s_$a), mislabeled)
            or carries(latest($s_$a), expired_kit);
        define $s_$a_ok = if(has_sample($s_$a), if($s_$a_doubted, 0, latest($s_$a)), 0);
    };
    on decide commit $s_open because enough using $s_bacteria_ok * $s_algae_ok;
    bind $s.open = "closed";
    bind $s.open = "open" when committed($s_open) and not reopened($s_open) and latest($s_open) == 1;
};
```

What each part does:

- **Members, not copies.** `entity NAME kind KIND at PLACE;` lists members.
  `for site as $s routed by site { … }` writes the body once per member and
  runs a member's rules only for events whose `site` names it. A `for` block
  may hold another, and `$s_$a` names one copy per pair.
- **Readings.** `readings S from E limit N;` keeps up to N readings, `S@1`,
  `S@2` and so on; the next is refused as `limit/history_limit`. `sample S = V
  supports C` adds one. `latest(S)` reads the newest; `reading(S, N)` names
  the Nth in `withdraw`, `qualify`, `unqualify`, `carries` and `in_basis`.
- **Decisions.** `decisions D limit N reopened by S, …;` is a series of
  revisions, `D@1`, `D@2`. `commit D because enough using EXPR` records one;
  its grounds are exactly what `EXPR` read, frozen. A second commit while one
  is in force is refused as `evaluation/decision_in_force`. `citing EXPR, …`
  records a decision with no value.
- **Taking things back.** `withdraw latest(S) because E` marks a reading
  wrong; `qualify latest(S) with C` and `unqualify latest(S) with C` add and
  take back a caveat. None of them changes a decision already made: a rule
  reopens it, here only when `in_basis(D, latest(S))`.
- **Pausing.** `hold D because E` keeps the revision in force; reopenings wait
  and `release D because E` applies them. A reopening because of withdrawn
  evidence never waits. `on_hold(D)` asks whether it is held.
- **Refusals.** `reject "…"` refuses the whole event, and nothing an earlier
  rule did remains. Typed parameters (`kind`, `in`, `min … max`) refuse a bad
  payload as `input/payload_invalid` before any rule runs.

## Scenarios

<!-- file: water.scenarios.json -->
```json
{
  "schema": "caveat-scenarios/0.1",
  "source": "water.cav",
  "scenarios": [
    {"id": "W01", "title": "Clean results open the site; a flag on one of them reopens it, and the first decision keeps its grounds",
     "steps": [
       {"send": "result", "payload": {"site": "lake", "assay": "bacteria", "outcome": "clean"}},
       {"send": "result", "payload": {"site": "lake", "assay": "algae", "outcome": "clean"}},
       {"send": "decide", "payload": {"site": "lake"}},
       {"expect": {"/bindings/lake/open": "open",
                   "/commitment_grounds/lake_open@1": {"evidence": {"$set": ["lake_algae@1", "lake_bacteria@1"]}}}},
       {"send": "flag", "payload": {"site": "lake", "assay": "algae", "reason": "expired_kit"}},
       {"send": "decide", "payload": {"site": "lake"}},
       {"expect": {"/bindings/lake/open": "closed",
                   "/commitment_grounds/lake_open@1": {"evidence": {"$set": ["lake_algae@1", "lake_bacteria@1"]}, "caveats": []}}},
       {"send": "flag", "payload": {"site": "lake", "assay": "algae", "reason": "expired_kit"}, "rejected": "Already flagged."}
     ]},
    {"id": "W02", "title": "While paused, a new result waits; resuming applies it, and a withdrawal never waits",
     "steps": [
       {"send": "result", "payload": {"site": "river", "assay": "bacteria", "outcome": "clean"}},
       {"send": "result", "payload": {"site": "river", "assay": "algae", "outcome": "clean"}},
       {"send": "decide", "payload": {"site": "river"}},
       {"send": "pause", "payload": {"site": "river"}},
       {"send": "result", "payload": {"site": "river", "assay": "algae", "outcome": "unsafe"}},
       {"expect": {"/bindings/river/open": "open"}},
       {"send": "resume", "payload": {"site": "river"}},
       {"expect": {"/bindings/river/open": "closed"}},
       {"send": "decide", "payload": {"site": "river"}},
       {"send": "result", "payload": {"site": "river", "assay": "algae", "outcome": "clean"}},
       {"send": "decide", "payload": {"site": "river"}},
       {"send": "pause", "payload": {"site": "river"}},
       {"expect": {"/bindings/river/open": "open"}},
       {"send": "withdraw", "payload": {"site": "river", "assay": "bacteria"}},
       {"expect": {"/bindings/river/open": "closed"}}
     ]},
    {"id": "W03", "title": "A decision made after a withdrawal is grounded on the withdrawal's reason as well",
     "steps": [
       {"send": "result", "payload": {"site": "lake", "assay": "bacteria", "outcome": "clean"}},
       {"send": "result", "payload": {"site": "lake", "assay": "algae", "outcome": "clean"}},
       {"send": "withdraw", "payload": {"site": "lake", "assay": "algae"}},
       {"send": "decide", "payload": {"site": "lake"}},
       {"expect": {"/bindings/lake/open": "closed",
                   "/commitment_grounds/lake_open@1": {"evidence": {"$set": ["lake_algae@1", "lake_algae_recheck", "lake_bacteria@1"]}}}}
     ]}
  ]
}
```

`"$set"` compares without order. An object matches only the members it lists.

<!-- run: npx --no-install caveat check water.cav -->
```text
water.cav: no warnings.
```

<!-- run: npx --no-install caveat test water.scenarios.json -->
```text
PASS W01  Clean results open the site; a flag on one of them reopens it, and the first decision keeps its grounds  (6 events, 1 rejected, 0 resumes)
PASS W02  While paused, a new result waits; resuming applies it, and a withdrawal never waits  (11 events, 0 rejected, 0 resumes)
PASS W03  A decision made after a withdrawal is grounded on the withdrawal's reason as well  (4 events, 0 rejected, 0 resumes)
3 passed, 0 failed (water.scenarios.json)
```

## The mistakes that cost the most

- **A withdrawal's reason joins later grounds.** Any `withdrawn(…)` read
  carries the reason as evidence, so `lake_open@1` in W03 rests on
  `lake_algae_recheck` as well as the readings. When a host shows "the
  reports a decision rested on", it must leave the reason out, or the
  program must not read `withdrawn(…)` in `using`.
- **A procedure's arguments reach everything it does.** `call P(X)` passes
  X's evidence and caveats to every step, samples included, even when the
  body never reads X. Pass a procedure only what it reads; never pass a flag
  to one that samples.
- **"Any caveat" has to be spelled out.** There is no form for "carries any
  caveat". Name every caveat, as `$s_$a_doubted` does, and add a new caveat
  there too.
- **`unqualify E with C` reaches every held occurrence of E.** To take a
  caveat back from the newest reading only, write `unqualify latest(S) with C`.
- **A guard passes its evidence to what it guards.** Grounds are only what
  `using` read; everything else that touched a value is lineage. Run
  `explain` when unsure.
- **Comments are `#` or `//`.** `--` is not a comment.

## The host

```js
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { decisionHistory } from 'caveat-lang/session';

const runtime = await loadRuntimeFromDirectory();
const source = await readFile('water.cav', 'utf8');
const session = runtime.open(source);              // or runtime.restore(source, save)
const outcome = session.dispatchView('result', { site: 'lake', assay: 'algae', outcome: 'clean' },
  { fields: ['commitments', 'decision_series', 'decision_journal', 'bindings'] });
if (outcome.outcome !== 'accepted') console.log(outcome.message);   // refused; nothing changed
const history = decisionHistory(outcome.view, 'lake_open');
const save = session.save({ compact: true, positional: true });
```

- `dispatchView` with `fields` builds only what the host reads; `dispatch`
  also builds the full snapshot and costs several times as much.
- The positional save writes each accepted event as a few digits. It holds
  about 4,096 / (2 × (parameters + 1)) events, 512 of this program's
  three-parameter events, and then the save is the full state save, which is
  far larger: 34,488 bytes for this program after 514 events, about 64 bytes
  more for each event after that (measured on 1.2.0). Events a host can send
  without end, such as flag and unflag, need a cap when the store has a size
  limit ([what grows the save](reference/docs/AI_AUTHORING.md#what-grows-the-save)).
