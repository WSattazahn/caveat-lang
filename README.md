# CAVEAT Language

[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/15891/badge)](https://www.bestpractices.dev/projects/15891)

**A decision ledger for agents: Caveat keeps an AI's reasons in a record you can check.**

Caveat is a rules language and a runtime for decisions that may need taking
back. It tracks evidence through computations, preserves the grounds of
recorded decisions, and reopens them under rules you write when evidence
changes. [Limits](docs/LIMITS.md) says up front what it does not do (check
that evidence is true, authenticate saves, know who sent an event) and what it
costs.

For teams that govern their agents, Caveat is also an audit trail for agent
decisions. `caveat explain` shows what each decision rested on: its evidence
and caveats, the permission it needed and why it was later reopened.
Withdrawing evidence adds to that record and never rewrites a decision's
original grounds. Your application stores the events; Caveat does not sign
them or authenticate the evidence, so it complements your platform's identity,
permission and logging controls rather than replacing them.

The published release is **1.4.1**, a patch release of 1.4.0. See the
[1.4.1 release record](docs/releases/v1.4.1.md) for what it fixes, the
[1.4.0 release record](docs/releases/v1.4.0.md) for what 1.4 adds and the
[1.0.0 release record](docs/releases/v1.0.0.md) for what 1.x promises.

> npm publication verified 2026-10-26T23:13:05.492Z: exact **caveat-lang@1.4.1**,
> tested Linux artifact SHA256 `ecb535b181f5f33836f4730ed8c1c0c8ac9a1622eb55e3d43e3e3c3492f281f5`.
> Published by `publish-npm.yml` from tag `v1.4.1` with an npm provenance attestation. Both `latest` and `next` were verified at 1.4.1 on 2026-10-26.
> [Publication record](docs/releases/v1.4.1-npm-publication.json).


When a tool result is corrected or a memory turns out to be stale, an application
needs to reconsider its decision without losing the reasons for the original.
Caveat is a programming language that keeps evidence and caveats with computed
values, freezes a decision's grounds, and records why it was reopened.

Try the published release, **1.4.1**, in an empty directory with Node 20 or
later. No Rust installation is needed. Install it:

```sh
npm init -y
npm install caveat-lang@1.4.1
```

Then paste this "hello, world": one piece of evidence, one decision resting on
it, and one event that withdraws the evidence and reopens the decision.

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

The decision was reopened when the CI result it rested on was withdrawn, and
it still says what it was first based on. For a fuller program with a caveat,
scenarios and a displayed value, `init` writes the getting-started starter:

```sh
npx --no-install caveat-lang init
npx --no-install caveat-lang test umbrella.scenarios.json
npx --no-install caveat-lang explain umbrella.cav events.jsonl
```

The two scenarios pass. The explanation shows `umbrella@1 = 70` reopened by
`sky`, still based on `rain_chance@1` with its `forecast_is_old` caveat. New
knowledge changed the decision's status; its original grounds remain visible.
[Getting started](kit/docs/GETTING_STARTED.md) walks through those files, and
[the language in brief](kit/docs/REFERENCE.md) covers the syntax.

## Start here

To see what Caveat does, give a coding agent this task in an empty directory:

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

The three scenarios pass, and the explanation still shows `umbrella@1` resting
on `rain_chance@1` and its `forecast_is_old` caveat after `sky` reopened it.

| To | Start with |
| --- | --- |
| Try Caveat by hand | [Getting started](kit/docs/GETTING_STARTED.md) |
| Give an agent an undo list in 15 minutes | [An agent with an undo list](docs/tutorials/agent-undo-list/README.md) |
| Give an agent Caveat's tools over MCP | [MCP server](#mcp-server) |
| Integrate Caveat into an agent or application | [Agent quickstart](kit/docs/AGENT_START.md) and the [Python client](kit/examples/agent-evidence/README.md) |
| Write Caveat source | [The language in brief](kit/docs/REFERENCE.md) and the [authoring guide](docs/AI_AUTHORING.md) |
| See what is proved and tested | [What is checked, and how](docs/VERIFICATION.md) |
| Report a capability Caveat lacks | [Agent feature requests](AGENTS.md#report-a-capability-missing-from-a-real-integration) |
| See what each release changed | [Release history](#release-history) |

The [1.0.0 verified publication record](docs/releases/v1.0.0.md) identifies
the exact tested npm artifact. 1.0.0 makes the
[compatibility commitment](docs/COMPATIBILITY.md) the promise for every 1.x
release: the language, the 19 refusal codes, the CLI, library exports, `serve`,
the MCP tools, named schemas and saves are stable, and 1.x minors only add. It
adds no language, schema, operation, field, CLI flag, export or MCP tool over
rc.20, and its Rust runtime is unchanged from rc.20.

Restoring a save still does not authenticate its history. A refused operation is
not completed; hosts must inspect its outcome. The package has no npm runtime
dependencies.

With 1.4.1 installed, use `npx --no-install caveat-lang doctor` to check it and
`npx --no-install caveat-lang demo agent` to see observation, assessment,
correction and revision in one run. Introduced in rc.7, the unambiguous
`caveat-lang` command remains preferred; `caveat` is supported shorthand.
See [CLI names](kit/docs/NAMES.md).

**Integrating an agent?** Follow the [agent quickstart](kit/docs/AGENT_START.md)
and the [official Python client](kit/examples/agent-evidence/README.md). Caveat
records supplied evidence and authored policy; it does not authenticate evidence
or authorize external actions. [Repository agent guidance](AGENTS.md) includes
how to report a capability missing from a real integration.

**What stands behind it?** [What is checked, and how](docs/VERIFICATION.md)
separates the Lean model theorems ([proofs/lean](proofs/lean/README.md)),
sampled native and WebAssembly conformance with mutation controls, runtime
tests, and the boundaries nothing proves. It also states the limits: windows
do not bound saves, nothing is authenticated, and hosts set their own resource
budgets.

## Principles and capabilities

For the thinking behind Caveat, read
[The Caveatist way](docs/WHY_CAVEAT.md#the-caveatist-way).

> Say what you know, and what it rests on.  
> Carry the caveats honestly.  
> Give uncertainty the attention its consequences deserve.  
> Act when there is enough to proceed.  
> Remember why you chose, and remain willing to choose again.

These are the project's principles. They are not additional license
conditions: Caveat is under the [MIT License](LICENSE), and using it does not
require adopting them.

- **Values carry their evidence.** Every number computed from an observation carries that evidence and its caveats through every sum, comparison and label.
- **Explanations are checked against recorded dependencies.** Anything a program shows can say *why*, and the runtime rejects an explanation that cites something outside the value's recorded dependencies. That does not prove the evidence true or the explanation complete.
- **Decisions remember.** A decision records what it was made on, reopens when the world disagrees, and keeps a journal.
- **Late knowledge is one line.** A late qualification reaches current values built on the evidence and future values that read it. Archived readings and decisions already made keep what they knew.

**[What Caveat is for](https://wsattazahn.github.io/caveat-lang/about.html)**, on one page: the problem, one example to run, and the limitations.

**[Try to break this](https://wsattazahn.github.io/caveat-lang/break.html).** One claim about withdrawal, as five rules a checker tests after every event of a program you write. If Caveat reopens too much or too little, the page fills in an issue for you.

**[Play the glowcap explainer](https://wsattazahn.github.io/caveat-lang/glowcap.html).** Four look-alike mushrooms, a belief, a trust decision that is made, doubted and remade, and a "why?" under everything on the page. The rules and explanations all live in [`game/glowcap.cav`](game/glowcap.cav); the page only renders them.

Start with **Change what you know** at the top of that page. Make a decision in one click, learn something new in a second, and see what changed, why the decision reopened, and the original reasons it kept.

**Trail Rescue** is a new, complete mechanic: spend three scout tokens, weigh
fallible reports, choose a tunnel and reconsider when its evidence changes or
ages. Its [requirements and 24 scenarios](experiments/trail-rescue/PROTOCOL.md)
were committed before implementation. The [Caveat program](game/trail_rescue.cav)
owns the rules; the [browser page](web/trail-rescue.html) displays its decisions
and frozen explanations. Build and run it with `npm run build`, `npm run serve`,
then open `http://127.0.0.1:4173/trail-rescue.html`.

The mechanic added [state caveat queries](spec/caveat-state-caveats-0.1.md):
`has_caveat(plan_basis, stale)` checks the actual grounds of a saved basis, and
`reopen route because caveated(plan_basis, stale)` cites the precise observations
that have gone stale. The journal now preserves elapsed time and the chosen
numeric value, and restore checks its consistency with commitments and evidence.
See the [implementation record](experiments/trail-rescue/RESULTS.md).

**Fresh-agent authoring:** six fresh contexts authored three specified policies
using a frozen documentation packet, without private-test feedback. Two first
submissions and all six final submissions passed the registered corpus: 20,512
final-source events and 4,788 restores. All four initial failures exposed an
unclear numeric-bound restriction, now documented. Two final programs still
have known clock-horizon contract violations outside the corpus. The
[study report](experiments/agent-authoring/v1/RESULTS.md) preserves those limits,
every revision and the independent verification.

Reactive source can now read the runtime clock directly with
[`elapsed()`](spec/caveat-elapsed-0.1.md): `bind hud.elapsed = elapsed();`.
It uses the same time as scheduled caveats and decision journals, survives
save/restore, and removes the need for a separate bounded state timer.

A [four-context follow-up](experiments/agent-authoring/v2/RESULTS.md) repeated
the two clock tasks with this read and the updated guide. Three first submissions
and all four final submissions passed: 13,704 event dispatches and 3,158 restores.
All four used `elapsed()` directly, without a separate state clock. The remaining
first-source failure was unsupported comment syntax, repaired by its author.
This is evidence on two repeated tasks, not a general reliability claim.

The reactive runtime also offers [structured dispatch outcomes](spec/caveat-dispatch-0.1.md).
`dispatch_outcome` distinguishes an authored policy rejection from invalid input,
state-bound failure and execution-budget exhaustion. Unclassified errors remain
fatal. Existing dispatch methods retain their behavior; new integrations can use
the explicit outcome contract when testing why an event was refused. A host that
redraws from the view uses `dispatch_view_outcome` (`session.dispatchView` in the
developer kit): the same outcome, with the view in place of the snapshot.

## What Caveat makes part of the language

These mechanisms can also be implemented and checked in a general-purpose
language, through application code or libraries. Caveat supplies them together
as language and runtime facilities; the comparison is about what an author
has to provide, not what another language can express.

| | Caveat | In a general-purpose language |
| --- | --- | --- |
| **Explanations** | `bind label = "Could be a duskcap" when … because contradiction;` The runtime checks cited evidence and caveats against recorded dependencies. | Implement dependency tracking and citation checks, or use a library that supplies them. |
| **Caveats through computation** | `qualified(1, taste, tasted_in_dark)` carries evidence and caveats through computations that use it. | Use provenance-aware values and operations to propagate the same information. |
| **Late caveats** | `qualify taste with taste_faded;` qualifies current values built on the taste while earlier decision records retain their frozen caveats. | Track dependencies or resolve qualifications when read, and freeze decision records. |
| **Decisions that remember** | `commit`, `reopen`, and decision-series revisions retain each revision's grounds and publish its changes in `decision_journal`. | Implement decision records, frozen bases and an ordered change journal. |
| **Grounds and lineage** | The runtime distinguishes a value's content grounds from its wider control lineage and enforces grounds ⊆ lineage. | Represent both kinds of dependency and enforce their relationship in code or a library. |
| **Atomic events** | `reject "already absorbed";` rolls back the event's changes to the Caveat session. | Use transactions, immutable state or explicit rollback to publish changes atomically. |

Citation checking establishes that cited dependencies were recorded; it does
not establish that supplied evidence or authored explanation prose is true.
Citations may omit dependencies, so the check does not establish completeness.

## The evidence: the glowcap benchmark

The [glowcap benchmark](experiments/glowcap/RESULTS.md) implements the same game beat in Caveat and in a general-purpose language, against:

- a frozen, pre-registered scenario suite;
- sixteen change requests, twelve committed before either implementation changed for those rounds;
- seeded differential fuzz comparing accepted and rejected events, views, and save/resume behavior.

[The results file](experiments/glowcap/RESULTS.md) has every round's change cost, size, first-run passes and verdict, including the rounds Caveat did not win, and the decision rules each round was judged by. Each round's weak spots became the language's work list.

The earlier losses led to [grounded explanations](spec/caveat-explanations-0.1.md), [grounds](spec/caveat-explanations-0.2.md), [reject](spec/caveat-reject-0.1.md), [define](spec/caveat-define-0.1.md), [typed parameters](spec/caveat-typed-parameters-0.1.md), [the view](spec/caveat-view-0.1.md), [late qualification](spec/caveat-late-qualification-0.1.md) and the [decision journal](spec/caveat-decision-journal-0.1.md).

Round 6 led to [incremental evaluation](spec/caveat-incremental-evaluation-0.1.md), a load-time [observation-order check](spec/caveat-observation-order-0.1.md), [procedures that take evidence](spec/caveat-procedure-symbols-0.1.md), [renewable evidence and timed caveats](spec/caveat-renewal-0.1.md), and [save/restore without event replay](spec/caveat-save-0.1.md). The replay passes all 38 scenarios and a separate twelve-life regrowth check beyond the old eight-life cap. Its evidence still has a declared limit of 1,024 lives per mushroom: the request for unlimited regrowth remains unmet. Its first save-format attempt exceeded the size limit; that failed run remains in the results.

What Caveat costs is in [Limits](docs/LIMITS.md), including the benchmark program's time per event and bundle size.

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

Read [Why Caveat](docs/WHY_CAVEAT.md) for side-by-side code and the Caveatist way of working. To write your first program, start from the [authoring guide](docs/AI_AUTHORING.md).

## MCP server

The package includes a local MCP server, `caveat-lang mcp`, that gives an agent
Caveat's authoring operations as tools. It runs over stdio, needs Node 20 or
later and no Rust, and is listed in the MCP Registry as
`io.github.WSattazahn/caveat-lang` ([server.json](server.json)). Add it to an
MCP client's configuration:

```json
{
  "mcpServers": {
    "caveat-language": {
      "command": "npx",
      "args": ["-y", "caveat-lang@1.4.1", "mcp"]
    }
  }
}
```

| Tool | Inputs | Result |
| --- | --- | --- |
| `caveat_validate` | source | Whether the program loads and what it declares |
| `caveat_check` | source, optional strict | Authoring diagnostics and suppressed warnings |
| `caveat_test` | source, scenarios | Scenario results, invariant checks and check warnings |
| `caveat_explain` | source, optional events | Decisions, their exact grounds, caveats and history |
| `caveat_dependents` | source, subject, optional events | What depends on the named evidence, stream or caveat |
| `caveat_dispatch` (from 1.5.0) | source, optional save, optional events | One step of a running ledger: the state after the events, and the save to pass back next time |
| `caveat_find` | source, records, query | Past decisions matching every word, with their grounds and whether they still stand (1.5 draft) |

Every tool works on inline source text and supplied events. The server has no
file, network or persistent-session tools, and it runs each call in a fresh
subprocess with a 10-second limit. It speaks MCP 2026-07-28 and the
2025-11-25 profile. [MCP setup and limits](kit/docs/MCP.md) covers the protocol,
result format, limits and a configuration for an installed copy. A tool's
report does not authorize an external action or authenticate its evidence.

## Release history

1.0.1 fixes three cases where a caveat travelling through a guard's stored
record was kept after it was taken back, or missed by a later `qualify`
(D012, D013 and D014). It adds no language, schema, operation, field, CLI flag,
export, MCP tool or refusal code, and restores every 1.0.0 save. The
[1.0.1 verified publication record](docs/releases/v1.0.1.md) identifies the
exact tested npm artifact and says which programs can see a difference.

1.0.2 fixes two cases where 1.0.1's fixes met a window or limit: departure
was not told of evidence a late `qualify` added to a stored record (D016), and
a caveat read through a stored record after its evidence departed could no
longer be taken back (D018). It adds nothing either, and restores every 1.0.0 and
1.0.1 save. The [1.0.2 verified publication record](docs/releases/v1.0.2.md)
says which programs can see a difference.

1.0.3 fixes a case where 1.0.1's D014 fix let a caveat that a plain `qualify`
put on a guard's stored record survive its take-back (D020). It adds nothing
either, and restores every 1.0.0, 1.0.1 and 1.0.2 save. The
[1.0.3 verified publication record](docs/releases/v1.0.3.md) says which
programs can see a difference.

1.1.0 adds language forms (`citing`, `retaining … because`, neutral and noted
samples, `reading(STREAM, N)`, `in_basis` and more), a compact save format,
opt-in `caveat check` warnings, a `serve` option and kit helpers, and one
refusal code. It changes one behavior to keep Late Qualification 0.1's promise:
a later `qualify` now reaches a value read through a decision (D015). Every
1.0.x save restores as it was recorded. The
[1.1.0 verified publication record](docs/releases/v1.1.0.md) lists each
addition and says how to find the programs the change reaches.

1.2.0 adds nested `for` blocks, holds (`hold … because`, `release`,
`on_hold`) and foreclosure (`irreversible forecloses …`), a positional replay
save and selected view fields, and two refusal codes, both for the new syntax.
A program that loads on 1.1.0 gives the same results on 1.2.0, and every 1.0.x
and 1.1.0 save restores as it was recorded. The
[1.2.0 verified publication record](docs/releases/v1.2.0.md) lists each
addition.

1.3.0 adds recorded outcomes: `resolve` records how a decision turned out,
and `resolved` and `outcome` read it back, so a program can score its own
forecasts. It adds two refusal codes, both for the new syntax. A program that
loads on 1.2.0 gives the same results on 1.3.0, and every earlier save restores
as it was recorded. The
[1.3.0 verified publication record](docs/releases/v1.3.0.md) lists each
addition.

1.4.0 adds agents that may send only their own events, withdrawing everything
an untrusted source said, `only_if`, `carries_any`, reversible foreclosure and
a strict irreversible mode, and in the kit `review`, ledger sets, what-if runs,
handoff and save helpers. It adds four refusal codes, two for new syntax and
two for inputs that ended the session before, and fixes saves that would not
restore (D021). A program that loads on 1.3.0 gives the same results on 1.4.0
apart from those fixes, and every earlier save restores as it was recorded. The
[1.4.0 verified publication record](docs/releases/v1.4.0.md) lists each
addition.

1.4.1 fixes two things. A value read through `committed(D)`, `reopened(D)` or
`in_basis(D, E)` now carries `withdrawn` when the decision's basis lists
withdrawn evidence, as Withdrawal 0.1 always said (D026), and a computed text
past its 65,536-byte limit is refused as `limit/text_limit` instead of ending
the session (D025). Saves restore in both directions between 1.4.0 and 1.4.1.
The [1.4.1 verified publication record](docs/releases/v1.4.1.md) has the
details.

rc.20 clarified when to use each MCP tool, its
inputs and its results. It also fixed human-readable explanations of caveats
carried by a decision's grounds, retained in its frozen basis, or present only
in its wider lineage. It changed no Rust runtime, language or save format.

rc.19 began the 1.0 feature freeze: it added no language, schema, operation,
field, CLI flag or export. It fixed restore of a
session that reopened a decision because of a caveat and then took that caveat
back; saves written by rc.18 in that state restore in rc.19.

rc.18 added `unqualify`, which takes back a caveat
that `qualify` added, on every held occurrence of the evidence. The take-back is
recorded, not erased, and decisions keep their basis. A save of a program that
uses `unqualify` does not restore in rc.17 or earlier; other saves do. View 0.2
now lists a binding's evidence in the order it was first observed, and
`explain` names the event and payload that observed each ground.

rc.17 introduced View 0.2, source digests in saves and the integration starter
(`caveat-lang/starter`); the digest identifies the source and does not
authenticate a save. rc.16 introduced departure of eligible retired history, host-drained archives
and collection of unreachable withdrawal groups; see the
[history integration guide](kit/docs/HISTORY.md). rc.15 introduced integer clocks, history windows and `caveat test` checking each
program first. rc.14 corrected the package's MCP server name to
`io.github.WSattazahn/caveat-lang`. rc.13 introduced interface JSON and TypeScript
declarations, stricter restore checks and refused invalid `id_text` operations.

## Status

CAVEAT 0.3 is executable, and the **0.4 game profile** adds executable knowledge requirements, evidence-dependent outcomes, and source-authored spatial presentation. The Rust reference runtime parses source, evaluates the epistemic graph, exposes the CAVEAT Map, and runs the same game program in a terminal or a WebAssembly browser session. The new features are specified in [Draft 0.4](spec/caveat-0.4-draft.md).

Other playable examples include **Light the Way**, a direct-control ferry rescue powered by the new reactive CAVEAT profile. Movement, scouting, collisions, damage, score, and outcomes are source rules. **The Last Beacon** remains a separate 3D island mystery whose available decisions, evidence, retained uncertainty, nine outcomes, locations, cameras, and paths are authored in `game/the_last_beacon.cav`. **Moon Garden** remains a short mobile-first mystery with both a 2D presentation and a separate **Moon Garden 3D** presentation driven by the same CAVEAT session. The earlier **The Door** scenario remains the world/action stress test.

**CAVEAT 3D 0.2** now consumes normalized Rust action-runtime executions (`Move`, `Inspect`, `Operate`, `Open`, `Observe`, `Stay`) and maps those commands to semantic place/entity/symbol presentation bindings. Moon Garden no longer needs normal per-action camera choreography. See `spec/caveat3d-0.2.md`. The remaining graphics problem is art-direction automation: semantic identifiers can now drive the scene, but attractive camera composition and assets still require authored presentation bindings.

## Light the Way — play immediately

[Play Light the Way](https://last-beacon-caveat.w4ltr0n.chatgpt.site). Hold and drag the lighthouse beam, or use the arrow keys. Steer the ferry around reefs into the green harbor. Hold the light over the current marker to take a reading while the ferry keeps moving. When the storm shifts, decide whether to correct the old plan manually or spend another second scouting. Damage, sampling progress, and rescued passengers appear directly in the scene. Retry immediately. No download or reading panels are required.

The new [reactive language profile](spec/caveat-reactive-0.1.md) adds bounded numeric state, typed events, arithmetic, ordered conditional rules, and atomic event execution. The same rules can read and change CAVEAT's actual evidence graph. In [the game source](game/light_the_way.cav), spotting a reef records evidence, pays to examine the associated caveat, reopens the initial course commitment, and makes the ferry slow near the known danger. The original chart claim and contrary evidence both remain in the graph.

```caveat
state boat_x = ferry.x min -9.5 max 13;
event tick dt min 0 max 0.1;
on tick when phase == 1 set boat_z = boat_z - boat_speed * dt;
```

The [reactive 0.2 extension](spec/caveat-reactive-0.2.md) adds reusable numeric functions, evaluated presentation bindings, explicit sound/visual cues, source-declared controls, and a source-declared clock. The rescue's screen transitions, feedback, visual state, and keyboard steering now live in the Caveat program. Cues publish only when their event commits; a failed calculation rolls back its graph changes and feedback together.

[Reactive 0.3](spec/caveat-reactive-0.3.md) carries evidence and caveats through numeric values, arithmetic, function calls, comparisons, and conditional decisions. A commitment made `using` a derived value automatically retains its caveats and records a frozen numeric basis plus `relies_on` evidence edges. The crosscurrent's source-authored observation and steering functions exercise these semantics; the browser continues consuming ordinary rendering values.

The crosscurrent demonstrates why that matters beyond moving code: a qualified forecast supports an initial navigation commitment; an opposing observation reopens it and leads to counter-steering while retaining the unresolved caveat. Observation changes the navigator's response, not the physical current. Both evidence histories remain inspectable. [Preserving Caveat's essence](docs/CAVEAT_ESSENCE.md) records the invariants and the remaining Rust/browser boundaries.

The current shifts twice during the crossing. A previously measured steering correction keeps its old value until a fresh sample supports a revised plan. Bright arrows show the surface flow; faint arrows preserve the last reading, and the HUD shows its age. Rescouting has an immediate cost: the same light must stay over the marker while the moving ferry still needs steering.

[Reactive 0.4](spec/caveat-reactive-0.4.md) adds bounded text expressions and conditional expressions. The standard functions `abs`, `min`, `max`, and `clamp` are implemented in [Caveat source](runtime/prelude.cav), replacing their Rust algorithms. Source functions also format the game's clock, numbers, and percentages; source state owns notice lifetimes and hull indicators. Rust still implements the evaluator and browser bridge. This is a first source-defined standard library, not a self-hosted compiler.

[Reactive 0.5](spec/caveat-reactive-0.5.md) adds bounded reading streams and decision series. Each `sample` creates a distinct evidence occurrence, even when the measured number repeats. `latest(flow)` reads its qualified value; a decision series records a new frozen basis after its previous decision is explicitly reopened. Old occurrences remain addressable in snapshots and graph relations. The game uses one sampling procedure and one navigation series across repeated weather changes, replacing its separate first-reading and storm-reading implementations.

The same capability is exercised by a [thermostat program](examples/thermostat_history.cav): cold, warm, and cold readings revise the heating command while preserving every reading and the unresolved calibration caveat. Run `cargo test --manifest-path runtime/Cargo.toml --test thermostat_history` to exercise its input history and rollback behavior. This is a separate source consumer of the generic language feature, with no thermostat logic in the interpreter.

[Reactive 0.6](spec/caveat-reactive-0.6.md) makes retained history available to source computation through `history_count`, zero-based `history_at`, and `fold_history`. Reducers are ordinary pure Caveat functions: summing, calculating a range, or choosing a history-based policy requires no corresponding Rust algorithm. Results retain the observations, caveats, and selection dependencies involved. Invalid indexes, reducer errors, and bounded-work failures roll back the whole event. The thermostat calculates mean/range/trend in source, while the game compares archived readings to report a measured flow reversal. This is bounded history computation, not yet arbitrary collections or self-hosting.

[Reactive 0.7](spec/caveat-reactive-0.7.md) adds reusable effect procedures. A `proc` can share input handling or a sequence of observation and revision steps across source events. Calls freeze their numeric arguments and entry guard, retain every argument's qualifications, and run within the calling event's atomic transaction. The game shares keyboard cleanup, held-input activation, and aim movement; the thermostat records and revises through a source procedure. Rust provides bounded calls and validation, while the behavior remains Caveat source.

[Explanations 0.1](spec/caveat-explanations-0.1.md) lets a binding say what it cites: `bind label.text = "…" when … because contradiction;`. The runtime checks the citation against the binding's lineage, so an explanation may leave dependencies out but can never cite evidence or a caveat the value and its conditions did not read. The full lineage stays available for audit.

[Explanations 0.2](spec/caveat-explanations-0.2.md) separates what a value is *based on* (its grounds) from everything that could have influenced it (its lineage). A rule’s guard, a skipped rule, the guard that revealed evidence, and a decision’s predecessor all stay in lineage but never enter grounds. Citations read grounds, `set x = e because c` narrows them, and grounds are always a subset of lineage.

[Reject 0.1](spec/caveat-reject-0.1.md) adds `reject "MESSAGE"`: an event that is not allowed fails atomically with that message, without a dummy state or a `require` trick.

[Define 0.1](spec/caveat-define-0.1.md) adds `define NAME = EXPRESSION;`, a named expression over state and the graph that is inlined wherever it is read, including per member inside a `for` block.

[View 0.1](spec/caveat-view-0.1.md) adds `dispatch_view`: the same transaction as `dispatch`, returning only what a host redraws after an event (bindings and their citations, cues, effects, commitments and their grounds, relations) as compact JSON.

[Typed Parameters 0.1](spec/caveat-typed-parameters-0.1.md) lets an event take an entity by name (`target kind mushroom`) or one of a list of names (`sort in glowcap duskcap`). The host sends names; the source reads positions, with `sort.duskcap` and `target.pool` as constants.

[Late Qualification 0.1](spec/caveat-late-qualification-0.1.md) adds `qualify EVIDENCE with CAVEAT`: a caveat learned after the fact reaches every current value built on that evidence, while decisions already made keep what they were made on.

[Decision Journal 0.1](spec/caveat-decision-journal-0.1.md) publishes every commitment and reopening, in order, with the evidence each was based on in the order it was observed.

[Observation Order 0.1](spec/caveat-observation-order-0.1.md) rejects, when a program loads, a rule that qualifies a value with evidence only a later rule of the same event can reveal: a use that can only fail. Uses that could succeed on some dispatch are left to the runtime.

[Procedure Symbols 0.1](spec/caveat-procedure-symbols-0.1.md) lets a procedure take evidence, a claim, a caveat, a reading stream or a decision series by name (`proc learn(e evidence, sort)`), so every way of observing something can share one set of rules. Each call is specialized for the names it passes when the program loads.

[Identifiers 0.1](spec/caveat-identifiers-0.1.md) lets an event carry text the program first learns while it runs, such as a commit SHA (`event pushed commit id;`). The program sees a numeric handle for each distinct text, and `id_text` shows the text again.

[Withdrawal 0.1](spec/caveat-withdrawal-0.1.md) records that an observation is no longer stood behind (`withdraw latest(checks) because recheck;`). Nothing is erased: decisions keep what they were made on, current values and later reads carry a `withdrawn` caveat, and `rests_on_withdrawn(D)` lets a program decide what the withdrawal means for a decision.

[Permission 0.1](spec/caveat-permission-0.1.md) records what permitted a decision, apart from what it rests on (`commit merge … permitted by latest(approvals) for head;`). A missing, withdrawn or mismatched grant refuses the commit as `policy/not_permitted`, and the frozen record says which grant permitted it, for which value.

[Reopening Triggers 0.1](spec/caveat-reopening-triggers-0.1.md) lets a decision series declare which readings reopen it (`decisions merge limit 8 reopened by pushes, checks opposing ready;`), instead of a hand-written reopening rule after every such reading. The reopening is exactly the authored one, run at the moment the reading is taken.

[Check 0.1](spec/caveat-check-0.1.md) is `caveat check`: advisory warnings about patterns worth a second look, such as rules repeated across events that one procedure could share, or a decision made on readings that nothing reopens. A warning is not a proven fault; `# caveat check: allow CODE` records a pattern that is intended. The kit's [one-page reference](kit/docs/REFERENCE.md) gives the language in brief, and its [worked example](kit/docs/WORKED_EXAMPLE.md) takes one program through every command, with output the tests check.

[Renewal 0.1](spec/caveat-renewal-0.1.md) gives evidence an identity that events create: `renewable taste_cave limit 256;` and `renew taste_cave` make the name mean a new, unobserved occurrence while earlier ones keep what they were about. `qualify taste_cave with taste_faded after 60` fades that occurrence on its own clock, and `carries(taste_cave, taste_faded)` asks whether it has.

[Save 0.1](spec/caveat-save-0.1.md) saves a session and restores it without replaying events: `WebReactiveSession.save()` and `WebReactiveSession.restore(source, saved)`. Restoring costs what loading costs plus the size of the save. Restore validates the saved names, values and histories; mutation tests check that an altered save is either refused or remains playable without a crash.

[Incremental Evaluation 0.1](spec/caveat-incremental-evaluation-0.1.md) makes an event cost what it touches: a binding is evaluated again only when something it reads changed, and a transaction copies only what its effects write. Round 6's 32-entity program went from 1.1 ms to 0.16 ms per event with identical results, checked against full evaluation after every event in the test suite.

Water time and accumulated rain travel now come from Caveat state too. The source-defined `wrap` function keeps travel bounded; the renderer maps those values onto its existing wave shader and seeded rain geometry. Pausing or replaying a session preserves the corresponding weather pose.

The browser sends input and elapsed time to generic `WebReactiveSession`, then draws its snapshot. It does not calculate the ferry's movement, collisions, damage, route rules, or rescue result. Rust implements the language interpreter; the game-specific rules are Caveat. [Design notes](docs/LIGHT_THE_WAY.md) explain the controls and the source/runtime/renderer boundary.

Keyboard policy also lives in Caveat: source-declared physical-key controls
retain independent presses and releases, combine aliases and opposing keys,
and steer during source ticks. The browser forwards those input facts. A
source-only remapping changes the playable controls; observing, examining,
and reopening still follow the same qualified event rules.

Pure Caveat functions are callable through the [source library API](spec/source-library-0.1.md) from Rust and WebAssembly. The Door's movement timing and turning policy now live in its Caveat source, and The Last Beacon's feedback classification and copy use source functions. The same compiled evaluator serves both hosts. Full program validation and host transaction boundaries remain explicit.

For headless authoring, `caveat-reactive validate` checks a reactive program and `caveat-reactive replay` executes JSONL event histories with qualified snapshots. The [authoring guide](docs/AI_AUTHORING.md) includes commands and a thermostat input fixture. [Development insights](docs/DEVELOPMENT_INSIGHTS.md) separates demonstrated behavior from hypotheses about broader usefulness and AI adoption.

The [Slime glow ability policy](docs/SLIME_GLOW_ABILITY.md) is a small host-integration
consumer: a reported mushroom absorption creates one qualified learning receipt,
and ordinary ability toggles retain that basis without growing a per-input
history. Glow controls require a separate host-verified renderer capability;
without it, the source reports the mushroom discovery without claiming light.
Native and real WebAssembly tests exercise 10,001 ready toggles, early
discovery, duplicate absorption, reset and source-only policy variation. The host's
world contact, inventory transaction and visible glow remain host responsibilities.

## The Last Beacon — story experiment

On stormbound Saint Orin, thirty-two ferry passengers are approaching a failing lighthouse. Spend three watches investigating uncertain evidence, try provisional plans, reopen them when the world disagrees, and choose between restoring the light, sending the island pilot, or holding the ferry offshore until daylight. Six interactions produce 324 legal decision sequences and nine outcomes across three final destinations. Earlier investigations unlock targeted preparations; performing those preparations changes what the same final order accomplishes. See the [game design and action contracts](docs/THE_LAST_BEACON.md).

The game also develops the language runtime. The generic Rust `GameSession` and WebAssembly `WebGameSession` apply world actions and epistemic effects atomically, expose the graph reached by the player's actual decisions, and restore saves by replaying source-checked selections. The parser now supports quoted semicolons, escaped and Unicode text, line comments, and source locations in errors. These capabilities are available to any CAVEAT program; see the [game-session specification](spec/game-session-0.1.md) and [source-text specification](spec/caveat-text-0.1.md).

These are actual game-source statements:

```caveat
require bridge_reserve observed reserve_charge;
resolve relight_beacon as beacon_guided when observed measured_pulses;
resolve relight_beacon as beacon_limited when observed hot_cable;
resolve relight_beacon as beacon_limited when observed split_beam;
resolve relight_beacon as beacon_unverified otherwise;
```

The runtime checks the reached evidence graph before an action. Declared but undiscovered evidence cannot unlock it; contradictory evidence remains recorded. The browser receives available choices, blocked reasons, and the selected outcome from CAVEAT. It supplies reusable graphics and controls. [Presentation declarations](spec/presentation.md) also place the world and compose its views without changing JavaScript.

[Play the earlier story](https://last-beacon-caveat.w4ltr0n.chatgpt.site/last-beacon.html). This hosted copy opens directly in your browser; downloading an HTML file is optional.

### Build and play

With rustup and Node.js 20 or newer installed, run from the repository root. `rust-toolchain.toml` pins the compiler, and rustup installs it with the WebAssembly target on first use:

```sh
cargo install wasm-bindgen-cli --version 0.2.104 --locked
npm ci
npm run build
npm run serve
```

The build also writes `dist/pkg-reactive/`: the same runtime without the sequential, graphics and 3D sessions, for hosts that only run reactive programs (`cargo build --no-default-features`). It is about a fifth smaller.

Open [Light the Way locally](http://127.0.0.1:4173/rescue.html). The earlier [story experiment](http://127.0.0.1:4173/last-beacon.html) remains available. The build compiles the Rust runtime to WebAssembly and assembles `dist/` with game sources, browser assets, and a local copy of Three.js. Existing games remain available in the same build.

To play the earlier Last Beacon story in a terminal, without a browser or JavaScript:

```sh
cargo run --manifest-path runtime/Cargo.toml --bin caveat -- --game game/the_last_beacon.cav
```

For an optional offline copy, run `npm run package:game` after building. Open `dist/Light-the-Way.html` in a full modern browser (`dist/The-Last-Beacon.html` contains the earlier story): it embeds the actual CAVEAT WebAssembly runtime, story, and graphics and works without a server or network connection. Some file-preview applications disable scripts or module imports; those previews cannot run the game. The launch screen now includes browser guidance, a retry action, and a timeout for failed or stalled imports. In the earlier story game, device-local saves preserve your watch. Choose with the buttons or keys **1–3**, open the journal with **J**, and toggle ambient sound with **M**.

The [workflow notes](docs/WORKFLOW.md) explain how the video's visual inspection and testing loop informed this implementation.

### Verify

```sh
cargo test --manifest-path runtime/Cargo.toml
cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings
npx playwright install chromium
npm run test:beacon
npm run test:rescue
npm run test:slime-glow
```

Reactive runtime tests check atomic rollback and malformed input, and rescue simulations verify a complete crossing, damage, timing, and the direct effect of observed evidence on motion. Browser rescue tests use real mouse, keyboard, and touch input. The story tests exhaust legal Beacon routes, checking unavailable-action rejection, investigation costs, retained caveats, reopening, physical destinations, evidence-dependent outcomes, and save restoration. Browser tests exercise the WebAssembly game and its presentation, starting their own local server. Run `npm run build` first; a separate `npm run serve` process is not required for tests. After `npm run package:game`, `npm run test:launch` also exercises blocked imports, a stalled module, and disabled-script previews. `npm run test:slime-glow` needs no browser: it drives the installed WebAssembly runtime directly and checks the host-integration policy's bounded toggles, verified renderer capability, qualified escape, atomic failure, and malformed input. It runs in the same continuous integration job that assembles `dist/`.

## Repository layout

- `spec/` — versioned language specifications
- `runtime/` — reference runtime
- `runtime/prelude.cav` — source-defined standard math and display functions
- `tests/` — canonical semantic tests
- `examples/` — example CAVEAT programs
- `game/` — playable CAVEAT scenarios, including The Last Beacon, Moon Garden, and The Door
- `web/` — browser presentations and generic world rendering
- `scripts/` — reproducible browser build, local server, and game verification

> Do not save CAVEAT by redefining it. If the computational model collapses into an existing paradigm, record the result.

## License

Caveat's code and documentation are available under the [MIT License](LICENSE).
Third-party components keep their own licenses: the Rust crates compiled into
the WebAssembly runtime are listed with their notices in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), and the site's copy of
three.js ships with its license as `vendor/THREE-LICENSE.txt`.

Report vulnerabilities privately as the [security policy](SECURITY.md)
describes; the [threat model](docs/THREAT_MODEL.md) sets out the trust
boundaries. [CONTRIBUTING.md](CONTRIBUTING.md) says how to report bugs and
what a pull request needs.

## Culture

Caveatism is the philosophy that grew up around Mr. Caveat, a mechanical fortune teller who always has a caveat; it lives in [`caveatism/`](caveatism/README.md) and is culture, not a contract of the language. The [Archive](caveatism/archive/The-Caveatist-Archive-v1.2.md) and [Atlas](caveatism/atlas/The-Caveatist-Atlas-v1.0.md) are its texts, and [the canon](caveatism/canon/README.md) keeps the Archive as a Caveat program with scenarios.

![Mr. Caveat holding a fortune ticket whose fine print runs off the card](caveatism/character/mr-caveat-hero.png)
