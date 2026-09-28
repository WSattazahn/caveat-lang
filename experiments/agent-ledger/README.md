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

The log records only go-aheads the user actually gave. Each names the head
commit it was for. Replayed, the ledger refuses every merge the agent made or
was about to make that day:

```text
    5  merge {"target":"pr21"}  refused (policy): No go-ahead for the current head.
   13  merge {"target":"pr26"}  refused (policy): Someone else has already merged it.
   15  merge {"target":"pr28"}  refused (policy): No passing checks on the current head.
   18  merge {"target":"pr28"}  refused (policy): No go-ahead for the current head.
   20  merge {"target":"pr29"}  refused (policy): No go-ahead for the current head.
```

The agent merged #21, #28 and #29 anyway. It took broad instructions as
permission to merge a particular pull request:
- "ok do 1 through 4" for #21;
- "proceed with your recommendation" for #28;
- "would you like to work on the 7 things" for #29.

The user's standing rule was that merging is a separate decision. For #26 the
user did say to merge it, but merged it themselves first.

### A correction

The first version of this experiment recorded those interpretations as
`approved` events, so it showed the three merges as allowed. The ledger's
agreement was only the agent checking its own assumption: it could not show
that the user had approved anything. Two things were corrected after an
outside review:
- a go-ahead now names the head commit it was given for, and a push clears
  it;
- the log records only explicit go-aheads.

The earlier version is in this directory's git history.

The agent now asks before every merge, naming the pull request and its full
head SHA.

## What the language made hard

Each item below is from the wishlist in the maintainer's notes. It says what
happened when the ledger needed it.

**1. Identifiers only known at run time: the largest barrier.**
- An event can name a pull request only if the source declares it
  (`entity pr29 kind pr`). Every new pull request means editing the program.
- A commit has to be a number. The ledger uses the first 8 hex digits of
  the SHA, so #29's checks reading has the value `1613487084`, not `602bdbec`.
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

**3. "What rests on this?": answered.** `caveat dependents` (#29) answers it.
After the history in scenario L05 it lists the merge decision that rests on
the broader go-ahead:

```text
What rests on pr29_scope in ledger.cav after 6 events (sequence 6)

Decisions
  pr29_merge@1 = 16  in force  based on pr29_scope
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
- The ledger refuses a merge without a go-ahead for the current head, but a
  `reject` rule that let the event through leaves no trace in the decision.
- With the condition in the commit rule's own guard, the go-ahead appears
  only as "could also have been influenced by".
- It became part of the decision's grounds only once it was folded into the
  decided value: `using qualified(latest(pr21_checks), pr21_go)`.
- The permission's scope is written out by hand too. Two things are needed:
  - a state recording which head each go-ahead was for;
  - a separate `approved_revisions` event for the broader permission, which
    then becomes the decision's grounds instead (scenario L05).

"The user said merge" is neither the value a decision rests on nor mere
lineage. It is permission, and Caveat has no word for it, or for its scope.

## Limits

- The agent typed every entry. Most were entered afterwards, from its
  transcript and GitHub. #28's and #29's checks and merge requests were
  entered as they happened. Nothing fed the ledger automatically.
- The ledger checks the inputs it is given against its rules. It cannot tell
  whether an input is true. That includes whether a message was a go-ahead
  for this pull request at this head, as the correction above shows.
- One day, one repository, and one kind of decision.
- **Capacity.** A program's reading and decision capacities may total at most
  1,024 ([reactive 0.5](../../spec/caveat-reactive-0.5.md), "Bounds and
  failure"). At `ledger-identifiers.cav`'s configured capacities, each pull
  request takes 88 of them: 32 check readings, 32 gate readings, 16 go-aheads
  and 8 merge decisions. So one session holds at most 11 pull requests
  (968). With a twelfth declared, the program does not load: "declared history
  capacity exceeds 1024". This is this ledger's limit, not the language's:
  smaller per-PR capacities hold more pull requests, and any history shared
  by all of them leaves room for fewer. `renewable $p_push limit 64` also
  allows 64 pushes per pull request. A 65th push is refused as
  `limit/renewal_limit`, and the other pull requests carry on (scenario I11).

## Follow-up: identifiers

[Identifiers 0.1](../../spec/caveat-identifiers-0.1.md) answers item 1 for
commits.
- [`ledger-identifiers.cav`](ledger-identifiers.cav) is the same ledger with
  `commit id`.
- [`session-2026-09-24-identifiers.jsonl`](session-2026-09-24-identifiers.jsonl)
  is the same history with full SHAs.

Replayed, it accepts and refuses exactly the events the numeric version does,
and shows the commits as themselves:

```text
  pr29.head = "602bdbec0a047a5319f53e83f336b9f7aec0e5ed"
```

[`ledger-identifiers.scenarios.json`](ledger-identifiers.scenarios.json)
tests two things here:
- two SHAs that share their first eight hex digits stay two commits;
- a refused event keeps no identifier.

Its later scenarios are described below.

Pull requests are still declared in the source: declarations made per
identifier are not part of that profile.

## Follow-up: withdrawal

[Withdrawal 0.1](../../spec/caveat-withdrawal-0.1.md) answers item 4. The
identifier ledger's `misread` event withdraws a pull request's latest check
result because the agent re-read it. It does not record a failure in its place.
A merge decision grounded on that result reopens, by a rule the ledger states,
and keeps what it was made on. Scenario I03 in
[`ledger-identifiers.scenarios.json`](ledger-identifiers.scenarios.json) tests
this, and `caveat explain` shows it:

```text
    pr29_merge@1 = 1  reopened
      based on pr29_checks@1, pr29_go
      pr29_checks@1 has since been withdrawn at #5 because pr29_recheck
```

## Follow-up: permission

[Permission 0.1](../../spec/caveat-permission-0.1.md) answers the new item.

In the identifier ledger, a go-ahead is a reading whose value is the commit it
approves. The merge rule names what permits it:

```caveat
on merge when target == $index and $p_revisions == 0
    commit $p_merge because enough using latest($p_checks)
    permitted by latest($p_approvals) for $p_head;
```

- **What changed.** The merge decision now rests on the checks alone. Its
  permission is recorded apart from its grounds, as
  `commitment_permissions`, and in the journal as `permitted_by`. The
  `qualified(…)` workaround and the hand-written approval check are gone.
- **The refusals.** Without a go-ahead for the head, the merge is refused as
  `policy/not_permitted`. The day's history is refused and accepted exactly as
  before. #21, #28 and #29 are now refused by the permission clause, not by a
  `reject` rule.
- **What stays.** The rule that reopens a decision when the head changes: the
  clause checks the head when the commitment is made, not afterwards. A
  go-ahead for later revisions is still a separate grant that a separate rule
  names.

## Follow-up: reopening triggers

[Reopening Triggers 0.1](../../spec/caveat-reopening-triggers-0.1.md) answers
the part of item 2 that is a language problem. A decision series can declare
which readings reopen it, instead of a hand-written rule after each one.

The ledger keeps its two reopening rules anyway. Both also require that the
pull request was not merged elsewhere (`$p_merged == 0`), and a declaration
cannot say that. Replacing them would change the ledger's policy, not shorten
it. Scenario I04 records the behavior that has to be kept: after someone else
merged the pull request, a failed check does not reopen the merge decision.

## Follow-up: keeping pull requests apart

Each pull request's rules stay with it only because every rule in the `for`
block starts with `target == $index`. A `for` block is a text expansion
([Repetition 0.1](../../spec/caveat-repetition-0.1.md)), so nothing else
keeps a rule to its own member, and `caveat check` does not look for a
missing selection. Two pull requests at the same SHA hold the same identifier
handle, so only their separate streams keep one's go-ahead from permitting the
other.

The first four scenarios each use one pull request. To measure what they
miss, one selection at a time was removed from each of the 31 rules that carry
one:
- every such program still loads and checks clean;
- I01-I04 caught 6 of the 31.

The sharpest one they missed drops the selection from `on approved`. pr28's
go-ahead is then recorded in pr29's stream, and at the same SHA it permits
pr29's merge.

I05-I11 put two or more pull requests in one scenario:
- **I05:** the same SHA, a go-ahead for only one of them, across two resumes.
- **I06:** interleaved events.
- **I07 and I08:** every kind of event on one pull request, pr28 in I07 and
  pr29 in I08.
  - After each event, the other one and the two untouched pull requests, pr21
    and pr26, must still have the same streams, decisions, values and
    bindings.
  - At the end, none of their evidence may be observed. Evidence cannot be
    un-observed, so one check at the end covers every event.
- **I09:** a pull request merged elsewhere blocks only its own merge, and
  observes its own evidence.
- **I10:** the same SHA with different results. Each result stays with its
  own pull request.
- **I11:** one pull request's 65th push is refused, and the others carry on.

**What they catch.** These are regression tests written with known slips in
view, not a proof that no slip can leak.
- **The 31 selection removals:** the scenarios catch all 31.
- **11 more slips from review.** A later review found slips of other kinds
  that passed the first version of these scenarios:
  - selecting by the shared commit instead of the pull request
    (`commit == $p_head`), which let pr28's pass at the same SHA make pr29
    green;
  - selections written as `target >= $index`, which leak only to
    lower-numbered pull requests;
  - a reveal hard-coded to one member.

  I07-I10 were extended for them, and each of the 11 now fails at least one
  scenario.

## Correction: no pull request starts approved

Random play found a fault in `ledger.cav`. `$p_approved_head` starts at 0,
the same as `$p_head`, so a pull request at head 0 counts as approved when no
go-ahead was given. The merge rule then cites `$p_go`, which nothing has
observed, and the event fails fatally instead of being refused:

```text
checks {"target":"pr26","commit":0,"result":"passed"}  accepted
merge  {"target":"pr26"}  fatal: event merge, rule 54: cannot qualify a value with unobserved evidence pr26_go
```

The runtime is right to refuse to qualify a value with unobserved evidence.
The fault is the program's. The day's history never reaches it, because no
commit in it is 0.

`ledger.cav`, its scenarios and the recorded session stay as they were, so
the results above are what was recorded.
[`ledger-approved-head.cav`](ledger-approved-head.cav) is the corrected copy.
Apart from comments saying why, one line differs:

```caveat
state $p_approved_head = -1 min -1 max 4294967295;
```

No commit is -1, so a head matches the approved head only after a go-ahead
for it. A go-ahead for commit 0 still counts.
[`ledger-approved-head.scenarios.json`](ledger-approved-head.scenarios.json)
tests it:
- **A01-A04:** without a go-ahead, a merge at head 0 is refused with "No
  go-ahead for the current head."; a go-ahead for commit 0 permits it; a
  go-ahead for later revisions permits it for its own pull request only. On
  `ledger.cav`, the first merge in each of the four is fatal.
- **L01-L05:** `ledger.scenarios.json`, unchanged.
- **I05-I11:** the isolation scenarios of `ledger-identifiers.scenarios.json`,
  with commits as numbers and without the `misread` event.

`ledger-identifiers.cav` does not have this fault. Its merge rule names the
go-ahead in a `permitted by` clause, which refuses a merge without one as
`policy/not_permitted`.

## Suggested order for the language work

1. Identifiers known only at run time. Without them the ledger cannot be used
   as events happen.
2. Withdrawing an observation while keeping its record. It is the one thing
   here that cannot be expressed at all.
3. Recording what authorized a decision.
4. Supersession. The rules can be written today; a feature would remove the
   chance of forgetting one.
