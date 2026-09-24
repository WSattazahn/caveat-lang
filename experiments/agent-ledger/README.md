# An agent's ledger in Caveat

An agent working on a repository keeps deciding things on grounds that
change. "Merge this pull request" rests on its checks passing on the head
commit, and on the user's go-ahead. A new push, a failed check, or someone
else merging it first each change what the agent may do. On 2026-09-24 the
agent that maintains this repository made exactly these mistakes:

- it waited 2 h 45 min on CI runs that had scrolled out of the list it read;
- it prepared to merge #26 minutes after the user had merged it;
- it kept a memory that "#23 awaits the user's merge" after #23 merged.

This experiment writes that bookkeeping as a Caveat program, using only the
language as it is, to find out which missing features would matter in real
use. It is the investigator's own use, not an outside project.

| File | What it is |
| --- | --- |
| [`ledger.cav`](ledger.cav) | Pull requests, their pushes, checks, local gates, go-aheads and merges |
| [`ledger.scenarios.json`](ledger.scenarios.json) | The ledger's rules, tested: `caveat test ledger.scenarios.json` |
| [`session-2026-09-24.jsonl`](session-2026-09-24.jsonl) | That day's real history for #21, #26, #28 and #29 |

```sh
npx --no-install caveat explain ledger.cav session-2026-09-24.jsonl
npx --no-install caveat dependents ledger.cav pr21_go session-2026-09-24.jsonl
```

## What it caught

Replaying the day's history, the ledger refuses the two merges the agent
should not have attempted:

```text
   14  merge {"target":"pr26"}  refused (policy): Someone else has already merged it.
   17  merge {"target":"pr28"}  refused (policy): No passing checks on the current head.
```

For #28 and #29 the agent recorded the checks as they finished. It asked
the ledger before merging, and merged only once the ledger decided to. Each
decision shows what it rests on:

```text
    pr29_merge@1 = 1613487084  in force
      based on pr29_checks@1, pr29_go
```

## What the language made hard

Each item below is from the wishlist in the maintainer's notes. It says what
happened when the ledger needed it.

**1. Identifiers only known at run time: the largest barrier.**
- An event can name a pull request only if the source declares it
  (`entity pr29 kind pr`). Every new pull request means editing the program.
- A commit has to be a number. The ledger uses the first 8 hex digits of
  the SHA, so the decision above reads `= 1613487084`, not `602bdbec`.
- The event log needs its own translation table to be read by a person.

**2. Newer observations that supersede older ones: expressible, by hand.**
- Eight rules per pull request handle it:
  - five say what a push means: renew the push evidence, reveal it, record
    the head, clear "checks pass", and reopen a merge decision still in
    force;
  - two refuse checks and gates for any commit other than the head;
  - one reopens a decision in force when a check fails.
- They work (scenario L02), but each is easy to forget. Forgetting one leaves
  a stale decision in force with nothing to show it.

**3. "What rests on this?": answered.** `caveat dependents` (#29) answers it
directly:

```text
What rests on pr21_go in ledger.cav after 23 events (sequence 21)

Decisions
  pr21_merge@1 = 2869574427  in force  based on pr21_go
```

**4. Withdrawing a wrong observation: not possible.** Suppose a check result
is recorded as passed, then found to have been misread. Late qualification
cannot mark it:
- `qualify pr21_ci with misread` does not load: a reading stream's evidence
  is never observed itself, only its readings are;
- a single reading cannot be named in `qualify`;
- decisions keep their grounds by design.

The only move is a new, contrary reading. That reopens the decision, but the
decision still cites the misread reading, with nothing to mark it as wrong
(scenario L04).

**5. A process interface: available.** `caveat serve` (#29) lets a hook keep
this ledger open. Here the history was replayed from a file instead.

**A new one: what authorized a decision.**
- The ledger refuses a merge without a go-ahead, but a `reject` rule that
  let the event through leaves no trace in the decision.
- With the condition in the commit rule's own guard, the go-ahead appears
  only as "could also have been influenced by".
- It became part of the decision's grounds only once it was folded into the
  decided value: `using qualified(latest(pr21_checks), pr21_go)`.

"The user said merge" is neither the value a decision rests on nor mere
lineage. It is permission, and Caveat has no word for it.

## Limits

- The agent typed every entry. The first 18 were entered afterwards, from
  its transcript and GitHub; those for #28's and #29's checks and merges
  were entered as they happened. Nothing fed the ledger automatically.
- The agent judged which messages counted as a go-ahead for which pull
  request. The ledger cannot check that an approval was meant for this pull
  request and this commit.
- One day, one repository, and one kind of decision.

## Suggested order for the language work

1. Identifiers known only at run time. Without them the ledger cannot be used
   as events happen.
2. Withdrawing an observation while keeping its record. It is the one thing
   here that cannot be expressed at all.
3. Recording what authorized a decision.
4. Supersession. The rules can be written today; a feature would remove the
   chance of forgetting one.
