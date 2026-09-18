# CAVEAT game session 0.1

`GameSession` is the generic bridge between a CAVEAT program, epistemic state,
physical action execution and a renderer. It does not recognize scenario names,
named destinations or particular player decisions.

## Execution boundary

A source `inspect` or `select` is an interaction boundary. Its written selection
is an authoring placeholder for batch evaluation; a game replaces it with the
player's choice. Initialization evaluates only statements before the first
boundary. An accepted selection evaluates that boundary and subsequent
statements up to, but excluding, the next boundary.

World declarations and all `display` declarations are available as static
presentation data. The world compiler validates a declaration-only program,
without evaluating future selections, charging future inspections or executing
their conditional effects. Live budget, graph, scenes, attention, commitments
and discoveries are derived exclusively from the executed prefix.

Consequently, a future invalid placeholder does not prevent initialization.
Invalid world declarations do prevent initialization. An invalid effect on the
selected execution path fails that interaction and leaves all state unchanged.
This API does not claim whole-program validation of every epistemic branch.

## Atomic application

An action must belong to the pending interaction, have a source action plan, be
available at the current logical place, satisfy open-barrier preconditions, and
fit the investigation budget when applicable. The session previews physical
effects and evaluates epistemic effects before publishing either. A failed
selection preserves the world, executed program, last feedback, turn count,
selection history and serialized save.

The action interpreter preserves opened entities across turns and returns its
ordered generic commands: `inspect`, `operate`, `open`, `move`, `observe`, `stay`.
`observe` is a physical event; its epistemic relation must also be authored with
`reveal` or `when_committed` in CAVEAT. Renderers must not invent that relation.

## Rust API

```rust
use caveat_runtime::game_session::GameSession;

let mut game = GameSession::from_source(source)?;
let before = game.snapshot()?;
let after = game.apply(selection)?;
let save_json = game.save();
let restored = GameSession::restore(source, &save_json)?;
```

`snapshot()` and `apply()` return `GameSnapshot`. `pending()` returns
`GamePending`. State structs derive `Serialize`; snapshots also support equality
for conformance tests. This adds a new API without removing the older session,
map, simulation or presentation-specific bridges.

`Session::apply` itself is also transactional. Its new `current_evaluation()`
accessor returns only the executed prefix; `program()` continues to return the
complete program for existing callers.

## Browser API

The same bridge is exported through wasm-bindgen:

```js
const game = new WebGameSession(source);
let snapshot = JSON.parse(game.snapshot());
snapshot = JSON.parse(game.apply(selection));
const save = game.save();
const resumed = WebGameSession.restore(source, save);
```

Construction, application and restoration throw a string on failure in
JavaScript. `snapshot()` returns JSON; a serialization/evaluation failure is
represented as the existing `{ "kind": "error", "message": "..." }` shape.

## Snapshot schema

The top-level `schema` is `caveat-game/0.1`.

| Field | Meaning |
| --- | --- |
| `source_id` | Stable source fingerprint for display and matching |
| `turn` | Number of successfully applied player selections |
| `scenes` | Scene text encountered in the executed prefix, in order |
| `labels` | Sorted object containing all source `display` identifiers and text |
| `world` | Static map world: `start_at`, `places`, `entities`, `connections`, `action_plans` |
| `state` | Current `current_place` and sorted `open_entities` |
| `pending` | One of the interaction states below |
| `budget` | `{initial, spent, remaining, exhausted}`, or `null` |
| `symbols` | Live symbols sorted by name, with `kind`, `source`, `consequence`, `display`, `attention` |
| `relations` | Current graph edges `{from, relation, to, origin: "live"}` in execution order |
| `commitments` | All live commitments `{action, open, retained, reopened_by}`, sorted by action |
| `discoveries` | Accumulated `{because, evidence, relation, target}` in discovery order |
| `selections` | Accepted player selection identifiers, in order |
| `last_execution` | `{action, from, to, commands}`, or `null` before the first turn |

`attention` is `unexamined`, `deferred`, `examining`, or `examined` for caveats,
and `null` for other symbols. Declared evidence is not automatically a discovery:
explicit reveal events and newly added evidence support/opposition relations
after an interaction populate the discovery list. A next-stage caveat's mere
qualification declaration is not evidence discovered by the player.

Source label conventions such as `<interaction>_body` or `<action>_result` are
allowed but uninterpreted by Rust. Labels and world definitions may include
future presentation content; clients should show them in their authored context.
Future selections and their derived live graph effects are never part of the
current snapshot.

### Pending states

```json
{"kind":"investigate","name":"scan","options":["lens_fault"],"cost":1,"budget":3}
{"kind":"choice","name":"route","options":["launch","wait"],"budget":2}
{"kind":"blocked","name":"route","options":[],"reason":"no action is available from the current world state","budget":2}
{"kind":"complete"}
```

Only physically available options are offered. Insufficient attention budget
produces `blocked` with reason `insufficient investigation budget`. A blocked
interaction is distinct from reaching the end of the source program. Applying
an action while blocked or complete fails without mutation.

## Save and replay

Saves are JSON with exactly `schema`, `source_id`, `source`, and `selections`.
Their schema is `caveat-save/0.1`. They store decisions, rather than trusting a
serialized world or epistemic graph, so restoration revalidates and replays every
turn through the same atomic interpreter.

`source_id` uses FNV-1a 64-bit over UTF-8 source bytes plus their byte length,
formatted `fnv1a64:<16 lowercase hex digits>:<length>`. It is an identifier, not a
cryptographic signature. Saves also carry the exact source text, and restoration
requires exact source equality in addition to fingerprint equality. Even a
whitespace change requires starting a new save. Source text makes saves portable
and inspectable; a caller sharing a save also shares the program it contains.

Malformed JSON, unknown fields, unsupported schema, source mismatch and an
invalid replay selection all fail. A replay error identifies its one-based turn.
No partly restored session is returned. Identical source and accepted selections
produce identical snapshots and save JSON across repeated executions.

## Conformance coverage

`runtime/tests/game_session.rs` covers initial snapshot isolation, source labels,
inspection cost, attention, persistent barriers, conditional discoveries,
retained caveats, reopening earlier commitments, ignored future placeholders,
atomic failures, blocked states, end-of-script behavior, deterministic replay,
source mismatch, save corruption and the browser JSON bridge. It also verifies
the legacy session's rollback and live summary behavior.
