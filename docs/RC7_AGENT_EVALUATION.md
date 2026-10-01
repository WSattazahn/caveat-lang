# Caveat rc.7 agent integration evaluation

Evaluation date: October 1, 2026 (America/Los_Angeles). The review is split between
`codex/rc7-agent-evaluation` (documentation/examples/tests) and
`codex/rc7-authoring-diagnostics` (runtime diagnostics and focused tests).
This is an evaluation and development candidate,
not an rc.7 publication or a claim that the unavailable Muse/Dot applications pass.

## Baseline and evidence

| Identity | Verified value |
| --- | --- |
| rc.5 tag's commit | `c7de1205ed278dd26e7adae43c115777a4eb435d` |
| rc.6 tag's commit | `21523d41cbd4077996ec0625e33e172d1c57a2d9` |
| Local and live GitHub main at inspection | `21523d41cbd4077996ec0625e33e172d1c57a2d9` |
| rc.7 development/evaluation base | `3eab7b37868b24586d739b6dc76fc682902bbe95` |
| Existing development branch | `codex/record-rc6-start-rc7`; PR [76](https://github.com/WSattazahn/caveat-lang/pull/76) was open/unmerged |
| Changes from rc.6 to main | None at inspection |
| Changes from rc.6 to development base | One documentation/package metadata commit; kit version advances to rc.7; no runtime edits |
| Publication | npm `next=0.1.0-rc.6`, `latest=0.1.0-rc.5`; no rc.7 publication at inspection |

The npm rc.6 tarball was independently read and hashed: 760,138 bytes,
SHA-256 `98a4b0b21c3dc5dead74c09835373838dc586650a4ee31a1d8ef85090d527447`.
That matches the GitHub release asset metadata and annotated tag. npm published
rc.6 at 2:00 p.m. PDT and GitHub published its prerelease at 2:54 p.m. PDT.
[rc.6 release](https://github.com/WSattazahn/caveat-lang/releases/tag/v0.1.0-rc.6).
No release, tag, main branch or published package was changed by this evaluation.

The original [combined brief and later whatif note](../experiments/agent-authoring/rc7/evidence/README.md)
are preserved as supplied. The user clarified that Muse and Dot's feedback was
pasted into chat, then organized into the brief. Original source, scenarios,
Python integrations, dependencies and run logs were not supplied or found in
the bounded local search. Muse's claimed unchanged starter copy cannot be
compared. New tests below are independent reproductions, not recovered originals.
The unrelated Dot rc.5 checker ZIP with 22 scenarios is excluded.

## Decisions

| Candidate | Muse evidence | Dot evidence | Reproduced on rc.7 baseline? | Existing solution? | Change justified? | rc.7 disposition |
| --- | --- | --- | --- | --- | --- | --- |
| lifecycle documentation | v2 separates observation/assessment and uses explicit reopening | second assessment rejected; flow unclear | Yes: duplicate refused; shipped supporting observation leaves old approval; every-observation policy reopens | Authored `reopened by` plus separate sessions | Yes, explain policy choices with executable examples | **DOCUMENT rc.7** |
| qualification documentation | v2 reports archived/future distinction unclear | No separate report | Yes: direct template-dependent value changes, archived occurrence and frozen grounds do not; Q1 reuses old clean grounds | Existing identity and qualification semantics | Yes, explain six distinct records and resampling | **DOCUMENT rc.7** |
| custom integration tests | v2 permit description checks new commitment, but original grounds checks unavailable | rejected operation must defeat old approval | Yes: new commitment with wrong or old grounds can pass a generic permit test; exact custom check refuses | `Attempt` plus an application-specific permit callback | Yes: L1-L8, Q1/Q2, exact sources and explicit carry-over | **IMPLEMENT rc.7** |
| stance-less reveal | v1 reports artificial `memory supports memory_trustworthy` workaround; v2 source unavailable | No report | Neutral syntax rejected; sampling does not observe template; exact v2 workaround unverified | A truthful stance may fit some models; does not represent neutral consultation itself | Real language-modeling gap; compatibility design not complete | **DEFER with concrete missing evidence**: independent observation-record design and complete save/renewal/order/withdrawal/consumer compatibility tests |
| whatif | wants counterfactual execution without risking live state | No report | Separate-runtime API branches work; injected actual WASM trap demonstrates shared-instance failure boundary | Save into another runtime/process with same source/build | Useful potential isolation feature, but public lifecycle/resource contract not complete | **DEFER with concrete missing evidence**: bounded disposable-runtime/process contract, crash/resource tests and complete operation acceptance suite; document existing recipe now |
| abstain | v1 distinguishes non-choice from choosing clarification; v2 successfully chooses clarification | No report | No original non-choice application to rerun; new primitive absent | A substantive clarification strategy already expresses a choice to clarify | Insufficient verified need for a new decision state | **DEFER with concrete missing evidence**: real non-choice integration plus complete five-transition state/query/restore/actionability contract |
| `because` diagnostic | known claim described as unknown numeric identifier | No report | Yes, on baseline WASM | Correct grounding checks existed; messages conflated kinds | Yes, diagnostic-only change with native and WASM tests | **IMPLEMENT rc.7** |
| `define` diagnostic | `define freshness_last()` gave generic identifier error | No report | Yes, on baseline WASM | `define NAME = â€¦`; `fn` only for pure explicit inputs | Yes, targeted hint without suggesting capture | **IMPLEMENT rc.7** |
| scenario matcher | wants Proceeding prefix while confidence varies | No report | Exact mismatch and unsupported matchers reproduced; structured action/score assertions work | Assert decision, exact numeric score and relevant display contract separately | No verified need beyond existing structural assertions | **DEFER with concrete missing evidence**: concrete presentation-only prefix/substring contract paired with numeric assertions; unrestricted regex not justified |

Deferrals are decisions for this candidate, not claims that the larger proposals
lack value. A truthful `memory_available` stance in the qualification example
is that example's authored model; it does not prove Muse's neutral-consultation
requirement has been solved.

## What rc.6 already supplied

The official `CaveatServer` handles subprocess launch, JSON-lines requests,
IDs/response matching, protocol and request failures, fatal session disposal,
timeouts and process-tree cleanup. `Attempt` records required operations,
accepted/rejected/unconfirmed/not-sent results, concurrency boundaries and a
frozen final result. It combines operation completion with the application's
current permit callback. A stored approval cannot substitute for a rejected or
unfinished operation, and a later attempt can succeed after a failed one.

These are reusable implementation responsibilities, verified by the unchanged
starter suite. Muse and Dot independently report that they avoided writing
this support code. Their original clients are unavailable, so no deleted-line
count, causal effect size or broad reliability claim is made. Task identity,
reopening policy, evidence selection and application-specific permits remain
the integrator's responsibility.

## Lifecycle and identity results

[Assessing again](../kit/examples/agent-evidence/README.md#assessing-again)
contains tested executable examples. `test_lifecycle.py` covers L1-L8 and task
isolation: complete serialized rejection rollback; unchanged shipped approval
after supporting evidence; every-observation revisions; opposition; rejected
required operation after approval; later valid retry; unfinished registration;
and save/restore with continued accepted and rejected events. The unchanged
starter suite separately covers actual interrupted and in-flight operations.
These overlapping checks are not independent proofs of the same behavior.

[Qualification and exact grounding](../kit/examples/agent-evidence/QUALIFICATION.md)
distinguishes template, occurrence, current dependent state, archived reading,
frozen decision and later observation. Q1 learns stale and reassesses without
resampling: the new decision still uses `freshness@1` without stale in its
grounds, while `direct`, which depends on `memory`, carries stale. A later
`freshness@2` inherits stale. This follows dependency identity, not a global
past/future rule. Q2 withdraws clarity, reopens, and refuses reassessment without
replacement at the application-policy level; a later valid attempt succeeds.

The three-source example requires exact intended clarity, memory and tool
occurrences. Clarity/tools must come from the current attempt; memory can carry
over only through an explicitly allowed occurrence ID. Tests reject each
missing source, unintended old evidence, wrong reading, missing/extra grounds,
and withdrawn evidence. All operations and a new commitment can succeed while
this stronger application permit check still refuses the result. This policy
is an example, not a universal freshness requirement.

## The whatif gate after the trap finding

[`whatif-isolation.test.mjs`](../kit/test/whatif-isolation.test.mjs) uses the
production session adapter and independent runtime loading. A tiny valid WASM
module executes `unreachable`; the resulting actual engine trap is injected at
the dispatch boundary. This is not a naturally Caveat-source-triggered fault
inside Caveat's compiled memory, and is not process/resource containment proof.

The tests establish:

- Two normal live sessions on runtime A survive the injected trap in B, retain
  complete save/snapshot/view, and continue dispatch/save/restore afterward.
- Reusing the trapped B instance or imported namespace is refused; newly loaded
  hypothetical runtimes work after earlier branches trap.
- The inverse control on one shared runtime invalidates its live siblings.
- Ordinary unclassified arithmetic fatal ends its session without the wider
  trap effect. Accepted branches and policy/input/evaluation/limit rejections
  do not change live state.
- The fixture exercises sequence/time, numeric state, identifiers, readings and
  IDs, decision revisions, qualification, withdrawal, reopening journal,
  effects and cues. Full live serialization is compared after branch steps.
- Exact source/runtime identity is used; mismatched source restoration fails.
- Appending a reading to current history and replacing an earlier event from
  its preceding checkpoint produce different histories.

The [two-process recipe](../kit/examples/agent-evidence/BRANCHING.md) is executable
and tested. No `whatif` API or CLI has been added. A public operation still needs
explicit start/checkpoint semantics, bounded events and time/resources, exact
runtime/source compatibility, fatal result representation, worker/process
ownership and disposal, crash/early-exit tests if using a child process, and
repeated-use acceptance tests. It can describe policy consequences; evaluating
whether an outcome is better remains external or authored.

## Why neutral reveal needs a separate design

The load probe `on consult reveal memory;` is rejected. Today `observed` is
computed from support/opposition edges. Making stance optional in the parser
alone would break that contract. Implementation must introduce a truthful,
authoritative observation record with occurrence identity, ordering and guard
provenance. That record must survive restore, qualification, withdrawal and
renewal without fabricating an edge or silently reopening decisions.

Affected consumers include `Effect::Reveal`, parser/validation, procedure
substitution, observation-order analysis, execution/predicates, snapshots and
effect reports, graph representation, restore's observed-evidence validation,
and kit explain/dependents. Existing stance-bearing sources and old saves need
explicit compatibility tests. Until that complete representation is specified
and tested, this evaluation does not ship a parser-only approximation.

## Fresh-agent trial

[Trial 01](../experiments/agent-authoring/rc7/trial01/RESULTS.md) started with a
fresh context and ordinary docs, without Muse/Dot feedback or a prescribed
reopening policy. It selected every-observation reopening and one session per
task, used the official caller, checked current grounds, preserved corrections,
and modeled late qualification through a direct template-dependent basis.
Its first candidate and final copy, manifests, receipts and commands remain
separate. Five scenarios and seven host tests passed initially with no repair;
the root independently reran validate, strict check, scenarios and host tests
against the rebuilt diagnostic runtime. The first run did not capture its WASM
hash; the independent rerun did. This is one self-authored synthetic trial,
not a reliability estimate or a controlled causal comparison.

## Verification and delivery

The final runtime is built from the working tree on base `3eab7b3` (dirty,
compiled, Windows); reactive WASM SHA-256:
`c110650b0059ca7022f93197a42d195f1a138b1283ddbd6afb5ffba8d84e78ff`.
The LF-normalized rebuild produced the same measured WASM hash as the first
build used by the completed runtime/browser checks. Final local receipts are
in `test-results/rc7-evaluation/`; `SUMMARY.json` records exact gate outcomes
and counts. A dirty candidate must not be described as a tagged release.

Counts stay separate:

| Suite | Outcome |
| --- | --- |
| Original official Python starter | 53 discovered; 52 passed; 1 POSIX-only test skipped on Windows |
| Muse v1 original | Four reported; not independently rerun |
| Muse v2 original | Seven reported; not independently rerun |
| Dot rc.6 original | Eight reported; not independently rerun |
| New lifecycle Python suite | 9 passed |
| New qualification Python suite | 2 passed |
| New exact-grounding Python suite | 9 passed |
| New documented branching Python suite | 1 passed |
| New native diagnostics | 4 passed (also inside both Rust configurations) |
| New WASM diagnostic checks | 4 passed (inside the 175-test kit suite) |
| New JS isolation checks | 6 passed |
| Fresh trial | 5 scenarios and 7 host tests passed; kept separate from starter suites |

Rust format, Clippy in both configurations, full and reactive-only tests, build,
WASM policy checks, Glowcap, Trail Rescue, scenario conversions and mutation
checks passed. Kit browser, Glowcap page and Trail Rescue page gates passed. Current Glowcap
and Trail Rescue mobile screenshots were also visually inspected; the user
retains final interactive judgment.
The historical authoring fixture check loads 12 v1/v2 final sources and runs
12 non-mutant v3/v4 scenario files on the current runtime; it does not rerun
v1/v2 private scoring or change frozen historical results.

The game workspace's `npm run typecheck`, `lint`, `test`, and `audit:all` were
attempted in this target repository. All four are unavailable here (missing
scripts); they are not green gates. Caveat's package/workflow gates above are
the applicable checks. The final kit suite passes all 175 tests. The installed package passes CLI,
library, browser, documentation, original 53-test and new 21-test Python checks.
Its 773,795-byte tarball has SHA-256
`12d03c9a7866cf516f96af6fc8de1544d9e324c5b2efb8cfb9118c58bd629bad`.
The portable [verification summary](../experiments/agent-authoring/rc7/evidence/verification-summary.json)
records these combined-candidate gate outcomes; raw gate logs are local-only. The native suites report
717 full-runtime and 705 reactive-only passes; these overlapping configurations
are not independent proofs. No release is authorized by these tests.

During verification, a CRLF rewrite violated `.gitattributes` and hid Rust
structs from a source-shape test; LF was restored. The one-page reference's
length guard caught excessive new prose; detailed guidance moved to the
normal authoring guide. Isolation-fixture expectations were corrected for
cue-versus-effect reporting and the actual different-program restore message.
One authoring-fixture launch overlapped the rebuild and found no runtime; its
failed receipt is retained, and all 24 checks passed after build completion.

Reviewable work separates naturally into documentation/examples/tests (including
trial evidence and the isolation design constraint) and diagnostic changes with
native/WASM tests. Verification above covers the combined working candidate
before the review split, not an isolated PR head or release certification.
Separate draft PRs preserve that review boundary. No tag, release or registry
publication is part of this delivery. Larger features have explicit deferral
gates above.
