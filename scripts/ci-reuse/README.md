# Round-1 carry

Every pull request here gets two CI rounds. Round 1 tests the change. The
thread then records "green CI and approval" by adding two lines to
`ledger/events.jsonl`, and that push starts round 2. Round 2 used to run every
job again on code that had not changed, and took as long as round 1.

Now round 2 runs every job that could see the new lines, and the jobs that
cannot see them carry round 1's result instead of running again.

## What still runs on that push

- **core semantics**, which replays the whole ledger with the new lines
  (`npm run test:ledger`), and the jobs that wait for it: the MCP workers,
  Mr. Caveat, Moon Garden 2D and 3D, and Legacy Door when it is in scope.
- **Evidence games browser QA** and **packed kit dependency security**. The
  security audit checks the packed kit against today's RustSec and npm
  advisories, which change from day to day, so round 1's answer can be out of
  date. The Evidence games job packs the kit that the audit checks.
- **hosted MCP server**, **native runtime, optimized builds** and **VS Code
  extension**, which end before the slowest jobs anyway.
- all of **carmack.yml**: its paths include the ledger's events, and its
  ledger workload reads them. Also python.yml when its paths match, the Lean
  scaffold and the format check.

## What carries

| Job | Check names |
| --- | --- |
| `native` | native runtime |
| `native-reactive` | native runtime, reactive-only and diagnostic features |
| `compatibility` | rc.15 compatibility and departure gates |
| `lean-conformance` | Lean runtime comparison |
| `reproducible` | reproducible WebAssembly |
| `last-beacon` | The Last Beacon browser QA |
| `light-the-way` | Light the Way (chromium), Light the Way (webkit) |

None of these jobs reads `ledger/events.jsonl`. To keep it that way, every
full run of them takes the file out of its checkout right after the checkout
([`hide-ledger-events.sh`](../../.github/ci/hide-ledger-events.sh)). If one of
them ever starts to read it, it fails in round 1 instead of carrying a result
that depended on the file. Git still treats the file as unchanged, so
`git status` stays clean and the build's record of a clean tree is the same.

## When a run carries

The regression scope job decides, in [`reuse.mjs`](reuse.mjs). A run carries
only when all of these hold, checked in this order:

1. The repository variable `ROUND1_REUSE` is not `off`.
2. The run is for a push to a pull request (`synchronize`) from this
   repository.
3. The pull request does not change `kit/package.json` or anything under
   `docs/releases/`. A release pull request's final head always runs
   everything.
4. The tree being tested is the previous head's tree with lines added at the
   end of `ledger/events.jsonl`, each ending in a newline, and nothing else.
5. The newest earlier run of this workflow on the previous head of this pull
   request (round 1) finished with success, less than 60 minutes ago.
6. Round 1 left exactly one tested-tree record. It says round 1 tested the
   previous head's own tree (so that head contained current main), that
   round 1 did not itself carry, and that it chose the same browser QA jobs.
7. Every carried check ended with success in round 1, and ran in full there.

Steps 1 to 4 use only git, so round 1 asks GitHub nothing. Anything that goes
wrong runs everything: GitHub not answering within 10 seconds, a missing
permission, a crash. The decision step continues on error and has a 2-minute
limit. If the regression scope job fails, the carried jobs run in full.

## What a carried job does

Its first step, "Carry round 1's result", leaves a notice that names round 1's
run, with its attempt, and the tree, and writes the same in the job summary.
Every other step is skipped, and the job ends with success under its usual
name. The carry step fails if it is ever reached outside a pull request run.

The **round-1 carry** job runs only in a run that carries. It names round 1,
the tree, when round 1 finished, and the jobs it carried. The PR ledger
records it with the other checks a merge rested on, so
`rested-on --check "round-1 carry"` lists every merge that rested on a carried
result. The done check says how many checks on a head were carried, and when
the sweep was one of them.

## What it does not change

- The run on main after a merge runs every job: a push never carries. Pages,
  the release gate and npm publishing accept only that run.
- A run started by hand (`workflow_dispatch`) never carries.
- The weekly game QA retry count ([`game-qa-retries.mjs`](../game-qa-retries.mjs))
  still opens an issue at the same flakiness per run. A carried run could not
  retry, so each job's retries are counted as if every one of its runs had
  run: the count is scaled by its runs over the runs that ran.

## Turning it off

Set the repository variable `ROUND1_REUSE` to `off` (Settings, Secrets and
variables, Actions, Variables). To run every job on one head, start the
runtime workflow by hand on the branch.

## How it is checked

`npm run test:ci-reuse`, in core semantics:

- [`reuse.test.mjs`](reuse.test.mjs): the accepted case and every refusal
  above, against a temporary git repository and a stand-in for GitHub; and
  that a failed decision runs everything and still leaves the record.
- [`workflow.test.mjs`](workflow.test.mjs) reads the runtime workflow: only
  the jobs listed here can carry; when they carry they run nothing but the
  carry step, and when they do not they run as before; a failed regression
  scope runs them in full; the decision has read-only access and continues on
  error; core semantics still replays the ledger and carmack still runs on a
  ledger push; and only known code names the ledger's events file.
