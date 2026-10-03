# CAVEAT 0.1.0-rc.10 development

Status: development open; implementation and verification pending. This is not
an npm publication or release-readiness record.

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
3. Correct the explanation/exclusivity claims, the source-text display example,
   and the view field documentation. Preserve factual historical release records.
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
- Reproductions, implementation, review and final verification: pending.
