# Release-readiness authoring exercise

This is an application of the frozen language to Caveat's own release work.
It records what was verified about rc.20 on October 15, 2026, and asks what
would be required to assess one future stable candidate. It does not declare
stabilization finished, approve 1.0, change the release procedure, or schedule
background work.

## Observed history and hypothetical cases

[`observed-2026-10-15.events.jsonl`](observed-2026-10-15.events.jsonl) is a
retrospective record of these actual observations, not a live monitor:

| Observation | Receipt and scope |
| --- | --- |
| rc.20 was published and its package verified | [Publication record](../../docs/releases/v0.1.0-rc.20-npm-publication.json), verified `2026-10-15T20:51:15.212Z`; package revision `d9e398e2a6e39c489401ce7785994226966bb364`, tarball SHA-256 `8b4e6b4855e822c824f854f36d4bda1b83c7792e784e00ed25505ecec9d4465a` |
| The later closeout main passed CI and Pages deployment | [Runtime 38813800644](https://github.com/WSattazahn/caveat-lang/actions/runs/38813800644) and [Pages 38819733092](https://github.com/WSattazahn/caveat-lang/actions/runs/38819733092), revision `6704c0a9fb827a066760493aec25349b0ffb2703`; this is distinct from the package revision |
| npm channels and the official MCP Registry named rc.20 | [Distribution observation](../../docs/releases/v0.1.0-rc.20-distribution.json), fetched `2026-10-15T21:02:59.932Z`; this says nothing about a future stable package |
| The observation period was still incomplete | [Approved rc.19 plan](../../docs/releases/rc19-development.md), Q1: two weeks of fixes only; [rc.20 continuation](../../docs/releases/path-to-1.0.md#maintenance-follow-up-2026-10-15) keeps the October 12 chronology. October 15 is before an October 26 review. |

The recorded `stabilization@1` decision means only that these preview receipts
were reviewed. Its grounds are `preview_publication`, `preview_ci`, and
`preview_listings`. The final `assess_stable` event is deliberately refused:
`No stable candidate selected.` The final bindings are
`preview.status = "verified rc.20 observations"`,
`stable.status = "missing candidate"`, and `stable.revisions = 0`.
No stable candidate, completed observation period, or stable owner approval
is present in this observed history.

[`release-readiness.scenarios.json`](release-readiness.scenarios.json) uses
fresh sessions. RR01 checks that actual history and adds duplicate-call and
save/resume probes. RR02–RR13 are explicitly hypothetical acceptance tests;
their identifiers beginning `hypothetical:1.0.0` do not identify a built or
published artifact. A successful hypothetical scenario is not release evidence.

## Authored policy

[`release-readiness.cav`](release-readiness.cav) accepts one stable candidate
per session. The host supplies one exact identity containing version,
revision, and tarball SHA-256, verifies it, and labels its kind. Preview
evidence cannot stand in for stable-candidate evidence. A different candidate
requires a fresh session; the old session remains available as a record.

Stable assessment requires passing checks, compatibility review, supported-
contract defect triage, and a completed dated observation-period review.
The checks, compatibility review, and triage must name the selected artifact.
The owner grant is recorded separately with
`permitted by latest(approvals) for candidate`. This is a model of reported
permission; the host must still obtain and enforce actual authorization.

Every new receipt in those channels, including an equally positive one,
reopens a stable assessment in force. This is an intentionally conservative
application policy, not a default of the language. Re-polling the current
receipt is refused without changing the session. The program only compares
the current receipt per channel: detecting stale replays of older receipts,
verifying receipt contents, and deciding whether a receipt is genuinely new
remain host responsibilities. A repeated ID with altered content is refused
too; correcting a receipt requires an explicit correction and replacement.

`correct_checks` withdraws the current check observation with a recorded
reason and reopens the decision if it rests on that observation.
`revoke_approval` withdraws the current grant and reopens a decision using
that grant. Neither rewrites frozen grounds. Replacement evidence earns a
new revision, and the unrelated preview decision remains unchanged. This
small exercise exposes corrections for checks and owner grants; extending
it to corrections of other receipt categories requires equivalent explicit
rules and scenarios, not silently overwriting the old observations.

The stable decision's numeric value is the sum of its four evidence-bearing
checks. It reads those grounds; it is not a confidence score. The displayed
status comes from the decision state. A value of `4` in a synthetic scenario
does not authorize publication.

## Acceptance cases

| Case | What must hold |
| --- | --- |
| RR01 | Real rc.20 observations do not approve stable 1.0; repeated preview calls refuse; resume agrees |
| RR02 | Preview identity, a different artifact, and changing candidates cannot borrow evidence |
| RR03 | Every missing evidence category and absent owner grant prevents assessment |
| RR04 | Failed checks, compatibility, triage, and incomplete observation period each prevent assessment |
| RR05 | Polling the current receipt, including with changed contents, changes nothing |
| RR06 | A new green receipt reopens; old grounds remain frozen; a new revision uses the new receipt |
| RR07 | Correcting checks withdraws evidence and reopens stable readiness; preview grounds remain unchanged |
| RR08 | Revoking approval reopens; checks stay unchanged; a new scoped grant permits reassessment |
| RR09 | A new failed check blocks reassessment without erasing the prior approval |
| RR10 | A ninth check observation is refused atomically at the declared history capacity |
| RR11 | A fifth decision revision is refused atomically at the declared decision capacity |
| RR12 | Invalid payloads and corrections without observations cannot invent evidence |
| RR13 | Fractional report values cannot update receipt identity without recording an observation |

The scenario runner additionally checks rejection atomicity, final save/restore,
and grounds within lineage for every case. The six explicit resume steps
check restored-state equivalence; those followed by events also check
continuation against the pre-resume session.

The program retains at most eight readings per channel, eight correction
occurrences per correction source, four stable decisions, one preview decision,
and 48 identifiers. Reaching a cap is a refusal, not automatic rotation. These
are finite histories, not a claim that windows universally bound saves or that
the application can run forever. Archive the record and deliberately start a
new review when its scope or capacity changes.

## Local preflight

The initial preflight used the already installed published
`caveat-lang@0.1.0-rc.20` under
`test-results/rc20-published-consumer/node_modules/caveat-lang`. Its lockfile
integrity matches the publication record. This is local CLI execution on
Windows, **not Glama execution**. It establishes no connected remote MCP server.

From the repository root, with that installation available:

```sh
node test-results/rc20-published-consumer/node_modules/caveat-lang/bin/caveat.mjs validate experiments/release-readiness/release-readiness.cav
node test-results/rc20-published-consumer/node_modules/caveat-lang/bin/caveat.mjs check experiments/release-readiness/release-readiness.cav --strict
node test-results/rc20-published-consumer/node_modules/caveat-lang/bin/caveat.mjs test experiments/release-readiness/release-readiness.scenarios.json --json
```

[`receipts/local-preflight.json`](receipts/local-preflight.json) records the
commands, execution and package identities, source hashes, and outcomes.
[`receipts/local-scenarios.json`](receipts/local-scenarios.json) is the full
scenario report. Validation passed, strict check produced no warnings, and
all 13 scenarios passed.

The first authoring run had two incorrect expectations: it assumed a
replacement check would be printed first in `because`. The journal preserves
acquisition order, so the older compatibility, triage, and window observations
precede the newly acquired check. Correcting those expectations resolved both
failures.

Adversarial review then found an actual application-policy defect: a numeric
`min 0 max 1` range admits fractions. A report with `passed = 0.5` (or
`complete = 0.5`) was accepted and updated the current receipt without sampling
any evidence, leaving a prior eligible decision in force. The
[pre-fix reproduction](receipts/fractional-before-fix.json) records that behavior
for all four affected channels against the original source hash. Explicit
binary-value guards now refuse those reports before any mutation; RR13 checks
the complete rollback and verifies that a corrected binary report can reuse
the refused receipt ID. This was an authored-policy bug, not a runtime defect.
The final local receipts identify the corrected source. Earlier successful
12-case results cover only the earlier source and narrower case set.

The inputs remain reports. Caveat does not fetch CI, compare downloaded bytes,
authenticate a source, advance calendar time, discover omitted defects, or
authorize an external release. The host must make those observations before
recording them. In particular, `window_reported complete=1` requires an actual
dated review under the owner's release policy; passing time or passing this
test file cannot supply it automatically.

## MCP execution

The [MCP verification summary](receipts/mcp-verification.json) records actual
JSON-RPC calls through the published rc.20 MCP server. Local stdio execution
covered all five tools on both `2025-11-25` and `2026-07-28`, using the corrected
multiline source and the equivalent comment-free, single-line form needed by
Glama's Inspector input. Both forms passed all 13 scenarios, strict check had
no diagnostics, and every local operation returned `exitCode: 0`.

| Final input | SHA-256 |
| --- | --- |
| Corrected multiline source | `df920a79d46bd802f836443bbbc6f71cae298032d80dfdec373dbbb9ab94f65b` |
| Inspector source | `83e17dfdcb776cefd0fc5a49ec388ceede44ae868024c821cc6e783bcc7efbc0` |
| Thirteen scenarios | `6b24010eb09bdca8c3816fbf7b2dd9d6b30e5b85ec9ca709dae8ee6b19984e03` |

Each source/profile combination also explained the supplied preview history
and traced `preview_publication`, then replayed the synthetic RR07 send events
and traced `checks@1`. The actual history retained its expected refusal of
stable assessment. The synthetic trace preserved the old decision's grounds
after correction and committed `stable_release@2` using replacement evidence.
The complete structured reports agree across both source forms and profiles
after normalizing only their exact source-hash values. Scenario checkpoint,
resume, and assertion steps ran through `caveat_test`; explanation and
dependency queries received only the corresponding send events.

Glama's authenticated MCP Inspector produced **four successful tool results
and two transport failures**. Before the fractional-value correction,
`caveat_validate`, `caveat_check`, and `caveat_explain` succeeded, while
`caveat_test` failed at the transport layer. With the corrected Inspector
source, `caveat_dependents` succeeded and matched a local replay of its three
supplied events exactly;
the subsequent `caveat_validate` attempt failed at the transport layer and
returned no tool result. Those failures remain recorded. The complete
corrected 13-case suite has therefore **not been verified on the hosted
deployment**, and Inspector access does not establish a connected Codex host
tool.

Full timestamped local requests, responses, source snapshots, and the mixed
remote observations remain in ignored `test-results/release-readiness-mcp`.
The tracked summary preserves their hashes and scope; no token-bearing
Inspector URL is included. These observations do not authenticate the supplied
release facts, finish the stabilization period, or authorize a stable release.
