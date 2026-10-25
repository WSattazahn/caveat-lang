# CAVEAT Untrusted Sources 0.1 (draft) — several sources, one confirmation per action, withdrawing a source, the undo list

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.4: specified on its branch, not yet in a release. It becomes stable when a 1.x release ships it. It adds to
[Withdrawal 0.1](caveat-withdrawal-0.1.md),
[Retention Reasons 0.1](caveat-retention-reasons-0.1.md),
[Past Revisions 0.1](caveat-past-revisions-0.1.md) and
[Dependents 0.1](caveat-dependents-0.1.md); it changes no program that loads
today.

The [prompt injection experiment](../experiments/prompt-injection/README.md)
runs an inbox agent on `caveat-lang@1.3.0`. Text read from an email carries
the caveat `untrusted`; an external send is an irreversible decision, so the
runtime refuses one resting on email text unless its commit retains the caveat
because the user confirmed. When a source turns out to be malicious, its
instruction is withdrawn and the reversible actions resting on it are reopened.
The experiment found four places where the language made the author work
around it (README, "Limits found", items 2 to 5):

1. **One action on several sources.** A send whose instruction came from one
   email and whose recipient came from another was written as one decision
   per pair of sources.
2. **Confirmation is global and permanent.** `user_ok`, once revealed, is the
   reason for every later send.
3. **Undo covers the current revision only.** A reopening questions the
   revision in force. Earlier revisions resting on the bad source are for the
   host to find.
4. **Withdrawing a whole source.** `withdraw latest(S)` withdraws one reading.
   Everything else the source said needs a rule per reading.

This profile answers each:

| Gap | Answer | Kind |
| --- | --- | --- |
| 1 | a nested routed `for`, which already grounds one decision per action on both sources | no change; a documented pattern and a test |
| 2 | `retaining CAVEAT because once EVIDENCE` | new syntax, one new refusal code |
| 3 | the undo list, `undoList(snapshot)` and `caveat undo-list` | a kit report, no syntax |
| 4 | `withdraw every NAME because EVIDENCE` | new syntax |

Both new forms refuse to load today, so no program that loads changes. The
kit report reads a snapshot and changes no session.

## 1. One action on several sources

A [routed](caveat-routed-repetition-0.1.md) `for` block may hold another, and
each adds its own route ([Nested Repetition 0.1](caveat-nested-repetition-0.1.md)).
An event that names two members of one kind is routed by one parameter in the
outer block and by the other in the inner one:

```caveat
event send_to instr kind source, recip kind source;
for source as $i routed by instr {
    decisions $i_send limit 1 irreversible;
    for source as $r routed by recip {
        on send_to when not has_sample($i_asks) or not has_sample($r_addr)
            reject "Missing instruction or recipient.";
        on send_to commit $i_send because enough using latest($i_asks) + latest($r_addr);
    };
};
```

Each copy of the inner rules is expanded as if
`instr == I and recip == R and …` had been written: the outer route first,
then the inner, then the guard (Routed Repetition 0.1, section 3, applied once
per block). So `send_to {"instr": "user", "recip": "newsletter"}` runs only
the copy for that pair, and `user_send@1` is grounded on `user_asks@1` and
`newsletter_addr@1`. It carries `untrusted` from the newsletter's address, and
the irreversible commit is refused as `evaluation/caveated_irreversible`
unless it retains the caveat. `caveat explain` names both sources.

There is one decision per instruction source, not per pair: the decision's
grounds say which sources it rests on. The source text is written once. Its
expansion holds one copy of the inner rules per pair, and one more factor per
further nested block (an attachment, a subject). Each copy's route compares an
event parameter with a number, and `and` stops at the first false operand, so
on an event only the copies of the named members read further than their
route.

Nothing here is new. Version 1.3.0 loads and runs this program as described;
this profile states the pattern, `runtime/tests/untrusted_sources.rs` pins it,
and the experiment's recipient program uses it.

## 2. One confirmation per action

### Syntax

```text
commit D because REASON […] retaining CAVEAT because once EVIDENCE {, …}
```

`once` may stand between `because` and the reason's name, in any retained
caveat that names a reason. `retaining k because once e` refuses to load today
("retaining expects CAVEAT [because EVIDENCE], …"), so no program that loads
changes. Evidence named `once` keeps its meaning: `retaining k because once`
names it, as before.

### Meaning

A `once` reason means everything `because EVIDENCE` means (Retention Reasons
0.1): it must be observed, it joins the grounds, and the commit records the
pairing in `retention_reasons`. In addition, when the commit runs:

- **The reason must be unspent.** An occurrence is *spent* once it is the
  reason of a retained caveat in any commitment the session holds, in force,
  reopened or superseded, whether that commitment named it with `once` or
  not. A commit whose `once` reason resolves to a spent occurrence is refused
  as `evaluation/reason_spent`, and records nothing.
- Two caveats of the same commit may name the same `once` reason. The
  occurrence is spent by that commitment, not by one of its caveats.

The check runs after the reasons are read, so an unobserved reason is still
`evaluation/unobserved_evidence`, and before the irreversible check, so a
spent reason is never reported as `caveated_irreversible`.

A spent occurrence stays spent: the record that spends it is the commitment's
`retention_reasons`, which nothing removes. A withdrawal of the reason does not
unspend it; a confirmation found false is withdrawn, and the decision it backed
is reopened by the program's own rules, as for any withdrawn grounds.

### A confirmation per send

The reason is usually renewable evidence, renewed on each confirmation:

```caveat
evidence user_ok from "the user confirming one pending send";
renewable user_ok limit 64;
on confirm when observed(user_ok) renew user_ok;
on confirm reveal user_ok;
on act commit $m_send because enough using latest($m_asks)
    retaining untrusted because once user_ok;
```

The first send after a confirmation spends `user_ok`. A second send before the
next confirmation is refused as `reason_spent`. Each confirmation is a new
occurrence, `user_ok@2` and so on, unspent until a send names it. Without
`once` the same program lets one confirmation cover every later send; 1.3.0
shows this for two irreversible decisions after one `confirm`. With plain
(not renewable) evidence a `once` reason can back one commitment in the whole
session.

`once` says which commitment used a confirmation. Which action the user
confirmed is the host's to say: a confirm event can name the action, and a
`reject` can refuse an `act` the confirmation was not for.

### No new record

The snapshot, the save, the views and `explain` are unchanged. `once` is a
check when the commit runs; what it checks against is the `retention_reasons`
map those already hold. A save written by a program with `once` restores as
any save does, and restore adds no check: two commitments that share a reason
are what a program without `once` produces, and restore never authenticates
history ([A3](caveat-save-0.1.md)).

## 3. The undo list

`rests_on_withdrawn(D)` asks about the current revision, and a program reopens
what it rests on. A revision that was superseded, or a decision the program has
no reopening rule for, can still have gone out on evidence since withdrawn.
[Past Revisions 0.1](caveat-past-revisions-0.1.md) lets a program ask about
one past revision by number. The host wants the whole list.

`undoList(snapshot)`, exported from `caveat-lang/explain`, returns:

```json
{
  "schema": "caveat-undo/0.1",
  "sequence": 9,
  "withdrawals": [{"evidence": "newsletter_asks@1", "because": "newsletter_bad", "sequence": 8, "event": "found_malicious"}],
  "decisions": [
    {
      "series": "newsletter_add_event", "id": "newsletter_add_event@1", "value": 1,
      "status": "reopened",
      "withdrawn": [{"evidence": "newsletter_asks@1", "because": "newsletter_bad", "sequence": 8, "event": "found_malicious"}]
    }
  ]
}
```

- `withdrawals` is the snapshot's withdrawal ledger, in its order.
- `decisions` holds every revision the snapshot holds, of every series and
  plain commitment, whose frozen grounds hold withdrawn evidence, in the
  order `explain` lists them. That is the rule `rests_on_withdrawn` reads,
  applied to every revision: grounds, not lineage, so evidence only a guard
  read is not listed.
- Each entry has `series`, `id`, `value`, `status` (`in force`, `reopened`
  or `superseded`, as `explain` says), `withdrawn` (the withdrawal records of
  the evidence in its grounds, in ledger order), `irreversible: true` for an
  irreversible series, and `outcome` when [Outcomes 0.1](caveat-outcomes-0.1.md)
  recorded one.

A revision `in force` is one the program has not reopened: the host decides
whether to undo it. A `reopened` one the program has flagged. A `superseded`
one is no longer current but may have acted. An `irreversible` one cannot be
called back; it is listed so that the host can tell someone.

`caveat undo-list [--runtime <dir>] [--json] PROGRAM EVENTS` dispatches the
events as `explain` does and prints the report, in text or as the JSON above.
It exits 0 when the events are accepted, whether or not anything is listed.
The text lists the entries in force first, then the reopened, then the
superseded.

The list is a reading of the snapshot. It reopens nothing, changes no status
and does not judge whether a withdrawal was right. An empty list means no held
revision's grounds hold withdrawn evidence; it does not authenticate the
evidence. A withdrawal of a record that has departed
([Departure 0.1](caveat-departure-0.1.md)) is not in the snapshot, and so not
in the list.

## 4. Withdrawing a whole source

### Syntax

```text
withdraw every NAME because EVIDENCE;      -- an effect, in a rule or procedure
```

NAME is a reading stream or declared evidence, renewable or not, or a
procedure's `evidence` parameter where `withdraw` takes one. `withdraw every
NAME` refuses to load today ("invalid reactive identifier"), so no program
that loads changes. Evidence named `every` keeps its meaning:
`withdraw every because R` withdraws it, as before.

### What it withdraws

When the effect runs, it lists the **held, observed** occurrences NAME stands
for, as [Dependents 0.1](caveat-dependents-0.1.md) reads a subject:

- a reading stream: its readings, oldest first;
- evidence: its occurrences, first to current (plain evidence is its one
  occurrence), then the readings of each stream that reads from it, streams
  in name order, readings oldest first.

A held occurrence is one that has not retired ([Windows 0.1](caveat-windows-0.1.md)).
An occurrence not yet observed, such as a renewable's fresh one, is not
listed. So `withdraw every newsletter_text because newsletter_bad` withdraws
the newsletter's text and every instruction and address read from it.

The reason resolves to its current occurrence and must be observed, or the
event is refused as `evaluation/unobserved_evidence`, as for `withdraw`. This
holds even when nothing is listed.

Then each listed occurrence not already withdrawn is withdrawn exactly as
`withdraw` withdraws one (Withdrawal 0.1, "What `withdraw` does"): a record in
`withdrawals`, the built-in `withdrawn` caveat on the occurrence and on every
current value whose lineage or grounds hold it, and an effect report
`{"kind": "withdraw", "evidence", "because"}`, one per occurrence in the order
above. One already withdrawn keeps its first record. When nothing is withdrawn
the effect records and reports nothing, and the event is not refused.

Nothing else changes: every frozen basis and grounds, the archives and the
journal stay as they were. A reading sampled later, from the same stream or
template, is a new observation and is not withdrawn. A program that keeps a
source out for good rejects further instructions from it, as the experiment
does with `observed($m_bad)`.

### Snapshot, save and views

There is no new record. Each withdrawal is an ordinary record of Withdrawal
0.1, so the snapshot, the save, restore's checks, the views and the withdrawal
collector see what they would see after the same withdrawals written one rule
each. A program with `withdraw every` uses withdrawal, and has the `withdrawn`
caveat, as one with `withdraw` does.

## Refusal code

`evaluation/reason_spent`, from 1.4.0: an executed `commit` names a
`once` reason whose current occurrence is already the reason of a retained
caveat in a commitment the session holds. It is a new code for new syntax
(compatibility commitment, "Rejection origins and codes"). Its origin is
`evaluation`, as for `already_committed`: the commit ran and the session's
record refused it.

## Not in this profile

- A runtime reference to a member chosen by an event parameter, such as a
  stream named by `recip` without a nested block. The nested routed block
  writes the same decision; the reference would be new expression syntax
  with its own checks.
- Spending a permission grant, or any reason other than a retained caveat's.
- Unspending a reason, or undoing a withdrawal.
- Withdrawing readings by value, or the readings sampled after the effect.
- The undo list in `caveat serve` or the MCP tools, and in the views. A host
  that holds a snapshot calls `undoList`.
- Revisions whose lineage, not grounds, holds withdrawn evidence, and revisions
  whose permission grant is withdrawn. Lineage is not what a decision rests on
  (Withdrawal 0.1); a withdrawn grant refuses later commits (Permission 0.1).

## Proofs

`proofs/lean/Caveat` models a withdrawal as a record and a caveat on one
occurrence. `withdraw every` is several such withdrawals in one transaction;
the model's theorems about frozen bases hold for each. A `once` reason is a
refusal before anything is recorded, which the model's atomic rollback covers.
The undo list reads the snapshot. The implementation adds runtime and kit
tests.

## Corpus sweep

`node experiments/untrusted-sources-sweep/sweep.mjs` reads every tracked
Caveat program and writes
[`RESULTS.json`](../experiments/untrusted-sources-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 203 programs; 53 of them withdraw, with 92 `withdraw NAME`, 22
  `withdraw latest(S)` and no `withdraw every`;
- 4 retained caveats name a reason, none with `once`;
- no program declares a name `every` or `once`;
- three events name two members of one kind: the experiment's `send_to` and
  `before_the_rain.cav`'s `confront` and `hold`. One program nests a routed
  block in another: the experiment's recipient program, rewritten here to
  the pattern of section 1 (it had written one decision per pair out by
  hand);
- no program asks `rests_on_withdrawn(revision(…))`.

No program reaches a new form, so no accepted program changes.
[The sweep's README](../experiments/untrusted-sources-sweep/README.md) also
records a run of every tracked program's scenarios on the published 1.3.0
package and on this branch, with the same results.

## Changes

- 2026-10-25: specified for 1.4 (prompt injection experiment, items 2 to 5).
