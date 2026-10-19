# The project's own decision ledger

caveat-lang's decisions about caveat-lang, kept in Caveat. `project.cav` is
the program, `events.jsonl` the record of what happened in the order it
happened, `project.scenarios.json` the tests of what the ledger must say,
and `replay-check.mjs` the check that the record still replays. It loads and
passes on the published 1.0.0, 1.0.2 and 1.0.3 runtimes as well as on `main`, so
every thread that has `caveat` installed can ask it.

This exists because the language's claim is that a decision should carry
what it rested on, with its caveats, and should be found when that evidence
is later withdrawn or doubted. From 2026-10-17 to 19 the project made a run of
such decisions (merging #259, shipping 1.0.1, reopening it, shipping 1.0.2,
reopening that, shipping 1.0.3)
and kept them in prose. This is the same run, kept in the language, in the
order the [release records](../docs/releases/) and the
[decision log](../experiments/agent-decisions/README.md) show. If the ledger
can't carry the project's own decisions, that is a finding about the
language; where it pinches is listed at the end.

## Ask it

```
caveat explain ledger/project.cav ledger/events.jsonl
caveat dependents ledger/project.cav release_tree ledger/events.jsonl
npm run test:ledger
```

`explain` after the recorded events, the part about the release:

```
  release: 3 of at most 8
    release@1 = 101  superseded
      based on release_tree (caveats: changes_saves)
      release_tree: observed at #11 cherry_pick {"suite_green":1}
      permitted by approvals@3 for 101
      could also have been influenced by approvals@3
      #13 ship: committed because release_tree (caveats: changes_saves)
      #17 review: reopened because release_tree (caveats: changes_saves, uncovered_windows)
    release@2 = 102  superseded
      based on release_tree@2 (caveats: changes_saves)
      release_tree@2: observed at #18 cherry_pick {"suite_green":1}
      also carrying, through its lineage: uncovered_windows
      permitted by approvals@5 for 102
      could also have been influenced by approvals@3, approvals@5, release_tree
      #24 ship: committed because release_tree@2 (caveats: changes_saves)
      #33 review: reopened because release_tree@2 (caveats: changes_saves, fix_regressed)
    release@3 = 103  in force
      based on release_tree@3 (caveats: changes_saves)
      release_tree@3: observed at #35 cherry_pick {"suite_green":1}
      also carrying, through its lineage: fix_regressed, uncovered_windows
      permitted by approvals@9 for 103
      could also have been influenced by approvals@3, approvals@5, approvals@9, release_tree, release_tree@2
      #36 ship: committed because release_tree@3 (caveats: changes_saves)
```

That is the 1.0.1 story as the language tells it: the release rested on a
green suite over the cherry-picked tree, with the caveat that its fixes change
the saves of some programs (D014). D016 and D018, both found after 1.0.1 was
published, showed that tree had never been exercised on a path with a window,
which put a doubt on that evidence after the fact; the decision's own grounds
stayed as recorded, the doubt reached the basis it was computed from, and
`review` reopened the release naming the evidence that carried it. 1.0.2
rests on a fresh tree and carries the old doubt only through its lineage.
D020 then showed that 1.0.1's own D014 fix had broken a take-back on that
tree, so `review` reopened 1.0.2 on the `fix_regressed` doubt, and 1.0.3
rests on the tree with the D020 fix.

The withdrawn inference reads the same way:

```
  attribution_all_nine supports divergences_were_runtime  caveats: inferred_not_traced, withdrawn  withdrawn at #3 because rerun_on_fix
```

## What is in it

| Kind | Names | Meaning |
| --- | --- | --- |
| claims | `divergences_were_runtime`, `fix_complete`, `patch_safe`, `runtime_correct`, `approved_by_owner` | what evidence bears on |
| caveats | `inferred_not_traced`, `changes_saves`, `uncovered_windows`, `latent_since_1_0_0`, `fix_regressed` | what a reader must see next to a piece of evidence |
| evidence | the all-nine inference, the D012 repro, the 1.0.0 bisect, fuzz reruns (a stream of divergence counts), CI runs (renewable), release trees (renewable), findings (a stream of D-numbers), named findings decisions rest on, the randomized checks, owner approvals (a stream of item codes) | sources, each declared with where it comes from |
| decisions | `merge_259`, `release` (value: the version), `scope_1_0_2`, `record_fix`, `gaps_1_1`, `latest_gap_1_1` | series: each may be reopened and revised, never overwritten |
| events | one per thing that happens: `rerun`, `ci_run`, `approve`, `merge`, `cherry_pick`, `ship`, `find`, `review`, the `decide_*` cards | what gets appended to the record |

A decision is refused until its grounds exist and the owner has approved
that item: `merge` needs a rerun with 0 divergences, a green CI run and
`approvals` carrying 259; `ship` needs a release tree and an approval for the
version; a second `ship` while the current release stands is refused until
`review` has reopened it. An approval is a reading in `approvals` whose
value is the item code: a PR number, a version (101, 102, 103), or a card option
(1021/1022 for 1.0.2's scope, 41–42 for the record fix's order, 1101–1103 for
the two forms' place in 1.1, 1111–1113 for `carries(latest(STREAM), C)` in 1.1).

## Keep it going

Every thread that makes or informs a decision appends to the record:

1. **A finding filed** (a D-entry): `{"event": "find", "payload": {"number": N}}`.
   If decisions will rest on it, declare it as named evidence with its
   caveats and add its `reveal` to the `find` rule, as `d016`, `d018`,
   `d019` and `d020` are.
2. **A rerun, a CI run, a release tree**: `rerun` with the divergence count,
   `ci_run` with `green` 0 or 1, `cherry_pick` with `suite_green` 0 or 1.
3. **The owner's answer to a card**: `approve` with the item code, then the
   decision's event with the option chosen. A new kind of decision is a new
   `decisions` series, a basis state, a `decide_*` event and a rule, in the
   shape of the ones here.
4. **A retraction**: a rule that `withdraw`s the evidence because of the
   evidence that retracted it, as `rerun_on_fix` withdraws the all-nine
   inference. Nothing is deleted from the record.
5. **A doubt learned later**: `qualify` the evidence with the caveat, then
   `review`. The decisions whose basis carries it are reopened by name.

Then `npm run test:ledger`. The record must replay with every event
accepted, the scenarios must pass, and L04 mirrors the record, so a new event
is added to both. Keep `explain` honest: if a decision's `based on` line does
not say what it actually rested on, the rule is wrong, not the record.

## Where it pinches

Kept here because the pinches are findings about the language, and the point
of the ledger is to find them on the project's own decisions.

- **A caveat on a reading.** A finding is naturally a reading in a stream
  (`findings`, value the D-number), but `qualify latest(findings) with …`
  does not load: a caveat can go on named evidence only. So every finding a
  decision rests on is declared twice, as a reading for the count and as
  named evidence for its caveats. This is one of the two gaps planned for
  1.1 (`qualify latest(stream)`).
- **A past decision resting on doubted evidence.** A late `qualify` leaves a
  decision's recorded grounds alone, by design, so finding the decisions that
  rest on newly doubted evidence goes through a state: each decision commits
  `using` a basis state computed from `qualified(…)` reads, the late caveat
  reaches the state, and `reopen D because caveated(STATE, CAVEAT)` names the
  evidence. One state per decision series. This is the other 1.1 gap (a
  question over past decision revisions), and the ledger shows the shape
  authors build by hand until it lands.
- **Approvals are numbers.** The owner's go is a reading whose value is an
  item code, checked by `permitted by latest(approvals) for CODE`. It works,
  and `explain` shows `permitted by approvals@3 for 101`, but a code table is
  a convention the program cannot check.
- **A 1.0 sample needs a stance.** `sample approvals = item` must say what
  claim it bears on; `approved_by_owner` exists for that alone.

Each of these is a line a reader can judge; none stopped the ledger from
carrying the run.
