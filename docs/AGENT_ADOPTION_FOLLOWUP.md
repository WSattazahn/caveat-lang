# Agent adoption follow-up

Recorded October 1, 2026 after the owner asked to retain the adoption proposal
as follow-up work. The owner subsequently authorized working through the
recommended order. Implementation proceeds on `codex/agent-adoption`; this
authorization does not approve publication or make every item a release gate.

The active rc.7 scope is the evaluated lifecycle/qualification guidance,
application grounding tests, authoring diagnostics, the `caveat-lang` executable
alias and coexistence test, and approved neutral reveal. Public `whatif`,
`abstain`, and scenario string matchers remain deferred. Neutral reveal was
explicitly approved after the earlier proposal to defer it.

## Execution order

1. Doctor and the deterministic agent demonstration — implemented and tested.
2. Onboarding contract and README first screen — implemented and checked.
3. Thin MCP authoring bridge — implemented; focused and official-client checks pass.
4. Packed-release dependency security checks — implemented; exact local candidate passes.
5. Preregistered cold-agent adoption experiment — protocol and scorer reviewed; registration next.
6. Dated name audit before 1.0.

Each item requires the acceptance evidence below before it is recorded as done.

## Work and acceptance evidence

| Follow-up | Desired outcome | Evidence needed before calling it complete |
| --- | --- | --- |
| `caveat-lang doctor [--json]` | Diagnose Node support, package/runtime/WASM identity, a working test session, and which installed executable each name resolves to. | Structured/text output agree; missing, incompatible and corrupted runtime cases fail clearly; both actual npm shim owners are exercised; inspecting another executable never runs it. |
| One-command agent demonstration | Show supplied evidence, a decision's exact grounds, a correction/new observation, reopening and preserved earlier grounds. | Fresh installed package completes a short deterministic run; output comes from real snapshots/explanations; file creation, if any, is explicit and refuses overwrites. |
| Thin first-party MCP authoring bridge | Expose existing validate, check, test, explain and dependents results to agent hosts. | A separate interface review defines source/file boundaries, resource limits, structured errors and process ownership; no persistent sessions or hypothetical isolation promises in the first version. |
| Onboarding contract | Help unfamiliar agents discover supported syntax, reproduce missing capabilities and report useful feedback. | Audit the existing repository AGENTS.md, authoring guide and Agent Feature Request template before adding another entrypoint; preserve user authorization for posting reports. |
| Packed-release dependency security gate | Assess the actual shipped npm artifact and Rust dependencies rather than only the development checkout. | Define shipped/development scope, advisory sources, reproducible receipts, severity handling and an explicit resolution process for High/Critical findings; verify the actual candidate artifact. No scan result is claimed here. |
| README first-screen revision | Show the problem, a few working commands and an observable payoff before detailed language theory. | Commands match the actually published version; an unfamiliar reader/agent can reproduce the displayed result. |
| Cold-agent adoption experiment | Measure discovery, authoring, checking, scenarios, diagnosis, explanation, host integration and honest handling of missing capabilities. | Register tasks/scoring and package/build identity before runs; give several fresh agents only the intended public package/docs; preserve first attempts, interventions, failures and final artifacts. Do not relabel the earlier single synthetic trial as this study. |
| Broader name audit before 1.0 | Record npm, GitHub, PyPI, crates.io, other package managers, papers, domains and trademark-database findings once. | Dated primary-source references, exact identities and field overlap; distinguish verified records from unanswered questions. The completed npm coexistence fixture is only one part of that audit. |

## Existing foundations

- [Repository agent instructions](../AGENTS.md) already describe the official
  caller, supported tools and a permission-aware feature-request path.
- [Agent Feature Request](../.github/ISSUE_TEMPLATE/agent-feature-request.md)
  already asks for versions, a minimal reproduction, expected/actual behavior,
  existing alternatives and an acceptance check.
- [CLI naming](../kit/docs/NAMES.md) documents the rc.7 alias, rc.6 distinction,
  and the competing `caveat` executable. The pinned regression fixture is in
  `scripts/fixtures/cli-collision/`.
- [The rc.7 evaluation](RC7_AGENT_EVALUATION.md) and
  [neutral reveal specification](../spec/caveat-neutral-reveal-0.1.md) retain the
  language's evidence, qualification and frozen-history requirements.

The proposed adoption work should make these capabilities easier to discover
and use. It does not authorize a memory-store product, a project rename,
publication, or feature-for-feature competition with another CAVEAT project.

## Execution receipts

Doctor/demo: commit `0d0f178`; 196 kit tests passed before the final demo review
fix, followed by 15 passing demo/CLI checks and 10 passing doctor checks for the
final owned modules. A preliminary installed tarball passed both collision
owners, doctor/demo aliases, library, Python and browser checks (SHA-256
`6c143069ef685aaa55fb581658731e890edcd0f2c01c10979d6acb3f1c1ef155`). Its dirty-build
metadata makes it development evidence, not a releasable candidate. The final
candidate package is checked again after the remaining package changes.

Onboarding: actual fresh `caveat-lang@0.1.0-rc.6` install/init/test/explain passed;
the example's two scenarios and displayed decision history match the README.
All 19 documentation/command tests passed. Existing feature-request instructions
already covered permission, reproducibility and acceptance; they were retained.

The requested root npm scripts typecheck, lint, test and audit:all are absent;
the attempted commands failed as missing scripts. They are not passing gates.

MCP bridge: [interface review](MCP_AUTHORING_DESIGN.md) precedes implementation.
All 32 final operation/protocol/declaration checks pass, including injected
worker faults and cancellation/EOF under stdout backpressure. The pinned
official SDK 1.31.0 discovers and calls all five tools against the checkout;
its same interoperability check is registered in the installed-package gate.
No live desktop-host installation is claimed.

Final local candidate: clean compiled revision `a24a6e91d75eeab466baa32a108ee9a242f43ac2`;
all 229 kit tests pass. The installed package gate passes both executable names,
collision owners, doctor/demo, the official MCP client, library, published-guide
flows, Python suites and Chromium 140.0.7339.186. Package report:
`test-results/kit-package/2026-10-02T00-40-22-944Z/report.json`.
Tarball SHA-256: `eea2c9239f3d648a1af740843abee5ed1eac1c972c327793bb9af2ede36b9a1f`
(810,225 bytes). This Windows-built candidate is the study artifact; publication
still requires the reviewed Linux release artifact and separate authorization.

The [security gate](PACKAGE_SECURITY.md) passed against those exact bytes on
2026-10-02 at 00:44 UTC: all five npm scopes reported zero findings; the complete
Rust build lockfile reported zero vulnerabilities and zero informational
warnings against RustSec commit `6de4455103aced2cba86e3b86e5c090b22827cf1`.
Receipt: `test-results/kit-security/2026-10-02T00-43-55-847Z-66076/report.json`.
Eight refusal controls pass. Playwright is patched to 1.55.1; the real competitor
fixture keeps caveat-cli 0.19.13 with its explicitly recorded smol-toml 1.7.1
override. These are known-advisory results at the recorded time, not a general
security certification. CI retains distinct receipt artifacts for each retry.

The study's [protocol](../experiments/agent-adoption/v1/PROTOCOL.md), task and
private scorer were reviewed before registration. Sixteen controls pass,
including a valid alternative implementation and targeted broken variants.
Trial results are recorded separately from these evaluator tests.
