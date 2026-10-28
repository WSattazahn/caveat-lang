# CAVEAT Find 0.1 (draft) — finding past decisions by what they rest on

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.5, building on [Ledger Sets 0.1](caveat-ledger-sets-0.1.md) and [Explanations 0.2](caveat-explanations-0.2.md). It is a report of the kit: it adds no syntax, changes no program and no session, and a program means what it meant before.

An agent that starts a task fresh does not know what was decided last time,
or why. A person in the same place searches their notes; a mathematician
searches Mathlib for the lemma whose name they forgot. Caveat records every
decision with its grounds, but finding one meant knowing the program, its
record and the revision, or reviewing a whole set (`caveat review --all`) for
the ones with outcomes.

Find answers the question an agent asks before it decides: has this been
decided before, on what, and does that decision still stand? It searches
the ledgers under a directory (or, through MCP, inline records of one
program) by plain words, and returns each matching decision revision with
its value, its standing, what it rests on and how it turned out.

## Example

With the two refund ledgers of Ledger Sets 0.1 under `ledgers/`:

```text
$ caveat find ledgers/ customer email
ledgers/ticket-a.cav: refund@1 = 80  in force
  based on customer_email ("the customer's email"), order_record ("the order system"); caveats: self_reported
  decided at #3 decide
  turned out 0 at #4 delivered_after_all because delivery_scan; came true: self_reported
  matched customer: evidence customer_email, source the customer's email
  matched email: evidence customer_email, source the customer's email

ledgers/ticket-b/program.cav: refund@1 = 80  in force
  ...

2 decisions found in 2 ledgers.
```

The first answer is the lesson: a refund decided on the customer's own email
turned out wrong, and the caveat it retained came true.

## 1. The command

```text
caveat find [--runtime <dir>] [--json] <directory> <word>...
```

Ledgers are found under the directory as `caveat review --all` finds them
(Ledger Sets 0.1, section 1). Each ledger's events are sent to one fresh
session, and every revision of every decision series in its explanation is
searched, current or not, with or without an outcome.

## 2. Matching

The query is the words given, lowercased and split on white space. A
revision is found when **every** word occurs, as a substring ignoring case,
in at least one of its terms:

| Kind | Terms |
| --- | --- |
| `decision` | the decision series' name |
| `revision` | the revision, such as `refund@1` |
| `program` | the program's path under the directory; through MCP, the record's name |
| `evidence` | each evidence in the revision's grounds, its lineage, and its outcome's `because` |
| `source` | each such evidence's declared source text |
| `display` | each such evidence's or caveat's display text, where declared |
| `claim` | each claim such evidence supports or opposes |
| `caveat` | each caveat in its grounds, lineage, retained caveats and outcome's materialized caveats |
| `reason` | each reason its caveats were retained for (Retention Reasons 0.1) |

There is no ranking and no fuzzy match: a word matches or it does not, so a
result can be checked by hand. A word matches part of a name, so `refund`
matches `refund_warranted`.

## 3. The report

`--json` prints one `caveat-find/0.1` object:

- `query` and `words`;
- `searched`: per ledger, `{ program, events, program_sha256, revisions, fatal }`,
  where `revisions` counts the revisions searched and `fatal` is null or
  `{ index, event, message }` for the event that failed; or
  `{ program, events, error }` for a ledger whose program does not load;
- `found`: per revision, in ledger, series and revision order:
  `program`, `events`, `program_sha256`, `series`, `revision`, `current`,
  `value`, `status` (`in force`, `reopened` or `superseded`, as `explain`
  says), `grounds`, `retained`, `retention_reasons` where any, `withdrawn`
  (evidence in its grounds withdrawn since), `decided` (`{ sequence, event }`
  of its commit, or null), `outcome` (as `explain` gives it, or null),
  `sources` (each grounds evidence's source text, or null), `history`, and
  `matched`: for each word, the terms it matched as `KIND TEXT`.

Text output shows each found revision with the lines of the example: what it
is based on, when it was decided, each later change, evidence withdrawn
since, how it turned out, and where each word matched; then
`N decisions found in M ledgers.` A revision in force on withdrawn evidence
says so: it still stands, but on something since taken back.

Exit status: 0 after the search, also when nothing matched; 1 when an event
failed fatally in a ledger (its decisions are of the session before it, and
the text says which ledger); 2 when the directory cannot be read, holds no
ledger, a ledger's files cannot be used or its program does not load (the
others are still searched), or no word is given.

## 4. Through MCP

The `caveat_find` tool takes `source` (one program), `records` (1 to 64
`{ name, events }`, unique non-empty names, at most 1,000 events each and
1 MiB in all) and `query`. Each record is a ledger of that program, its
`name` standing for the program's path: the report's `events` is the record's
name, `program` is `inline.cav`, and query words match the name. Its report
is the `caveat-find/0.1` object with `program` added, inside the authoring
result (`exitCode` as in section 3; 2 when the source does not load). Like
the other tools it reads no files.

## 5. In the kit

`kit/lib/ledgers.mjs`, typed in `kit/lib/ledgers.d.mts`, not a package export
while this profile is a draft:

| Function | Returns |
| --- | --- |
| `queryWords(query)` | the query's words |
| `findDecisions(open, ledgers, query)` | the `caveat-find/0.1` report; `ledgers` as for `reviewLedgers`, with an optional `name` that words match instead of `program` |
| `formatFound(report)` | the text output |

## What it is not

- It is not semantic search. Words match names and declared texts, not
  meaning: an agent asking about "returns" does not find `refund`. Good
  evidence and caveat names, and source texts that say where evidence came
  from, are what make past decisions findable.
- It does not judge a decision or say which one to follow. It shows what was
  decided and on what; deciding again is the agent's.
- Each ledger is replayed in full, once. It is for looking back before a
  decision, not a hot path.

## Implementation

- `kit/lib/ledgers.mjs` and `kit/lib/ledgers.d.mts`: `queryWords`,
  `findDecisions`, `formatFound`.
- `kit/bin/caveat.mjs`: `find`.
- `kit/lib/authoring.mjs` and `kit/lib/mcp.mjs`: `caveat_find`.
- `kit/test/find.test.mjs`: matching, standing, a fatal event, a program that
  does not load, and the command over a directory; `kit/test/authoring.test.mjs`
  and `kit/test/mcp.test.mjs`: the tool, its argument checks and limits;
  `scripts/test-mcp-client.mjs`: the tool through the official client against
  the packed kit.

## Changes

- 2026-10-27: drafted and implemented for 1.5.
