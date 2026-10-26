# CAVEAT Ledger Sets 0.1 (draft) — reviewing past ledgers, and saying a decision read one

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.4, building on [Review 0.1](caveat-review-0.1.md) and [Outcomes 0.1](caveat-outcomes-0.1.md). It is a report of the kit: it adds no syntax, changes no program and no session, and a program means what it meant before.

An agent can learn from its own mistakes only the way a person does from
notes: it reads what happened last time before it decides. Outcomes 0.1
records how a decision turned out, and Review 0.1 says which evidence it
turned on and which caveat came true. Two things were missing for using that
record the next time:

1. **Finding the past ledgers.** A review takes one program, one events file
   and one revision. Learning from a history of decisions means reviewing
   every decision that has an outcome, across many records, without naming
   each revision by hand.
2. **Saying a decision read one.** When an agent reads a past review and then
   decides differently, the new record says nothing of it. The lesson shows
   only in the host's own log, so a later reader cannot tell which past
   mistake a decision was meant to avoid.

This profile answers both in the kit. Nothing here changes a model: the
agent reads a record, and a rule a person or an agent tightened keeps the
lesson. A demo found both gaps: a refund agent refunded on a customer's own
email, a delivery scan later showed it was wrong, and the second time the
agent read that review and checked the carrier's tracking first.

## Example

Round 1 (`ledgers/ticket-a.cav` and `ledgers/ticket-a.events.jsonl`):

```caveat
claim refund_warranted;
evidence order_record from "the order system";
evidence customer_email from "the customer's email";
evidence delivery_scan from "the carrier's signed delivery scan";
caveat self_reported consequence material;
self_reported qualifies customer_email;
decisions refund limit 4;
state price = 0 min 0 max 500;
state missing = 0 min 0 max 5;
event read_order; event read_email; event decide; event delivered_after_all;
on read_order reveal order_record supports refund_warranted;
on read_order set price = qualified(80, order_record);
on read_email reveal customer_email supports refund_warranted;
on read_email set missing = qualified(1, customer_email);
on decide when missing >= 1 commit refund because enough using price * missing retaining self_reported;
on delivered_after_all reveal delivery_scan opposes refund_warranted;
on delivered_after_all resolve refund = 0 because delivery_scan materialized self_reported;
```

Round 2 (`ledgers/ticket-b/program.cav` and `ledgers/ticket-b/events.jsonl`)
declares the past review it reads as evidence, reveals it before deciding,
and needs tracking:

```caveat
evidence lesson from "caveat-review:<SHA-256 of ticket-a.cav>#refund@1";
event recall;
on recall reveal lesson;
on decide when missing >= 1 and lost < 1 reject "Read tracking first.";
on decide when missing >= 1 and lost >= 1 commit refund because enough using price * missing * lost retaining self_reported;
```

`caveat review --all ledgers/` reviews both refunds. The second ends:

```text
Consulted: lesson, the review of refund@1 of program 718016c155de; ledgers/ticket-a.cav: self_reported came true, decisive
```

## 1. Finding ledgers

```text
caveat review [--runtime <dir>] [--json] --all <directory>
```

A **ledger** is a program and its record. Under the directory, searched
through every subdirectory except those whose names start with `.` and
`node_modules`, in path order:

- `NAME.cav` beside `NAME.events.jsonl` is a ledger;
- otherwise, a directory's only `.cav` beside `events.jsonl` is a ledger.

Each ledger's events are sent to a fresh session, as `explain` does. Every
revision of a decision series that has an outcome (Outcomes 0.1), or that
consulted a past review (section 2), is reviewed as Review 0.1 says.

Text output is each review, with its Consulted lines, then
`N revisions reviewed in M ledgers.` `--json` prints one
`caveat-ledger-review/0.1` object per line: the review's fields, with
`schema` replaced and `program`, `events` (the paths) and `program_sha256`
(the program's source SHA-256, as saves bind it) added. A ledger whose
program does not load is reported on standard error and reviewed no
further.

Exit status: 0 after the reviews; 1 when an event failed fatally in a
reviewed ledger (its reviews are of the session before it); 2 when the
directory cannot be read, holds no ledger, or a ledger's files or program
cannot be used (the others are still reviewed).

## 2. Consulted reviews

Evidence whose source text is exactly

```text
caveat-review:<64 lowercase hex digits>#<SERIES>@<N>
```

names a **consulted review**: the review of revision `SERIES@N` of the
program whose source has that SHA-256. It is ordinary evidence. The runtime
gives the source no meaning, so a program means what it meant before, and a
program can rest a rule on it (`observed(lesson)`) like any other evidence.

A review lists each consulted review the record observed, by any `reveal`,
before the deciding event, as `consulted`: `{ evidence, program_sha256,
revision, in_lineage }`. `in_lineage` is true when the evidence is in the
revision's lineage, so the decision read it and did not only see it. The
text adds a line per item:

```text
Consulted: EVIDENCE, the review of REVISION of program <first 12 hex digits>[ (in its lineage)]
```

With `--all`, each item is resolved against the set. `found` is
`{ program, events }` of the ledger whose source has that SHA-256, or null.
`came_true` lists `{ caveat, decisive }` from that revision's review, or is
null when the set holds no review of it. The line ends with what was found:

- `; PROGRAM: CAVEAT came true[, decisive]; …` or `; PROGRAM: nothing came true`
- `; PROGRAM has no such revision with an outcome`
- `; no ledger in the set has that program`

The single-ledger `caveat review` lists `consulted` without resolving it,
since it is given one ledger.

What it is not:

- It does not check that the agent understood the review, only that the
  record says it read one, and which.
- A SHA-256 names exact source bytes. A program edited since its review was
  consulted is a different program, and is not found.

## 3. In the kit

`kit/lib/ledgers.mjs`, typed in `kit/lib/ledgers.d.mts`, not a package export
while this profile is a draft:

| Function | Returns |
| --- | --- |
| `consultedSources(snapshot)` | the program's consulted-review evidence, `{ evidence, program_sha256, revision }` |
| `reviewWithConsulted(open, events, revision, options)` | `review(…)` with `consulted` |
| `reviewLedgers(open, ledgers, options)` | an entry per reviewed revision, or `{ error }` per ledger that cannot be used; `ledgers` are `{ program, events_file, source, events }` |
| `formatConsulted(consulted)` | the Consulted lines |

Finding files is the command's: the library reads no files.

## Limits

- Each revision is a full Review 0.1 run, so a large set is slow. It is for
  looking back, not a hot path.
- Ledgers are found by file names only. A manifest, or decisions without
  outcomes as a dataset (`caveat export --corpus`), is left for when a set
  needs it.
- A consulted review is recorded by the program's author or the agent's
  host. Nothing stops a record from claiming a review it did not read; the
  reviewed ledger and its SHA-256 are there for anyone to check.

## Implementation

- `kit/lib/ledgers.mjs` and `kit/lib/ledgers.d.mts`.
- `kit/bin/caveat.mjs`: `review --all`, and Consulted lines on `review`.
- `kit/test/ledgers.test.mjs`: the source form, a set with a revision that
  has an outcome and a later one that consulted its review, unresolved
  reviews, consulted evidence after the deciding event and in the lineage,
  a program that does not load, and the command over a directory.

## Changes

- 2026-10-25: drafted and implemented for 1.4, from the learn-from-ledger demo.
