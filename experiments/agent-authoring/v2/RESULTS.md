# Fresh-agent clock-task follow-up — 2026-09-23 UTC

**Three of four first submissions and all four final submissions passed every
registered case. All four final sources read `elapsed()` directly and contain
no separate clock accumulator.** The remaining first-source failure was
unsupported comment syntax, repaired by its author from public validation.

This repeats the two clock tasks from [v1](../v1/RESULTS.md) with four fresh
contexts, updated documentation and the new clock read. It demonstrates those
changes working together on these tasks. The small repeated-task sample does
not isolate the effect of the intrinsic or predict reliability on unseen work.

## Registered outcomes

| Run / task | First submission | Final submission | Source versions | Runtime checks |
| --- | --- | --- | ---: | ---: |
| A1 / cold-storage dispatch | 114/114 cases | 114/114 cases | 1 | 5 |
| A2 / cold-storage dispatch | 114/114 cases | 114/114 cases | 1 | 5 |
| B1 / access review | 116/116 cases | 116/116 cases | 1 | 5 |
| B2 / access review | Load error | 116/116 cases | 2 | 8 |

Cold-storage first and final pass rates are 2/2; access-review rates are 1/2
first and 2/2 final. B2's first validation rejected line 1:
`cannot parse statement: -- Fixed assertions`. Its author removed three `--`
comment lines. Apart from line-ending normalization, the policy source was
unchanged. That parse failure remains in the denominator and the preserved
[first source](runs/B2/versions/01.cav) and [check log](runs/B2/checks/01.jsonl).
No candidate had a numeric-bound or runtime-clock load error in this cohort.

Across final sources, **460/460 case executions** passed, covering **13,704
event dispatches** (5,664 accepted, 8,040 rejected) and **3,158 restore
operations**. These totals exclude duplicate shadow dispatches and first-source
scoring. There are 30 targeted scenarios and 200 seeded 40-step sequences,
repeated for each task's two authors. Case executions are not independent
author trials. See [raw scores](results/scored.json) and [corpus](corpus.json).

The verifier checks exact admission, every required HUD field, ordered
observations and journals, direct caveats, frozen grounds, and runtime snapshot
elapsed/sequence at initialization, after each step and after final restore.
Rejected events preserve complete saves and views. Restores preserve full
snapshots/views and subsequent behavior against an uninterrupted shadow.

## What changed from the earlier clock trials

For the corresponding A/B tasks, v1 had **0/4 first-source corpus passes and
4/4 final-source corpus passes**. Two final sources nonetheless rejected valid
events beyond their state clock caps; the other two used scaled/split clocks
whose large-time branches were untested. Those outcomes and all v1 artifacts
remain unchanged.

Post-freeze source review found that all four v2 finals use
`bind hud.elapsed = elapsed();`, an explicit `advance` clock and native delayed
qualifications. None mirrors time in state, saves observation timestamps in
bounded state, or rescales/splits time. No additional policy, provenance or
reopening defect was identified in source review. This is a separate review
observation, not an added primary scoring rule or exhaustive correctness proof.

The registered corpus is unchanged and still reaches only about 11.4 elapsed
units for A and 10.8 for B. It does not simulate extremely long sessions. Separate
runtime regressions exercise a deliberately manufactured valid save above
`1e12`, then advance and restore it; that evidence is not a long replay of the
author programs.

Two verification differences also matter when comparing cohorts: v2 includes
snapshot elapsed/sequence in its primary checks, and its public runner forwards
explicit null payloads correctly. v1 retains its original scores, documented
transport correction and separate metadata supplement. No v2 scoring correction
was needed after registration.

## Freeze, isolation and retained evidence

The runtime/documentation revision is
`c4b25e18ffdab96f328c45da4266069255c32ecf`. The [protocol](PROTOCOL.md), exact
A/B contracts, unchanged domain oracle/corpus, scorer, prompts, public tools
and packet manifest were registered in
`b44a0190771b319bdbd616b34b3e10fd3381d94d` before dispatch. The pinned compiled
reactive WASM SHA-256 is
`d92dd03d9a368b035a9d9b2a8a604139afaaf0e716344994e6ebd98471c285ec`.

All four authors were new contexts with `fork_turns: none`, in waves A1/B1 then
A2/B2. Each received its assigned task and the public packet, including the
updated guide with its final earlier-results section removed by the registered
transformation. Authors could submit three source versions and use twelve
validation/replay checks. All finished before private scoring. There were no
supervisor candidate edits, private-test hints or outcome-based extra trials.

[Integrity checks](results/integrity.json) verify registered inputs, pinned
references/runtime, source archives, final hashes, logs and budgets. Every author
reports no access-rule deviation in its notes: [A1](runs/A1/notes.md),
[A2](runs/A2/notes.md), [B1](runs/B1/notes.md), [B2](runs/B2/notes.md). The shared
filesystem was not an OS sandbox; instructions and disclosures cannot rule out
unobserved reads. The root-only SMOKE run is excluded from all author totals.

Authors inherited the same session model/settings. Exact model revision, token
use and compute budgets were not exposed. There was no hard wall-clock/token
cutoff. [Launch timestamps](launches.json) were recorded after each pair was
dispatched and are descriptive, not speed measurements. First submission means
first source executed through the runner; private drafting is unobserved.

The public runner and scorer parse/stringify JSON. Raw duplicate keys and
nonfinite numeric spellings are not covered by this study. Repeated tasks,
prior investigator knowledge, one model family and the joint documentation/runtime
change limit broader conclusions. [Reproduction instructions](README.md)
preserve exact packet hashes and explain the registered Windows build requirement.

## Runtime validation and subsequent guidance

The new intrinsic has 16 dedicated native tests and five module-compatibility
regressions. All **461 Rust tests** pass; formatting and clippy pass. The clean
compiled browser build passes seven elapsed-clock WASM checks, including
fractional timing, clock-only binding invalidation, rollback, journal agreement,
restore, large values and signed zero. Glowcap, Trail Rescue and Slime Glow
policy checks pass, as do Glowcap's browser flow and Trail Rescue's desktop
and mobile flows. These are local checks, not a claim that CI ran. See the
[validation record](results/runtime-validation.json) and
[WASM report](results/elapsed-clock-wasm.json).

After the cohort finished, the live authoring guide gained a short reminder
that `#` and `//` introduce line comments and `--` does not. That reminder is
not in the frozen packet, has not been tested with another fresh cohort, and
receives no credit for these results. No additional syntax was added.
