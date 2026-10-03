# CAVEAT Dependents 0.1

The kit's reverse explanation reports what rests on an evidence occurrence,
a reading stream, evidence template, or caveat. It reads a snapshot without
running a program or changing any decision.

The public `dependents(snapshot, subject)` function is exported from
`caveat-lang/explain`. The CLI `dependents PROGRAM NAME [EVENTS]`, the Serve
`dependents` operation and the inline authoring operation use the same report.

## Matching and roles

An evidence name selects itself and its renewable occurrences. A reading stream
selects its readings; its evidence template selects those readings too. A
concrete occurrence such as `checks@1` selects that occurrence. A caveat selects
records whose corresponding caveat channel contains it. An unknown subject is
an error; a declared subject without dependents has empty result arrays.

The report contains decision revisions, decision changes, current state values,
and displayed values. Each match records its exact `via` names. Decisions and
values distinguish content grounds from possible influence through lineage;
displayed values distinguish citations from lineage. A grant matching the
queried evidence is labeled `permission` when the query does not also match the
decision's grounds. This precedence selects one reported basis for each item.

The revision status remains `in force`, `reopened`, or `superseded`, according
to the decision series and its journal. A withdrawal neither invents a contrary
observation nor changes that status. Reopening requires an authored policy.

## Withdrawal information

The schema remains `caveat-dependents/0.1`. The following fields are additive:

- Top-level `withdrawals` lists withdrawn occurrences matching the subject,
  even if no decision, change, value or display depends on them.
- Each decision, change, value and display has a `withdrawn` array. It contains
  only withdrawal records for the queried evidence in that item's reported
  basis. A stream query can therefore show a withdrawal on an earlier revision
  while a later revision grounded on a fresh reading has an empty array. A
  direct query for the old reading can still show its lineage influence there.

Each record is `{ evidence, because, sequence, event }`: the exact withdrawn
occurrence, reason occurrence, event sequence and event name from the snapshot's
withdrawal ledger. Arrays retain ledger order; a snapshot omitting the ledger
produces empty arrays. No historical grounds, permissions, readings or journal
entries are rewritten to add a withdrawal caveat.

For caveat queries, withdrawals are associated only with occurrences that the
snapshot explicitly records that caveat as qualifying. An item's annotation
also requires that evidence to occur in its selected grounds, citations,
lineage, or decision-change witnesses. An authored extra caveat does not invent
an evidence-qualification edge. Other withdrawn inputs are not added merely
because the same item depends on them. Querying the built-in `withdrawn` caveat
uses its actual qualification edges, with the same matching rules.

The human report names every matching withdrawal and includes its occurrence,
sequence, event and reason beside affected items. Its formatter accepts older
reports that omit these additive fields. An empty array means no matching
withdrawal was recorded; it does not authenticate evidence or grant permission
for an external action.

See [Withdrawal](caveat-withdrawal-0.1.md),
[Permission](caveat-permission-0.1.md), and
[Grounds](caveat-explanations-0.2.md) for the underlying contracts.
