# Design note: general work around a checked record

Status: proposed 2026-10-31, for the owner's plan of that day: make Caveat
usable for general work while keeping its checked record intact. This note adds
no syntax and changes no specification, promise or output. Each step below is
a series of pull requests ([Pull requests, in order](#pull-requests-in-order)),
and a step that changes what a program means starts with a specification PR.
Receipts are `path: Lx–Ly` at `main` `36928e8`.

## In short

- **Step 1, reach.** Any program can call today's Caveat as its "why" layer.
  JavaScript hosts already can, and so can MCP hosts for a program that
  restricts no event to an agent. Python needs its package on PyPI and the
  parts it still lacks; other languages get the same operations over local
  HTTP.
- **Step 2, a general layer.** Ordinary code (loops, math, files, network
  calls) runs beside a Caveat program as *tasks*, written first in Python or
  JavaScript. A task gets a request and a `report` that sends only its answer
  to that request. The record keeps when each request started, and its answer
  or the runner's stop.
- **Step 3, any language, boxed in.** A task can be a WebAssembly component
  built from any language that compiles to one. It runs in a box that gives it
  the request, `report`, and only the abilities the person running it grants,
  such as one web address or one folder.
- **Step 4, a Caveat task language.** A small language for tasks, checked
  against its program before it runs, running in step 3's box. It is a choice,
  never a requirement, and the guide recommends it only where the examples
  show it does better.
- **Step 2 trusts its task code; step 3 does not.** A Python or JavaScript
  task runs in the runner's process and could reach the record's files by
  going around `report`. Step 3's box closes that, as long as no grant reaches
  the store or code that runs outside the box ([The wall](#the-wall)).
- **The first general layer needs no language change.** A decision the program
  commits is the request; the task's answer is the outcome recorded on that
  decision; [Agents 0.1](../../spec/caveat-agents-0.1.md) says which task may
  answer and lets the runner record each start in the program's own record. A
  program built this way runs on `caveat-lang@1.4.1` (see
  [A worked example](#a-worked-example)).
- **Nothing here needs 2.0.** Every step adds to 1.x. A rule that calls code
  outside the program is left out by the owner's plan of 2026-10-31, under
  which results reach the record only through declared events. Replay is a
  reason too, because it would run that code again.
- **The owner chose this order** on 2026-10-31
  ([The owner's choice](#the-owners-choice)).

## The two layers

| | Checked core | General layer |
| --- | --- | --- |
| What it is | A Caveat program, in today's language, unchanged | Tasks: Python or JavaScript functions (step 2), WebAssembly components from any language (step 3), or files in the Caveat task language (step 4) |
| What it may do | Change the record, one accepted event at a time | A Python or JavaScript task: anything its process may do, such as loops, math, files and network. A boxed task: only what the person running it grants |
| What it may not do | Call out: the language has no foreign calls and no file, process, host clock or network access (`docs/THREAT_MODEL.md: L45–50`) | Change the record, except by sending a declared event |
| How it is checked | Load checks, `check`, scenarios and explanations; every event is accepted or refused whole | Each event it sends is checked by the program's rules; its own code is its author's |
| On replay | Rebuilt from the source and the events | Not run again |

Caveat is already "meant for the decisions an application records, not for
inner loops" (`docs/LIMITS.md: L47–49`). The general layer is where the loops
go.

## The wall

These rules hold for every task, in every step and every task language. A
Python or JavaScript task (step 2) is held to them only while its code uses
what it is handed. A boxed task (steps 3 and 4) can change the record only
through `report`, whatever its code does, as long as no grant reaches the
store or code that runs outside the box. Its own code still decides whether
it asks `waiting` before an outside call and uses its key (rules 6 and 7).
[Who the wall holds against](#who-the-wall-holds-against) says more.

Rules 1, 2, 3, 5, 6 and 7 say what enforces them and name a test, which the
PR that builds the rule adds. Rules 4 and 8 add no check of their own: the
view a task gets is read-only, and the runtime's limits apply to every event.

1. **Only an accepted event changes the record.** A task holds no session,
   save, store or archive. It holds a `report` bound to its request, which
   sends only the events its agent's `sends` list names, as that agent, and
   nothing else. An event no agent lists is not among them, though the
   runtime would accept it from any agent (`spec/caveat-agents-0.1.md: L68`).
   *Enforced by* the runner, which sends each report through the starter
   host's `send(…, { agent })` (PR 3) and refuses an event outside the task's
   list before it reaches the host, as `scopeSession` does for a bare session
   (`kit/lib/handoff.mjs: L55–70`); and by the runtime, which refuses an event
   from an agent the program does not let send it as `input/wrong_agent`
   (`spec/caveat-agents-0.1.md: L60–70`).
   *Test:* a task that reports an event outside its list is refused and the
   saved record is byte for byte unchanged; a test lists what a task receives
   and finds no session, host, store or save.
2. **A result is a claim by the task that produced it.** It enters as an event
   payload, and the program's rules decide what evidence it becomes and which
   caveats it carries. The record names the agent that sent it and the request
   it answers, and the runner, not the task, says which request that is: it
   sets the answer's `revision` itself. The runner never sends an answer for a
   task, its `send` refuses the agent of any task and its own, and each task
   has an agent of its own (runner step 2). So in a runner's record
   `by payments` means the payments task sent it, while the runner is the
   store's only writer (runner step 8). The record still vouches for nothing:
   Caveat records what a host tells it and authenticates no sender
   (`docs/THREAT_MODEL.md: L25–28`, `spec/caveat-agents-0.1.md: L100–104`).
   *Enforced by* the runner, which sets `revision` itself and whose `send`
   refuses the agent of any task and its own.
   *Test:* a report whose payload names another revision is refused and
   nothing is sent; `runner.send`, `caveat run`'s standard input and its
   `--http` each refuse the runner's agent and every task's agent, and
   `caveat run` takes no `restore`.
3. **The program never calls out.** No rule can call a task, host code or the
   outside world. Loading, checking, testing, explaining, replaying and
   restoring a program run no task. This keeps the threat model's promise
   that running a program, an event or a save "must not execute host code,
   touch files, start processes or reach the network"
   (`docs/THREAT_MODEL.md: L20–21`) for every tool but the runner, including
   the hosted MCP server, which must never fetch a URL because a program
   asked. The runner is `caveat run`, `python -m caveat_lang run`, and the
   `caveat-lang/run` and `caveat_lang.run` exports. Its job is to run the
   tasks the person running it names. There an accepted event can start task
   code, so whoever can send the runner events can make its tasks act, within
   what the program accepts; the threat model says so when the runner ships.
   A Python or JavaScript task has the process's access; a boxed task
   (step 3) has only what that person grants.
   *Enforced by* the language, which has no call-out form (unchanged), and by
   keeping task code in host files the runtime never reads.
   *Test:* `caveat test`, `explain` and the MCP tools run a program whose
   runner has tasks, and no task runs.
4. **Reading changes nothing.** A Python or JavaScript task may read the view
   it is handed. A boxed task gets only its request and whether it is still
   waiting. Reading spends no attention, observes no evidence and writes
   nothing. What a task learns by reading and then reports enters as that
   task's claim (rule 2).
5. **Replay never runs the general layer again.** The record is rebuilt from
   the program's source and its events alone, and results are already in the
   events. A runner's session names agents, so it keeps no replay log and its
   compact save is its state save (`spec/caveat-agents-0.1.md: L137–140`). Its
   store therefore keeps the accepted events itself, each with its agent, in
   the event log of step 1, as the threat model advises a host that needs the
   history behind a save (`docs/THREAT_MODEL.md: L71–74`). With the log, an
   accepted event is durable once its line is written, and the checkpoint
   only shortens a restart. Restoring the checkpoint, `explain` of the log, or
   `replayOnto` from it gives the same record without running any task, and a
   task that would answer differently today changes nothing already recorded.
   `carrySave` does not apply: it takes only a replay save.
   *Enforced by* the store's log, which keeps every accepted event, and by
   the language, which cannot call a task (rule 3).
   *Test:* replaying a runner's log through `caveat explain`, with no tasks
   loaded, gives the record the runner holds.
6. **One standing answer per request, and no started request is dropped.** A
   request is a committed revision such as `refund@2`. An answer is what
   settles a request: a success, such as `refunded`, or a failure, such as
   `refund_failed`. The program records it with `resolve`, and a second answer
   is refused as `evaluation/already_resolved` while the first answer's
   evidence stands (`spec/caveat-outcomes-0.1.md: L139–146`). To correct an
   answer, the program withdraws the evidence it was read from, and the
   journal keeps both (`spec/caveat-outcomes-0.1.md: L159–170`). It names that
   reading by its place in its stream, as
   `withdraw reading(failures, K) because …`, even after later readings, for
   as long as the stream holds it
   (`spec/caveat-reading-index-0.1.md: L53–60`, `L73–79`).
   `resolve revision(SERIES, K)` lands an answer on the request it answers
   even after a newer one exists (`spec/caveat-outcomes-0.1.md: L117–119`).
   Every started request gets an answer from its task, or the runner's stop
   event when it gave up without one. A stop is not an answer: the request
   stays committed and unanswered, so the example refuses every new refund
   until a person answers it as `ops` or the program reopens it. A waiting
   request the runner has not started stays visible in the view. Each start
   carries a random key, recorded in the start event, that the outside
   service uses to do the work once: a task tried again uses the same key,
   and a key never repeats in a new ledger.
   *Enforced by* the program's `resolve`, which refuses a second answer, and
   by the runner, which sends the stop event when it gives up.
   *Test:* a task that throws after its outside call is tried again with the
   same key; when its tries run out, a `lookup` that finds the work reports the
   answer as the task's agent, the program accepts it, and the runner sends no
   stop; with no `lookup`, or one that leaves the request without an answer,
   the runner sends the stop event and lists the request for a person, and a
   restart does not call that `lookup` again; a second answer is refused as
   `already_resolved`, and the runner sends no stop for it; a crash before a
   start is durable, after the outside call, or after an answer is accepted but
   before it is durable, each ends with one start and one answer, and the
   test's service does the work once for each key.
7. **Work starts only for a waiting request, and the record shows when.** A
   request is waiting while it is committed, not reopened and not resolved.
   Caveat's "in force" means committed and not reopened, and an answered
   request stays in force (`spec/caveat-outcomes-0.1.md: L128–131`), so
   `permits` alone is not enough. The runner reads the view and sends the
   program's start event, as its own agent, for the revision it read. The
   program refuses a start unless that revision is the current one and
   waiting. `committed` and `reopened` ask only about the current revision,
   so the guard also compares the start's revision with
   `history_count(SERIES)`. If an event between the read and the start leaves
   that revision no longer current or waiting, the program refuses the start,
   and the runner reads the view again. The runner calls the task only once
   the start is durable. Once the request reopens, the runner starts nothing
   more for it and signals a task already running. It cannot take back an
   outside call already made: a task asks `request.waiting()` just before its
   outside call, and only the outside service can close the moment between
   that check and the work, for example by holding a payment until a second
   call. An answer that still arrives is recorded on its request (rule 6), and
   the record shows the start came before the doubt. A go-ahead withdrawn
   after the commit reopens nothing by itself
   (`spec/caveat-permission-0.1.md: L91–94`); a program that must stop the
   work then reopens the request, as the example does when fraud is flagged.
   Work that cannot be undone should be asked for by an `irreversible strict`
   series, which commits only when it retains each `material` or worse caveat
   its grounds, guard or grant carry
   ([Strict Irreversible 0.1](../../spec/caveat-strict-irreversible-0.1.md));
   such a series holds one revision (limit 4 under
   [What the first version cannot do](#what-the-first-version-cannot-do)).
   *Enforced by* the program, which refuses a start for a request that is not
   the current one or not waiting, and by the runner, which calls a task only
   once its start is durable.
   *Test:* a start that names a reopened, answered, superseded, missing or
   already started revision is refused, and no task runs; a start whose
   append fails is written again, and only then is its task called; after a
   crash between the start and the call, the restart tries the task with the
   recorded key; a running task's signal fires when its request reopens;
   after a restart, the runner does not try again a request that was reopened
   meanwhile, and starts no request whose start the program refused.
8. **The core keeps its limits; loops live outside.** Every event stays within
   the work, history and text limits. Long computation runs in tasks. A
   JavaScript task runs on the runner's event loop, so one that computes
   without awaiting holds up the runner, its time limits and the
   application's sends until it returns; such a task does its computing in a
   worker thread of its own, or is written as a boxed task (step 3). A result
   too large for one event is split across events or refused. A program's
   decision series, and each history declared with a `limit`, are bounded
   over its whole life (limit 5); a reading stream declared with a `window`
   retires its oldest reading instead (`spec/caveat-reactive-0.5.md: L220–222`).

### Who the wall holds against

The wall guards the record against task code that uses only what it is
handed: a bug, a wrong answer, a model's mistake. In step 2 it does not hold
against code written to get around it. A Python or JavaScript task runs in the
runner's process, which writes the store, so such code can import the kit,
open the store and write a checkpoint that names any agent, and restore
accepts any save the program could produce (`docs/THREAT_MODEL.md: L67–74`).
A language-model task with tools that read files or run code can be steered
there by text other agents wrote into a request's notes. Running the runner in
a container or as another user does not help, because the task and the store
are then in the same box. So in step 2, untrusted code, and models with file
or code tools, stay out of tasks. Step 3's box is what holds the wall against
a task: it gives the task nothing but the request, `report` and the abilities
the person running it grants, and never the store or code the runner loads.

## Step 1: reach

### What exists

| Surface | For | Where | What is missing |
| --- | --- | --- | --- |
| JavaScript library: sessions, the starter host, handoff, explain, scenarios, saves | Node 20 and later, browsers | `kit/package.json` exports | The starter sends without an agent (`kit/lib/starter.mjs: L126`), the typed session and the typed host take no options (`kit/lib/types.d.mts: L31–32`, `kit/lib/starter.d.mts: L139–141`), and it keeps no event log |
| `caveat serve`: JSON lines on standard input and output | Any language that can start a process | `spec/caveat-serve-0.1.md: L5–9` | A client in each language; Python's is an example, `kit/examples/agent-evidence/caller.py` |
| MCP tools, local and hosted | Agents in MCP hosts | `kit/docs/MCP.md: L57–65` | `caveat_explain` and `caveat_dependents`, and from 1.5 `caveat_dispatch` and `caveat_find`, take events without an `agent` (`kit/lib/mcp.mjs: L36–47`, `L69–83`, `spec/caveat-agents-0.1.md: L159–162`). So a program that restricts an event to an agent can be tested through MCP, since `caveat_test`'s steps name the agent, but not explained, traced, continued or searched. `caveat_dispatch` keeps one ledger across calls from 1.5 |
| The Python package `caveat_lang`: the same runtime in-process, the same bytes as the npm kit | Python 3.9 and later | `python/README.md: L36–44`, `L150–156` | Not on PyPI. No explain, dependents, what-if, scenario runner, persisting host or handoff |

### What step 1 adds

- **Python on PyPI.** `pip install caveat-lang`. From the release that first
  ships it, the package's version is the npm package's version, because both
  carry the same runtime and are held to the same bytes, and a check holds the
  version in `python/` to `kit/package.json`, allowing a `.postN` suffix only
  on a post-release's branch. A fix to the Python package alone ships as the
  next patch release of both packages, such as `1.5.1`. A post-release such
  as `1.5.0.post1` is kept for changes that touch no code, such as a wrong
  README, as
  [Python's version rules](https://packaging.python.org/en/latest/specifications/version-specifiers/)
  ask. It is built from a branch off the release's tag, and its publish
  refuses a build whose code differs from the tag's.
  [The compatibility commitment](../COMPATIBILITY.md) gains a row for the
  Python API, with the Python versions and wheel platforms it covers. The
  same-bytes check runs on the Linux x86_64 wheel only, so the row names the
  wheels held to the npm kit's bytes. Today that commitment covers the npm
  package only (`docs/COMPATIBILITY.md: L7`).
- **Python parity.** `explain`, `dependents` and `what_if` in Python: a port
  of the kit's explain and dependents reports and new what-if methods in the
  native module. The differential test, which replays every scenario file
  through both packages (`python/README.md: L150–156`), compares dispatch
  outcomes, snapshots, views, saves, `check` and `interface` today; it gains
  comparisons of the explain and dependents reports and what-if outcomes, and
  the Python workflow also runs on changes under `kit/`.
- **MCP events that name their agent.** Each MCP tool that takes events
  accepts an optional `agent` on each, so an MCP host can explain, trace,
  continue and search a program that restricts events to agents. The hosted
  server gets it at its next version bump.
- **One small API, the same in each language.** It is the starter host's,
  plus four additions:

  | Call | JavaScript (`caveat-lang/starter`) | Python (new) | Any language (`caveat serve --http`, new) |
  | --- | --- | --- | --- |
  | Open a ledger kept on disk | `openHost({ runtime, source, store })` | `open_host(source, store=DIR)` | `--state FILE` |
  | Send an event, as an agent | `send(event, payload, { agent })`, the option new | `send(event, payload, agent=…)` | `{"op":"dispatch", …, "agent": …}` |
  | Send only some events, as one agent | `scope(sends, { agent })`, new | `scope(sends, agent=…)` | none: whoever holds the token is the host |
  | Is a decision in force? | `permits(decision)` | `permits(decision)` | `view` |
  | Is a request waiting for its answer? | `waitingRequest(series)`, new | `waiting_request(series)` | `view` |
  | What does it rest on? | `assessment(decision)`, `history(decision)` | the same | `explain` |
  | What rests on this evidence? | `dependents(name)`, new | `dependents(name)` | `dependents` |

  `waitingRequest` returns the current revision when it is committed, not
  reopened and not resolved, and nothing otherwise. `permits` stays true after
  a request is answered, because resolving does not reopen
  (`spec/caveat-outcomes-0.1.md: L128–131`), so the starter guide's pattern of
  acting while `permits` is true (`kit/docs/STARTER.md: L40–42`) is for
  decisions no task answers. A series a task answers is acted on only by its
  runner. The call is not named for "open": in the view, a commitment's
  `open: true` means it was reopened (`kit/lib/starter.mjs: L213`).
- **An opt-in event log.** The store keeps the accepted events, each with its
  agent, in an append-only log beside the checkpoint, because a session that
  names agents keeps no replay log (rule 5). The log is opt-in:
  `openHost({ …, log: true })` keeps it, and the runner always sets it. A host
  opened without it writes, reads and checks no log, so its stores, restarts
  and `durable` mean what they mean in 1.4. A store keeps the log through new
  optional members, and `log: true` refuses a store without them.
  - Line N of the log is the event at sequence N, from the store's first
    event, with only `event`, `payload` and `agent`, so `caveat explain` reads
    it as it is. A runner refuses a store whose log does not start there.
  - The checkpoint names the SHA-256 of the log's lines up to its own
    sequence. Opening refuses a checkpoint that names no digest, such as one
    a host without the log wrote; a log shorter than the checkpoint; and a
    log whose lines up to it have another digest. It drops a last line cut
    short, and sends the lines past the checkpoint again.
  - So with the log, an event is durable once its line is written, and one
    whose checkpoint write failed comes back after a restart; the starter
    guide says so. If an append fails, the host writes no checkpoint, and
    before its next append it reads the log back and writes the line again
    only if it is missing.
  - A store with a log has one writer. A host opened with the log, and every
    runner, holds a lock on its store while it runs, and refuses a store
    another process holds. The lock ends with its process, so a crash leaves
    none behind. A Python host takes the same lock as a Node host, so each
    refuses a store the other holds; Node has no file-lock call, so that lock
    cannot be Python's `fcntl.flock`.
  - A host without the log takes no lock, as in 1.4. A checkpoint that names
    a log has a schema name of its own, and a host without the log refuses
    it, as the 1.4 starter refuses any schema name but its own
    (`kit/lib/starter.mjs: L57–58`). So no host without the log, of 1.4 or
    later, writes to a store with a log, whether its runner is running or
    stopped.

  A Python store keeps the same files as the Node file store, so a ledger
  written by one opens in the other.
- **Local HTTP.** `caveat serve --http PORT` carries Serve 0.1's requests and
  responses, one per POST, so Go, Java, C#, Ruby or a shell script can call
  Caveat without a stdio client. It listens on the loopback address only,
  which keeps out other machines but not other programs on the same one or a
  web page in its browser. So it also requires a secret token, which it gives
  on its ready line; it refuses a request that has an
  `Origin` header, a `Host` other than a loopback name, or a body that is not
  JSON; and it leaves `restore`, `drainArchive` and `close`
  (`kit/lib/serve.mjs: L12–21`) off unless a flag turns them on. Whoever holds
  the token is the host and can name any agent. It is specified first, as an
  added section of [Serve 0.1](../../spec/caveat-serve-0.1.md), as `--state`
  was, and its PR updates the [threat model](../THREAT_MODEL.md) and the kit's
  declared capabilities
  ([package security](../PACKAGE_SECURITY.md#declared-capabilities)).
- **Typed Python.** `caveat types --python` writes a program's events and
  payloads as Python types, as `caveat types` does for TypeScript.
- **A guide**, "Use Caveat from your program": which of the library, the
  host, `serve` or MCP to use, in each language.

## Step 2: the general layer

### The program asks, a task answers, the record keeps both

Everything a first general layer needs is in the language already:

| Need | In today's language | Where it is defined |
| --- | --- | --- |
| A request | A committed revision of a decision series, `refund@2` | [Reactive 0.5](../../spec/caveat-reactive-0.5.md) |
| Its arguments | The revision's value, and the notes on the readings it rests on, which the runner reads from the snapshot, because the view has no reading streams and shows a note only in the effects of the event that took it | [Reading Notes 0.1](../../spec/caveat-reading-notes-0.1.md) |
| Who may answer | `agent payments sends refunded, refund_failed;` | [Agents 0.1](../../spec/caveat-agents-0.1.md) |
| When work started | `agent runner sends refund_started, refund_stopped;`, and a start the program accepts only for the current revision, while it is waiting | [Agents 0.1](../../spec/caveat-agents-0.1.md) |
| The answer | `resolve revision(refund, N) = 1 because latest(receipts)` | [Outcomes 0.1](../../spec/caveat-outcomes-0.1.md) |
| One standing answer | A second `resolve` is refused as `already_resolved` while the first answer's evidence stands | Outcomes 0.1, "When a resolve is refused" |
| Permission to commit | `permitted by latest(approvals) for pending`, checked when the commit runs | [Permission 0.1](../../spec/caveat-permission-0.1.md) |
| Cancelling | Reopening the revision: the runner starts nothing more for it and signals a task already running | [Reactive 0.5](../../spec/caveat-reactive-0.5.md) |
| Work that cannot be undone | `decisions … irreversible strict` | [Strict Irreversible 0.1](../../spec/caveat-strict-irreversible-0.1.md) |

### A worked example

A refund agent. Support sends a ticket, a manager approves it, the runner
records when a payments task starts, the task pays, and a fraud check can
raise doubt about the latest ticket at any point. Here a *ticket* is
support's ask, a reading in `tickets`, and a *request* is the refund it leads
to, `refund@N`. The program decides; the runner and the task only report.

```caveat
claim refund_due;
evidence ticket from "the support ticket";
evidence manager from "a manager's go-ahead";
evidence runner_log from "the task runner";
evidence provider from "the payment provider";
evidence provider_error from "the payment provider's error";
evidence fraud_check from "the fraud team";
readings tickets from ticket limit 16;
readings approvals from manager limit 16;
readings starts from runner_log limit 16;
readings stops from runner_log limit 16;
readings receipts from provider limit 16;
readings failures from provider_error limit 16;
decisions refund limit 16;
identifiers limit 256;
state pending = 0;
state asked = 0;
state started = 0;
state stopped = 0;

agent support sends request_refund;
agent manager sends approve;
agent runner sends refund_started, refund_stopped;
agent payments sends refunded, refund_failed;
agent ops sends refunded, refund_failed;
agent fraud sends flag_fraud;

event request_refund order id, amount min 0 max 10000;
event approve order id;
event refund_started revision min 1 max 16, key id;
event refund_stopped revision min 1 max 16;
event refunded revision min 1 max 16, receipt id;
event refund_failed revision min 1 max 16;
event flag_fraud order id;

# One refund at a time, each approved once, and never on a withdrawn ticket.
on request_refund when committed(refund) and not reopened(refund) and not resolved(refund)
    reject "A refund is still waiting for its answer.";
on request_refund sample tickets = amount supports refund_due noting order;
on request_refund set pending = order;
on request_refund set asked = 1;
on approve when asked == 0 reject "There is no new ticket to approve.";
on approve when withdrawn(latest(tickets)) reject "This ticket was withdrawn.";
on approve sample approvals = order;
on approve when committed(refund) and not reopened(refund)
    reopen refund because latest(approvals);
on approve commit refund because enough using latest(tickets)
    permitted by latest(approvals) for pending;
on approve set asked = 0;

# The runner records one start per request, only while it is waiting.
on refund_started when not committed(refund) or reopened(refund) or resolved(refund)
    reject "No refund is waiting to be paid.";
on refund_started when revision != history_count(refund) reject "This is not the current refund.";
on refund_started when started == revision reject "This refund has already started.";
on refund_started sample starts = revision noting key;
on refund_started set started = revision;
on refund_stopped when resolved(revision(refund, revision)) reject "This refund is already answered.";
on refund_stopped sample stops = revision;
on refund_stopped when revision == history_count(refund) set stopped = revision;

# An answer names the request it answers: refund@N.
on refunded sample receipts = revision noting receipt;
on refunded resolve revision(refund, revision) = 1 because latest(receipts);
on refund_failed sample failures = 1;
on refund_failed resolve revision(refund, revision) = 0 because latest(failures);

on flag_fraud when order != pending reject "This order is not the latest ticket.";
on flag_fraud reveal fraud_check opposes refund_due;
on flag_fraud withdraw latest(tickets) because fraud_check;
on flag_fraud when committed(refund) and not reopened(refund) and rests_on_withdrawn(refund)
    reopen refund because fraud_check;

bind refund.status = if(not committed(refund), "none",
    if(reopened(refund), "reopened", if(resolved(refund), "paid or failed",
        if(stopped == history_count(refund), "needs a person", "asked"))));
```

On `caveat-lang@1.4.1` from npm, on 2026-11-01, the program loads, `check`
reports no warnings, and eight scenarios pass. Each row below is a step in one
of them, and the scenario checks its result, message included:

| Sent | As | Result |
| --- | --- | --- |
| `refund_started` before any approval | `runner` | refused, `policy/reject`: "No refund is waiting to be paid." |
| `refund_started` for revision 1 | `payments` | refused, `input/wrong_agent` |
| `refunded` for revision 1 | `support` | refused, `input/wrong_agent` |
| `refunded` for revision 2, which does not exist | `payments` | refused, `evaluation/not_committed` |
| `refunded` for revision 1, after its start | `payments` | accepted; `refund.status` is `paid or failed` |
| `refunded` for revision 1 again | `payments` | refused, `evaluation/already_resolved` |
| a second `approve` of the same ticket | `manager` | refused, `policy/reject`: "There is no new ticket to approve." |
| `approve` after `flag_fraud` withdrew the ticket's reading `tickets@1` | `manager` | refused, `policy/reject`: "This ticket was withdrawn." |
| `flag_fraud` naming an order that is not the latest ticket's | `fraud` | refused, `policy/reject`: "This order is not the latest ticket." |
| a second `refund_started` for `refund@1` | `runner` | refused, `policy/reject`: "This refund has already started." |
| `refund_started` for revision 1, or 3, once `refund@2` is the current one | `runner` | refused, `policy/reject`: "This is not the current refund." |
| `refund_started` after `flag_fraud` reopened `refund@1` | `runner` | refused, `policy/reject`: "No refund is waiting to be paid." |
| `refunded` after `flag_fraud` reopened a started `refund@1` | `payments` | accepted, and recorded on the request it answers |
| `refund_stopped` for a started `refund@1` | `runner` | accepted; `refund.status` is `needs a person` |
| `request_refund` after that stop | `support` | refused, `policy/reject`: "A refund is still waiting for its answer." |
| `refunded` after that stop | `ops` | accepted: a person settles what the runner gave up on |
| `refund_stopped` for a reopened `refund@1`, after `refund@2` was stopped | `runner` | accepted, and `refund.status` stays `needs a person` |

For the late answer, `caveat explain` prints each step with its sender, and
the events show the start came before the doubt:

```text
Events
    1  request_refund by support {"order":"B-2","amount":900}  accepted
    2  approve by manager {"order":"B-2"}  accepted
    3  refund_started by runner {"revision":1,"key":"k-7"}  accepted
    4  flag_fraud by fraud {"order":"B-2"}  accepted
    5  refunded by payments {"revision":1,"receipt":"re_10"}  accepted
…
    refund@1 = 900  reopened
      based on tickets@1
      tickets@1: observed at #1 request_refund by support {"order":"B-2","amount":900}
      tickets@1 has since been withdrawn at #4 because fraud_check
      permitted by approvals@1 for 1
      could also have been influenced by approvals@1
      #2 approve by manager: committed because tickets@1
      #4 flag_fraud by fraud: reopened because fraud_check
      #5 refunded by payments: resolved = 1 because receipts@1
```

The example becomes a tracked program, with its scenarios, in the examples
PR (11 below). It is not tracked yet, so the corpus sweeps do not change with
this note.

### What the first version cannot do

Running the example, and reading the specifications it uses, found five limits:

1. **One request at a time per series.** Only the current revision can be in
   force, so the program refuses a new ticket while a refund waits for its
   answer. A second ticket sent before the first was approved replaced
   `pending`, and approving the first was then refused as
   `policy/not_permitted` ("approvals@1 permits 1, not 2"). Asking again
   after an answer means reopening the answered revision, which the journal
   records as doubt: `#7 approve by manager: reopened because approvals@2`.
2. **One number per request.** A request's arguments are its revision's value
   and the notes on what it rests on. The task finds the order number in the
   note on the reading its revision rests on, which the runner hands it as a
   ground; `refund@2` can rest on `tickets@3`.
3. **No replay save.** Once an event names an agent, the session keeps no
   replay log, so its compact save is the full state save
   (`spec/caveat-agents-0.1.md: L137–140`). A runner's checkpoints grow with
   the record, and its history lives in the store's log (rule 5). A replay
   save that keeps agents is not designed yet. Held to the bound every replay
   save has today, at most 4,096 bytes with its header
   (`spec/caveat-replay-save-0.1.md: L28–34`, `L75`), it would help only small
   ledgers.
4. **Irreversible work is one request per series.** An irreversible series
   holds one revision (`spec/caveat-foreclosure-0.1.md: L82`). A program that
   pays many refunds it cannot take back needs a series per refund, which
   today means a routed `for` block over members fixed in the source. The
   example uses a plain series for that reason.
5. **A ledger answers a fixed number of requests, ever.** Each request is a
   new revision, and a decision series has no window
   (`kit/docs/REFERENCE.md: L91`). A series holds at most 256 revisions, and
   a program's histories hold at most 1,024 records between them; a full
   history refuses the next record as `limit/history_limit`
   (`spec/caveat-reactive-0.5.md: L216–232`). So a ledger answers no more
   requests than its series' limit, and fewer if another history fills first.
   The example's histories each hold 16, so its 17th `request_refund` is
   refused at the door instead of being accepted and never committed.
   Identifiers never depart either: a session holds at most N of them, N at
   most 65,536, and at most 1 MiB of their text
   (`spec/caveat-identifiers-0.1.md: L25`, `L87–90`). The example adds up to
   three for each request: an order, a start key and a receipt.

None of these breaks the wall. Limit 5 needs an answer that ships with the
runner (PR 12): a window for decision series, which would let old requests
depart as old readings can, or a Requests profile, and with either, a way for
identifiers to depart or a count of how many requests a ledger can answer.
Limits 1, 2 and 4 need one only if the examples show they matter. Limit 3
needs a replay save that keeps agents: a new schema, written only when a host
asks for it with a new save option, as Replay Save 0.2's form is, so that
`save()` and `save({ compact: true })` stay byte for byte as Agents 0.1 says.
Only small ledgers gain from it.

### The runner

The runner is the one place tasks run. It is built on the starter host with
the event log on, so it persists and restarts as the starter does
([starter guide](../../kit/docs/STARTER.md)). A sketch of the JavaScript form,
`caveat-lang/run`; names are settled in its PR:

```js
const runner = await openRunner({
  runtime, source, store: await fileStore('refund-store'),
  agent: 'runner',                  // the program says: agent runner sends refund_started, refund_stopped;
  tasks: [{
    agent: 'payments',              // agent payments sends refunded, refund_failed; no other task uses it
    answers: 'refund',              // a decision series of the program
    start: 'refund_started', stop: 'refund_stopped',
    attempts: 3, timeout: 30_000,
    async perform(request, report) {
      // request: { id: 'refund@2', revision: 2, key, value, grounds, signal, waiting }, and a read-only view
      if (!request.waiting()) return;     // checked just before the outside call
      const paid = await payments.refund({ amount: request.value, idempotencyKey: request.key, signal: request.signal });
      if (paid.declined) return report('refund_failed', {});
      return report('refunded', { receipt: paid.id });    // the runner adds revision: 2
    },
    async lookup(request, report) {       // at most once, when the runner gives up on the request
      const found = await payments.findByKey(request.key);
      if (found?.paid) return report('refunded', { receipt: found.id });
    },
  }],
});
await runner.send('request_refund', { order: 'A-18', amount: 15 }, { agent: 'support' });
```

The Python form, `caveat_lang.run`, reads the same:

```python
runner = caveat.open_runner(source, store="refund-store", agent="runner")

@runner.task(agent="payments", answers="refund", start="refund_started",
             stop="refund_stopped", attempts=3, timeout=30)
def pay(request, report):
    if not request.waiting():
        return
    paid = payments.refund(amount=request.value, idempotency_key=request.key)
    if paid.declined:
        return report("refund_failed", {})
    report("refunded", {"receipt": paid.id})

@pay.lookup
def find(request, report):
    found = payments.find_by_key(request.key)
    if found and found.paid:
        report("refunded", {"receipt": found.id})
```

From the command line, `caveat run PROGRAM --tasks MODULE --store DIR` runs
JavaScript tasks, and `python -m caveat_lang run` does the same for Python.
Each takes the application's events as `caveat serve` takes `dispatch`, on
standard input or over `--http`, and sends them as `runner.send` does (step 7
below), so it refuses the runner's agent and any task's agent. Neither takes
`restore` or `drainArchive`, whatever its flags: a restore would replace the
record with nothing in the log (rule 5), and the store already keeps the
archive.

What the runner does:

1. It opens the program from its store with the event log on, or fresh when
   the store is empty, and holds the store's lock while it runs (step 8). It
   refuses a store whose log does not start at the store's first event. On
   opening, the store sends again the events its log holds past the
   checkpoint (the event log in [what step 1 adds](#what-step-1-adds)).
2. It checks each task against the program's
   [interface](../../spec/caveat-interface-0.1.md): the task's agent and the
   runner's agent are declared, and no other task uses the task's agent; each
   event in the task agent's `sends` list has a `revision` parameter; the start
   event has a `revision` parameter and a `key` parameter of type `id`, the
   stop event has a `revision` parameter, both are sent by the runner's agent
   alone, which is no task's agent, and no other task uses either one as its
   start or stop; each `revision` parameter allows every revision the series
   can hold; and `answers` names a decision series that no other task answers.
   It refuses to start otherwise. The interface does not say which series an
   event resolves, so the runner cannot check that a task's events answer its
   own series. The program gives each task an agent of its own, and lets that
   agent send only the events that answer that task's series.
3. When it opens, and after each accepted event once that event is durable, it
   reads the view. If an accepted event's append fails, the runner tries the
   write again, as the starter's `flush()` does, until the event is durable.
   For a task whose series' current revision is waiting and not yet started, it
   sends the start event with a new random key, for the revision it read. When
   the program refuses a start, the runner reads the view again and notes the
   refusal in its own file. It sends no other start for that request, even
   after a restart, and lists the request for a person if it is still waiting.
   Once the start is durable, the runner notes the try in its own file (below)
   and calls the task with the request: its id, revision, key, value and
   grounds, each ground with its value and note when it is a reading (from the
   snapshot's reading streams), a signal that fires when the request reopens or
   is resolved, `waiting()`, and a read-only view.
4. `report` is bound to that request. It sends only the task agent's events,
   as that agent, through the host's single queue; it sets `revision` to the
   request's revision and refuses, without sending, a payload that names
   another. It returns what the host returns: whether the event was handled,
   accepted and durable, or the rejection.
5. A try succeeds when the program accepts the task's answer. When a task
   throws, runs out of time, or returns without an accepted answer, the runner
   does not know whether the outside work was done, so it reports nothing. It
   closes that try's `report` and tries again with the same key while the
   request is waiting, up to `attempts`. A refusal as `already_resolved` means
   another answer came first, so the request has its answer and nothing more is
   sent for it. When a started request has no answer and the runner will not
   try it again, because the tries ran out, the request was reopened, or the
   program refused what the task reported, the runner calls the task's
   `lookup(request, report)` if it has one. It notes the call in its own file
   first, as it does a try, and gives `lookup` the task's time limit; a
   `lookup` that throws or runs out of time reports nothing. `lookup` asks the
   outside service by key; it reports an answer only when the service has one,
   and a failure only when the service says the work will not be done. If the
   request still has no answer after `lookup`, or the task has no `lookup`, the
   runner sends the stop event as its own agent and lists the request for a
   person, with any report the program refused and its rejection. A stop
   refused because an answer came first ends the request too. A person settles
   a stopped request by sending an answer as an agent the program lets answer,
   the example's `ops`. A reported failure is already the answer: the program
   refuses a later success unless the failure's evidence is withdrawn. The
   runner never sends a refused event again, and the application's events and
   other tasks go on.
6. After a restart, a request with a start and no answer or stop is tried
   again with its recorded key while it is waiting and its tries have not run
   out. Tries count across restarts, because the runner notes each try before
   it calls the task, so a task that crashes or hangs the runner is not tried
   on every restart. Otherwise the runner goes on as in step 5: it calls
   `lookup` if the task has one, and sends the stop event if the request
   still has no answer. It calls `lookup` at most once for a request, so if
   its file shows a call from before the restart, it sends the stop event
   without calling it again, and a `lookup` that crashes or hangs the runner
   is not called on every restart either.
7. `runner.send` sends the application's events. It refuses the runner's
   agent and any task's agent.
8. A store has one writer. The runner holds the store's lock while it runs,
   so the application and a person send events through the running runner:
   `runner.send` in its process, or `caveat run`'s standard input or
   `--http`.

What the runner started, with which key, and how each request was answered or
stopped are events in the store's log, read back after a restart. Beside the
log, the runner keeps a file of its own, which the record never reads: each try
and each call of `lookup` it began, so they count across restarts; each start
the program refused, so a request handed to a person is not started again after
a restart; and each report the program refused, so the person who settles a
stopped request sees what the task tried to report. Restoring a store from an
older backup forgets the starts made since, so their requests could start again
under new keys and the outside work be done twice. A person restores one only
after checking the outside service for work done since the backup.

When the runner ships, the places that say Caveat does not authorize external
actions (`docs/THREAT_MODEL.md: L25–28`, `AGENTS.md: L32–33`,
`README.md: L148–149`, `docs/AI_AUTHORING.md: L6–7`,
`spec/caveat-foreclosure-0.1.md: L98–99`), and the starter guide's "Taking an
external action is still up to the application" (`kit/docs/STARTER.md: L19`),
say what becomes true: the runtime still never acts, and the runner starts
only the tasks it is given, for requests waiting when it starts them. The
threat model gains task code as an input, the person's own code, run with the
process's access by the runner and by nothing else, and the runner's own file
beside the store. Its promise about the host process names the runner, in its
four forms, as the only part of either package that runs code because events
arrived (rule 3).

### Later: Requests, and a window for decision series

Limit 5 needs an answer before the runner ships (PR 12). A window for decision
series would let old requests depart as old readings can; today a series
refuses one, because departure waits on a rule for revision lineage. A
Requests profile would answer limit 5 too, and, if the examples hit limits 1,
2 or 4, let a program hold many waiting requests at once, some of them
irreversible, give each named arguments, name the agent that answers, accept
answers with fields, and record a cancellation. Either way identifiers still
never depart, so PR 12 also lets them depart, or the runner's guide says how
many requests a ledger can answer. The acceptance scenario: two refunds asked
before either is paid, each answered once, and a fraud flag on one cancels
that one only. Each is new syntax, so a 1.x addition, specified before it is
built.

## Step 3: tasks in any language, boxed in

A task can be a WebAssembly component, built from any language that compiles
to one. The runner treats it as any other task: the same requests, without
the view and the signal, the same `report`, the same wall. What step 3 adds
is a box around the task's code.

- **Only what is granted.** In WebAssembly, "All I/O in Wasm goes through its
  imports and exports" ([WASI 0.2 launch](https://bytecodealliance.org/articles/WASI-0.2)).
  The runner gives a task the request, `report` and `waiting`, and links only
  the abilities the person running it grants, such as calls to one web
  address, one folder, or the clock. Toolchains import many standard WASI
  interfaces a task never uses, so the runner links each one the component
  imports with nothing behind it unless granted: no folder, no environment
  variable, no web address. When it opens (runner step 2), it reads what each
  component imports, and refuses to open if one imports something outside
  WASI and the `caveat:task` world, naming it, so no start is recorded for a
  task the box would refuse.
- **Never the store, and no folder code runs from.** The runner refuses a
  grant that reaches its store, or any file its process loads as code: the
  program, the application, task modules, components and their packages,
  compared after links are resolved. A task that could write a task module
  could have it run, unboxed, after the next restart. The same holds for any
  folder that other code runs from, such as a project someone builds or
  tests, and the runner cannot see those. So the person grants a task no
  folder that code outside the box runs from. Then a boxed task cannot go
  around `report` as a step-2 task can
  ([who the wall holds against](#who-the-wall-holds-against)).
- **Stopped when its time runs out.** A Python or JavaScript task that runs
  past its time cannot be forced to stop inside its process; the runner can
  only close its `report` and stop waiting, and for a JavaScript task only
  once it awaits. A boxed try runs off the runner's thread, where the runner
  can end it at any point: in Node, a worker it terminates; in Python, a
  child process, or a thread whose web calls the runner itself cuts off. The
  runner ends a try when its time runs out, so an abandoned try does nothing
  more. On the runner's own thread, a task waiting on a web call would hold
  up every other event, and in Node a task in a loop could not be stopped at
  all. Wasmtime's own time limit stops a task only once the web call it waits
  on returns.
- **Any language.** The author uses whatever language compiles to a
  component, and the runner does not need to know which.

A sketch of the box from the task's side, in WIT, the component model's
interface language; names are settled in step 3's design note:

```wit
package caveat:task@0.1.0;

world task {
  record ground { name: string, value: option<f64>, note: option<string> }
  record request { id: string, revision: u32, key: string, value: f64, grounds: list<ground> }
  variant reply { accepted, refused(string) }

  // All Caveat gives a task, besides what the person grants.
  import waiting: func() -> bool;
  import report: func(event: string, payload: string) -> reply;
  export perform: func(req: request) -> result<_, string>;
  // At most once for a started request without an answer, when the runner gives up on it.
  export lookup: func(req: request) -> result<_, string>;
}
```

A ground that is not a reading, such as revealed evidence, has no value. A
task with nothing to look up returns an error from `lookup`, and the runner
sends the stop event and lists the request for a person. A boxed task gets
no view and no signal: it learns that its request reopened by asking
`waiting`. Step 4's interpreter gets its task file from the runner, not from
a granted folder.

And from the person's side, granting one ability:

```js
tasks: [{
  agent: 'payments', answers: 'refund', start: 'refund_started', stop: 'refund_stopped',
  component: 'pay.wasm',
  grant: { http: ['api.payments.example'] },   // nothing else: no files, no environment, no other hosts
}]
```

Step 3's design note settles four questions, from these facts:

- **Which WASI release.** WASI 0.2 has been stable since January 2024. WASI
  0.3, which adds native async, shipped on 11 June 2026, is supported "in
  Wasmtime 46 and later … and in jco", and implementations "may continue to
  support WASI 0.2 alongside 0.3" ([WASI roadmap](https://wasi.dev/roadmap)).
- **Which engine.** In Node, jco runs components. Its WASI shim gives a
  component the network and the process's environment unless the runner
  turns them off, and its only network setting is on or off. In Python, the
  `wasmtime` package does: "Components are also supported in `wasmtime-py`",
  for 64-bit Python 3.9 and later ([PyPI](https://pypi.org/project/wasmtime/)).
  Its web calls take no list of hosts, and its version 50.0.1 has no way to
  call a WASI 0.3 async export (tried on 2026-11-01). It "follows the
  Wasmtime versioning scheme, with a new major version being released every
  month", and a new major version "can contain changes that break code
  written against the previous major version". So in both languages the
  runner itself holds a task to one web address; in Python that means
  writing the web calls the box offers.
- **How the engine arrives.** The npm package has no dependencies, and adding
  one "requires an explicit contract review" (`docs/PACKAGE_SECURITY.md: L45–46`);
  the Python package declares none (`python/pyproject.toml: L14`). The note
  chooses between that review and having the person running tasks supply the
  engine, as `openHost` is given its `runtime` today
  (`kit/lib/starter.mjs: L265`).
- **What the threat model promises.** Boxed tasks are a new input: what one
  can reach is what was granted, and no grant reaches the store or code the
  runner loads. A folder a task may write is only as safe as the code that
  later runs from it.

## Step 4: a Caveat task language

The owner asked for a Caveat language that does this work well: a strong
choice, never a requirement. It runs in step 3's box, so to the runner it is
one more boxed task, with the same wall, protocol and grants as any other.

What would make it the strong choice, each shown by its examples before the
guide says so, and each tested in PR 19:

- **Checked against its program before it runs.** A task file names its
  program and the agent it answers for. From the program's
  [interface](../../spec/caveat-interface-0.1.md), the checker knows which
  events that agent may send and what each carries, so a report of an event
  the agent may not send, or with a misspelled or missing field, stops the
  task from loading instead of being refused while it runs.
- **Abilities written in the file.** The task lists what it needs, such as
  one web address, at its top. The runner shows that list when it asks the
  person to grant it. It gives the interpreter, for that file, only what the
  list names and the person grants.
- **Results that say what they came from.** Each value keeps which inputs it
  was computed from: the request's fields and each answer an ability gave. A
  report can carry that list in a field the program declares, so the program
  can note it on a reading it records
  ([Reading Notes 0.1](../../spec/caveat-reading-notes-0.1.md)). A reading
  keeps one note, so the example, whose receipt reading notes the receipt,
  would record the list on a second stream.
- **Runs that can be checked again.** A task's only inputs are its request
  and what its imports return: `waiting`, `report` and each granted ability.
  When the person running a task asks for it, the runner writes those inputs
  to a file beside the store, which the record never reads. Anyone can then
  run the task again on them and get the same reports. That checks the task;
  it never changes the record, which replay still rebuilds without running
  tasks (rule 5).
- **Small and familiar.** It reads like a Caveat program, and its reference
  is meant to fit on one page an agent can load, so the cost of one more
  language stays small.

It is built as an interpreter in Rust, the runtime's language, compiled to a
component that runs in step 3's box; a compiler comes only if measured speed
needs one. Task files get their own extension, so no tool that loads programs
takes a task for one. It is new syntax in its own files, specified before it
is built, so it is a 1.x addition that changes no program.

## 1.x or 2.0

The [versioning policy](../COMPATIBILITY.md#versioning-policy) lets a 1.x
minor add "new syntax, schemas, operations, optional fields, CLI flags or
exports" so long as anything that worked before works the same, with the same
output bytes (`docs/COMPATIBILITY.md: L152–155`).

| Change | Kind | Release |
| --- | --- | --- |
| PyPI, and a promise for the Python API | a new channel; a promise added | 1.x |
| Python `explain`, `dependents`, `what_if`, host, handoff | new functions in the Python package | 1.x |
| `send(…, { agent })`, `scope`, `waitingRequest(series)` and `dependents(name)` on the starter host, and an opt-in event log with its lock | an optional argument (an added overload in the TypeScript declarations), new methods, a new `openHost` option, new optional store members and a new checkpoint schema for a store with a log | 1.x |
| The starter's checkpoint and file store, opened from both languages | their files named as promised schemas, checked by opening each language's store in the other | 1.x |
| An `agent` on the MCP tools' events | an optional field | 1.x |
| `caveat serve --http`, `caveat types --python` | new CLI flags | 1.x |
| `caveat-lang/run`, `caveat_lang.run`, `caveat run`, `python -m caveat_lang run` | new exports and new commands, promised from their first release | 1.x |
| A window for decision series, or Requests, and identifiers that can depart | new syntax, with any new codes for the new syntax only | 1.x |
| A replay save that keeps agents | a new schema behind a new save option; `save()` and `save({ compact: true })` unchanged | 1.x |
| Boxed WebAssembly tasks | new options, a new interface and new CLI flags | 1.x |
| The Caveat task language | new syntax in its own files, with its own specification | 1.x |

Two changes are sometimes taken for 2.0 work. This plan makes neither:

- **A rule that calls out of the program.** It would be new syntax, which a
  1.x minor may add, but a program using it could replay to a different
  record, since restoring sends each event through the same path again
  (`spec/caveat-replay-save-0.1.md: L66–69`), and running it would execute
  host code, which the threat model rules out
  (`docs/THREAT_MODEL.md: L20–21`). This is the wall the owner asked for, so
  this plan does not add it, in 1.x or 2.0, unless the owner reopens that
  choice.
- **Requiring a sender on every event.** Today an event no agent lists may be
  sent with no agent (`spec/caveat-agents-0.1.md: L68–70`); requiring one would
  refuse events 1.x accepts. The runner does not need it: each task sends
  through a scope of its own agent's events, whatever the default.

So nothing joins the [2.0 candidates](../2.0-CANDIDATES.md); the list records
that this was looked at.

## Pull requests, in order

Each merges on its own. PRs 1 to 5 come first; 6 to 8 can land while 9 is
being built, and 12 is specified then too, before 9 merges, so the runner's
promised API fits the request form it picks. Step 3 starts once both runners
(9 and 10) have merged, and step 4 once the JavaScript runner runs boxed
tasks (14).

Each way of writing a task is promised from the first release that ships it,
so its examples come first, while its draft can still change: no release
ships the runner before 11 and 12 have merged, boxed tasks before 16 has, or
the task language before 19 has. CI's security gate holds every new packed
file to the declared capabilities
([package security](../PACKAGE_SECURITY.md#declared-capabilities)), so the
PRs below do not list them.

**This note**

0. This design note, the 2.0 list's record of it, and the owner's answer in
   the ledger: `approve` events with new card codes, 1501 for Python and
   JavaScript first, then 1503 for WebAssembly from any language and 1502 for
   a Caveat task language inside it, added to the ledger README's code table.

**Step 1: reach**

1. **PyPI.** A publish workflow for the wheels `python.yml` already builds,
   gated as the npm publish is on a tested `main` revision, or, for a
   post-release, on a tested run of its branch off the release's tag. With it:
   - the version rule and its check;
   - the compatibility row for the Python API, naming the wheels held to the
     npm kit's bytes, checked by a step every pull request runs;
   - the threat model reviewed as a whole for the PyPI package. It says that
     the sandbox, and the fresh instance after memory exhaustion, hold for the
     WebAssembly runtime only: the Python runtime is native code in the host's
     process, and a failed allocation there ends the process;
   - before each PyPI publication, the security gate audits
     `python/Cargo.lock` with RustSec and scans the wheels' Python files for
     declared capabilities;
   - PyPI in the release gate and the distribution closeout
     (`scripts/check-distribution.mjs`), and a ledger event for a PyPI
     publication.

   Owner steps: create the PyPI project with a trusted publisher that names
   this workflow and a `pypi-publish` environment; create that environment
   with the owner as its required reviewer and deployments limited to `v*`
   tags, as `npm-publish` is (`docs/CONSOLIDATION_PLAN.md: L476–478`); and
   approve each publish. The release thread may own this one.
2. **Python: `explain`, `dependents` and `what_if`,** held to the kit's bytes
   by the differential test.
3. **Starter: `send(…, { agent })`, `scope(sends, { agent })`,
   `waitingRequest` and `dependents(name)`,** and the opt-in event log with
   its lock, its checkpoint schema and its open check, a test that a host
   without the log refuses a store with one, and the threat model review a
   new input needs.
   The log's format is settled and reviewed here, because this PR ships it,
   and the starter guide's "What the stores guarantee" says what `durable`
   means with the log. `scope` sends through the host's queue and checkpoint,
   which `scopeSession` on a bare session cannot. `TypedHost.send`,
   `TypedSession.dispatch` and `dispatchView` keep their 1.4 signatures and
   gain an overload that takes the options. Each new overload comes before
   the 1.4 signature, because `Parameters<…>` and `ReturnType<…>` read the
   last one, and the type tests gain a 1.4-style caller that wraps each
   changed method through `Parameters<…>` and must still compile.
4. **MCP: an optional `agent` on each event** of `caveat_explain`,
   `caveat_dependents`, `caveat_dispatch` and `caveat_find`.
5. **Python host and handoff:** `open_host` with a file store the Node store
   can open and the reverse, the event log and the same lock as the Node
   host, `permits`, `waiting_request`, `assessment`, `history`, `dependents`,
   `scope` and `replay_onto`, and a test that a Python host and a Node host
   each refuse a store the other holds. With the threat model review that a
   new filesystem capability in the Python package needs.
6. **`caveat serve --http`,** specified first as an added section of Serve
   0.1: one request per POST, what becomes of the ready line, how `close` and
   a fatal error end the listener, the token and request checks, and a server
   started without `--http` answering as before, byte for byte. With the
   threat model and capability updates.
7. **`caveat types --python`.**
8. **The guide,** "Use Caveat from your program", linked from the README,
   `AGENTS.md`, `llms.txt` and `python/README.md`, the PyPI page.

**Step 2: the general layer**

9. **The runner in JavaScript,** `caveat-lang/run` and `caveat run`, with the
   wall's tests (rules 1, 2, 3, 5, 6 and 7), crash-and-restart tests like the
   starter's, a test that a second runner, or a starter host opened with the
   log, does not open a store a running runner holds and that a runner killed
   while it holds the lock opens its store again, the wording changes above,
   and the threat model review. Its contract (the API, the request object, the
   runner's own file and `caveat run`'s exit statuses) is written down and
   reviewed before it merges, because every export is a promise from its first
   release.
10. **The runner in Python,** held to the JavaScript runner by one shared
    fixture: the same program, the same scripted task answers and keys, the
    same record bytes and event logs. With the threat model review that
    running Python task modules needs.
11. **Three examples,** each with scenarios and a README that lists what the
    language made its author work around: the refund agent above, with
    scenarios for a time-out followed by a late receipt, a restart while a
    started request is no longer waiting, and a payload that names another
    request; a pipeline that reruns only the steps a bad input reached,
    because each step's decision rests on its inputs; and an agent that checks
    claims with Claude as a task, from `python/examples/claim_checker.py`,
    with no tools that read files or run code.
12. **A way past limit 5:** a window for decision series or Requests,
    specified before 9 merges and shipped no later than the runner, with a way
    for identifiers to depart or the runner's guide saying how many requests a
    ledger can answer. If 11 shows the need, the rest of Requests, and a
    replay save that keeps agents.

**Step 3: tasks in any language, boxed in**

13. **A design note for boxed tasks,** settling the four questions in step 3,
    with the threat model's new promise.
14. **The JavaScript runner runs WebAssembly tasks,** with the box's tests: a
    component that imports something outside WASI and the task world does not
    open; a grant that reaches the store is refused, and so is one that
    reaches a task module, the program or a package; a task granted one web
    address cannot reach another or read the runner's environment; a task
    past its time, in a loop or in a web call that never answers, is stopped
    and tried again, and events sent meanwhile are accepted; a task cannot
    report outside its agent's events or for another request. With the
    threat model review, and the contract review and gate coverage, that a
    new dependency or a new shipped component needs.
15. **The Python runner runs them,** held to the JavaScript runner by the
    shared fixture and the same tests, with the threat model review, and the
    contract review and gate coverage, that a new dependency or a new shipped
    component needs.
16. **The refund task compiled from two languages,** each run by both runners
    to the same record bytes.

**Step 4: a Caveat task language**

17. **Its design note, then its draft specification.**
18. **The interpreter,** a component that runs in the box, and a check of
    task files against their program, with the threat model review that task
    files, a new input, need, and the review and gate coverage a new shipped
    component needs. When the person asks for it, the runner keeps each boxed
    try's inputs, from its abilities, `waiting` and `report`, in a file beside
    the store, with the threat model review that this file needs.
19. **The same tasks in typed Python, TypeScript and the Caveat task
    language,** typed with `caveat types --python` and `caveat types`, each
    also as a boxed component (16). Measured: mistakes caught before running,
    from a list of seeded mistakes; what each author had to work around;
    length; and time per task. Each strength gets a test: a task that uses an
    ability its file does not list does not load; a report carries the inputs
    its value came from; a rerun from kept inputs gives the same reports, byte
    for byte; and agents write the tasks from the one-page reference, with the
    share that pass first time counted. The guide recommends the Caveat task
    language only where this shows it does better.

## The owner's choice

Asked: which language are tasks written in? The options were (1) Python and
JavaScript, run by Caveat's runner; (2) a new Caveat language for tasks; and
(3) WebAssembly modules from any language, boxed in.

Decided by the owner on 2026-10-31: option 1 first, then option 3, with a
Caveat language for tasks that runs inside option 3 as a strong choice, never
a requirement. Steps 2, 3 and 4 follow that order. The wall and the request
protocol are the same under each, so each step adds a way to write a task and
none replaces another. Each way is promised for all of 1.x from the first
release that ships it, which comes after its examples (11 for the runner, 16
for boxed tasks, 19 for the task language); that is also why each step's API
is written down and reviewed before it merges.

## Precedents

Read on 2026-10-31 and 2026-11-01. The credibility score is how far the
source can be relied on for the quoted point (0 to 10).

- **Temporal, durable workflows** (vendor's official documentation; 8).
  "A Workflow Definition must be deterministic to support replay";
  Activities, where API calls go, "execute outside the replay path"; on
  replay the Worker "uses the results stored in the `ActivityTaskCompleted`
  Events", so "Activities don't run again"
  ([workflow definition](https://docs.temporal.io/workflow-definition),
  [event history](https://docs.temporal.io/encyclopedia/event-history/event-history-typescript)).
  Rule 5 is this rule.
- **Elm ports** (the language's official guide; 8). "Ports are about creating
  strong boundaries!", and "The compiler does not know what goes on in
  JavaScript" ([ports](https://guide.elm-lang.org/interop/ports.html)).
  Rules 1 and 3: one narrow, declared channel each way.
- **Deno permissions** (the runtime's official documentation; 8). "Unless
  you specifically enable it, a program run with Deno has no access to
  sensitive APIs", and "Code can not escalate its privileges without user
  consent" ([security](https://docs.deno.com/runtime/fundamentals/security/)).
  Rule 3: access comes from the person running the code. Step 2's runner
  does not box task code in; step 3's box grants abilities as Deno does.
- **WASI and the WebAssembly component model** (the standard's official
  sites; 9). WASI 0.2 is "officially based on the Wasm component model", and
  "All I/O in Wasm goes through its imports and exports"
  ([WASI 0.2 launch](https://bytecodealliance.org/articles/WASI-0.2)); WASI
  0.3 shipped on 11 June 2026 ([roadmap](https://wasi.dev/roadmap)). Step 3's
  box is this rule. Python support is from the `wasmtime` package's PyPI page
  (the project's own release page; 8).
- **Python's version rules** (the Python Packaging Authority's
  specification; 9). Some projects use post-releases "to address minor errors
  in a final release that do not affect the distributed software", and "The
  use of post-releases to publish maintenance releases containing actual bug
  fixes is strongly discouraged"; a request for a version greater than 1.5.0
  "MUST NOT allow a post-release of the given version"
  ([version specifiers](https://packaging.python.org/en/latest/specifications/version-specifiers/)).
  The PyPI version rule follows them.
- **In this repository** (10). The [Lean bridge](lean-bridge.md) turns a
  host-side check into an ordinary event, "so nothing in the 1.x promise
  moves" (`docs/design/lean-bridge.md: L132–135`), and the
  [agent-evidence example](../../kit/examples/agent-evidence/README.md) drives a
  program from Python. The general layer makes that pattern the runner's job.

## What this note does not do

- It authenticates no result and no sender, as today
  (`docs/THREAT_MODEL.md: L25–28`).
- It does not box in Python or JavaScript task code
  ([who the wall holds against](#who-the-wall-holds-against)); step 3 boxes
  WebAssembly tasks.
- It lets no tool other than a runner run a task, boxed or not: not
  `caveat test`, not `explain`, not the MCP tools, not the hosted MCP server.
- It does not design the box or the Caveat task language; steps 3 and 4 each
  start with their own design note.
- It proposes no hosted ledger service. One would store people's records and
  needs its own decision.
- It does not publish the Rust crate, whose API stays outside the promise
  (`docs/COMPATIBILITY.md: L89`).
- It changes the output of no program that exists.

## Changes

- 2026-10-31: proposed.
- 2026-10-31: the owner chose the task languages: Python and JavaScript first,
  then WebAssembly tasks from any language, with a Caveat task language in
  the same box (steps 3 and 4).
- 2026-10-31: revised after an independent review. The runner records each
  start in the program's record and never answers for a task, so a time-out
  is no longer recorded as a failure; an answer is bound to its request; the
  store keeps an event log; the example refuses a repeated or withdrawn
  approval; a fifth limit, a ledger answers a fixed number of requests;
  `serve --http` needs a token; MCP events take an agent; and the wall says
  who it holds against.
- 2026-11-01: revised after a second independent review. A store with a log
  has one writer, and the log is opt-in, holds only what `caveat explain`
  reads, and is checked against the checkpoint; a start names its revision,
  and the program refuses it unless that revision is the current one; each
  task has an agent of its own; tries count across restarts, and `lookup`
  comes before a stop; "waiting" names a request that is committed, not
  reopened and not resolved, which Caveat's "in force" does not; the example
  calls support's ask a ticket, shows a stopped refund as needing a person,
  and names the order a fraud flag is about; the box links standard
  interfaces with nothing behind them, refuses grants that reach code the
  runner loads, and runs each try off the runner's thread; no release ships a
  way of writing tasks before its examples; and the PyPI package gets its own
  threat model review, audit and publish environment.
- 2026-11-01: corrected after a final check. A host without the log refuses
  a store with one, by the checkpoint's schema name, and Python and Node take
  the same lock; `caveat run` sends its input as `runner.send` does and takes
  no restore; the runner notes, times and calls `lookup` once, sends the stop
  whenever a request is still unanswered after it, remembers refused starts
  across restarts, and writes a failed append again; a late stop for an
  older request no longer hides the current one's; and the version check
  allows a post-release's suffix.
