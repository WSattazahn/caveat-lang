# First mushroom, one learned ability

[`slime_glow_ability.cav`](../examples/slime_glow_ability.cav) owns the bounded
policy for a Slime RPG's first mushroom: verified absorption learns Glow and
enables it, further input toggles the learned ability, and repeated absorption
cannot enable it again. Discovering the mushroom before the cave story threshold
still teaches Glow. Cave entry changes the objective, not permission to learn.

This is a policy consumer of the existing reactive interpreter. No language
syntax, Rust evaluator behavior, material value, or shader is added. The
runtime does not authenticate contact with a world object or prove a visible
glow was rendered; those are host responsibilities.

## Source and host contract

The three declared events accept exactly `{}`:

| Event | Host report | Source decision |
| --- | --- | --- |
| `cave_entered` | Existing authored cave threshold completed | Change the unfinished objective to absorbing the mushroom |
| `absorb_mushroom` | The exact authored first mushroom was reached and accepted for absorption | Learn and enable once, record a qualified receipt, mark the item consumed |
| `toggle_glow` | Player requested the ability toggle | Toggle only if learned |

Identity, contact, inventory capacity and scene/session ownership must be checked
by the existing host systems. They must not add a second learned/toggle state
machine. A host cannot replace this verification by sending an arbitrary object
ID to Caveat: this small source has no dynamic inventory or entity-ID API.

The successful `keep_glow` commitment freezes value `1`, the `first_mushroom`
evidence, and the `single_absorption` caveat. The caveat says that this first
absorption does not establish the effects of other mushrooms. It does not
prevent using the ability. The engine's authored mushroom rule determines the
fictional effect; the language preserves its reported basis.

Bindings are direct presentation/state projections:

| Target | Properties |
| --- | --- |
| `ability` | `name`, `text`: strings; `learned`, `active`: booleans |
| `item` | `available`, `consumed`: booleans |
| `objective` | `text`: string; `complete`: boolean |
| `journal` | `visible`: boolean; `text`, `qualification`: strings |

The full snapshot also retains `qualified_values`, `binding_qualifications`,
`commitment_bases`, symbols and relations. Preserve the full learning receipt
when exposing its basis; a boolean UI projection is not a replacement for it.

The wrapper [`SlimeGlowPolicy`](../web/slime-glow-policy.js) forwards events and
returns the full snapshots. Its exports are `new SlimeGlowPolicy(source)`,
`snapshot()`, `caveEntered()`, `absorbMushroom()`, `toggleGlow()`, `reset()` and
`free()`. Initialize the generated WASM module first. `free()` is idempotent;
other calls after free throw. `reset()` creates a fresh session from the same
source and then frees the old session. It does not erase the old graph in place.
The source owns initial state; the host owns round and component lifetime.

Dispatch on input/absorption/threshold events, never on render or physics ticks.
Toggling changes numeric state with the original learning qualifications. There
is one fixed learning commitment and no readings or decision series. Thus input
does not accumulate a history occurrence for every button press. No compaction,
metadata truncation, unobserved evidence, or stateless reconstruction is needed.

Atomicity ends at the session boundary. A failed dispatch publishes no Caveat
changes. It cannot roll back a host inventory write, item removal or rendered
effect. The integrating host must stage/validate its existing absorption
transaction so an interpreter failure cannot silently consume an item without
granting its effect; it must not repeat a committed host action to repair UI.

## Build, test and copy into a host

From the repository root:

```sh
cargo test --manifest-path runtime/Cargo.toml --test slime_glow_ability
npm run build
npm run test:slime-glow
```

`npm run build` builds the locked Rust runtime for `wasm32-unknown-unknown` in
release mode, runs exactly `wasm-bindgen 0.2.104 --target web`, and copies the
canonical example and wrapper into `dist/`. It is a WASM interpreter plus
generated JavaScript bridge, not a compiler that translates Caveat to JavaScript.

Copy these files preserving the relative layout:

```text
slime_glow_ability.cav
slime-glow-policy.js
pkg/caveat_runtime.js
pkg/caveat_runtime_bg.wasm
```

The generated `pkg/caveat_runtime.d.ts` is also available for typed hosts. Fetch
the source as text, initialize the module with the exact packaged WASM bytes or
URL, and construct one policy per round. Package digests should cover the exact
bytes installed; no runtime fetch from a sibling checkout or remote repository
is required. Copies of source/wrapper must be replaced from this build, not
independently edited to create a second policy.

The real-WASM test checks that the copied source and wrapper match their
canonical inputs. `test-results/slime-glow-wasm.json` records their SHA-256
digests, generated glue/WASM digests, source identity, and footprint observations.
These identify tested files; the runtime's FNV source ID is not a cryptographic
integrity check. The build command is the provenance for generated artifacts.

## Evidence and limits

Seven native tests and the real WASM wrapper test cover locked input, early
discovery, one-time qualified acquisition, duplicate absorption after disabling,
10,001 toggles, new-round reset, malformed input and late atomic failure.
Changing only `set active = learned` to `set active = learned * 0` changes
automatic activation without changing the learned ability or its receipt.

After 10,001 toggles the graph still has six symbols and five relations, with
unchanged commitment bases and empty reading/decision histories. Snapshot size
varies only with fields such as event name, temporary effects and sequence
digits; it does not archive every toggle. This demonstrates the bounded policy
and bridge. It does not verify Vessel integration, visible lighting, browser
frame time, authored placement or controller feel; those require the real game.
