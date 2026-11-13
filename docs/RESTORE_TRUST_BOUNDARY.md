# Restore acceptance and host trust

Status: design review written for rc.10 and updated for rc.11, which
implements the source-capability check it proposed, for rc.12, which
applies the same check to a caveat's attention and to the keys of the
qualification tables and the withdrawal record's event, and for rc.13, which
applies it to the caveats a saved provenance carries and to memberships
without their records. The save schema is unchanged.

## What acceptance establishes

Reactive restore loads the supplied source, checks the save's schema and source
fingerprint, then applies the saved delta and recomputes bindings. Its checks
establish the specific structural and consistency properties listed in the
[save contract](../spec/caveat-save-0.1.md#restore-validation). They do not establish
that the saved account is reachable through a complete historical event sequence.

| Property | Current runtime check | Host responsibility |
| --- | --- | --- |
| Format and program matching | Required fields, known schema/fields and matching source fingerprint | Select the intended source and runtime; the fingerprint is an identifier, not authentication |
| Saved graph | Known endpoints, allowed relation kinds and endpoint kinds; each `qualifies` pair one the source can make; each caveat's attention one its declarations, or its examinations and budget, can leave; additional checks for recorded observations and commitments | Establish whether the supplied account is one the host trusts |
| Current values | Present states are declared, finite and in range; grounds fit lineage | Preserve the saved delta; omitted entries deliberately retain source initialization |
| Historical decisions | Journal, bases, grounds, revisions and graph agree under the documented checks; selected event/effect reachability is checked | Establish that the recorded inputs, guard outcomes and permissions actually occurred |
| Persistence | The runtime's original save text preserves numeric values, including signed zero | Store and return that string intact; define ownership, integrity and freshness requirements |
| Size | None: restore takes a state save of any length (D070), and its time and memory grow with the save, faster in the steps the save contract lists, among them a decision journal whose causes share caveats (D039) and a windowed history that holds many records (D071). A restore that runs out of WebAssembly memory traps the instance and every session on it; it is not a refusal | Bound the size of a save from a party the host does not trust before restoring it, as the MCP server does at 2 MiB |

The implementation is in [reactive_save.rs](../runtime/src/reactive_save.rs):
`restore_json`, `apply_save`, `restore_graph`, `restore_states` and
`check_saved_journal` perform the relevant checks. The source fingerprint
is computed by `source_identity` in [reactive.rs](../runtime/src/reactive.rs).
`event_can_change_decision` intentionally ignores historical guard values,
which the saved delta cannot reconstruct. `restore_records` likewise requires
each saved cue and effect to be one a rule effect of the saved last event
could make, through `reached_effects`, without evaluating its conditions.
`check_last_event_spending`, run after every other check, requires the costs
of the last event's `examine` effects, a caveat examined twice counting
twice, added to what the declarations spent, to fit the budget's `spent`
(D053).

Since rc.11, a single added `["phantom", "qualifies", "sensor"]` relation is
refused when no mechanism of the loaded source can add that pair, and accepted
when one can, whether or not its guard held: a source-reachable effect is not
proof that its event ran. `origin: "live"` identifies current graph membership,
including restored edges. It does not attest where an edge came from.

Deleting a changed state entry can restore its initializer while a
prior commitment keeps its frozen value and grounds. Requiring every declared
state would reject legitimate sparse saves and would not establish their origin.
Parsing and re-encoding a save in JavaScript can change `-0` to `0`; the runtime
cannot recover a sign the supplied text no longer contains.

The executable regression fixtures characterize these current boundaries. If a
future schema or validated compatibility policy tightens acceptance, update the
corresponding fixture and contract together rather than treating acceptance of
an edited save as a permanent requirement.

## The source-capability check (rc.11)

Restore asks whether each restored `qualifies` relation, and each pending
scheduled qualification, could be created by some mechanism in the loaded
source. It refuses impossible pairs without claiming that possible ones
occurred. The compatibility policy for schema 0.1 is conservative: refuse a
(caveat, evidence) pair only when no mechanism can create it, and accept when
one can. Saves the runtime writes are therefore accepted by construction, and
genuine saves written by every published rc from rc.3 to rc.10 were checked
to restore.

`qualification_sources` in [reactive_save.rs](../runtime/src/reactive_save.rs)
enumerates the mechanisms from the compiled program, not from text: declared
qualifications; `qualify` effects in rules and in the procedures they reach,
including symbol-instantiated procedures, which compile to concrete names at
each call site; reading-template inheritance, so a stream's readings take
whatever can qualify its template; renewable occurrences, which carry their
evidence's declared caveats; delayed qualifications; and withdrawal's
generated `withdrawn` qualification. An occurrence is checked as the stream or
renewable evidence the source names. Pending scheduled records are checked
against `qualify ... after` effects only, so a schedule cannot be inserted for
a pair that only an immediate path makes.

The fixtures in
[restore_capability.rs](../runtime/tests/restore_capability.rs) and the
[restore boundary fixtures](../kit/test/restore-contract.test.mjs):

- refuse an injected or retargeted immediate relation and a retagged pending
  schedule with an explicit restore error;
- restore genuine saves from each direct, procedure, inherited, renewal,
  withdrawal and delayed path, then play on and save and restore again with
  occurrence identity and historical grounds intact;
- accept a relation a guarded rule could make though its guard never held,
  keeping a syntactically possible effect distinct from proof that it ran;
- keep the sparse-state and exact-text signed-zero cases unchanged.

## Attention (rc.12)

A save's `graph.attention` once restored with a kind check only, so a save
could mark any caveat `examined` without spending its budget, and
`examined(...)` guards then held as if the examination had happened (findings
123, 188–191). Restore now asks the same question of attention that it asks
of qualifications: could this program's declarations, or some sequence of
its events, have left it? It refuses, with an explicit restore error:

- `deferred` and `examining`, which no reactive event leaves, unless the
  declarations leave the caveat so (see below);
- `unexamined` on a caveat the declarations leave otherwise, since no
  reactive event leaves a caveat unexamined (D046);
- `examined` in a program with no attention budget;
- `examined` on a caveat no `examine` reaches, in a rule or in a procedure a
  rule calls, procedures specialized for the caveat they are passed included;
- examined caveats whose least examination costs, one examination each, sum
  with what the declarations spent to more than the budget's `spent`;
- `examine` effects of the last event whose costs, each paid in full, sum
  with what the declarations spent to more than the budget's `spent`
  (D053).

A declaration leaves a caveat `examined`, `examining` or `deferred` as the
program loads (`examine stale cost 1;`, `examine stale;`, `defer stale;`).
Since D043 the attention the declarations leave is accepted as it stands,
and a caveat they leave `examined` adds nothing to the least cost, since no
event need have examined it, unless its examination record shows an event
examined it again (D046). Since D046 what the declarations spent is counted
with the least costs, and an examination record needs a rule or procedure
that examines its caveat, since a declaration leaves none. Before D043
restore checked that attention as if an event had left it. It refused
every save that still held a declared `examining` or `deferred`, or a
declared `examined` that no rule examines, and a declared `examined`
that a rule could examine whenever `spent` fell short of the least costs:
with `examine stale cost 1;` and a rule that examines `stale` for 3, the
save written before any event was refused.

The policy is conservative. Guards are not evaluated, and an examination is
counted once at its cheapest, so attention the program could have left is
accepted though no event of the session left it: with `stale` and `deep`
each examinable, a save that spent enough for both can mark both examined
after only one was (since rc.13, with the examination record each
examination leaves). Only the last event's examines are also counted at the
costs it reports, all together (D053). That sum is checked apart from the
least costs, not added to them, so neither check refuses a save that meets
each, though no session spends so little: where one event examines `c` for
5, another examines `a` and `b` for 3 each and a third examines both for 1
each, a session that runs the first and then the second has spent 11, and a
save after them that says 7 restores, since the last event's 6 fits it and
so do the least costs, 1 each for `a` and `b` and 5 for `c`. Removing an
examination an event made restores too, leaving that caveat at the
attention the program loads it with (unexamined, unless the declarations
leave it otherwise), just as removing a `qualifies` relation restores
without it; since the table-key checks below, its examination record must
go with it. Neither check establishes what the session did.

The fixtures in [restore_attention.rs](../runtime/tests/restore_attention.rs),
[d043_declared_attention_restores.rs](../runtime/tests/d043_declared_attention_restores.rs),
[d046_declared_attention_forgeries_refused.rs](../runtime/tests/d046_declared_attention_forgeries_refused.rs),
[d053_last_event_examines_fit_spent_together.rs](../runtime/tests/d053_last_event_examines_fit_spent_together.rs),
[save_restore.rs](../runtime/tests/save_restore.rs) (every attention value on
every caveat of the fuzz's played save), the
[restore boundary fixtures](../kit/test/restore-contract.test.mjs) and the
installed-package check in
[test-kit-package.mjs](../scripts/test-kit-package.mjs) refuse each forgery
above, restore genuine saves after no, one and several examinations by rule
and procedure and play on and restore again, and accept attention the
program could have left though its events did not.

## Qualification tables and withdrawals (rc.12)

The observation, examination and reopening qualification tables, the
predicate guards kept for skipped effects, and the withdrawal record's
`event` once restored with their values checked and their keys trusted
(findings 86, 89, 96, 118–120). Restore now refuses a record that could not
belong where it is: an observation of evidence neither observed nor made by a
renewal, or on a name that no `reveal`, `sample` or `renew` in a rule, or in a
procedure a rule reaches, leaves a record on (D052), an examination of a
caveat the save leaves unexamined or that no rule or procedure examines
(D046), a reopening of a commitment no `reopens` relation names, a predicate
guard on a name of the wrong kind, and a withdrawal whose event reaches no
`withdraw` of that evidence for that reason, in a rule or a procedure a rule
calls. Evidence a declaration relates to a claim is observed as the program
loads, and has no observation record until an event reveals it with a stance
its declarations do not give it. A renewal leaves one on the occurrence it
makes only when its guard rests on something, which needs a condition or call
argument on its way that reads a state, the graph or a history.

The policy is the same as above. A predicate guard on a name of the right
kind restores though no skipped effect kept it, an observation record
restores on a name a reached effect leaves records on though none left one
there (whether a renewal's guard can rest on anything is read from the
source, from what the conditions and call arguments on its way read, not
from what they held), and a withdrawal by an event that could withdraw that
evidence restores whether or not its guard held.
Round 7's F263 found that removing a withdrawal record together with its
`withdrawn qualifies` relation restores, so `withdrawn(...)` then reads
false; that is still accepted. A save without a record the session made is
one the source could have written had the event not happened, and restore
does not establish which events happened. The fixtures are in
[restore_tables.rs](../runtime/tests/restore_tables.rs) and
[d052_observation_record_needs_an_observing_rule.rs](../runtime/tests/d052_observation_record_needs_an_observing_rule.rs).

## Caveats in saved provenance (rc.13)

Each saved provenance (a state's lineage and grounds, a commitment's basis
and grounds, a qualification record, a predicate guard, a reading, a journal
entry) once restored with its caveats checked only as declared names. A
caveat written into an observation, examination or reopening record, a guard
or a basis then restored live and reached `using` reads and the
`examined()`, `reopened()` and `committed()` guards (F330, F333,
F335–F337). Restore now refuses a caveat that no mechanism of the loaded
source can attach to that provenance.

`caveat_sources` in [reactive_save.rs](../runtime/src/reactive_save.rs)
derives, from the compiled program, the caveats a value resting on each
evidence can carry: what can qualify that evidence (the pairs
`qualification_sources` allows, and the extra caveats a `qualified(...)`
names for it), what a declaration qualifies on a claim the evidence can bear
on, and what qualifies those caveats in turn. A value can also carry a caveat
with no evidence: `examined(...)` carries the caveat it reads and what
qualifies it, and a commitment carries its `retaining` caveats. A caveat is
accepted when it is one of those, or one some evidence of the provenance can
carry; for grounds, evidence of the lineage or basis they lie within.

The policy is the same as above. Guards are not evaluated, so a caveat that
a rule whose guard never held could attach restores, and a caveat that some
evidence of a provenance could carry restores in that provenance whether or
not it came from that evidence. A caveat the source attaches only to other
evidence, or to nothing, is refused. Like the other checks, this bounds a
save by what the program could have written; it does not establish what the
session did. The fixtures are in
[restore_caveats.rs](../runtime/tests/restore_caveats.rs), the save fuzz in
[save_restore.rs](../runtime/tests/save_restore.rs) (a caveat nothing
attaches, added to every provenance of the played save) and the
[restore boundary fixtures](../kit/test/restore-contract.test.mjs).

## Departure markers and names in saved provenance (D054, D061)

When a record of a windowed history departs, each provenance that named it
keeps a marker of that history in the name's place
([Lineage Compaction 0.1](../spec/caveat-lineage-compaction-0.1.md)).
Restore checked a marker's history, range, count and date against what the
save says departed, but not which provenance held it. A marker copied into a
provenance its program never carries that history into restored, and the
caveats the departed records carried came with it: a commitment on a stream
sampled without reading `a` could keep a caveat of `a`'s departed readings
(D054). Restore now refuses a marker or a record's name in a provenance,
and a record that an outcome, a hold, a deferral, a `reopens` relation an
event added or a withdrawal names as its cause, when no rule, procedure,
reopening trigger or declaration of the loaded source can carry that
history's records into the part of the session that holds it. A relation a declaration makes is the program's: a
save holds none, and restore makes it again from the source, so its cause,
which can be a caveat, a claim or a commitment, is not read.

D054 applied this only to a program with a window, where a record can
depart. A program without one keeps no marker, but its provenances and causes
name records all the same, and the caveats and grounds that follow a name
came with a forged one: a lineage citing evidence that no rule carries into
its state restored, and later events kept it (D061). Restore applies the
bound to every program now. One part is wider in a program without a window:
a procedure's parameter there grounds on what its argument carries, its
whole lineage, as well as on what the argument gives grounds, because
releases before rc.12 grounded a parameter so (F268), and only such a program
can have a save that old, since windows came in rc.15. The rc.11 save that
`runtime/tests/parameter_grounds.rs` restores holds such grounds.

`history_reach` in [reactive_reach.rs](../runtime/src/reactive_reach.rs)
derives the bound once per loaded program, when restore first needs it, and
in debug builds as the program loads. Each part of what a session holds has
its own bound, from what writes it: a state's lineage, and its
grounds, which take what its initializer's and its sets' values or citations
ground on, as the runtime evaluates grounds, and so never the guard of the
rule that sets the state, though what a value reads can hold the guard of
the rule that wrote it; a stream's readings, their observation and
`observed` records, and its selection, which only a skipped sample's guard
writes; an evidence's observation and `observed` records; the `withdrawn`
records of an evidence or of a stream's readings, which take the guards of
its skipped withdrawals; a series' or a commitment's bases, its grounds
(what its commits use, cite and give as reasons, as they ground on), its
outcomes and the evidence they name (only what its resolves select), its
reopening records and the evidence that reopened it or that a hold defers
(only what its `reopen` effects select, the stream a reopening trigger
watches and the evidence a declaration reopens it with), the reasons of its
hold (only what its `hold` effects select), its `committed` record, which
takes the guards of a commitment's skipped commits (a series' skipped commit
writes its selection instead), its `reopened` record, which takes the guards
of its skipped reopenings, and its selection, which only a skipped commit's
guard writes; the guards scheduled on each evidence or stream, which take
what its `qualify ... after` effects evaluate; a caveat's records; the
withdrawals' reasons (only what a withdrawal's `because` selects); and each
cue, which takes the guards of its own emits. What a `release` selects goes
into the reopening record, and the series keeps none of it as a cause.

What an effect evaluates (its guard, its values, what it cites or selects, a
call's arguments) flows into what the effect writes, a procedure's steps run
on what any call of it passes, a reopening trigger reads its condition and
its stream, and the declarations carry what they leave. A read carries the
parts the runtime reads for it, and no other part of what it names: a read
of evidence or of a stream, the record's name and what its observation and
`observed` records can hold, and of a stream also its readings and its
selection; of a series or a commitment, a value read the bases, `grounds_of`
the grounds, `outcome` and `resolved` the outcomes, `committed` and
`in_basis` the bases, the reopening records and the `committed` records, and
`reopened` and `on_hold` those and the `reopened` records, each with the
selection; and a withdrawal predicate, the `withdrawn` records of what it
asks about and the reason of each withdrawal, read through its records, and
nothing else of what it asks about. `in_basis` carries nothing of the
evidence it looks for, and `qualified` nothing of a caveat it attaches by
name. In grounds, `qualified`, `observed` and `carries` give only the name
of the evidence they ask about, and a cause an outcome, a series or a
withdrawal keeps is the name its `because` selects. Two reads take a name
from what the session holds and read it through its records, so there the
bound follows each name a part can keep to what that name's records can
hold: a `because caveated(STATE, CAVEAT)` reads, with the state's lineage,
each evidence the state's grounds name, and `reopened` and `on_hold`, and
with them the next revision's basis, read each evidence that reopened the
commitment again, as its records stand then. A part's content also flows
into what takes it whole: grounds into their lineage or basis, a cause into
its outcome or reopening, a selection into what inherits it, and a reopening
and a skipped one's guard into the next revision's basis. The guard of a
qualification or a withdrawal, and a scheduled guard when it applies, joins
each state lineage and each observation, examination or predicate record
that lists the record it qualifies then; readings, bases, outcomes,
reopening records, grounds, selections, scheduled guards and cues keep what
was read when they were written, and no record or guard joins them later: a
qualification adds only caveats to a state's grounds. So a part can hold a
history's records, and its marker, only when these flows lead to it from a
read of that history or a name of one; a reading also holds its own name. A
journal entry's name never enters a provenance, so a marker of the journal
is refused anywhere.

The bound has to contain everything the runtime can write, and where the
runtime reads less than a whole part it is knowingly wider than the read. A
value read of a stream, and `has_sample`, is given what the readings'
observation and `observed` records can hold, though the runtime reads only
the readings and the selection for them. What `reopened` finds in the
records of what reopened a commitment is bounded with the reopening records,
so `committed` and `in_basis`, which carry those, are given it too.
`outcome`, `resolved` and `in_basis` are given the series' selection, which
the runtime reads for `resolved` only while the series has no outcome, and
for the other two never; a selection flows into the outcomes and the bases
anyway, so this widens no bound. A withdrawal predicate is given the reason
of every withdrawal, whichever it finds. And `withdrawn(latest(S))` is given
the `withdrawn` records of `S`'s readings, where the guard of a skipped
withdrawal of the current reading is kept, though the runtime does not read
them for it.

The policy is the same as above. Guards are not evaluated, so a marker that
a rule whose guard never held could carry restores, and a marker some rule
could carry into a part restores there whether or not one did. A marker of a
history that nothing carries into the part that holds it is refused, even
where another part of the same state, stream or series can carry it: a
commitment's grounds, for one, can hold a marker, and with it the caveat its
journal entry records, only where what its commits use, cite or give as
reasons can carry it. Like the other checks, this bounds a save by what the
program could have written; it does not establish what the session did.

Each part's content moves only into parts whose bound contains its own, so a
session restored from a save this check accepts stays within the bound as it
plays on, and the saves it writes pass the check. Debug builds rely on that:
after a program loads and after every event they check that each marker,
name and cause a session holds is within the bound, so a flow the bound
misses fails any test that plays it, rather than refusing a save the runtime
wrote. Since restore reads the bound in every program (D061), every session
it makes is within it, and the checks apply to each. The fixtures are in
[d054_markers_no_rule_carries_refused.rs](../runtime/tests/d054_markers_no_rule_carries_refused.rs)
and [d061_windowless_names_bounded.rs](../runtime/tests/d061_windowless_names_bounded.rs).

## Memberships and their records (rc.13)

The qualification-table check above runs from the record to the graph: a
record must belong to a membership. Version Lab found the other direction
unchecked (F317–F322). A save with an observation, examination or reopening
record removed restored; the next genuine events wrote the record again, but
the graph's `reopens` and `relies_on` relations stayed as the edited save
left them, and no surface reported the skew. Every event that observes
evidence, examines a caveat or reopens a commitment leaves its record, an
empty one when nothing guarded it, so restore now refuses observed evidence,
an examined caveat or a reopened commitment without its record. Evidence and
attention a declaration sets as the program loads have none and need none.

This is again a bound on what the program could have written, not proof of
what happened. A record removed together with its membership (the
examination with the caveat's attention, say) restores, as a save the
program could have written had that event not happened. The corpus run for
this check found the invariant held in every genuine save: 79,855 written by
this runtime from 151 programs, and those written by the published rc.3 to
rc.12 from the same event traces. The fixtures are in
[restore_membership.rs](../runtime/tests/restore_membership.rs).

## Establishing trust in a checkpoint

The host should decide which component may produce a checkpoint and which
component may replace it. A trusted store can preserve the exact save bytes.
Where saves pass through untrusted storage or clients, an authenticated envelope
can bind those bytes to the intended source, runtime, session and sequence; the
host must also define freshness or rollback policy. A digest supplied alongside
an equally editable save does not establish that origin. Designing keys,
permissions and verification is a separate host task.

An authoritative event log can support replay and comparison when exact inputs,
ordering, source and runtime are retained. Replay establishes what the supplied
inputs produce; trusting those inputs remains a separate requirement.
[GameSession](../spec/game-session-0.1.md#save-and-replay) already replays saved
selections, whereas reactive saves apply a delta. Their restoration mechanisms
must not be conflated. Neither mechanism authenticates external observations.

For rc.10, the completed action is a precise contract and an executable example
of preserving save text. Source-capability enforcement and authenticated host
persistence remain separately scoped work.
