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
wrote a line Serve 0.1 does not allow, stopped reading requests, or did not
exit after `close` or a failed session, it stops every process the launch
started, not only the first: npx runs `caveat` under a shell. On POSIX that
is the launch's own process group; on Windows, the process tree, with
`taskkill /T`. If that fails, it stops the first process, and the error it
raises says that processes started under it may still be running. A server
that closes correctly exits by itself.

## The caller's rule

A workflow attempt succeeds only when its required operations succeed and its relevant, current CAVEAT assessment permits the intended result. A stored earlier assessment cannot substitute for a rejected required operation.

`Attempt` carries it out. `require` sends a required operation; once one is
rejected, or raises an error, the attempt has failed and sends nothing more.
`finish` reads the current snapshot and asks the application's own test
whether it permits the result. Here that test is `assessment_permits`: the
verdict is `approved`, and the approval in force was committed during this
attempt. `assess_answer` is this example's attempt: it names the required
operations, `observe` then `assess`.

A rejected event rolls back everything it did, so an earlier approval stays
visible after it. That is how the protocol works. The rule, not the session,
decides that the attempt failed.

| Situation | Conclusion |
| --- | --- |
| A required operation is rejected after an earlier approval | This attempt did not succeed, even if the old approval remains visible. |
| Evidence supporting an approval is withdrawn | Apply the program's explicit withdrawal/reopening policy; do not assume the old approval remains usable. |
| A test deliberately requests a forbidden action and gets the expected refusal | The TEST passed; the requested ACTION did not succeed. |
| A later, separate attempt receives valid replacement evidence and an accepted reassessment | Evaluate that new attempt; do not permanently mark every future attempt failed because an earlier one failed. |

Each row is a test in `test_caller.py`. Other tests there cover a malformed
request, a program that does not load, the limits, each condition of
`assessment_permits`, and, with a stand-in the tests write to a temporary
directory, a server that breaks the protocol.

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
