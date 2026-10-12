# The release gate

This is a synthetic policy for a study, not advice about releasing software. All policy
belongs in the program; the host only sends events and reads the view.

### Inputs

| Event | Payload | Meaning |
| --- | --- | --- |
| `propose` | `head`: integer in `1..999` | The head now under consideration |
| `check` | `run`: integer in `1..999`; `passed`: `0` or `1` | A CI run's result |
| `review` | `approved`: `0` or `1`; `relayed`: `0` or `1` | A reviewer's verdict. `relayed` means a bot posted it on the reviewer's behalf |
| `scan` | `findings`: integer in `0..99` | The security scanner's result |
| `approve` | `head`: integer in `1..999` | The release manager's grant to merge that head |
| `decide` | none | Make a merge decision |
| `recheck` | none | The latest check's run was re-queued as flaky: its result is withdrawn |
| `db_aged` | none | The scanner's database is found stale |
| `db_refreshed` | none | The scanner's database was refreshed |

An event with an unknown name, a missing or extra parameter, or a parameter
that is not an integer in its range (a fraction is refused) is refused. So is
every event the rules below refuse. A refused event changes nothing: no
observation, caveat, decision or view value, and the session's sequence does
not advance. Refusal wording is up to the author.

### What the program knows

- Every accepted `check`, `review`, `scan` and `approve` is a distinct
  observation, identified in this study by the 1-based line of the event in
  the history file that recorded it (`#3` is the observation recorded by line
  3). Earlier observations are retained; a later one never overwrites an
  earlier one.
- A check with `passed` 1 supports the claim `ready`; with `passed` 0 it
  opposes it. The program accepts at most **eight** checks; a ninth is refused.
- A review with `approved` 1 supports `ready`; with `approved` 0 it opposes it.
  A review with `relayed` 1 carries the caveat `secondhand` (consequence
  `material`) from the moment it is recorded; a first-hand review carries no
  caveat. At most **four** reviews; a fifth is refused.
- A scan with `findings` 0 supports `ready`; otherwise it opposes it. At most
  **four** scans; a fifth is refused.
- An `approve` records a grant for its `head`. At most **four**; a fifth is
  refused. A grant is an observation too, identified by its line, but it is
  never something a decision rests on.
- `propose` sets the current head. It is accepted whenever its payload is
  valid; it changes no decision and no observation.
- The caveat `deps_unreviewed` (consequence `low`) is declared and is carried
  by no observation: the team has chosen to merge without a dependency review.

### The decision

The decision series is `merge`, with capacity four. "The latest check" is the
most recent check that is not withdrawn; "the latest review", "the latest
scan" and "the latest grant" are the most recent of each. `decide`:

- is refused unless at least one check, one review and one scan have been
  recorded, the latest check exists, and the latest of each supports `ready`;
- is refused while a merge decision is in force; after a decision, the next
  `decide` is accepted only once that decision has been reopened;
- is refused when no grant has been recorded, or when the latest grant is not
  for the current head; the grant is what **permitted** the decision, and it
  is not part of what the decision rests on;
- is refused once four decisions have been made;
- otherwise commits a decision whose **value is the current head** and which
  **rests on exactly three observations: the latest check, the latest review
  and the latest scan** at that moment, with whatever caveats those three
  carry at that moment, and which **retains `deps_unreviewed`** by choice.
  What a decision rests on, and the caveats it was made with, are frozen when
  it is made; later observations, caveats and take-backs never change them.

### Corrections

- A later check with `passed` 0, or a later scan with `findings` above 0,
  reopens the decision in force, if any, citing that observation. The reason
  text (below) becomes `check failed` or `scan found issues`.
- **Withdrawal.** If an observation that the decision in force rests on is
  withdrawn, that decision is reopened; reason text `check withdrawn`.
  Withdrawing an observation that no decision in force rests on, or a grant,
  reopens nothing. Nothing is erased: a withdrawn observation stays in the
  record as withdrawn. The only event that withdraws anything is `recheck`:
  it withdraws the most recently recorded check, with the re-queue as its
  reason, and is refused when no check has been recorded or when that check
  is already withdrawn.
- `db_aged` qualifies the latest scan with the caveat `db_stale` (consequence
  `material`). It is refused when no scan has been recorded or when the
  latest scan already carries `db_stale`. A decision already made keeps the
  caveats it was made with; if the decision in force rests on that scan, the
  same event reopens it; reason text `scan database stale`.
- `db_refreshed` takes `db_stale` back from every scan that carries it. It is
  refused unless the latest scan carries `db_stale`. It reopens nothing,
  decides nothing and changes no view value; a new `decide` may then rest on
  the same scan without the caveat.
- While the latest decision is reopened, a further failing check, scan with
  findings, `recheck` or `db_aged` records its observation or caveat as above
  but reopens nothing and leaves `reason` as it was. `propose` with a new head
  and a new grant reopen nothing.

### The view

After every event, the host reads:

```js
{ status, head, revisions, reason }
```

- `status`: `undecided` before any decision; `merge` while the latest decision
  is in force; `reopened` while it is reopened.
- `head`: the current head, `0` before the first `propose`.
- `revisions`: the number of merge decisions made so far.
- `reason`: `""` before any decision; `ready` when a decision is in force; on
  reopening, one of `check failed`, `scan found issues`, `check withdrawn`,
  `scan database stale`, unchanged until the next decision.

Nothing else is required. Whatever the program keeps beyond this is the
author's choice.

### Interface

A Caveat program is `gate.cav`. The host is the package's own session; a
history line `{"event": "check", "payload": {"run": 7, "passed": 1}}` is sent
as that event, and the view is the displayed values `gate.status`,
`gate.head`, `gate.revisions` and `gate.reason`. A line is refused when the
runtime returns a rejected outcome of any origin (policy, input, evaluation or
limit); a fatal outcome fails the case. A TypeScript program is `gate.ts`,
exporting `createPolicy(): { dispatch(line), view() }`, where `dispatch`
receives the parsed history line, returns normally when the event is accepted
and throws when it is refused, leaving the view unchanged.

A reviewer who has not seen your work will later have to establish, from
what you hand over, what each decision rested on.
