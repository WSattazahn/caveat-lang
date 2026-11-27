# CAVEAT Agents 0.1 — who sent each event

**Status:** stable from 1.4.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.4. It adds to
[Dispatch 0.1](caveat-dispatch-0.1.md),
[Interface 0.1](caveat-interface-0.1.md),
[Save 0.1](caveat-save-0.1.md),
[Serve 0.1](caveat-serve-0.1.md) and
[Scenarios 0.1](caveat-scenarios-0.1.md); it changes no program that loads
today.

A main agent hands work to a sub-agent, and both send events to one program.
The [handoff prototype](../kit/examples/handoff/README.md) found the runtime
could not tell them apart: nothing stopped the sub-agent from sending the main
agent's `main_return` and releasing its own hold. The kit's `scopeSession`
keeps that boundary in the host, as an allowlist of events. The ledger then
says what happened and why, but not who made it happen: `explain` shows
`#4 sub_report: committed`, and whether a sub-agent or the main agent sent
`sub_report` is a naming convention.

This profile lets the program say which agent may send which event, and the
runtime check it and record the answer:

```caveat
agent main sends delegate, main_return;
agent sub sends sub_report;
agent reviewer;
```

The host says who sent each event:

```js
session.dispatch('sub_report', { n: 3 }, { agent: 'sub' });
```

`main_return` from `sub` is refused as `input/wrong_agent`, and an accepted
event's records carry its agent, so `explain` prints
`#4 sub_report by sub: committed because …`.

## 1. Syntax

```text
agent NAME;
agent NAME sends EVENT, EVENT, …;
```

- `NAME` is an identifier. Agents are their own namespace: an agent may share
  a name with an event, a state or evidence, and no expression reads one.
- Each `EVENT` must be a declared event, listed once per agent. Several agents
  may list the same event.
- A name is declared once. `agent` with no `sends` declares an agent that may
  send only the events no agent lists.
- A module may not declare an agent: who sends an event is the program's to
  say, as its attention budget is.

Before this profile a line beginning `agent` did not load, so no program that
loads today changes meaning.

## 2. Meaning

The host may name an agent with each event. The runtime refuses the event as
`input/wrong_agent` when:

- the agent is not one the program declares, including any agent at all in a
  program that declares none; or
- some agent's `sends` lists the event, and the host named no agent or one that
  does not list it.

An event no agent lists may be sent by any declared agent or with no agent.
That lets a program restrict only the events that matter, such as a release of
a hold or an irreversible commit.

The check comes after the event is found and before its parameters are
checked: an undeclared event stays `input/unknown_event`, a payload that is not
valid JSON or of the wrong type stays `input/payload_invalid`, and an event
from the wrong agent is `wrong_agent` whatever its parameters. A refusal
changes nothing, as every refusal does ([Dispatch 0.1](caveat-dispatch-0.1.md),
"Atomicity").

An event sent with no agent behaves exactly as before this profile.

## 3. What the runtime records

When the host names an agent and the event is accepted, the agent is recorded
beside the event on everything that event wrote:

| Record | Field |
| --- | --- |
| Decision journal entries (`committed`, `reopened`, `held`, `deferred`, `released`, `resolved`) | `agent` |
| Reading occurrences | `agent` |
| Withdrawals | `agent` |
| Taken-back qualifications | `agent` |
| The snapshot and save, for the last event | `last_agent` |

Each field is left out when the event named no agent, so a session no agent
sent to writes the same snapshot, view and save bytes as before. Evidence
revealed outside a reading stream is attributed through the event that
revealed it, which `explain` already reports (issue #199), not by a field of
its own.

The record is what the host said. Like an [origin](caveat-authorship-0.1.md#4-honest-boundary),
it is not an authentication: Caveat does not know that the process that said
`sub` is the sub-agent. A host gives each agent its own handle, for example
`scopeSession(session, events, { agent: 'sub' })`, so that an agent cannot
choose its own name.

## 4. Interface

The interface lists `agents`, in name order, each with the events its
declaration `sends` in declared order. The field is left out when the program
declares no agent, so the interface of every program that loads today is
unchanged.

```json
"agents": [
  {"name": "main", "sends": ["delegate", "main_return"]},
  {"name": "reviewer", "sends": []},
  {"name": "sub", "sends": ["sub_report"]}
]
```

## 5. Save and restore

A state save holds each record's `agent` and the session's `last_agent`.
Restoring refuses a save when:

- a record or `last_agent` names an agent the program does not declare;
- a record of an event some agent `sends` names no agent, or one that does not
  send it;
- two records of one event, or a record of the last event and `last_agent`,
  name different agents; or
- `last_agent` is present without a last event.

These are the saves no host could make by sending events, so this follows the
rule that restore refuses only what no source mechanism can produce. It does
not check that the agent really sent the event.

A replay save ([Replay Save 0.1](caveat-replay-save-0.1.md),
[0.2](caveat-replay-save-0.2.md)) lists events without agents. A session that
has accepted an event with an agent keeps no replay log from then on, so its
compact save is the state save. A session no agent sent to saves as before.

## 6. Hosts

| Surface | How the agent is named |
| --- | --- |
| Kit session | `dispatch(event, payload, { agent })`; `dispatchView`, `dispatchViewDelta` and `whatIf` take the same option. A runtime build without agents throws `CaveatError("load")` for an event sent with one, and the session is untouched. |
| Handoff helper | `scopeSession(session, events, { agent })` sends every event as that agent and lists it on each `accepted` entry, and its `whatIf` asks as that agent; `replayOnto` sends an entry that names an agent as that agent. |
| Scenarios | a `send` step's optional `"agent"`. |
| `serve` | a `dispatch` request's optional `"agent"`. |
| `explain`, `dependents`, `review` | an events file line's optional `"agent"`. |

The runtime's WebAssembly session has `dispatch_outcome_as`,
`dispatch_view_outcome_as`, `dispatch_view_fields_outcome_as`,
`dispatch_view_delta_outcome_as`, `what_if_view_outcome_as` and
`what_if_view_fields_outcome_as`, each taking the agent last. The what-if
forms answer what the dispatch form would return for that agent, and change
nothing ([Dispatch 0.1](caveat-dispatch-0.1.md#the-what-if-path)); before
them (1.4.0) the what-if path took no agent, so a host could not ask about
an event some agent sends (D027). The MCP tools
are unchanged in this draft: `caveat_test` runs scenarios, so its sends take
`agent` with them, but `caveat_explain` and `caveat_dependents` take events
without one.

## 7. Explain, dependents and review

Each journal change, reading, withdrawal and event that names an agent shows
it after the event, as `#4 sub_report by sub`. The JSON reports gain the same
optional `agent` field beside each `event`: a history entry, an evidence item,
a withdrawal note, an observing event, a dependents change, and the review's
`decided_by` and dropped events. A report of a session no agent sent to is
unchanged.

## What it does not do

- **Authenticate.** The agent is the host's word (section 3).
- **Let rules read the sender.** No expression asks who sent the event; a
  program that must act differently for two agents gives them two events.
  A `sent_by(AGENT)` condition can come in a later profile if a real program
  needs one.
- **Merge parallel sub-agents.** Two agents writing from one save still fork
  it; `replayOnto` merges by replay, and per-agent streams keep the order from
  mattering ([handoff example](../kit/examples/handoff/README.md)).
- **Carry agents in a replay save.** Section 5.
- **Scope agents to modules.** Section 1.

## Corpus sweep

[`experiments/agents-sweep/sweep.mjs`](../experiments/agents-sweep/README.md)
reads every tracked `.cav` file and lists each statement that begins with the
word `agent`, and each use of the word `agent` or `sends` outside quoted text
and comments. Before this profile such a statement did not load. Given a
published package, it also loads each tracked program in both runtimes and
compares the interface and the initial snapshot byte for byte.

## Implementation

- Runtime: `Directive::Agent` in `runtime/src/reactive.rs`, the check in
  `check_agent`, the `_as` dispatch entry points, the records' `agent` fields;
  the restore check `check_saved_agents` in `runtime/src/reactive_save.rs`;
  `RejectionCode::WrongAgent`; the interface's `agents`.
- Tests: `runtime/tests/agents.rs`, and the kit's `kit/test/agents.test.mjs`;
  the what-if forms in `runtime/tests/what_if.rs` and `kit/test/what-if.test.mjs`.

## Changes

- 2026-10-31: a three-word relation about evidence named `agent` (`agent supports c;`, `opposes`, `qualifies`) is a relation again. From 1.4.0 the `agent` statement took every line that starts with the word and refused it, although the program loaded on 1.3.0 (D035). Neither statement has a three-word form with a relation word in second place.
- 2026-10-26: the what-if path takes an agent (`what_if_view_outcome_as`,
  `what_if_view_fields_outcome_as`, `whatIf(event, payload, { agent })`, and
  `whatIf` on a scoped session). Found by the Conway test of the 1.4 features
  (D027): with every event of interest listed by an agent, `whatIf` could only
  answer `wrong_agent`. Add-only, from 1.5.0.
