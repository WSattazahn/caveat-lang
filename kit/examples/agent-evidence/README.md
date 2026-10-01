# Agent evidence: calling Caveat from an application

This example shows an application, such as an agent's harness, using a Caveat
program as the place its assessment is made and recorded. The application
sends evidence in, obtains an assessment, withdraws evidence found to be
wrong, and asks for a new assessment. It talks to the program through
`caveat serve`, from Python.

- `assessment.cav`: the program.
- `caller.py`: a small client for [Serve 0.1](../../docs/reference/spec/caveat-serve-0.1.md),
  standard library only, Python 3.9 or later.
- `test_caller.py`: tests that drive a real `caveat serve` process through the
  situations below.

For the language, read the [worked example](../../docs/WORKED_EXAMPLE.md) and
[Caveat on one page](../../docs/REFERENCE.md). This page does not repeat them.
Its links work where the package installs it,
`node_modules/caveat-lang/examples/agent-evidence/`. In a copy, find the same
documents under `node_modules/caveat-lang/`.

## Run it

In a project directory:

```sh
npm init -y
npm install caveat-lang@next
cp -r node_modules/caveat-lang/examples/agent-evidence .
cd agent-evidence
npx --no-install caveat validate assessment.cav
npx --no-install caveat check assessment.cav
python3 -B -m unittest -v test_caller
```

The example ships in the package from `0.1.0-rc.6`. With an earlier release
there is no `examples/` folder, so `cp` finds nothing: take this folder from
the repository instead,
[kit/examples/agent-evidence](https://github.com/WSattazahn/caveat-lang/tree/main/kit/examples/agent-evidence),
and put it in the project directory. Its files run, unchanged, on
`0.1.0-rc.5`.

`cp -r` is for a POSIX shell, such as Git Bash. In PowerShell, use
`Copy-Item -Recurse` instead. Use `python` if there is no `python3`. `-B`
keeps Python from writing bytecode into the folder.

The caller starts `npx --no-install caveat serve assessment.cav`, so it finds
the package installed in the project. npx adds a few seconds to each start.
To start `caveat` another way, set `CAVEAT_COMMAND` to a JSON array of words,
such as `["node", "../node_modules/caveat-lang/bin/caveat.mjs"]`, or pass
`command=` to `CaveatServer`. In JSON, write a Windows path with forward
slashes or doubled backslashes.

## The program

`assessment.cav` has four events:

- `observe confidence` records an observation. `confidence` must be in
  `0..100`; anything else is refused as `input/bound_exceeded`. From 70 up it
  supports the answer, below 70 it opposes it. An opposing observation
  reopens an approval in force
  ([reopening triggers](../../docs/reference/spec/caveat-reopening-triggers-0.1.md)).
- `assess` commits an approval that rests on the latest observation. The
  program refuses it, with its own `reject`, when there is no observation,
  when the latest one is withdrawn, when an approval is already in force, or
  when the latest observation is below 70.
- `retract` withdraws the latest observation, because of a recheck. This is
  the program's explicit withdrawal policy. The withdrawal keeps its record,
  and the approval keeps the grounds it was made on
  ([withdrawal](../../docs/reference/spec/caveat-withdrawal-0.1.md)). A
  withdrawal reopens nothing by itself; a later rule reopens an approval that
  rests on the withdrawn observation. The program refuses `retract` when
  there is no observation, or when the latest one is already withdrawn.
- `erase` is forbidden. The program always refuses it.

The displayed value `assessment.verdict` is `none`, `approved` or `reopened`.

A session keeps at most 16 observations and 8 assessments. After that,
`observe` or `assess` is refused with origin `limit` and code
`history_limit`, and every later attempt in the session fails. To go on, the
application starts a new server.

## Three levels of an answer

Every dispatch answers three separate questions. Read them in this order, and
do not take one for another.

1. **Was the request handled?** `ok` is true, or false with an error `kind`.
   Kind `request` means the line was not a valid request: nothing changed, and
   the server keeps serving. Any other kind, usually `fatal`, means the
   session cannot be used, and the server exits with status 1. A program that
   does not load is reported by the first line, with `ready: false`, and the
   server exits with status 2.
2. **Was the event accepted?** `outcome` is `accepted` or `rejected`, with
   `origin`, `code` and `message` when rejected
   ([dispatch outcomes](../../docs/reference/spec/caveat-dispatch-0.1.md)). A
   rejected event is a successful request, and the session is unchanged.
3. **Does the application's decision permit what it intends?** Only the
   current session says. Read it after the attempt, from the snapshot, or from
   an `explain` or `dependents` report.

This response is valid, and the operation it answers did not succeed:

```json
{"id":2,"ok":true,"outcome":"rejected","origin":"input","code":"bound_exceeded","message":"confidence must be finite and in 0..100","sequence":1}
```

`caller.py` raises `CaveatRequestError` for level 1 kind `request`, including
a line the server could not read, which it answers with an `id` of null.
It raises `CaveatSessionFailed` for any other kind, and `CaveatLoadError` for
a program that does not load. A dispatch returns a `Dispatch` whose `accepted` answers
level 2. An outcome or origin the contract does not name raises
`CaveatProtocolError`; it is never read as a rejection.

When the caller gives up on a server, because it wrote nothing in time,
wrote a line Serve 0.1 does not allow, stopped reading requests, did not
exit after `close` or a failed session, or a request was interrupted before
its response was read, it stops every process the launch
started, not only the first: npx runs `caveat` under a shell. On POSIX that
is the launch's own process group; on Windows, the process tree, with
`taskkill /T`, while the first process runs. If that fails, it stops the
first process, and the error it raises says that processes started under it
may still be running. A server that closes correctly exits by itself.

## The caller's rule

A workflow attempt succeeds only when its required operations succeed and its relevant, current CAVEAT assessment permits the intended result. A stored earlier assessment cannot substitute for a rejected required operation.

That is the summary. In full: an attempt succeeds only when every required
operation has explicitly recorded accepted completion, no required operation
has failed, and the relevant assessment permits success. An earlier approval
cannot cover an incomplete operation.

`Attempt` carries it out:

- `begin(event, payload)` registers a required operation and returns an
  `Operation`, whose `send()` dispatches it. `require(event, payload)` does
  both, for plain code. An operation counts only once its acceptance is
  recorded; success is never assumed because no failure was recorded.
- Once a required operation is rejected, or raises anything, an interruption
  or a cancelled task included, the attempt has failed and sends nothing more.
  What `send()` raised goes on to the caller.
- `finish(permits)` decides, once, and returns a frozen `AttemptResult`;
  calling it again returns the same one. It reports each operation as
  `accepted`, `rejected`, `unconfirmed` or `not sent`. When every operation
  sent has an answer, it reads the current snapshot and asks the
  application's own test whether it permits the result. Here that test is
  `assessment_permits`: the verdict is `approved`, and the approval in force
  was committed during this attempt.
- After `finish`, nothing more is registered or sent in the attempt: a
  `send()` that had not started raises `AttemptFinished` without contacting
  the server. A response that arrives later is recorded on its `Operation`,
  and does not change the result. A retry is a new `Attempt`, started once
  nothing of the finished one is still sending (see below).

`assess_answer` is this example's attempt: it names the required operations,
`observe` then `assess`.

**Unconfirmed is not a rejection.** It means `send()` raised, or was still
under way when the attempt finished: the server may or may not have applied
the operation. A `CaveatRequestError`, for one, says it did not; the status is
still unconfirmed, with the error kept beside it. Either way the attempt did
not succeed. While an operation is registered or still sending, a worker
may own the pipe; after an unconfirmed one, what the session holds is not
known. In both cases `finish` does not talk to the server: the result has
`permits` false and no snapshot.

A rejected event rolls back everything it did, so an earlier approval stays
visible after it. That is how the protocol works. The rule, not the session,
decides that the attempt failed.

| Situation | Conclusion |
| --- | --- |
| A required operation is rejected after an earlier approval | This attempt did not succeed, even if the old approval remains visible. |
| Evidence supporting an approval is withdrawn | Apply the program's explicit withdrawal/reopening policy; do not assume the old approval remains usable. |
| A test deliberately requests a forbidden action and gets the expected refusal | The TEST passed; the requested ACTION did not succeed. |
| A later, separate attempt receives valid replacement evidence and an accepted reassessment | Evaluate that new attempt; do not permanently mark every future attempt failed because an earlier one failed. |
| A required operation is interrupted after an earlier approval | It is unconfirmed, not rejected. This attempt did not succeed; the old approval does not cover it. |
| The attempt is finished while a worker is still sending a required operation | This attempt did not succeed. A later acceptance is recorded on the operation; a retry is a new attempt, once that operation is no longer sending. |

Each row is a test in `test_caller.py`. Other tests there cover a malformed
request, a program that does not load, the limits, each condition of
`assessment_permits`, and, with a stand-in the tests write to a temporary
directory, a server that breaks the protocol.

### The pattern to copy

```python
from caller import CaveatServer, assess_answer

with CaveatServer("assessment.cav") as server:
    result = assess_answer(server, 85)
    if result.succeeded:
        ...
```

Leaving the `with` block closes the session and waits for the server, or
stops it. Abandoning a `CaveatServer` without closing it or leaving the
`with` block is outside what this starter supports.

The server answers one request at a time; requests from several threads take
turns. A request interrupted between writing it and reading its response, by
`KeyboardInterrupt` for example, leaves the pipe out of step: the next line
could answer it. The caller then gives up on the server and stops it, as
above, and lets the interruption go on; any later request raises
`CaveatProtocolError`. A new attempt needs a new server.

### Sending from a worker

Register the operation with `begin` before handing `send` to a worker,
because cancelling the awaiting task does not stop a call already running in
the executor.

```python
import asyncio

async def observe(attempt, confidence):
    operation = attempt.begin("observe", {"confidence": confidence})
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(None, operation.send)
```

If `finish` runs while `send` is still running, the operation is unconfirmed
and the attempt did not succeed. Its request may still reach the server
afterwards, and land in whatever attempt is running then. So before a retry on
the same server, wait until that `Operation`'s `status` is no longer
`sending`, for example by awaiting the executor's future; or retry on a new
server.

## Change it

To make the approval rest on more evidence, declare it and its event in the
program, and add its `require` call to `assess_answer`. Then make
`assessment_permits` check that the grounds of the approval in force include
it, and give the evidence its own withdrawal and reopening rules. Grounds are
sets: compare them as sets. Tests that call `assess_answer` keep working.

## From code instead

A Node application can hold the session in process instead of starting a
server: `loadRuntimeFromDirectory()` from `caveat-lang/node` gives a runtime,
and `runtime.open(source)` a session. Or `createServer` from
`caveat-lang/serve` handles the same lines without I/O. See the
[package README](../../README.md#use-a-session-from-code). The three levels
and the rule are the same.

## Limits

- A source label, such as `"a tool observation the agent reported"`, describes
  evidence. It does not authenticate it. The program believes what the
  application sends.
- An accepted event, or an approval, permits nothing beyond what the
  application's own rule says it permits. It is not permission for any other
  action. To record who permitted a decision, see
  [permission](../../docs/reference/spec/caveat-permission-0.1.md).
- There is no receipt store, no hashing and no binding to files. Which
  records to keep, and how to tie them to what they describe, is specific to
  each application, and deliberately left out.

The [authoring guide](../../docs/reference/docs/AI_AUTHORING.md) covers
withdrawal, permission and reopening at more length.
