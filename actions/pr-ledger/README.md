# Caveat PR ledger

A repository that merges many pull requests written by coding agents needs to
answer one question after the fact: **this check, this review or this agent
turned out to be wrong, so which merges rested on it?** GitHub keeps the
checks and reviews, but not as grounds a merge rested on, and a later finding
has nowhere to go.

This Action keeps that record in [Caveat](../../README.md). Each merged pull
request becomes a decision revision whose grounds are exactly what it rested
on: every check that passed on its head, every reviewer whose latest review
approved, and its author (the coding agent it names, or the account that
opened it). A failed check or a request for changes is recorded, as evidence
against, but is not part of what the merge rested on. A finding later
withdraws the readings it covers. It never rewrites a merge's grounds, so
Caveat's `dependents` report can still name every merge whose grounds hold a
withdrawn reading.

## Use it

Record every merge. The record lives on its own branch (`caveat-pr-ledger` by
default), so nothing is pushed to the branch you merge into.

```yaml
name: PR ledger
on:
  pull_request_target:
    types: [closed]
permissions:
  contents: write        # push the record to its branch
  pull-requests: read
  checks: read
  statuses: read
concurrency:
  group: pr-ledger
  cancel-in-progress: false
jobs:
  record:
    if: github.event.pull_request.merged
    runs-on: ubuntu-latest
    steps:
      - uses: WSattazahn/caveat-lang/actions/pr-ledger@<commit sha>
```

`pull_request_target` gives a token that can push for pull requests from
forks too. The Action never checks out or runs the pull request's code; it
reads what GitHub says about it.

When something turns out wrong, record the finding and get the merges that
rested on it, from a `workflow_dispatch` job or a terminal:

```yaml
      - uses: WSattazahn/caveat-lang/actions/pr-ledger@<commit sha>
        with:
          command: wrong
          check: e2e            # or review: LOGIN, or author: NAME
          from: 812             # the first and last pull request it covers;
          to: 840               # or pr: N for one; neither for every merge
          note: e2e ran against a cached build, so it passed without testing the change
```

| Command | What it does |
| --- | --- |
| `record` (default) | records the pull request `pr` (by default the one that triggered the run) |
| `backfill` | records the `last` (default 30) most recently merged pull requests |
| `wrong` | withdraws the readings of a `check`, `review` or `author` on the merges named, then lists every merge that rested on anything found wrong |
| `affected` | lists every merge that rested on anything found wrong |
| `rested-on` | lists the merges whose grounds hold a `check`, `review` or `author` (on one `pr`, or on all), changing nothing |
| `show` | what one merge `pr` rested on |

Output goes to the job log and its summary. The same commands run locally
with `node actions/pr-ledger/pr-ledger.mjs COMMAND … --repo OWNER/NAME --dir
FOLDER`, given `CAVEAT_PR_LEDGER_KIT` (a caveat-lang package folder) and,
for private repositories, `GITHUB_TOKEN`.

## The runtime

The Action runs the released `caveat-lang` package, downloaded from the npm
registry and checked against a SHA-256 before it is unpacked:
`caveat-version` 1.3.0 by default, with its tarball's checksum as
`caveat-sha256` (the same one [the release record](../../docs/releases/v1.3.0-npm-publication.json)
lists). Nothing is installed with scripts, and the package has no
dependencies. Pin the Action itself by commit SHA.

## The record

Each chapter folder holds `chapter.json`, the record itself, and
`program.cav` and `events.jsonl` written from it, so anyone can ask it
directly:

```
npx caveat-lang@1.3.0 explain pr-ledger/chapter-001/program.cav pr-ledger/chapter-001/events.jsonl
npx caveat-lang@1.3.0 dependents pr-ledger/chapter-001/program.cav check_e2e pr-ledger/chapter-001/events.jsonl
```

The program is [one template](pr-ledger.cav.template) with one `source`
entity per check, reviewer and author the chapter has seen. A Caveat program
declares at most 1,024 history entries across its decision revisions and
readings, so a chapter holds at most 64 merges, and fewer when they rest on
more than 15 sources (24 sources allow 40 merges). The next merge starts a
new chapter; queries read them all.

## What it does not do

- It records what GitHub says when it runs. `record` on the merge event sees
  the checks as they stood at merge; `backfill` sees them as they stand now,
  after any re-runs.
- The author is what the pull request says: a bot account, a "Generated
  with" line in its description, or a `Co-authored-by` trailer naming Claude
  Code, Codex, Copilot, Devin, Cursor, Jules or Aider. It is not an
  authentication, and Caveat's `agent` statement (Agents 0.1) is not used,
  so the record replays on 1.3.0.
- It blocks nothing and reopens nothing by itself. A finding is a record;
  what to do about the merges it names is yours.
- A finding withdraws readings; it cannot be taken back. Record a new merge
  of the fix instead.

`npm run test:pr-ledger` runs the host's tests ([pr-ledger.test.mjs](pr-ledger.test.mjs))
on this repository's kit, and [PROOF.md](PROOF.md) is the record of this
repository's own merges.
