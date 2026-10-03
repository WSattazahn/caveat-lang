# CAVEAT 0.1.0-rc.10 development

Status: development open; the accepted scope is implemented. Verification and
review are tracked in [PR #94](https://github.com/WSattazahn/caveat-lang/pull/94).
This is not an npm publication or release-readiness record.

## Starting point

Development starts from completed rc.9 documentation/main revision
`1ce7969b4f386c1dc96a204a0f1a253b5e9a567b`. The published rc.9 package is frozen at
`7ff92b145ac9ce6f55588911c52ddc780b122916`; its tarball, tag, receipts and npm
channels remain the release baseline. See the [rc.9 record](v0.1.0-rc.9.md).
Only `kit/package.json` owns the candidate version. Bundled installation blocks
are generated from that version; they describe the candidate's own identity.
Current public installation guidance continues to identify published rc.9.

## Accepted scope

1. Resolve the recovery contract for exhausted examination attention and empty
   `caveated()` witness selection. Reproduce the rc.9 outcomes, classify these
   specific recoverable failures explicitly, and preserve whole-event rollback,
   history, evidence, sequence numbers, cues and subsequent session usability.
   Keep unrelated fatal errors fatal and align the dispatch/feature specs.
2. Make withdrawal visible in `dependents` JSON and text. Retain the decision's
   authored status and frozen grounds; show the withdrawn observation, its
   reason and recorded event/sequence. Cover evidence, streams, revisions and
   permission dependencies without implying that withdrawal automatically reopens.
3. Correct the explanation/exclusivity claims and view field documentation;
   verify the reported source-text display example against the runtime. Preserve factual historical release records.
4. Turn the selected Meta Muse reports into small repository regression cases.
   Keep reported measurements separate from independently reproduced results.

Meta Muse's supplied FINDINGS(2).md sections 33/35 and 62 motivate the recovery
cases; REPORT.md's withdrawal comparison motivates `dependents`. The supplied
reports remain external evidence: their broad conclusions, performance claims,
version summaries and proposed `abstain` syntax are not accepted as new language
contracts. No new syntax, broad failure reclassification or adoption claim is in
this scope.

## Acceptance and verification

- Pin the reproduction source, events, exact published package and actual
  outcomes before changing the relevant behavior.
- Require focused failing-then-passing tests for each recovery/visibility fix.
  Exercise native full and reactive-only Rust, built WASM, kit/CLI consumers,
  structured outcomes, save/restore, and a successful event after refusal.
- Run the repository's formatting, both Rust lint/test configurations, build,
  kit tests, package documentation checks, browser/installed-package checks,
  existing Lean gates and Runtime CI on the final review commit. Preserve the
  registered historical study inputs. Add independent review of the final diff.
- Keep verification receipts outside tracked source, identify the exact tested
  revision and report failures or unavailable checks explicitly.

Publication is a later step under the [release procedure](../CONSOLIDATION_PLAN.md#candidate-and-release-gates):
final-main and Pages verification, a retained clean Linux package, exact-package
security/install checks, a reviewed tag and release authorization precede npm
publication. Opening this development cycle does not publish a candidate.

## Progress

- Development scope and candidate identity opened.
- Exhausted examination attention now refuses with `limit/attention_limit`;
  an empty reopening witness selection refuses with
  `evaluation/empty_caveated_selection`. Both roll back the whole event and
  allow a later event. No evidence or attention is invented to recover.
- `dependents` now includes query-matched withdrawal records in JSON and text,
  including occurrence, reason, event and sequence. Decision status and frozen
  grounds still come from the authored history. The
  [dependents contract](../../spec/caveat-dependents-0.1.md) defines matching,
  permission roles and compatibility with earlier report data.
- Public explanation claims now distinguish checked dependency references from
  true evidence, truthful prose and complete explanations. The comparison with
  general-purpose languages describes supplied facilities rather than exclusive
  expressive power. The view field table now includes `decision_journal`.
- The reported display syntax defect did **not** reproduce with published rc.9:
  the unquoted label works when the specification's minimal example is supplied
  a reactive event. Quoting the label changes its key and loses the intended
  evidence-label association. The valid syntax is retained, the event requirement
  is documented, and an executable documentation test protects both the label
  association and the view fields.
- Regression cases cover the two exact rc.9 fatal outcomes, rollback after partial
  work, both dispatch result shapes, restore and continued execution, native and
  WASM runtimes, repository hosts and a fresh package installation. Withdrawal
  cases cover direct/stream/template/caveat queries, permissions, historical
  revisions, unrelated sources and CLI/Serve/inline authoring consumers.
- Browser screenshot review also exposed a missing UTF-8 declaration in the
  legacy Door page. A charset-free HTTP reproduction decoded its title as
  Windows-1252; the declaration corrects it, and the existing route check now
  asserts the exact rendered title. No scene or material settings change.
- Investigation of a CI process-cleanup failure reproduced a race in the Python
  test checker: a reaped process could be reported running, including after a
  stopped observation. The checker now rechecks a missing process conservatively
  and retains a confirmed stop; deterministic regressions also retain failure
  for a genuinely live process. This proves the checker defect, not the exact
  cause of that CI failure. Production process shutdown is unchanged.
- Reproduction sources, runtime identity, before/after outputs and local command
  receipts are retained under `test-results/rc10-development/` in the working
  checkout; these generated receipts are not source files. PR checks supply the
  independent clean Linux build, package/security, browser and Lean evidence.
  Their completion must be checked on the final review commit.

## Verification boundaries

The four game-workspace aliases `npm run typecheck`, `npm run lint`,
`npm run test` and `npm run audit:all` were attempted in this repository;
all report a missing script. They do not replace Caveat's documented Rust,
kit and Runtime workflow commands listed above.

The local builder uses Windows and has the owner's separate rc.9 root dependency
installation edits. Its build identity therefore records a dirty checkout and
is not a clean reproducible release artifact. Those dependency edits are excluded
from this work. The clean Linux PR gates remain required. Existing Lean checks
cover their stated fragment; this work does not claim new proofs for examination
attention, reopening selectors or withdrawal reporting.
