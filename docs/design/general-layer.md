# Design note: general work around a checked record

Status: proposed 2026-10-31, for the owner's plan of that day: make Caveat
usable for general work while keeping its checked record intact. This note adds
no syntax and changes no specification, promise or output. Each step below is
its own pull request, and a step that changes what a program means goes as a
specification PR first. Receipts are `path: Lx–Ly` at `main` `36928e8`.

## In short

- **Step 1, reach.** Any program can call today's Caveat as its "why" layer.
  JavaScript and MCP hosts already can. Python needs its package on PyPI and
  the parts it still lacks; other languages get the same operations over
  local HTTP.
- **Step 2, a general layer.** Ordinary code (loops, math, files, network
  calls) runs beside a Caveat program as *tasks*. A task can read the record
  and send the events the program lets it send. Nothing else it does reaches
  the record.
- **The first general layer needs no language change.** A decision the program
  commits is the request; the task's answer is the outcome recorded on that
  decision; [Agents 0.1](../../spec/caveat-agents-0.1.md) says which task may
  answer. A program built this way runs on `caveat-lang@1.4.1` (see
  [A worked example](#a-worked-example)).
- **Nothing here needs 2.0.** Every step adds to 1.x. The one change that would
  need 2.0, a rule that calls code outside the program, is what the wall
  forbids in every version.
- **One choice is the owner's:** the language tasks are written in
  ([The owner's choice](#the-owners-choice)).

## The two layers

| | Checked core | General layer |
| --- | --- | --- |
| What it is | A Caveat program, in today's language, unchanged | Tasks: ordinary functions in a host language |
| What it may do | Change the record, one accepted event at a time | Anything its process may do: loops, math, files, network |
| What it may not do | Call out: the language has no foreign calls and no file, process, host clock or network access (`docs/THREAT_MODEL.md: L45–50`) | Change the record, except by sending a declared event |
| How it is checked | Load checks, `check`, scenarios and explanations; every event is accepted or refused whole | Each event it sends is checked by the program's rules; its own code is its author's |
| On replay | Rebuilt from the source and the events | Not run again |

Caveat is already "meant for the decisions an application records, not for
inner loops" (`docs/LIMITS.md: L47–49`). The general layer is where the loops
go.

## The wall

These rules hold in both steps and for every task language. Each says what
enforces it; the PR that builds it adds the test named.

1. **Only an accepted event changes the record.** A task holds no session,
   save, store or archive. It holds a `report` function that sends the events
   the program lets its agent send, as that agent, and nothing else.
   *Enforced by* the runner, which builds `report` on
   `scopeSession(session, sends, { agent })` (`kit/lib/handoff.mjs: L122–126`)
   and hands the task only `report` and a read-only view; and by the runtime,
   which refuses an event from an agent the program does not let send it as
   `input/wrong_agent` (`spec/caveat-agents-0.1.md: L60–70`).
   *Test:* a task that reports an event outside its list is refused and the
   saved record is byte for byte unchanged; a test lists what a task receives
   and finds no session, host, store or save.
2. **A result is a claim by the task that produced it.** It enters as an event
   payload, and the program's rules decide what evidence it becomes and which
   caveats it carries. The record names the agent that sent it and the request
   it answers. It vouches for neither: Caveat records what a host tells it and
   authenticates no sender (`docs/THREAT_MODEL.md: L25–28`,
   `spec/caveat-agents-0.1.md: L100–104`).
3. **The program never calls out.** No rule can call a task, host code or the
   outside world. Loading, checking, testing, explaining, replaying and
   restoring a program run no task. Only a runner runs tasks: the ones the
   person running it names, with that process's access. This keeps the
   threat model's promise that running a program, an event or a save
   "must not execute host code, touch files, start processes or reach the
   network" (`docs/THREAT_MODEL.md: L20–21`) for every tool, including the
   hosted MCP server, which must never fetch a URL because a program asked.
   *Enforced by* the language, which has no call-out form (unchanged), and by
   keeping task code in host files the runtime never reads.
   *Test:* `caveat test`, `explain` and the MCP tools run a program whose
   runner has tasks, and no task runs.
4. **Reading is open and changes nothing.** A task may read the view or a copy
   of the snapshot, as any host can. Reading spends no attention, observes no
   evidence and writes nothing. What a task learns by reading and then reports
   enters as that task's claim (rule 2).
5. **Replay never runs the general layer again.** The record is rebuilt from
   the program's source and its events alone. Results are already in the
   events, so a save, `explain`, `carrySave` or a handoff replay gives the same
   record without running any task, and a task that would answer differently
   today changes nothing already recorded.
   *Test:* replaying a runner's event log through `caveat explain`, with no
   tasks loaded, gives the record the runner holds.
6. **One answer per request, and every started request ends.** A request is a
   committed revision such as `refund@2`. The program records its answer with
   `resolve`, and a second answer is refused as `evaluation/already_resolved`
   (`spec/caveat-outcomes-0.1.md: L139–146`). `resolve revision(SERIES, K)`
   lands an answer on the request it answers even after a newer one exists
   (`spec/caveat-outcomes-0.1.md: L117–119`). For a request it started, the
   runner sends either the task's answer or the task's failure event, so none
   disappears. A request it has not started stays open and visible in the
   view. After a crash a task may run twice for one request, so each request
   comes with a stable key the outside service can use to do the work once.
7. **Work runs only for a request in force.** The runner starts a task only for
   a revision that is committed, not reopened and not resolved in the current
   view, and checks again just before it starts. Once the revision reopens,
   the runner starts nothing for it. An answer that still arrives is recorded
   on its request (rule 6), so the record says the money moved even though
   doubt came first. Work that cannot be undone should be asked for by an
   `irreversible strict` series, which commits only when it retains each
   `material` or worse caveat its grounds, guard or grant carry
   ([Strict Irreversible 0.1](../../spec/caveat-strict-irreversible-0.1.md));
   such a series holds one revision (limit 4 under
   [What the first version cannot do](#what-the-first-version-cannot-do)).
8. **The core keeps its limits; loops live outside.** Every event stays within
   the work, history and text limits. Long computation runs in tasks, under the
   runner's own timeouts. A result too large for one event is split across
   events or refused.

The wall guards the record, not the computer. Python and JavaScript task code
runs with the access of the process that runs it. Someone who needs task code
boxed in runs the runner under operating-system limits, such as a container
or a separate user, or picks option 2 or 3 in
[The owner's choice](#the-owners-choice).

## Step 1: reach

### What exists

| Surface | For | Where | What is missing |
| --- | --- | --- | --- |
| JavaScript library: sessions, the starter host, handoff, explain, scenarios, saves | Node 20 and later, browsers | `kit/package.json` exports | The starter sends without an agent (`kit/lib/starter.mjs: L126`) |
| `caveat serve`: JSON lines on standard input and output | Any language that can start a process | `spec/caveat-serve-0.1.md: L5–9` | A client in each language; Python's is an example, `kit/examples/agent-evidence/caller.py` |
| MCP tools, local and hosted | Agents in MCP hosts | `kit/docs/MCP.md: L57–65` | Nothing for reach; `caveat_dispatch` keeps one ledger across calls from 1.5 |
| The Python package `caveat_lang`: the same runtime in-process, the same bytes as the npm kit | Python 3.9 and later | `python/README.md: L36–44`, `L150–156` | Not on PyPI. No explain, dependents, what-if, scenario runner, persisting host or handoff |

### What step 1 adds

- **Python on PyPI.** `pip install caveat-lang`. The package takes the npm
  package's version from the release that first ships it, because both carry
  the same runtime and are held to the same bytes, and
  [the compatibility commitment](../COMPATIBILITY.md) gains a row for its API.
  Today that commitment covers the npm package only
  (`docs/COMPATIBILITY.md: L7`).
- **Python parity.** `explain`, `dependents` and `what_if` in
  Python, held to the kit's output by the existing differential test, which
  already replays every scenario file through both
  (`python/README.md: L150–156`).
- **One small API, the same in each language.** It is the starter host's,
  plus two additions:

  | Call | JavaScript (`caveat-lang/starter`) | Python (new) | Any language (`caveat serve --http`, new) |
  | --- | --- | --- | --- |
  | Open a ledger kept on disk | `openHost({ runtime, source, store })` | `open_host(source, store=DIR)` | `--state FILE` |
  | Send an event, as an agent | `send(event, payload, { agent })`, the option new | `send(event, payload, agent=…)` | `{"op":"dispatch", …, "agent": …}` |
  | Is a decision in force? | `permits(decision)` | `permits(decision)` | `view` |
  | What does it rest on? | `assessment(decision)`, `history(decision)` | the same | `explain` |
  | What rests on this evidence? | `dependents(name)`, new | `dependents(name)` | `dependents` |

  A Python store keeps the same checkpoint and archive files as the Node file
  store, so a ledger written by one opens in the other.
- **Local HTTP.** `caveat serve --http PORT` carries Serve 0.1's requests and
  responses, one per POST, so Go, Java, C#, Ruby or a shell script can call
  Caveat without a stdio client. It listens on the loopback address only. It
  adds a network listener, so its PR updates the
  [threat model](../THREAT_MODEL.md) and the kit's declared capabilities
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
| Its arguments | The revision's value, and the notes on the readings it rests on | [Reading Notes 0.1](../../spec/caveat-reading-notes-0.1.md) |
| Who may answer | `agent payments sends refunded, refund_failed;` | [Agents 0.1](../../spec/caveat-agents-0.1.md) |
| The answer | `resolve revision(refund, N) = 1 because latest(receipts)` | [Outcomes 0.1](../../spec/caveat-outcomes-0.1.md) |
| Only one answer | A second `resolve` is refused as `already_resolved` | Outcomes 0.1, "When a resolve is refused" |
| Permission to act | `permitted by latest(approvals) for pending` | [Permission 0.1](../../spec/caveat-permission-0.1.md) |
| Cancelling | Reopening the revision; the runner then starts nothing for it | [Reactive 0.5](../../spec/caveat-reactive-0.5.md) |
| Work that cannot be undone | `decisions … irreversible strict` | [Strict Irreversible 0.1](../../spec/caveat-strict-irreversible-0.1.md) |

### A worked example

A refund agent. Support asks, a manager approves, a payments task pays, and a
fraud check can raise doubt at any point. The program decides; the task only
answers.

```caveat
claim refund_due;
evidence ticket from "the support ticket";
evidence manager from "a manager's go-ahead";
evidence provider from "the payment provider";
evidence provider_error from "the payment provider's error";
evidence fraud_check from "the fraud team";
readings requests from ticket limit 16;
readings approvals from manager limit 16;
readings receipts from provider limit 16;
readings failures from provider_error limit 16;
decisions refund limit 16;
identifiers limit 256;
state pending = 0;

agent support sends request_refund;
agent manager sends approve;
agent payments sends refunded, refund_failed;
agent fraud sends flag_fraud;

event request_refund order id, amount min 0 max 10000;
event approve order id;
event refunded revision min 1 max 16, receipt id;
event refund_failed revision min 1 max 16;
event flag_fraud;

on request_refund when committed(refund) and not reopened(refund) and not resolved(refund)
    reject "A refund is still being paid.";
on request_refund sample requests = amount supports refund_due noting order;
on request_refund set pending = order;
on approve sample approvals = order;
on approve when committed(refund) and not reopened(refund)
    reopen refund because latest(approvals);
on approve commit refund because enough using latest(requests)
    permitted by latest(approvals) for pending;

# A task's answer names the request it answers: refund@N.
on refunded sample receipts = revision noting receipt;
on refunded resolve revision(refund, revision) = 1 because latest(receipts);
on refund_failed sample failures = 1;
on refund_failed resolve revision(refund, revision) = 0 because latest(failures);

on flag_fraud reveal fraud_check opposes refund_due;
on flag_fraud withdraw latest(requests) because fraud_check;
on flag_fraud when committed(refund) and not reopened(refund) and rests_on_withdrawn(refund)
    reopen refund because fraud_check;

bind refund.status = if(not committed(refund), "none",
    if(reopened(refund), "reopened", if(resolved(refund), "paid or failed", "asked")));
```

On `caveat-lang@1.4.1` from npm, on 2026-10-31, the program loads, `check`
reports no warnings, and two scenarios pass:

| Sent | As | Answer |
| --- | --- | --- |
| `refunded` for revision 1 | `support` | refused, `input/wrong_agent` |
| `refunded` for revision 2, which does not exist | `payments` | refused, `evaluation/not_committed` |
| `refunded` for revision 1 | `payments` | accepted; `refund.status` is `paid or failed` |
| `refunded` for revision 1 again | `payments` | refused, `evaluation/already_resolved` |
| `refunded` after `flag_fraud` reopened the request | `payments` | accepted, and recorded on the request it answers |

For the last case, `caveat explain` prints the whole chain, each step with its
sender:

```text
    refund@1 = 900  reopened
      based on requests@1
      requests@1: observed at #1 request_refund by support {"order":"B-2","amount":900}
      requests@1 has since been withdrawn at #3 because fraud_check
      permitted by approvals@1 for 1
      could also have been influenced by approvals@1
      #2 approve by manager: committed because requests@1
      #3 flag_fraud by fraud: reopened because fraud_check
      #4 refunded by payments: resolved = 1 because receipts@1
```

The example becomes a tracked program, with its scenarios, in the examples
PR (10 below). It is not tracked yet, so the corpus sweeps do not change with
this note.

### What the first version cannot do

Running the example, and reading the specifications it uses, found four limits:

1. **One request at a time per series.** Only the current revision can be in
   force, so the program refuses a second refund while one is unpaid. A
   second request sent before the first was approved replaced `pending`, and
   approving the first was then refused as
   `policy/not_permitted` ("approvals@1 permits 1, not 2"). Asking again after
   an answer means reopening the answered revision, which the journal records
   as doubt: `#5 approve by manager: reopened because approvals@2`.
2. **One number per request.** A request's arguments are its revision's value
   and the notes on what it rests on; the task finds the order number in the
   note on `requests@N`.
3. **Larger checkpoints.** Once an event names an agent, the session keeps no
   replay log, so its compact save is the full state save
   (`spec/caveat-agents-0.1.md: L137–140`). The starter checkpoints compact
   saves where it can (`kit/lib/starter.mjs: L38`), so a runner's checkpoints
   grow with the record.
4. **Irreversible work is one request per series.** An irreversible series
   holds one revision (`spec/caveat-foreclosure-0.1.md: L82`). A program that
   pays many refunds it cannot take back needs a series per refund, which
   today means a routed `for` block over members fixed in the source. The
   example uses a plain series for that reason.

None of these breaks the wall. Limits 1, 2 and 4 are what a later Requests
profile would answer, if the examples show they matter; limit 3 needs a replay
save that keeps agents, a new schema beside the old ones.

### The runner

The runner is the one place tasks run. It is built on the starter host, so it
persists and restarts as the starter does ([starter guide](../../kit/docs/STARTER.md)).
A sketch of the JavaScript form, `caveat-lang/run`; names are settled in its PR:

```js
const runner = await openRunner({
  runtime, source, store: await fileStore('refund-store'),
  tasks: [{
    agent: 'payments',              // the program says: agent payments sends refunded, refund_failed;
    answers: 'refund',              // a decision series of the program
    failed: request => ['refund_failed', { revision: request.revision }],
    async perform(request, report) {
      // request: { id: 'refund@2', revision: 2, key, value, grounds }, and a read-only view
      const receipt = await payments.refund({ amount: request.value, idempotencyKey: request.key });
      return report('refunded', { revision: request.revision, receipt: receipt.id });
    },
  }],
});
await runner.send('request_refund', { order: 'A-18', amount: 15 }, { agent: 'support' });
```

The Python form, `caveat_lang.run`, reads the same:

```python
runner = caveat.open_runner(source, store="refund-store")

@runner.task(agent="payments", answers="refund",
             failed=lambda request: ("refund_failed", {"revision": request.revision}))
def pay(request, report):
    receipt = payments.refund(amount=request.value, idempotency_key=request.key)
    report("refunded", {"revision": request.revision, "receipt": receipt.id})
```

From the command line, `caveat run PROGRAM --tasks MODULE --store DIR` runs
JavaScript tasks, and `python -m caveat_lang run` does the same for Python.

What the runner does:

1. It opens the program from the store, or fresh when the store is empty.
2. It checks each task against the program's
   [interface](../../spec/caveat-interface-0.1.md): the agent is declared, the
   answer and failure events are in that agent's `sends`, and `answers` names a
   decision series. It refuses to start otherwise.
3. After each accepted event, whether sent by the application or by a task, it
   reads the view. When a task's series has a current revision that is
   committed, not reopened and not resolved, and the runner has not started it,
   the runner notes the start in its own journal beside the store and calls the
   task with the request: its id, revision, key, value and grounds (each with
   its value and note), and a read-only view.
4. `report` sends through the host's single queue, one event at a time, as the
   task's agent and only within its `sends`, and returns the host's answers:
   `handled`, `accepted`, `durable` or the rejection.
5. When a task throws or runs out of time before an answer is accepted, the
   runner sends the task's failure event. If that is refused, the runner stops
   and says why; it does not retry in a loop.
6. After a restart, a started request with no answer is started again with the
   same key, so the outside service can do the work once (rule 6).

When the runner ships, the places that say Caveat "does not authorize actions"
(`docs/THREAT_MODEL.md: L25–28`, `kit/docs/STARTER.md: L19–21`, `AGENTS.md: L31–32`)
say what becomes true: the runtime still never acts, and the runner runs only
the tasks it is given, for decisions in force. The threat model gains task
modules as an input: the person's own code, run with the process's access, and
never loaded by any other command.

### Later, and only with evidence: Requests

If the examples hit limits 1, 2 or 4, a Requests profile would let a program hold
many open requests at once, some of them irreversible, give each named arguments, name the agent that
answers, accept answers with fields, and record a cancellation. Its acceptance
scenario: two refunds asked before either is paid, each answered once, and a
fraud flag on one cancels that one only. It is new syntax, so it is a 1.x
addition, specified before it is built.

### Later, and only if chosen: a Caveat work language

Option 2 in [The owner's choice](#the-owners-choice) starts with its own
design note. The wall and the runner's protocol stay as they are; a work
language would be one more way to write a task.

## 1.x or 2.0

The [versioning policy](../COMPATIBILITY.md#versioning-policy) lets a 1.x
minor add "new syntax, schemas, operations, optional fields, CLI flags or
exports" so long as anything that worked before works the same, with the same
output bytes (`docs/COMPATIBILITY.md: L151–154`).

| Change | Kind | Release |
| --- | --- | --- |
| PyPI, and a promise for the Python API | a new channel; a promise added | 1.x |
| Python `explain`, `dependents`, `what_if`, host, handoff | new functions in the Python package | 1.x |
| `send(…, { agent })` and `dependents(name)` on the starter host | an optional argument and a new method on an export | 1.x |
| `caveat serve --http`, `caveat types --python` | new CLI flags | 1.x |
| `caveat-lang/run`, `caveat_lang.run`, `caveat run` | a new export and a new command | 1.x |
| Requests, if built | new syntax, with any new codes for the new syntax only | 1.x |
| A replay save that keeps agents | a new schema beside the old ones | 1.x |
| A work language, if chosen | new syntax in its own files | 1.x |

What would need 2.0, and why this plan does neither:

- **A rule that calls out of the program.** What an accepted program computes
  would then depend on code outside its source and its events, so the same
  save could replay differently: "changing what an accepted program computes"
  is a 2.0 change (`docs/COMPATIBILITY.md: L155–157`). It would also break the
  host-process promise (`docs/THREAT_MODEL.md: L20–21`). This is the wall
  itself, so no version does it.
- **Requiring a sender on every event.** Today an event no agent lists may be
  sent with no agent (`spec/caveat-agents-0.1.md: L68–70`); requiring one would
  refuse events 1.x accepts. The runner does not need it: each task sends
  through a scope of its own agent's events, whatever the default.

So nothing joins the [2.0 candidates](../2.0-CANDIDATES.md); the list records
that this was looked at.

## Pull requests, in order

Each merges on its own. PRs 1 to 4 come first; 5 to 7 can land while 8 is
being built.

**This note**

0. This design note, and the 2.0 list's record of it. Docs only.

**Step 1: reach**

1. **PyPI.** A publish workflow for the wheels `python.yml` already builds,
   gated as the npm publish is on a tested `main` revision; the compatibility
   row for the Python API; the release channel checklist. Owner steps: create
   the PyPI project with a trusted publisher, and approve the first publish.
   The release thread may own this one.
2. **Python: `explain`, `dependents` and `what_if`,** held to the kit's bytes
   by the differential test.
3. **Starter: `send(…, { agent })` and `dependents(name)`.**
4. **Python host and handoff:** `open_host` with a file store the Node store
   can open and the reverse, `permits`, `assessment`, `history`,
   `dependents`, `scope` and `replay_onto`.
5. **`caveat serve --http`,** loopback only, with the threat model and
   capability updates.
6. **`caveat types --python`.**
7. **The guide,** "Use Caveat from your program", linked from the README,
   `AGENTS.md` and `llms.txt`.

**Step 2: the general layer**

8. **The runner in JavaScript,** `caveat-lang/run` and `caveat run`, with the
   wall's tests (rules 1, 3, 5 and 6), crash-and-restart tests like the
   starter's, and the wording changes above.
9. **The runner in Python,** held to the JavaScript runner by one shared
   fixture: the same program, the same scripted task answers, the same record
   bytes.
10. **Three examples,** each with scenarios and a README that lists what the
    language made its author work around: the refund agent above; a pipeline
    that reruns only the steps a bad input reached, because each step's
    decision rests on its inputs; and an agent that checks claims with Claude
    as a task, from `python/examples/claim_checker.py`.
11. **If 10 shows the need:** the Requests specification, then its runtime
    PRs; and a replay save that keeps agents, if checkpoints grow too large.
12. **If the owner picks option 2 or 3:** a design note for that task
    language.

## The owner's choice

Which language are tasks written in?

1. **Python and JavaScript, run by Caveat's runner (recommended).** People
   and agents already write both, and it ships in small steps on what Caveat
   has today. Task code runs with the computer's normal access, so the wall
   protects the record, not the machine.
2. **A new Caveat work language.** One language for the whole job, and it
   could limit what task code can touch. It is a large build, and agents would
   have one more language to learn.
3. **WebAssembly modules, from any language.** Task code is fully boxed in,
   but the setup is heavy for users and agents.

The wall and the request protocol are the same under each, so option 1 first
leaves 2 or 3 open for later.

## Precedents

Read on 2026-10-31. The credibility score is how far the source can be relied
on for the quoted point (0 to 10).

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
  Rule 3: access comes from the person running the code. Unlike Deno, the
  runner in option 1 does not box task code in.
- **In this repository** (10). The [Lean bridge](lean-bridge.md) turns a
  host-side check into an ordinary event, "so nothing in the 1.x promise
  moves" (`docs/design/lean-bridge.md: L132–135`), and the
  [agent-evidence example](../../kit/examples/agent-evidence/README.md) drives a
  program from Python. The general layer makes that pattern the runner's job.

## What this note does not do

- It authenticates no result and no sender, as today
  (`docs/THREAT_MODEL.md: L25–28`).
- It does not box in Python or JavaScript task code (see the end of
  [The wall](#the-wall)).
- It lets no tool other than a runner run a task: not `caveat test`, not
  `explain`, not the MCP tools, not the hosted MCP server.
- It proposes no hosted ledger service. One would store people's records and
  needs its own decision.
- It does not publish the Rust crate, whose API stays outside the promise
  (`docs/COMPATIBILITY.md: L89`).
- It changes the output of no program that exists.

## Changes

- 2026-10-31: proposed.
