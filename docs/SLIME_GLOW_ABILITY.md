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

The four declared events accept exactly `{}`:

| Event | Host report | Source decision |
| --- | --- | --- |
| `cave_entered` | Existing authored cave threshold completed | Change the unfinished objective to absorbing the mushroom |
| `clearing_started` | Explicit authored free-start clearing profile selected | Show the clearing's squeeze-and-absorb guidance until learned |
| `absorb_mushroom` | The exact authored first mushroom was reached and accepted for absorption | Learn and enable once, record a qualified receipt, mark the item consumed |
| `toggle_glow` | Player requested the ability toggle | Toggle only if learned |

Identity, contact, inventory capacity and scene/session ownership must be checked
by the existing host systems. They must not add a second learned/toggle state
machine. A host cannot replace this verification by sending an arbitrary object
ID to Caveat: this small source has no dynamic inventory or entity-ID API.

The optional clearing context does not award Glow or change its learning basis.
It records the host's authored-scene report and replaces unfinished guidance with
"Squeeze through the gap. Approach the mushroom and press E to absorb it."
Re-reporting context is idempotent. Hosts that omit this event retain the original
cave objective. A fresh round has no context; the host must report its selected
scene again, including when it stages a pre-learning acquisition candidate.

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
`snapshot()`, `caveEntered()`, `clearingStarted()`, `absorbMushroom()`, `toggleGlow()`, `reset()` and
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

Eight native tests and the real WASM wrapper test cover locked input, authored
clearing context and its reset, unchanged old-scene guidance, early
discovery, one-time qualified acquisition, duplicate absorption after disabling,
10,001 toggles, new-round reset, malformed input and late atomic failure.
Changing only `set active = learned` to `set active = learned * 0` changes
automatic activation without changing the learned ability or its receipt.

After 10,001 toggles the graph still has eight symbols and five relations, with
unchanged commitment bases and empty reading/decision histories. Snapshot size
varies only with fields such as event name, temporary effects and sequence
digits; it does not archive every toggle. These tests establish the bounded
policy and bridge independently of the host integration below. An additional
1,001 WASM toggles with both scene contexts observed retain eight symbols and
six relations and the same acquisition receipt.

## First live Vessel integration — 2026-09-19

The installed WASM policy now runs inside Vessel's ordinary Slime mode. Browser
review followed the existing jar escape, authored cave squeeze and physical
contact with the placed mushroom. The existing absorption animation removed the
world mushroom, inventory contained one item, and the installed source supplied
the learned/active bindings and qualified `keep_glow` receipt. Keyboard `G` and
the HUD button toggled the source state; the journal displayed its acquisition
and qualification. Pause, inventory/journal modal guards and held-key repeat
suppression passed. These are observed controls and progression, not a claim
that the pending light effect has been rendered.

The host calls Caveat at input, threshold and absorption boundaries. It does
not serialize snapshots on each render or physics tick. Existing engine code
still owns contact detection, inventory writes, absorption animation and UI
lifetime. It consumes source bindings without copying the learning/toggle rules
into TypeScript.

Before acquisition, the host stages the Caveat decision in a candidate session
with the same source and any existing scene context. It validates the
result before attempting the normal inventory write, then publishes the
candidate only after that write succeeds. A full bag discards the candidate;
a failed policy dispatch never attempts the write. In both cases the existing
pickup flow leaves the world item available. Reconstructing this pre-learning
candidate is valid for this exact source's small event contract; it is not a
general license to replace arbitrary sessions and drop their history. Tests
using the installed WASM also cover loader/reset/unmount races, source-only
variation, observer failures and synchronous round changes during the inventory
callback.

All four installed artifact byte identities are checked during installation and
host tests. The runtime loader checks source and WASM digests before creating a
session; it loads the JavaScript wrapper and generated glue as ordinary local
modules. Do not describe that narrower runtime check as verification of all
executed module bytes.

Host evidence is retained in the private Vessel checkout under
`.cache/slime-glow-review/` (including `controls.json`, runtime reports and
screenshots) and `tests/lib/slime/slimeGlowAbilityState.spec.ts`. The protected
emitted-light/material work still awaits the user's separate approval. Visible
illumination, general game feel, frame-time improvements, standalone Slime
export and broader language usefulness remain unverified by this milestone.
