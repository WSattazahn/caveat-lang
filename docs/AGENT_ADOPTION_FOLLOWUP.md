# Agent adoption follow-up

Recorded October 1, 2026 after the owner asked to retain the adoption proposal
as follow-up work. This is a backlog, not additional rc.7 implementation or a
release gate approved by that discussion.

The active rc.7 scope is the evaluated lifecycle/qualification guidance,
application grounding tests, authoring diagnostics, the `caveat-lang` executable
alias and coexistence test, and approved neutral reveal. Public `whatif`,
`abstain`, and scenario string matchers remain deferred. Neutral reveal was
explicitly approved after the earlier proposal to defer it.

## Proposed work and acceptance evidence

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
