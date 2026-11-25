# CAVEAT Language

[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/15891/badge)](https://www.bestpractices.dev/projects/15891)

**A decision ledger for agents: Caveat keeps an AI's reasons in a record you can check.**

Most programs and agents act on evidence, then lose track of *why*. Later a tool result is corrected, a memory turns out stale, or an assumption fails, and three hard questions follow: **what did this decision rest on when it was made, what else rests on that same evidence, and what needs to be reconsidered?**

Caveat is an embeddable rules language and WebAssembly runtime (Node.js, browser, MCP, Python, no npm dependencies, no Rust needed to install) built for decisions that may need taking back. Every value carries the evidence and caveats it was computed from; every committed decision freezes its exact grounds; and when evidence is withdrawn or qualified later, rules you write reopen the affected decisions without erasing history.

[Limits](docs/LIMITS.md) states up front what Caveat does **not** do (verify that evidence is true, sign saves, or authenticate who sent an event) and what it costs.

> **Published release: `caveat-lang@1.5.0`** (fifth minor release of 1.x; stable under the [1.x compatibility commitment](docs/COMPATIBILITY.md)).
> npm publication verified `2026-11-25T03:28:26.088Z`, tested Linux tarball SHA-256 `96d62570255d5ecc22351195beb45c7e1fc78e6edddda8e35694c0a1ac728dfd`, published by `publish-npm.yml` from tag `v1.5.0` with npm provenance. npm `latest` and `next`, the MCP Registry (`io.github.WSattazahn/caveat-lang`), and the hosted MCP worker are all verified at `1.5.0`.
> [1.5.0 release record](docs/releases/v1.5.0.md) · [Publication receipt](docs/releases/v1.5.0-npm-publication.json) · [What Caveat is for (one page)](https://wsattazahn.github.io/caveat-lang/about.html)

## When Caveat is useful

Use Caveat when decisions get revisited and someone — a user, a reviewer, or another agent — will later ask *"why was this done, and what changes now that this reading was wrong?"*

| Use case | What Caveat gives you | Example |
| --- | --- | --- |
| **Agent undo lists & tool corrections** | Withdraw a bad tool output; reopen only the actions whose frozen grounds depended on it, keeping every original reason intact | [15-minute agent undo-list tutorial](docs/tutorials/agent-undo-list/README.md) |
| **Release, deploy & approval gates** | Record test readings, named thresholds, caveats, and explicit grants (`permitted by`) separately from grounds; reopen when a check is withdrawn or ages | [Engineering readiness](kit/examples/readiness/README.md) · [PR ledger Action](actions/pr-ledger/README.md) |
| **Multi-agent handoffs** | Pass a compact save to a sub-agent, enforce foreclosures (`irreversible forecloses`), hold changes mid-task, and keep sub-agent caveats on return | [Handoff example](kit/examples/handoff/README.md) |
| **Auditable decision memory** | Inspect decisions with `caveat explain`, trace forward impact with `caveat dependents`, search past ledgers with `caveat find`, or step a ledger over MCP (`caveat_dispatch`, `caveat_find`) | [Project's own ledger](ledger/README.md) · [Retracted studies](kit/examples/retractions/README.md) |

**When plain code is better:** one-off actions nobody will revisit, hot inner loops (events take tens of microseconds in WebAssembly), or programs where no one will ever ask *"why?"*.

## Try it in 60 seconds

In an empty directory with Node 20 or later (no Rust installation needed):

```sh
npm init -y
npm install caveat-lang@1.5.0
```

Paste this "hello, world": one piece of evidence, one decision resting on it, and one event that withdraws the evidence and reopens the decision:

```sh
cat > hello.cav <<'CAV'
claim safe_to_ship;
evidence ci from "the CI run";
evidence rerun from "a rerun of CI";
readings tests from ci limit 2;
decisions ship limit 2;

event ci_passed;
event ci_was_wrong;

on ci_passed sample tests = 1 supports safe_to_ship;
on ci_passed commit ship because enough using latest(tests);
on ci_was_wrong reveal rerun opposes safe_to_ship;
on ci_was_wrong withdraw latest(tests) because rerun;
on ci_was_wrong reopen ship because rerun;
CAV
printf '%s\n' '{"event":"ci_passed"}' '{"event":"ci_was_wrong"}' > events.jsonl
npx --no-install caveat-lang explain hello.cav events.jsonl
```

Part of what it prints:

```text
    ship@1 = 1  reopened
      based on tests@1
      tests@1: observed at #1 ci_passed
      tests@1 has since been withdrawn at #2 because rerun
      #1 ci_passed: committed because tests@1
      #2 ci_was_wrong: reopened because rerun
```

The decision was reopened when the CI result it rested on was withdrawn, and it still records what it was originally based on.

To check the installation, watch a multi-revision agent lifecycle (`observe` → `assess` → `retract` → replace), or scaffold the starter project with scenarios:

```sh
npx --no-install caveat-lang doctor
npx --no-install caveat-lang demo agent
npx --no-install caveat-lang init
npx --no-install caveat-lang test umbrella.scenarios.json
npx --no-install caveat-lang explain umbrella.cav events.jsonl
```

`caveat-lang` is the unambiguous CLI command; `caveat` is supported shorthand ([CLI names](kit/docs/NAMES.md)).

## Start here

| If you want to… | Start here |
| --- | --- |
| **Understand the value in 2 minutes** | [What Caveat is for](https://wsattazahn.github.io/caveat-lang/about.html) · [Why Caveat](docs/WHY_CAVEAT.md) |
| **Give an agent an undo list in 15 minutes** | [An agent with an undo list (tutorial)](docs/tutorials/agent-undo-list/README.md) |
| **Walk through Caveat step by step** | [Getting started](kit/docs/GETTING_STARTED.md) · [Worked example](kit/docs/WORKED_EXAMPLE.md) |
| **Connect an agent over MCP (local or hosted)** | [MCP server](#mcp-server) · [MCP setup & limits](kit/docs/MCP.md) |
| **Write `.cav` programs cleanly** | [Authoring card](kit/docs/AUTHORING_CARD.md) · [Caveat on one page](kit/docs/REFERENCE.md) · [Authoring guide](docs/AI_AUTHORING.md) |
| **Embed Caveat in Node, browser, or Python** | [Agent quickstart](kit/docs/AGENT_START.md) · [Integration starter](kit/docs/STARTER.md) · [Python `caveat serve` example](kit/examples/agent-evidence/README.md) · [Python native package](python/README.md) |
| **Use Caveat in a game** | [Caveat in a game](kit/docs/GAMES.md) · [Intel betrayal example](kit/examples/intel-betrayal/README.md) |
| **Test withdrawal guarantees in your browser** | [Try to break this](https://wsattazahn.github.io/caveat-lang/break.html) |
| **Check what is proved, tested, and promised** | [What is checked, and how](docs/VERIFICATION.md) · [Compatibility commitment](docs/COMPATIBILITY.md) · [Limits](docs/LIMITS.md) |
| **Browse every language & kit specification** | [Documentation index](kit/docs/README.md) · [`spec/`](spec/) |
| **Report a missing capability** | [Agent feature requests](AGENTS.md#report-a-capability-missing-from-a-real-integration) |
| **See what each release changed** | [Release history](#release-history) |

### Try it with a coding agent

Give a coding agent this prompt in an empty directory:

```text
Install the caveat-lang npm package (Node 20 or later) in this empty directory
and run `npx --no-install caveat-lang init`. Read
node_modules/caveat-lang/docs/REFERENCE.md before writing any Caveat source.
In umbrella.cav, raise the rain chance needed to commit the umbrella decision
from 50 to 60. Add a scenario to umbrella.scenarios.json showing that a 55%
forecast leaves the advice at "No decision yet". Run
`npx --no-install caveat-lang check umbrella.cav` and
`npx --no-install caveat-lang test umbrella.scenarios.json`, then run
`npx --no-install caveat-lang explain umbrella.cav events.jsonl` and tell me
what umbrella@1 rests on and why it was reopened. Change no other files.
```

The three scenarios pass, and `explain` shows `umbrella@1` still resting on `rain_chance@1` and its `forecast_is_old` caveat after `sky` reopened it.

## Principles and capabilities

For the thinking behind Caveat, read [The Caveatist way](docs/WHY_CAVEAT.md#the-caveatist-way).

> Say what you know, and what it rests on.  
> Carry the caveats honestly.  
> Give uncertainty the attention its consequences deserve.  
> Act when there is enough to proceed.  
> Remember why you chose, and remain willing to choose again.

These are the project's principles, not license conditions: Caveat is MIT-licensed and using it requires adopting none of them.

- **Values carry their evidence.** Every number computed from an observation carries that evidence and its caveats through every sum, comparison, function call, and label.
- **Explanations are checked against recorded dependencies.** Anything a program shows can say *why* (`because`), and the runtime refuses a citation outside the value's recorded dependencies (`evaluation/ungrounded_citation`). That does not prove the evidence true or the explanation complete.
- **Decisions remember.** A decision freezes its grounds (and separately its grant, `permitted by`, and named `threshold`s), reopens under authored rules or triggers when evidence changes or is withdrawn, and keeps an ordered `decision_journal`.
- **Late knowledge qualifies without rewriting the past.** `qualify` and `unqualify` reach current values built on the evidence; earlier decision records keep what they knew at commit time.
- **Refusals leave no trace.** An authored `reject`, missing permission, out-of-range input, or resource-limit refusal rolls back the entire event atomically; hosts inspect structured [dispatch outcomes](spec/caveat-dispatch-0.1.md) before acting.
- **Complements platform governance.** Caveat records supplied evidence and authored policy; it does not sign saves, authenticate evidence, or authorize external actions ([threat model](docs/THREAT_MODEL.md), [restore trust boundary](docs/RESTORE_TRUST_BOUNDARY.md)).

## What Caveat makes part of the language

These mechanisms can also be implemented in a general-purpose language through application code or libraries. Caveat supplies them together as language and runtime facilities; the comparison is about what an author has to build and keep in sync, not what another language can express.

| | Caveat | In a general-purpose language |
| --- | --- | --- |
| **Explanations** | `bind label = "Could be a duskcap" when … because contradiction;` The runtime checks cited evidence and caveats against recorded dependencies. | Implement dependency tracking and citation checks, or use a library that supplies them. |
| **Caveats through computation** | `qualified(1, taste, tasted_in_dark)` carries evidence and caveats through computations that use it. | Use provenance-aware values and operations to propagate the same information. |
| **Late caveats** | `qualify taste with taste_faded;` qualifies current values built on the taste while earlier decision records retain their frozen caveats. | Track dependencies or resolve qualifications when read, and freeze decision records. |
| **Decisions that remember** | `commit`, `reopen`, `hold`/`release`, `resolve`, and decision-series revisions retain each revision's grounds and publish changes in `decision_journal`. | Implement decision records, frozen bases, and an ordered change journal. |
| **Grounds and lineage** | The runtime distinguishes a value's content grounds from its wider control lineage and enforces grounds ⊆ lineage. | Represent both kinds of dependency and enforce their relationship in code or a library. |
| **Atomic events** | `reject "already absorbed";` rolls back the event's changes to the Caveat session. | Use transactions, immutable state, or explicit rollback to publish changes atomically. |

Citation checking establishes that cited dependencies were recorded; it does not establish that supplied evidence or authored explanation prose is true, or that a citation is exhaustive.

## The evidence: the glowcap benchmark

The [glowcap benchmark](experiments/glowcap/RESULTS.md) implements the same game beat in Caveat and in TypeScript against:

- a frozen, pre-registered scenario suite;
- sixteen change requests across eight rounds (including blind rounds by fresh authors);
- seeded differential fuzz comparing accepted and rejected events, views, and save/resume behavior.

[The results file](experiments/glowcap/RESULTS.md) records every round's change cost, size, first-run passes, and verdict — including the rounds TypeScript won — and the weak spots that became Caveat's language work list. Fresh-agent authoring trials are also recorded in [`experiments/agent-authoring/`](experiments/agent-authoring/v1/RESULTS.md) ([v1](experiments/agent-authoring/v1/RESULTS.md), [v2](experiments/agent-authoring/v2/RESULTS.md), [v3](experiments/agent-authoring/v3/RESULTS.md), [v4](experiments/agent-authoring/v4/RESULTS.md)).

What Caveat costs is in [Limits](docs/LIMITS.md), and what is formally proved versus tested is in [What is checked, and how](docs/VERIFICATION.md) ([Lean model](proofs/lean/README.md)).

## A taste

```caveat
event absorb target kind mushroom, sort in glowcap duskcap;

for mushroom as $m {
    on absorb when $m_here and $m_consumed == 1 reject "already absorbed";
    on absorb when $m_here and sort == sort.glowcap
        set support = support + qualified(1, absorb_$m);
    on absorb when $m_here and sort == sort.duskcap and committed(trust)
        reopen trust because absorb_$m;
    on tick when $m_tasted_at >= 0 and now - $m_tasted_at >= 60
        qualify taste_$m with taste_faded;

    bind $m.label = "Probably a glowcap" when $m_unknown and probably_safe because support;
    bind $m.label = "Could be a duskcap — taste first" when $m_unknown and uncertain because contradiction;
};
```

Read [Why Caveat](docs/WHY_CAVEAT.md) for side-by-side TypeScript and Caveat code. To write your first program, start with the [authoring card](kit/docs/AUTHORING_CARD.md) and [authoring guide](docs/AI_AUTHORING.md).

## MCP server

The package includes a local MCP server, `caveat-lang mcp`, that gives an agent Caveat's seven authoring and ledger operations as tools. It runs over stdio, needs Node 20 or later and no Rust, and is listed in the MCP Registry as `io.github.WSattazahn/caveat-lang` ([server.json](server.json)). Add it to an MCP client's configuration:

```json
{
  "mcpServers": {
    "caveat-language": {
      "command": "npx",
      "args": ["-y", "caveat-lang@1.5.0", "mcp"]
    }
  }
}
```

A free, stateless Streamable HTTP copy of the same seven tools (running published `caveat-lang@1.5.0`) is also hosted at `https://caveat-mcp.waltersattazahn.workers.dev/mcp` ([source and limits](hosted-mcp/README.md)).

| Tool | Inputs | Result |
| --- | --- | --- |
| `caveat_validate` | source | Whether the program loads and what it declares |
| `caveat_check` | source, optional strict | Authoring diagnostics and suppressed warnings |
| `caveat_test` | source, scenarios | Scenario results, invariant checks, and check warnings |
| `caveat_explain` | source, optional events | Decisions, their exact grounds, caveats, and history |
| `caveat_dependents` | source, subject, optional events | What depends on the named evidence, stream, or caveat |
| `caveat_dispatch` (from 1.5.0) | source, optional save, optional moduleBindings, optional events | One step of a running ledger: the state after the events, and the compact save to pass back next time |
| `caveat_find` (from 1.5.0) | source, records, query | Past decisions matching every word, with their grounds and whether they still stand |

Every tool works on inline source text and supplied events. The server has no file, network, or persistent-session tools, and runs each call in a fresh subprocess with a 10-second limit. It speaks MCP `2026-07-28` and `2025-11-25`. [MCP setup and limits](kit/docs/MCP.md) covers the protocol, result format, limits, and host configs. A tool's report does not authorize an external action or authenticate its evidence.

## Interactive demos and games

Everything below runs in the browser on the WebAssembly runtime (demonstrations, not part of the npm package's compatibility promise):

- **[What Caveat is for](https://wsattazahn.github.io/caveat-lang/about.html)** — one-page overview, runnable example, and limitations.
- **[Try to break this](https://wsattazahn.github.io/caveat-lang/break.html)** — one precise claim about withdrawing evidence, checked step by step against five rules in your browser.
- **[Glowcap explainer](https://wsattazahn.github.io/caveat-lang/glowcap.html)** — start with **Change what you know**: make a trust decision in one click, learn something new in a second, and see what reopened and what stayed frozen (`game/glowcap.cav`).
- **[Trail Rescue](https://wsattazahn.github.io/caveat-lang/trail-rescue.html)** — weigh fallible scout reports, choose a tunnel, and reconsider when evidence ages (`game/trail_rescue.cav`, [protocol](experiments/trail-rescue/PROTOCOL.md), [results](experiments/trail-rescue/RESULTS.md)).
- **[Light the Way](https://wsattazahn.github.io/caveat-lang/rescue.html)** — direct-control ferry rescue powered by the reactive runtime (`game/light_the_way.cav`, [design notes](docs/LIGHT_THE_WAY.md)).
- **[The Last Beacon](https://wsattazahn.github.io/caveat-lang/last-beacon.html)** — 3D island mystery with nine evidence-dependent outcomes (`game/the_last_beacon.cav`, [design notes](docs/THE_LAST_BEACON.md)).
- **[Moon Garden](https://wsattazahn.github.io/caveat-lang/)** ([3D version](https://wsattazahn.github.io/caveat-lang/moon-garden-3d.html)) & **[Mr. Caveat](https://wsattazahn.github.io/caveat-lang/mr-caveat.html)** — earlier story demos and the mechanical fortune teller.

### Build and play

With `rustup` and Node.js 20 or newer installed, run from the repository root (`rust-toolchain.toml` pins the compiler and installs the WebAssembly target on first use):

```sh
cargo install wasm-bindgen-cli --version 0.2.104 --locked
npm ci
npm run build
npm run serve
```

The build also writes `dist/pkg-reactive/` (the reactive-only runtime built with `--no-default-features`, about a fifth smaller). Open `http://127.0.0.1:4173/about.html`, `glowcap.html`, `trail-rescue.html`, `rescue.html`, or `last-beacon.html`.

To play *The Last Beacon* in a terminal without a browser:

```sh
cargo run --manifest-path runtime/Cargo.toml --bin caveat -- --game game/the_last_beacon.cav
```

For standalone offline HTML copies (`dist/Light-the-Way.html`, `dist/The-Last-Beacon.html`), run `npm run package:game` after building.

### Verify

```sh
cargo test --manifest-path runtime/Cargo.toml
cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings
npm run test:kit
npm run test:ledger
npm run test:slime-glow
npx playwright install chromium
npm run test:beacon
npm run test:rescue
```

Run `npm run build` before the Node and Playwright suites (`npm run test:kit`, `npm run test:kit-package`, `npm run test:beacon`, `npm run test:rescue`, `npm run test:slime-glow`, `npm run test:trail-rescue`, `npm run test:launch`). See [developing the kit](docs/KIT_DEVELOPMENT.md) and [what is checked, and how](docs/VERIFICATION.md) for the full test matrix.

## Release history

All 1.x releases honor the [compatibility commitment](docs/COMPATIBILITY.md): 1.x minors only add, and earlier 1.x saves continue restoring. Full verification receipts and checksums live in [`docs/releases/`](docs/releases/).

- **1.5.0** ([release record](docs/releases/v1.5.0.md)): adds six language profiles (`sum`/`any`/`all` across members, `grounds_of`, `sample … with` reading caveats, named `threshold`s, `work skips other routes;`, and `state … unset` / `is_set` with refusal code `evaluation/unset_state`), `caveat find` and `caveat_find`, the `caveat_dispatch` MCP tool, incremental reused views (`dispatchView(..., { reuse: true })`) with compact binary wire patching, opt-in `check` (`--withdrawals`, `--names`), `explain` (`--later-caveats`, `--lineage-notes`) and `review --all --summary` flags, and `whatIf(..., { agent })`. Delivers the module-binding correction (`moduleBindings` restore option), removes restore's 16 MiB state-save cap, fixes runtime and restore validation findings (D027–D105), and speeds up event dispatch and view patching.
- **1.4.1** ([release record](docs/releases/v1.4.1.md)): fixes `withdrawn` propagation through `committed(D)`, `reopened(D)`, and `in_basis(D, E)` (D026), and refuses oversized computed text as `limit/text_limit` instead of ending the session (D025).
- **1.4.0** ([release record](docs/releases/v1.4.0.md)): adds `agent` sender declarations, `withdraw every`, `only_if`, `carries_any`, reversible foreclosure, `irreversible strict`, `caveat review`, ledger sets (`review --all`), `session.whatIf`, `caveat-lang/handoff`, and `caveat-lang/saves`.
- **1.3.0** ([release record](docs/releases/v1.3.0.md)): adds recorded outcomes (`resolve`, `resolved`, `outcome`, `materialized`) so programs can score forecasts against what happened.
- **1.2.0** ([release record](docs/releases/v1.2.0.md)): adds nested `for` blocks, holds (`hold … because`, `release`, `on_hold`), `irreversible forecloses`, positional replay saves, and selected view fields.
- **1.1.0** ([release record](docs/releases/v1.1.0.md)): adds `citing`, `retaining … because`, neutral and noted samples, `reading(STREAM, N)`, `in_basis`, `changed` triggers, compact replay saves, and late-qualification propagation through decision reads (D015).
- **1.0.3** ([release record](docs/releases/v1.0.3.md)), **1.0.2** ([release record](docs/releases/v1.0.2.md)), **1.0.1** ([release record](docs/releases/v1.0.1.md)): patch fixes for caveat take-back and windowed departure interaction (D012–D014, D016, D018, D020).
- **1.0.0** ([release record](docs/releases/v1.0.0.md)): first stable release establishing the 1.x compatibility promise; earlier 0.1.0 release candidates (`rc.1`–`rc.20`) are archived in [`docs/releases/`](docs/releases/).

## Repository layout

- `runtime/` — Rust reference runtime (native + WebAssembly) and `runtime/prelude.cav`
- `kit/` — the `caveat-lang` npm package (CLI, Node/browser library, MCP server, starter, docs, examples, tests)
- `spec/` — versioned language, runtime, save, view, CLI, and kit specifications
- `docs/` — guides (`AI_AUTHORING.md`, `WHY_CAVEAT.md`, `LIMITS.md`, `COMPATIBILITY.md`, `VERIFICATION.md`, tutorials, release records)
- `hosted-mcp/` — Cloudflare Worker hosting the published `caveat-lang` MCP server over Streamable HTTP
- `plugins/` & `skills/` — Anthropic (Claude Code/Cowork/apps) and OpenAI (ChatGPT/Codex) Caveatist plugin & skill packaging
- `python/` — native PyO3 Python bindings (`caveat_lang`) matching the npm kit byte-for-byte
- `actions/pr-ledger/` — GitHub Action tracking what merged PRs rested on in Caveat
- `editors/vscode/` — VS Code TextMate syntax-highlighting extension for `.cav` files
- `ledger/` — Caveat's own project decision ledger (`project.cav`, `events.jsonl`, scenarios)
- `proofs/lean/` — Lean 4 mathematical model and conformance theorems
- `examples/` & `tests/` — standalone `.cav` examples and canonical semantic tests
- `game/` & `web/` — playable browser demos, explainers, and `about.html` / `break.html`
- `scripts/` — build, packaging, release-gate, distribution-check, and verification tooling
- `caveatism/` — optional Caveatist philosophy, Archive, Atlas, canon program, and Mr. Caveat artwork

> Do not save CAVEAT by redefining it. If the computational model collapses into an existing paradigm, record the result.

## License

Caveat's code and documentation are available under the [MIT License](LICENSE). Third-party components keep their own licenses: the Rust crates compiled into the WebAssembly runtime are listed with their notices in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), and the site's copy of three.js ships with its license as `vendor/THREE-LICENSE.txt`.

Report vulnerabilities privately as the [security policy](SECURITY.md) describes; the [threat model](docs/THREAT_MODEL.md) sets out the trust boundaries. [CONTRIBUTING.md](CONTRIBUTING.md) says how to report bugs and what a pull request needs.

## Culture

Caveatism is the philosophy that grew up around Mr. Caveat, a mechanical fortune teller who always has a caveat; it lives in [`caveatism/`](caveatism/README.md) and is culture, not a contract of the language. The [Archive](caveatism/archive/The-Caveatist-Archive-v1.2.md) and [Atlas](caveatism/atlas/The-Caveatist-Atlas-v1.0.md) are its texts, and [the canon](caveatism/canon/README.md) keeps the Archive as a Caveat program with scenarios.

![Mr. Caveat holding a fortune ticket whose fine print runs off the card](caveatism/character/mr-caveat-hero.png)
