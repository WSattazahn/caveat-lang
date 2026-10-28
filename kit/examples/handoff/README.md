# Handing work to a sub-agent

A main agent hands a database cleanup to sub-agents, and the decision ledger
goes with the work. The sub-agent sees why it was asked and what is closed
off, can't do what the main agent foreclosed, acts only within its grant, and
its report brings its own caveats back. Every input is illustrative, and no
model is called: each agent is a fixed list of events.

- `handoff.cav`: the program every agent shares. Events named `main_*` are
  the main agent's and `sub_*` a sub-agent's.
- `handoff.scenarios.json`: twelve scenarios that state its behavior.
- `host.mjs`: the handoff with two sub-agents, each in a scoped session, and
  their work merged by replay.

## Run it

In a project with this package installed, copy the folder out of
`node_modules/caveat-lang/examples/handoff` and run:

```sh
npx --no-install caveat-lang test handoff.scenarios.json
node host.mjs
```

## What the language does

| Need | How | Scenario |
| --- | --- | --- |
| The sub-agent's work rests on the main agent's reasons | `delegate` cites `committed(plan)`, and `edit_queries` cites `committed(delegate)` | H01 |
| It can't do what the main agent closed off | `notify_customers … irreversible forecloses rename_column, drop_column`: refused as `evaluation/foreclosed` | H02, H03 |
| It acts only within a grant | `permitted by latest(grants) for 2`; a revoked grant is refused as `not_permitted` | H04, H04b |
| New requests and its objections don't change the plan mid-task | `hold plan` defers them; `release` on the main agent's return applies them | H05 |
| A wrong reason never waits | a withdrawal reopens the plan through the hold, and reaches the sub-agent's work | H06 |
| A withdrawn reason grounds nothing new | a new plan or ship on a withdrawn schema review or report is refused (`caveat check --withdrawals` reports a commit without that guard) | H11 |
| Its doubts come back with its report | a `linux_only` report blocks the irreversible `ship` until the main agent retains the caveat on purpose | H07, H08, H09 |

## What the host does

This program declares no agents, so the runtime does not know who sends an
event: without help, a sub-agent could send `main_return` and release the
main agent's hold (H10). `caveat-lang/handoff` keeps that boundary in the
host. From the 1.4 draft a program can also keep it itself; see
[Letting the program check the sender](#letting-the-program-check-the-sender).

```js
import { replayOnto, scopeSession } from 'caveat-lang/handoff';

const ledger = main.save({ compact: true });                 // the handoff
const a = scopeSession(runtime.restore(source, ledger), subEvents);
a.dispatch('main_return');
// {outcome: "rejected", origin: "input", code: "unknown_event",
//  message: "main_return is not one of this session's events: sub_edit, …"}
a.dispatch('sub_edit');                                      // the runtime decides as usual
a.accepted;                                                  // [{event: "sub_edit", payload: {}}]
```

- `scopeSession(session, events)` returns a session that sends only the
  listed events. Any other event is refused as `input/unknown_event` and
  never reaches the session. Reading it (`snapshot`, `view`, `save`) is not
  limited: the ledger is what is handed over. `accepted` lists the events it
  sent that the runtime accepted.
- `replayOnto(session, events)` sends each event to another session in
  order. One that is refused is a conflict, reported with its refusal; the
  rest are still sent.

To merge two sub-agents that started from the same ledger, restore one's save
and replay the other's `accepted` events onto it:

```js
const merged = runtime.restore(source, a.save());
const { conflicts } = replayOnto(scopeSession(merged, subEvents), b.accepted);
```

In `host.mjs` both sub-agents commit `edit_queries`, so B's commit is a
conflict, refused as `evaluation/decision_in_force` because A's is in force;
B's objection still applies and waits on the hold. The merge depends on the
order: both write the one `reports` stream, so whichever is replayed last is
`latest(reports)`. When sub-agents work in parallel, give each its own stream
and decision (`for agent as $g routed by agent`), and the order stops
mattering.

## Letting the program check the sender

[Agents 0.1](../../docs/reference/spec/caveat-agents-0.1.md), a 1.4 draft,
lets the program say which agent may send which event:

```caveat
agent main sends main_request, main_handoff, main_return;
agent sub sends sub_edit, sub_report;
```

A scope made with `{ agent }` sends every event as that agent, so the
runtime checks it and records it:

```js
const a = scopeSession(runtime.restore(source, ledger), subEvents, { agent: 'sub' });
a.dispatch('main_return');      // refused as input/wrong_agent, by the program
a.accepted;                     // [{event: "sub_edit", payload: {}, agent: "sub"}]
```

Each journal entry, reading and withdrawal the event wrote names `sub`, and
`explain` shows `#6 sub_edit by sub: committed …`. `replayOnto` sends an
entry that names an agent as that agent, so a merged ledger keeps who did
what. `handoff.cav` itself is unchanged: adding the lines above to a copy is
how to try it.

## What it does not do

- A scoped session is a convention of the host, not authentication. Code
  that holds the unscoped session can send anything, and with agents declared
  can name any agent: the agent is the host's word.
- Every agent runs the same program: a save binds its source's SHA-256.
- Only an irreversible decision forecloses. "We ruled out a rewrite" is a
  decision that can be undone, so `sub_rewrite` is refused by a guard with a
  message, and `explain` shows no link between the two
  ([foreclosure](../../docs/reference/spec/caveat-foreclosure-0.1.md),
  "Questions settled", 5).
