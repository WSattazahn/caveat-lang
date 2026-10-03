# Restore acceptance and host trust

Status: focused design review for rc.10. This documents the current boundary and
proposes a separate validation follow-up; it changes no runtime or save schema.

## What acceptance establishes

Reactive restore loads the supplied source, checks the save's schema and source
fingerprint, then applies the saved delta and recomputes bindings. Its checks
establish the specific structural and consistency properties listed in the
[save contract](../spec/caveat-save-0.1.md#restore-validation). They do not establish
that the saved account is reachable through a complete historical event sequence.

| Property | Current runtime check | Host responsibility |
| --- | --- | --- |
| Format and program matching | Required fields, known schema/fields and matching source fingerprint | Select the intended source and runtime; the fingerprint is an identifier, not authentication |
| Saved graph | Known endpoints, allowed relation kinds and endpoint kinds; additional checks for recorded observations and commitments | Establish whether the supplied account is one the host trusts |
| Current values | Present states are declared, finite and in range; grounds fit lineage | Preserve the saved delta; omitted entries deliberately retain source initialization |
| Historical decisions | Journal, bases, grounds, revisions and graph agree under the documented checks; selected event/effect reachability is checked | Establish that the recorded inputs, guard outcomes and permissions actually occurred |
| Persistence | The runtime's original save text preserves numeric values, including signed zero | Store and return that string intact; define ownership, integrity and freshness requirements |

The implementation is in [reactive_save.rs](../runtime/src/reactive_save.rs):
`restore_json`, `apply_save`, `restore_graph`, `restore_states` and
`check_saved_journal` perform the relevant checks. The source fingerprint
is computed by `source_identity` in [reactive.rs](../runtime/src/reactive.rs).
`event_can_change_decision` intentionally ignores historical guard values,
which the saved delta cannot reconstruct.

A single added `["phantom", "qualifies", "sensor"]` relation can pass the
current graph checks when its endpoints have the required kinds, even if no
source operation can add that pair. Subsequent evaluation can use the injected
qualification. `origin: "live"` identifies current graph membership, including
restored edges. It does not attest where an edge came from.

Deleting a changed state entry can likewise restore its initializer while a
prior commitment keeps its frozen value and grounds. Requiring every declared
state would reject legitimate sparse saves and would not establish their origin.
Parsing and re-encoding a save in JavaScript can change `-0` to `0`; the runtime
cannot recover a sign the supplied text no longer contains.

The executable regression fixtures characterize these current boundaries. If a
future schema or validated compatibility policy tightens acceptance, update the
corresponding fixture and contract together rather than treating acceptance of
an edited save as a permanent requirement.

## A bounded next validation step

A useful next question is whether each restored relation could be created by
some mechanism in the loaded source. A conservative source-capability check
could reject impossible pairs without claiming their events actually occurred.
It needs an explicit compatibility decision for saves accepted by schema 0.1.

Checking only written `qualify` rules would be insufficient. The analysis must
cover declared qualifications, reachable symbol-instantiated procedures,
reading-template inheritance, renewable occurrences, delayed qualifications,
and withdrawal's generated qualification. Pending scheduled records need the
same analysis: checking only present graph edges leaves a later insertion path.
Use the compiled effect graph and occurrence/template relationships, not a text
search. Keep uncertainty conservative and define which unsupported cases are
accepted or refused before implementation.

Acceptance cases for that separate change should include:

- Refuse an injected immediate relation and pending schedule when no source
  mechanism can create the pair, with an explicit restore error.
- Accept genuine saves from each direct, procedure, inherited, renewal,
  withdrawal and delayed path; preserve occurrence identity and historical grounds.
- Keep a syntactically possible effect distinct from proof that its guard ran.
- Retain sparse-state semantics and exact-text signed-zero behavior.
- Run native, full/lean WASM, host and installed-package cases, including a
  successful subsequent event and save/restore cycle for each accepted fixture.

No stronger validator is implemented by this review.

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
