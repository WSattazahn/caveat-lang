# Fresh-agent authoring results — 2026-09-23 UTC

**Two of six first submissions and all six final submissions passed every
registered test. Two final submissions nevertheless have known contract
violations beyond the tested time range.** The result supports the feasibility
of authoring these evidence-aware policies from documentation, including
self-directed repair. It does not establish generally reliable first-attempt
authoring or complete compliance over every valid history.

The [protocol](PROTOCOL.md), tasks, independent oracle, cases, prompts and
reference manifest were committed in `fba9c78` before author dispatch. Six
fresh contexts used the runtime and documentation from `04f72dc`, with no
parent conversation, implementation access or private feedback. Two contexts
authored each task. Each could submit three source versions and run twelve
validation/replay checks. All six finished before private scoring began.

## Registered outcomes

| Run / task | First submission | Final submission | Source versions | Runtime checks |
| --- | --- | --- | ---: | ---: |
| A1 / cold-storage dispatch | Load error | 114/114 cases | 3 | 8 |
| A2 / cold-storage dispatch | Load error | 114/114 cases | 2 | 5 |
| B1 / access review | Load error | 116/116 cases | 2 | 7 |
| B2 / access review | Load error | 116/116 cases | 2 | 6 |
| C1 / repair queue | 114/114 cases | 114/114 cases | 1 | 6 |
| C2 / repair queue | 114/114 cases | 114/114 cases | 1 | 8 |

The four load errors all name the same restriction: reactive numeric bounds
must be ordered and within `+/-1e12`. They remain failures in the first-source
denominator. Every author repaired solely from their own public runner output;
the supervisor never edited candidates or supplied private findings.

Across final candidates, **688/688 case executions** passed, covering **20,512
event dispatches**: 6,912 accepted and 13,600 rejected. **4,788 restore operations**
also passed. These totals exclude duplicate shadow dispatches and first-source
scoring. The corpus contains 44 targeted scenarios and 300 seeded sequences of
40 steps; each task's corpus is repeated for its two authors. Case executions
are not 688 independent tasks or agent trials.

The verifier compared admission, every required HUD field, ordered observations
and journal entries, direct evidence qualifications, and exact commitment
grounds. Rejections preserved complete saves and views. Restore preserved full
snapshots/views and future behavior against an uninterrupted session. Grounds
remained subsets of runtime provenance. See [raw scores](results/scored.json),
[corpus coverage](corpus.json) and [source/budget integrity](results/integrity.json).

## Failures and remaining uncertainty

The frozen reactive profile describes `-1e12..1e12` as a default state range,
without clearly saying that explicit bounds cannot enlarge it. All four
authors of clock tasks tried larger bounds. That is a repeated documentation
failure, not four unrelated policy mistakes.

- **A2 and B1:** their final sources mirror elapsed time in a single state
  capped at `1e12`. A valid `advance` that exceeds this cap rejects, violating
  tasks that impose no total-time horizon. Both authors disclosed this, and
  source review confirms it. This was not reached through a long replay.
- **A1:** its final source rescales the clock by a power of two. The branch
  first occurs beyond `2^39` elapsed units and was not exercised.
- **B2:** its final source splits the clock into high and low parts. Crossing
  its `2^20` split boundary was not exercised.
- **C1 and C2:** no remaining contract defect was identified on the registered
  corpus or subsequent source review. This is not an exhaustive proof.

The registered histories reach only about **11.4 elapsed units for A** and
**10.8 for B**. A1 and B2's representations appear sound on source review,
but their large-time paths lack runtime evidence. A2 and B1 are therefore
**corpus passes with known contract violations**, not unconditional successes.

## Verification review and corrections

Before candidate scoring, root review corrected a transport error that converted
an invalid null payload into an empty object. Every candidate received the same
correction; the oracle and inputs stayed frozen. The public self-check wrapper
kept its original behavior. The correction, original commit and transport
demonstration are preserved in [VERIFIER_CORRECTIONS.md](VERIFIER_CORRECTIONS.md).

Independent review after scoring found a second coverage gap: the registered
projection checks displayed time and journal metadata, but omits runtime elapsed
and sequence between decisions. A separate [metadata supplement](session-metadata.mjs)
now checks both snapshot fields at initialization, after every corpus step and
after final restore, for every first and final source. It also proves that
deliberately corrupting either field fails. All six final sources pass; the
original first-source load failures remain. [Supplemental results](results/session-metadata.json)
are separate from the unchanged registered scores. The supplement's first
draft incorrectly expected elapsed on the compact view; its failed output and
correction are also retained. Neither review found an additional candidate
policy failure within the registered histories.

## What changed afterward

The live [reactive profile](../../../spec/caveat-reactive-0.1.md) now states the
hard inclusive numeric declaration limits. The [authoring guide](../../../docs/AI_AUTHORING.md)
now asks authors to plan accumulator duration, units and representation, and
distinguishes numeric state from the runtime clock. The frozen packet, original
failures, final sources and runtime were not changed. The improved guidance
has not been tested with another fresh cohort.

Clock mirroring was the recurring difficulty: authors needed to display time
that the runtime already maintains, then invented extra representations to
avoid state bounds. A source-level way to read the runtime clock is a concrete
language/tooling follow-up suggested by this study; it is not implemented or
credited as a result here.

## Limits and reproduction

All authors inherited the same session model/settings. Exact model revision,
token use and compute budget were not exposed. There was no token or time
cutoff, and parallel elapsed times are not a speed benchmark. `record.started`
means first runner invocation; dispatch times are in [launches.json](launches.json).

Isolation used fresh contexts and instructions on a shared filesystem, with no
OS sandbox or exhaustive read telemetry. All authors reported no access
deviations; none was observed. Prompts, every submitted source revision, check
input/output and author disclosure are retained in `runs/A1` through `runs/C2`.
Draft work before first submission is not observable. `runs/SMOKE` is excluded.

The synthetic policies were detailed, capacities small, and seeded sequences
share fixed prefixes. Rejections dominate some runs: the repair corpus has
624 accepted and 2,780 rejected events per author. There was one seed per task,
no other model family, no human adoption study, and no production deployment.
Raw nonfinite values and duplicate JSON keys were not faithfully exercised by
the public JSON normalization path. No comparative reliability, token-cost or
performance advantage is inferred.

All nine oracle/scoring tests and the integrity audit pass. The runtime and
browser application did not change; their previous checks were not rerun for
this study. Use the [reproduction commands](README.md#reproduce-verification)
to rebuild the pinned packet and re-evaluate every preserved candidate.
