# Restore acceptance and host trust

Status: design review written for rc.10 and updated for rc.11, which
implements the source-capability check it proposed, and for rc.12, which
applies the same check to a caveat's attention. The save schema is
unchanged.

## What acceptance establishes

Reactive restore loads the supplied source, checks the save's schema and source
fingerprint, then applies the saved delta and recomputes bindings. Its checks
establish the specific structural and consistency properties listed in the
[save contract](../spec/caveat-save-0.1.md#restore-validation). They do not establish
that the saved account is reachable through a complete historical event sequence.

| Property | Current runtime check | Host responsibility |
| --- | --- | --- |
| Format and program matching | Required fields, known schema/fields and matching source fingerprint | Select the intended source and runtime; the fingerprint is an identifier, not authentication |
| Saved graph | Known endpoints, allowed relation kinds and endpoint kinds; each `qualifies` pair one the source can make; each caveat's attention one its examinations and budget can leave; additional checks for recorded observations and commitments | Establish whether the supplied account is one the host trusts |
| Current values | Present states are declared, finite and in range; grounds fit lineage | Preserve the saved delta; omitted entries deliberately retain source initialization |
| Historical decisions | Journal, bases, grounds, revisions and graph agree under the documented checks; selected event/effect reachability is checked | Establish that the recorded inputs, guard outcomes and permissions actually occurred |
| Persistence | The runtime's original save text preserves numeric values, including signed zero | Store and return that string intact; define ownership, integrity and freshness requirements |

The implementation is in [reactive_save.rs](../runtime/src/reactive_save.rs):
`restore_json`, `apply_save`, `restore_graph`, `restore_states` and
`check_saved_journal` perform the relevant checks. The source fingerprint
is computed by `source_identity` in [reactive.rs](../runtime/src/reactive.rs).
`event_can_change_decision` intentionally ignores historical guard values,
which the saved delta cannot reconstruct. `restore_records` likewise requires
each saved cue and effect to be one a rule effect of the saved last event
could make, through `reached_effects`, without evaluating its conditions.

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
of qualifications: could some sequence of this program's events have left
it? It refuses, with an explicit restore error:

- `deferred` and `examining`, which only a game session sets;
- `examined` in a program with no attention budget;
- `examined` on a caveat no `examine` reaches, in a rule or in a procedure a
  rule calls, procedures specialized for the caveat they are passed included;
- examined caveats whose least examination costs, one examination each, sum
  to more than the budget's `spent`.

The policy is conservative. Guards are not evaluated, and an examination is
counted once at its cheapest, so attention the program could have left is
accepted though no event of the session left it: with `stale` and `deep`
each examinable, a save that spent enough for both can mark both examined
after only one was. Removing an attention entry restores too, leaving that
caveat unexamined, just as removing a `qualifies` relation restores without
it. Neither check establishes what the session did.

The fixtures in [restore_attention.rs](../runtime/tests/restore_attention.rs),
[save_restore.rs](../runtime/tests/save_restore.rs) (every attention value on
every caveat of the fuzz's played save), the
[restore boundary fixtures](../kit/test/restore-contract.test.mjs) and the
installed-package check in
[test-kit-package.mjs](../scripts/test-kit-package.mjs) refuse each forgery
above, restore genuine saves after no, one and several examinations by rule
and procedure and play on and restore again, and accept attention the
program could have left though its events did not.

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
