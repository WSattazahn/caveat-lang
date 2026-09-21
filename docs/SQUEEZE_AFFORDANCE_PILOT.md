# An explainable squeeze-entry policy

This exploratory example consumes copied Vessel editor passage facts and
synthetic movement probes. Caveat classifies a probe as outside, an entry
candidate, occupied, or unsupported, and preserves the observations and
qualifications behind each classification. It is not connected to Vessel's
live movement, collision, camera, or rendering code.

## Run it

```sh
cargo run --quiet --manifest-path runtime/Cargo.toml --bin caveat-reactive -- validate examples/squeeze_affordance.cav
cargo run --quiet --manifest-path runtime/Cargo.toml --bin caveat-reactive -- replay examples/squeeze_affordance.cav examples/squeeze_affordance_replay.jsonl
cargo test --manifest-path runtime/Cargo.toml --test squeeze_affordance
```

The existing replay command emits the complete snapshots as JSONL. Inspect
`bindings.decision`, `bindings.reason`, `reading_streams`, `commitment_bases`,
and `decision_series.traversal`. No new runner or language feature is needed.

The supplied replay reports the cave passage, probes outside while retreating,
aims an intended segment into the entry margin, reports its center inside the
authored box, and retreats. Its decisions are `0, 1, 2, 0`. Loading the
apothecary fixture reopens the previous decision; probing it records `-1` and
an explicit unsupported-rotation explanation.

## Where the facts came from

[`squeeze_affordance_fixtures.json`](../examples/squeeze_affordance_fixtures.json)
contains two small excerpts from Vessel's `public/scenes/SlimeWorld1.json`:
the axis-aligned `Cave Squeeze Box` and the rotated apothecary `SqueezeBox`.
It records object IDs, original transforms, role, asset/collider profile,
capture date, source Git revision, and source-file SHA-256. These identify the
copied input; they do not authenticate future inputs or certify its geometry.

Both objects use `primitives/cube`, fixed physics, and an explicit `squeeze`
role. Vessel's `useSlimeObstacles.ts` supplies that primitive's known local
half-extents `[0.5, 0.5, 0.5]` and zero mesh-center offset. The fixture derives
half-extents as `abs(scale) * 0.5`, preserving all reported rotation values.
The `profile` number is a fixture assertion that this narrow object profile
was checked, not an independent runtime check of a Vessel scene file.

The apothecary object's reported Y rotation is `32.477603615619394`. This
prototype deliberately reports it as unsupported instead of dropping its
rotation. Its rule accepts rotation magnitudes no greater than `1e-8`, positive
half-extents, and profile `1`. The reactive event records the maximum absolute
rotation component. Direct `SourceLibrary` calls apply `abs` to their rotation
argument as well, so a negative signed rotation cannot bypass the restriction.
Fixture identities must be exactly `1` or `2`; fractional identifiers fail the
whole passage event rather than receiving another fixture's label. Other
collider kinds and mesh bounds require a different adapter or policy and are
outside this example.

Probe positions are constructed around these copied bounds. They are not a
recording of an actual player crossing. The source labels them accordingly.

## What a decision means

| Code | Meaning |
| --- | --- |
| `-2` | No classification yet; a passage report and subsequent probe are needed. |
| `-1` | The reported profile, rotation, or bounds are unsupported. |
| `0` | Neither the reported center nor its intended XZ segment contacts the entry volume. |
| `1` | The center or intended segment contacts the padded entry volume. |
| `2` | The reported center is inside the unpadded authored volume, including its boundary. |

The entry policy uses a normal body radius of `0.35` and an extra margin of
`0.22`, matching the inspected Vessel constants. It checks Y as well as XZ.
For intended movement, it intersects the segment with the expanded box using
three separation tests. A zero-length probe can still contact the padded
volume through its current position.

`Occupied` here describes the reported center in an authored box. It does not
mean that Vessel's movement squeeze state has engaged or that the Beat 9 story
threshold has completed. Those are separate current systems: Vessel movement
uses stable normal-body-radius contact, collision uses anticipatory entry and
wall filtering, and the story requires an eligible traversal into a stricter
interior threshold. This prototype's unpadded center occupancy is deliberately
distinct from the actual movement-contact rule.

Vessel deliberately authors squeeze volumes larger than visible openings.
Consequently, volume overlap cannot prove that the body fits through a gap.
The three retained qualifications make that boundary explicit: authored
volume, copied-snapshot age, and synthetic intent. This example changes no
physical state, wall exclusion, body deformation, material, shader, or GPU
setting.

## What the interpreter preserves

Each passage event archives the center, half-extents, rotation magnitude,
profile, and fixture identity as distinct reading occurrences. Each accepted
probe archives its position and intended endpoint. The current assessment uses
the latest values, while a `traversal` decision freezes the classification and
its evidence/caveats. These are provisional assessment decisions, not commands
to cross a physical obstacle.

A new probe explicitly reopens the prior assessment before committing a new
revision. A changed passage report reopens it immediately but does not invent
a new probe or classification. A probe before any passage creates neither
readings nor a decision. Earlier observations and decision bases remain
unchanged, including when later data describes a different fixture.

The source's pure functions also compile through `SourceLibrary`, so an
independent host can evaluate the same geometric and explanation policy. A
bare library call does not create observation history, authenticate metadata,
or reopen a decision. This example uses `ReactiveSession` and its existing
validate/replay interface for those persistent graph semantics.

## Verification and authoring findings

Seven native tests cover fixture derivation, deterministic replay, boundary
contact, intended entry, retreat, vertical misses, unsupported rotation/profile
and zero-sized bounds, missing observations, malformed input, and late decision
capacity rollback. They check frozen prior readings/bases and the three retained
qualifications. An independent slab-intersection implementation checks the
source's separation formula on 625 paths, including parallel and zero-length
segments, edge contact, and corner contact.

Changing only `entry_padding()` from `0.22` to `0` changes an entry candidate
into an outside assessment. The test keeps every archived reading and the
decision's qualifications identical. This demonstrates source-owned tolerance
policy with preserved facts; it does not establish that either tolerance is
better for gameplay.

Authoring exposed a practical expression limit: the first nested slab/min/max
form exceeded the existing 4,096-node expansion budget. The source was revised
to use the smaller separation formula; the runtime limit stayed unchanged.
The event declarations also needed comma separators during initial validation.
A fixture derivation assertion needed a small floating-point tolerance because
decimal parsing and halving differed by one ULP. These corrections are part of
the result, not evidence of error-free first-pass authoring.

## A subsequent integration experiment

The next useful step is a read-only comparison against Vessel's actual
`extractSceneObstacles` output and captured position/intent traces, with a scene
revision and an explicit bounds/profile contract. Compare the existing entry
query with this policy, and inspect disagreements before changing gameplay.
Preserve list ordering when several passages match; this example considers one
passage at a time. Include actual radius changes, rotation handling, wall
filtering, and the separate story threshold before claiming behavioral parity.

Keep the comparison outside Vessel's frame loop initially. This bounded
history example is an authoring/replay tool, not a per-frame telemetry store.
No live WASM integration, frame-time advantage, safer collision behavior,
material change, or general AI reliability improvement has been demonstrated.
