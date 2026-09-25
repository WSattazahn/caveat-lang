# CAVEAT Permission 0.1

A decision can rest on evidence and still need someone's permission. In the
[agent ledger](../experiments/agent-ledger/README.md), "merge this pull
request" rests on its checks, but it may happen only with the user's go-ahead
for that commit. The language had no place for the go-ahead:
- a `reject` rule that let the merge through left no trace;
- folding the go-ahead into the decided value mixed why the merge was safe
  with who allowed it;
- the go-ahead's scope, which commit it covered, took hand-written rules.

```caveat
claim may_merge;
evidence go from "the user's go-ahead in chat";
readings approvals from go limit 16;
event approved commit id;
on approved sample approvals = commit supports may_merge;
on merge commit merge because enough using latest(checks)
    permitted by latest(approvals) for head;
```

## Syntax

```text
commit D because REASON [using V] permitted by EVIDENCE [retaining C, …];
commit D because REASON [using V] permitted by latest(STREAM) [for X] [retaining C, …];
permission_withdrawn(DECISION)                                  -- a predicate
```

`for X` needs a grant with a value, so it takes `latest(STREAM)` only. The
grant must name declared evidence or a declared reading stream; anything else
is an error when the program loads.

## Checked when the commit runs

The clause resolves its grant to a concrete occurrence:
- named evidence resolves to its current occurrence;
- `latest(STREAM)` resolves to the stream's current reading.

The commit is refused as `policy/not_permitted` when any of these holds:
- the grant is missing: the evidence is not observed, or the stream holds no
  reading yet;
- the grant is [withdrawn](caveat-withdrawal-0.1.md);
- with `for X`, the grant's value is not equal to X.

A refusal is a returned outcome under [Dispatch outcomes
0.1](caveat-dispatch-0.1.md), and the session stays usable. As with every
refusal, the whole event rolls back:
- earlier rules' effects;
- readings sampled in the same event, including the grant itself;
- new identifiers;
- any decision record.

Missing grants are recognized as such. Other failures keep their own
classification: an error while evaluating `using` or `for X`, a work limit,
or a history limit. They are never reported as a denial.

A bare expected rejection in a [scenario](caveat-scenarios-0.1.md) matches any
`policy` refusal, so it matches `not_permitted` too. Name the code to tell it
from the program's own `reject`: `{"origin": "policy", "code":
"not_permitted"}`.

## What a permitted commitment records

Each commitment made with the clause has a frozen **permission record**:
- `grant`: the concrete occurrence, such as `approvals@2`;
- `scope`: when `for X` was given, `{"granted": G, "required": X}`, the two
  values compared;
- `caveats`: the caveats the grant carried at that moment.

The decision journal's `committed` entry names the grant as `permitted_by`.

A permission is not what a decision is based on. The grant does not enter the
decision's [grounds](caveat-explanations-0.2.md). It controls whether the
decision happens, so the grant and the scope expression's dependencies enter
its lineage, as any control dependency does.

The record is frozen. Later grants, withdrawals and changes to X do not
rewrite it. `for X` is checked when the commit runs. It is not a lasting
guarantee: if the head changes afterwards, the decision still records
permission for the old head, and a program that must reconsider says so, as
the ledger's push rule does. A clause never falls back to an older grant or
to a broader one. A program that accepts broader permission names that grant
in its own `permitted by`, chosen by its own guard.

## Revocation

A grant is taken back with `withdraw`, as any observation is.
`permission_withdrawn(D)` is true when the grant recorded on D's current
revision has been withdrawn. It asks about that recorded grant, not the latest
one. It carries the withdrawal's reason, as `rests_on_withdrawn` does. Nothing
reopens by itself.

## Snapshot and save

The snapshot lists `commitment_permissions`, by commitment. A save holds the
same map, and leaves it out when it is empty, so a save made before this
profile restores with none. Restoring refuses:
- a record for an unknown commitment;
- a grant that is not known evidence;
- a scope with values that are not finite;
- caveats that are not declared;
- disagreement with the journal: each permitted commitment's `committed`
  entry must name its grant, and no entry may name a grant without a record.

## Not in this profile

This profile does not include:
- who is entitled to grant: Caveat does not authenticate sources;
- expiry: a scheduled caveat does not by itself withdraw a grant or refuse a
  commit, so enforcing expiry takes explicit rules;
- several permissions on one commitment;
- permission for effects other than `commit`.
