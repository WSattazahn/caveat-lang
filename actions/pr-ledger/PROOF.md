# The PR ledger on this repository's own merges

Run on 2026-10-25 from a working copy, on the released caveat-lang 1.3.0
(tarball SHA-256 `5c3eb33e…f136`, as the Action pins it), against the public
GitHub API. Until the [PR ledger workflow](../../.github/workflows/pr-ledger.yml)
runs on `main`, this page is the record; afterwards the `caveat-pr-ledger`
branch is.

## Backfill

```
node actions/pr-ledger/pr-ledger.mjs backfill --last 60 --repo WSattazahn/caveat-lang --dir proof
```

60 merges, #257 to #328, in two chapters: 43 merges over 22 sources, then 17
over 23 (the first chapter closed when the 23rd source, a QA job, would have
left room for 41 readings each and it already had 43 merges). Every merge
replays with every event accepted.

- 57 merges name Claude Code as their author; #309, #310 and #311 name no
  agent, so they rest on the account that opened them.
- #309 merged with `packed kit dependency security` failing, as the checks
  stand now. The record keeps that as evidence against, outside what the
  merge rested on:

  ```
  #309 Record 1.2.0 GitHub, hosted MCP and Glama closeout
    merged 2026-10-21T01:02:23Z at 453a697; main@1 in chapter 2
    merged despite: check "packed kit dependency security"
  ```

## A check later found wrong

D021 (fixed by #327) is a save that does not restore, reachable through
Holds 0.1 (#295) on every release from 1.2.0 to 1.3.0. The Rust suite, the
`native runtime` check, passed throughout. Recording that as a finding:

```
node actions/pr-ledger/pr-ledger.mjs wrong --check "native runtime" --from 295 --to 325 \
  --note "D021: passed while a Holds 0.1 reopening could write a save that does not restore" --dir proof
```

```
Found wrong: check "native runtime" on 30 merges.
#295  merged 2026-10-19T19:19:20Z  superseded  Holds 0.1: keep a decision in force without dropping its reopenings (1.2 draft)
    rested on check "native runtime", found wrong (check_native_runtime@28)
…
#325  merged 2026-10-25T03:45:10Z  superseded  Highlight nested for blocks in the VS Code extension
    rested on check "native runtime", found wrong (check_native_runtime@15)
```

The 30 merges are #295 to #323 and #325 (#324 has not merged), the 1.2.0
candidate (#307) and the 1.3.0 candidate (#317) among them. #324 and later
are outside the range the finding names; a merge resting on `native runtime`
after #325 is not listed.

Caveat itself says the same, by hand, from the files the record writes:

```
npx caveat-lang@1.3.0 dependents proof/chapter-001/program.cav check_native_runtime@28 proof/chapter-001/events.jsonl
```

```
Decisions
  main@28 = 295  superseded  based on check_native_runtime@28
    check_native_runtime@28 withdrawn at #1015 during wrong because finding
  main@29 = 297  superseded  could have been influenced by check_native_runtime@28
  …
```

Only `main@28`, #295's merge, has the reading in its grounds. The later
revisions list it as lineage (each merge supersedes the one before it, and
lineage keeps the predecessor's basis), so the Action reports grounds only.

## What this shows

- A merge's grounds are what the record says it rested on, and a finding
  made five days later names those merges without changing them.
- The record carries a fact the merge itself hid (#309's red check).
- The limit that shapes the record is a program's declared history (1,024),
  which is why the record comes in chapters.
