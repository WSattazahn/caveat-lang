# First mushroom discovery and an optional rendered ability

[`slime_glow_ability.cav`](../examples/slime_glow_ability.cav) owns the bounded
policy for a Slime RPG's first mushroom: verified absorption records discovery
once, with a qualified learning receipt. Glow controls and claims require a
separate host report that its renderer is implemented and verified. With that
capability reported before absorption, acquisition enables Glow and further
input toggles it; repeated absorption cannot enable it again. Discovering the
mushroom before the cave story threshold still counts. Cave entry changes the
objective, not permission to discover.

The optional authored ruin scene instead asks the player to find a way out.
Only a verified crossing into its garden completes that objective; the mushroom
remains an optional discovery. Leaving a squeeze zone or absorbing food is not
by itself evidence of escape.

This is a policy consumer of the existing reactive interpreter. No language
syntax, Rust evaluator behavior, material value, or shader is added. The
runtime does not authenticate contact with a world object or prove a visible
glow was rendered; those are host responsibilities.

## Source and host contract

The six declared events accept exactly `{}`:

| Event | Host report | Source decision |
| --- | --- | --- |
| `cave_entered` | Existing authored cave threshold completed | Change the unfinished objective to absorbing the mushroom |
| `clearing_started` | Explicit authored free-start ruin/garden profile selected | Ask the player to find a way out of the ruined room |
| `ruin_escaped` | Player crossed from the room, through the named passage, beyond its garden boundary | Complete the authored ruin objective, retaining the context and crossing evidence |
| `absorb_mushroom` | The exact authored first mushroom was reached and accepted for absorption | Record discovery once, retain its qualified receipt, consume the item; activate only with renderer capability |
| `glow_renderer_ready` | The host implemented and verified the rendered Glow effect | Expose Glow labels and allow toggles after acquisition; do not grant discovery or silently activate an existing discovery |
| `toggle_glow` | Player requested the ability toggle | Toggle only after acquisition and verified renderer capability |

Identity, contact, inventory capacity and scene/session ownership must be checked
by the existing host systems. They must not add a second learned/toggle state
machine. A host cannot replace this verification by sending an arbitrary object
ID to Caveat: this small source has no dynamic inventory or entity-ID API.

The optional clearing context does not award Glow or change its learning basis.
It records the host's authored-scene report and sets guidance to "Find a way out
of the ruined room." Only `ruin_escaped` in that context completes the objective
as "You escaped into the garden." The qualified `escaped` value inherits both
`clearing_context` and `ruin_exit`. The host checks actual post-collision positions:
inside the room, then within `obj_clearing_squeeze`, then beyond its positive-Z
garden boundary. Spawning outside, backing out, skipping the corridor and leaving
through a side do not count. The source cannot establish these geometric facts
from an empty event payload. An unqualified squeeze-exit notification is inadequate.

Re-reporting context or escape is idempotent. Hosts that omit the scene context
retain the original cave objective; an escape event there does nothing. A fresh
round has no context or escape. Hosts re-report the selected scene on reset and
preserve a verified crossing during async loading. When staging a pre-learning
acquisition candidate, replay the current scene context and any verified escape.
Mushroom acquisition must neither erase nor create the escape result.

Without the renderer capability, absorption completes the original cave objective
as "Mushroom absorbed." It does not complete the authored ruin objective. The ability
is named "Mushroom discovery", remains inactive, and offers no toggle. Its
journal says "You absorbed one mushroom. Its effects are not yet established."
The host must consume `ability.toggleAvailable` when displaying controls and
must not report `glow_renderer_ready` merely to make a button appear. The source
also rejects unready toggle events. Reset removes the capability report; a host
with a verified renderer must report it again for a new session or staged
candidate. Vessel currently leaves this capability unset while protected
lighting work awaits separate approval.

The successful `keep_glow` commitment freezes value `1`, the `first_mushroom`
evidence, and the `single_absorption` caveat. The caveat says that this first
absorption does not establish the effects of other mushrooms. It does not
prevent using the ability. The engine's authored mushroom rule determines the
fictional effect; the language preserves its reported basis. The supported
claim is `mushroom_discovered`. The retained `keep_glow` commitment identifier
records that learning decision, not proof that light exists or was rendered.
Renderer capability has its own evidence and does not alter the learning receipt.

Bindings are direct presentation/state projections:

| Target | Properties |
| --- | --- |
| `ability` | `name`, `text`: strings; `learned`, `active`, `toggleAvailable`: booleans |
| `item` | `available`, `consumed`: booleans |
| `objective` | `text`: string; `complete`: boolean |
| `journal` | `visible`: boolean; `text`, `qualification`: strings |

The full snapshot also retains `qualified_values`, `binding_qualifications`,
`commitment_bases`, symbols and relations. Preserve the full learning receipt
when exposing its basis; a boolean UI projection is not a replacement for it.

The wrapper [`SlimeGlowPolicy`](../web/slime-glow-policy.js) forwards events and
returns the full snapshots. Its exports are `new SlimeGlowPolicy(source)`,
`snapshot()`, `caveEntered()`, `clearingStarted()`, `ruinEscaped()`, `glowRendererReady()`,
`absorbMushroom()`, `toggleGlow()`, `reset()` and
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

A host that pins digests should take these files from the Linux CI build, not
a local one. `rust-toolchain.toml` pins the compiler and the build remaps the
cargo home and checkout paths, so any Linux build of a revision produces the
same bytes; CI's `reproducible WebAssembly` job rebuilds from another path to
prove it. A Windows build still records `\` separators and differs. Each
`deploy-pages` run keeps the pinned files as the `caveat-runtime-<sha>` artifact
for 90 days, and `build-info.json` beside them names the revision, compiler and
host.

## Evidence and limits

Ten native tests and the real WASM wrapper test cover locked input, authored
clearing context and its reset, context-qualified escape independent of mushroom
acquisition, unchanged old-scene guidance, early
discovery, one-time qualified acquisition, duplicate absorption after disabling,
10,001 ready toggles, factual unready discovery, capability before/after discovery,
new-round reset, malformed input and late atomic failure.
Changing only `set active = learned * renderer_ready` to
`set active = learned * renderer_ready * 0` changes
automatic activation without changing the learned ability or its receipt.

After 10,001 ready toggles the graph still has twelve symbols and six relations, with
unchanged commitment bases and empty reading/decision histories. Snapshot size
varies only with fields such as event name, temporary effects and sequence
digits; it does not archive every toggle. These tests establish the bounded
policy and bridge independently of the host integration below. An additional
1,001 unready inputs with both scene contexts observed keep twelve symbols and six
relations without changing any bindings. Another 1,001 ready toggles after a
late capability report keep twelve symbols, seven relations and the same receipt.
Another 1,001 duplicate escape reports preserve the same graph and bindings;
the first crossing adds one fixed evidence relation rather than a frame history.
These policy checks establish no claim about how narrow, convincing or enjoyable
the host's passage looks. That requires inspection of the actual authored scene.

## First live Vessel integration — 2026-09-19

The first installed WASM policy ran inside Vessel's ordinary Slime mode. Browser
review followed the existing jar escape, authored cave squeeze and physical
contact with the placed mushroom. The existing absorption animation removed the
world mushroom, inventory contained one item, and the installed source supplied
the learned/active bindings and qualified `keep_glow` receipt. Keyboard `G` and
the HUD button toggled the source state; the journal displayed its acquisition
and qualification. Pause, inventory/journal modal guards and held-key repeat
suppression passed. These are observed controls and progression, not a claim
that the pending light effect has been rendered. That first policy nevertheless
showed Glow claims without an implemented renderer. The capability check above
corrects that mismatch: the default source now describes discovery factually and
withholds Glow controls. The earlier toggle review is historical evidence, not
proof that the current default should expose those controls.

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
