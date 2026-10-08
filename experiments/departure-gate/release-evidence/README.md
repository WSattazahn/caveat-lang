# rc.16 retained development evidence

These are selected, unmodified historical reports and raw observations from the
local development work through `590fae59aa4295ac5209f930a1e58ac152bd4603`.
They are not new measurements of the integrated release revision. Source commits
remain in this branch's ancestry. The rc.16 release candidate report records fresh
integrated verification separately.

| Record | Finding carried into release review |
| --- | --- |
| [Renewal removal](renewal-removal-profile/REVIEW.md) | Profiling only; batching deferred, quadratic bulk scaling remains on the agenda. |
| [Event phases](event-phase-profile/REVIEW.md) | Profiling only; archive construction motivated the approved early-boxing experiment. |
| [Early boxing](early-boxing/REVIEW.md) | Primary mutual3000 additional requested-heap peaks decreased 20.485%; finite behavioral comparisons and registered gates passed. |
| [High-degree confirmation](high-degree-confirmation/REVIEW.md) | Eight alternating pairs: successful release +6.67115% median paired latency (+15.45 microseconds); growth +0.82154%, rejected release +2.75810%. Only successful release crossed the strict 5% review trigger. |
| [Adversarial pass](adversarial-security/REVIEW.md) | Candidate/archive/withdrawal/collection cases and valid controls passed. Original Muse F256/F358/F362/F363/F364 inputs were unavailable and were not reproduced. |

Release decision: retain early boxing with the measured successful-release
tradeoff. The result does not satisfy a workload budget requiring at most 5%
successful-release degradation. No unrestricted interactive-workload guarantee
is made. Required-chain retention, archive growth, temporary peaks, retained
allocation capacity and bulk-event latency remain follow-up engineering work.
Absence of the unavailable Muse inputs alone is not a release blocker; a concrete
failure or material missing case should reopen that assessment.

Each `raw-observations.zip` preserves original relative paths for raw streams,
attempt records, frozen contracts, identities, fixtures and relevant result files.
`manifest.json` lists every copied report and ZIP member with its original SHA256
and byte length. Executables, redundant source mirrors, caches and private receipt
stores are excluded. For the renewal investigation both registration attempts are
retained; adversarial harness failures remain recorded beside the corrected runs.

The historical REPRODUCE documents describe their **complete original bundles**;
not every dependency they name is duplicated in this selected repository record.
Do not interpret those old permissions or absolute host paths as current commands.
Complete original bundles remain available with the release handoff. Their source
and all original observation files have been preserved without modification.

Run this repository selection's read-only integrity and high-degree arithmetic
check from any directory:

```text
python -B experiments/departure-gate/release-evidence/verify.py
```

This verifies retained bytes and recalculates the three high-degree medians from
all eight raw pairs. It does not rerun Caveat, authenticate observations, repeat
the full historical independent reviews, or validate a release artifact.
