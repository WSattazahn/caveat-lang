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
full run of them puts the file out of the job's reach right after the checkout
([`hide-ledger-events.sh`](../../.github/ci/hide-ledger-events.sh)):

- The file leaves the checkout, so reading it fails and a walk of the folders
  does not find it.
- Git gives one placeholder line for its contents, so a read through git
  (`git show`, `git archive`, `git grep`) gets that line instead of the
  events.

If one of them ever starts to read the file, the read fails in round 1 or
gets the placeholder, never the events. HEAD and the index still name the
real contents, so `git status` stays clean and the build's record of the
revision and of a clean tree is the same. A job could still go around the
placeholder on purpose (`git --no-replace-objects`); none of these does.

Two things are outside the hide's reach:

- **Core's browser build.** Core semantics replays the ledger, so it keeps the
  events, and it makes the browser build with them in reach. The Lean
  comparison, both games' browser QA and reproducible WebAssembly use that
  build. Reproducible
  WebAssembly rebuilds it with the events out of reach and requires every
  file to match, so a build that came to depend on them fails in round 1.
- **Ids.** The approval push makes a new commit, so its id, its tree's id and
  the id git gives the events file all change. A job whose result depended
  on an id rather than on the files would have to leave this list. None
  does: these jobs write the revision into their reports, and no result
  depends on which commit it is.

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
7. Every carried check ended with success in round 1, ran in full there, and
   finished less than 60 minutes ago. A re-run of other jobs keeps a job
   with the time it really ran, and that is the time that counts.

Steps 1 to 4 use only git, so round 1 asks GitHub nothing. Anything that goes
wrong runs everything: GitHub not answering within 10 seconds, a missing
permission, a crash. The decision step continues on error and has a 2-minute
limit. If the regression scope job fails, native runtime, the reactive-only
suite, the compatibility gates, the Lean comparison and reproducible
WebAssembly run in full. The Last Beacon and Light the Way need its games
output, so they are skipped then, as they were before.

A run carries only in the attempt that decided. A re-run of failed jobs keeps
the regression scope's outputs from the earlier attempt, so the carried jobs
it re-runs run in full: a re-run never carries past the 60 minutes or past
`ROUND1_REUSE`.

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
the sweep was one of them. It tells a carried check by the notice its carry
step left, so a job run again in full afterwards counts as run.

## What it does not change

- The run on main after a merge runs every job: a push never carries. Pages,
  the release gate and npm publishing accept only that run.
- A run started by hand (`workflow_dispatch`) never carries.
- The weekly game QA retry count ([`game-qa-retries.mjs`](../game-qa-retries.mjs))
  still opens an issue at the same flakiness per run. A carried run could not
  retry, so each job's retries are counted as if every one of its runs had
  run: the count is scaled by its runs over the runs that ran. It tells a
  carried check run by the notice the carry step leaves on it, so a later
  attempt that ran the job in full counts as run.

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
  scope still runs the jobs that did not need it before; they carry only in
  the attempt that decided; the decision has read-only access and continues
  on error; core semantics still replays the ledger and carmack still runs on
  a ledger push; only known code names the ledger's events file; the hide
  puts the events out of reach of the checkout and of git and leaves the
  revision and `git status` as they were; and reproducible WebAssembly
  compares every file of the browser build the carried jobs use.
