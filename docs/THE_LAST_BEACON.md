# The Last Beacon

The Last Beacon is a six-decision mystery on stormbound Saint Orin. Thirty-two ferry passengers are approaching a failing lighthouse. The player investigates, tests provisional plans, and decides which uncertainties to carry into a final order. Earlier knowledge changes both available actions and how that order turns out.

[`game/the_last_beacon.cav`](../game/the_last_beacon.cav) owns the game: evidence, attention, requirements, commitments, reopening, outcome rules, prose, world topology, coordinates, cameras, and traversal paths. Rust evaluates that source. The browser draws semantic templates and animates generic commands. The same program can run in the native terminal without a browser.

This scenario uses the implemented features described in the [CAVEAT 0.4 draft](../spec/caveat-0.4-draft.md), [game-session contract](../spec/game-session-0.1.md), and [presentation specification](../spec/presentation.md). It demonstrates an executable epistemic language, not a claim that CAVEAT is already a general-purpose replacement for Rust or JavaScript.

## The decision structure

Three watches provide three units of attention. Each watch buys one investigation, then one commitment. Reading the journal and rotating the scene are free: this is an attention budget, not a real-time countdown.

| Interaction | What changes | Available options |
| --- | --- | --- |
| `first_watch` | Observe a defect in the lens, chart, or radio | Three investigations |
| `first_signal` | Pursue the observed defect or compare records at the court | One unlocked test plus `compare_records` |
| `changed_water` | Investigate the tide, reserve, or ferry position | Three investigations |
| `revised_plan` | Prepare the supported solution or preserve equipment | One unlocked preparation plus `preserve_options` |
| `final_watch` | Check cooling, channel clearance, or anchorage | Three investigations |
| `final_signal` | Activate the beacon, dispatch the pilot, or transmit a hold | Three orders with nine possible outcomes |

There are **324 legal complete sequences**: `3 × 2 × 3 × 2 × 3 × 3`. The source declares four options at each provisional choice, but requirements leave two available on any particular playthrough. Unavailable actions explain which observation is missing.

Both provisional commitments reopen after new evidence, including the cautious alternatives. Reopening preserves the action and its retained doubts; it does not undo the world. The final commitment retains all nine caveats, with exactly three examined and six unexamined. Examination does not imply resolution: the reserve can produce voltage while still overheating under load.

## Knowledge changes behavior

The language, rather than browser conditionals, determines availability:

```caveat
require trust_beam observed salt_seam;
require bridge_reserve observed reserve_charge;
```

Merely declaring evidence does not satisfy these requirements. An observation must have an active support or opposition relation in the executed graph. A missing observation remains missing even when its descriptive label is present in static presentation data.

Final results are ordered source rules evaluated against the state **before** the final action:

```caveat
resolve relight_beacon as beacon_guided when observed measured_pulses;
resolve relight_beacon as beacon_limited when observed hot_cable;
resolve relight_beacon as beacon_limited when observed split_beam;
resolve relight_beacon as beacon_unverified otherwise;
```

The cooling check enables a complete approach. Either preparation evidence or a targeted first test permits a limited attempt. Without those observations, lighting the lamp does not convince the captain to approach. The same final button therefore has materially different consequences. Both earlier commitments matter: pursuing the field test or preparing the system yields observations that the cautious alternatives do not.

There are no hidden random death rolls or certainty scores. Contradictory support and opposition remain visible. The final graph records the operation actually performed—beacon activated, pilot dispatched, or hold transmitted—while the resolved outcome records whether it achieved its purpose. It never asserts a successful rescue solely because an order was issued.

## Three orders, nine outcomes

| Order and destination | Verified final check | Relevant earlier test or preparation | Neither available |
| --- | --- | --- | --- |
| `relight_beacon` → `lantern_room` | `beacon_guided`: all passengers arrive under measured pulses | `beacon_limited`: partial approach, daylight tow, emergency fuel cost | `beacon_unverified`: captain refuses the approach; rescue unresolved |
| `launch_pilot` → `harbor` | `pilot_rescue`: passenger transfer completed before dawn | `pilot_delayed`: transfer begins but the falling tide strands the remaining passengers aboard | `pilot_stalled`: launch cannot establish a safe passage; no transfer completed |
| `hold_offshore` → `observatory` | `hold_verified`: confirmed anchorage and arranged tow | `hold_repositioned`: search finds holding ground at the cost of engine damage | `hold_unlocated`: no confirmed holding ground; rescue support requested |

For each final order, the legal decision tree contains 36 verified, 22 partial, and 50 unsupported outcomes. The outcome basis identifies the actual observation that matched; an `otherwise` result has no supporting basis.

For example, these two routes differ at only the preparation:

```text
radio_echo → compare_records → reserve_unknown → bridge_reserve → anchor_hold → relight_beacon
  = beacon_limited, because the bypass exposed hot_cable

radio_echo → compare_records → reserve_unknown → preserve_options → anchor_hold → relight_beacon
  = beacon_unverified, because no relevant test or preparation established the approach
```

Changing that last investigation to `beam_heat` instead produces `beacon_guided`. A final check can repair an earlier information gap, but attention spent elsewhere cannot silently perform it.

## Physical and presentation contracts

Seven places form one connected island. The court connects to the lighthouse base, observatory, tidal archive, and harbor. The base connects to the elevated lantern room; the harbor connects to the breakwater. The first two choices explicitly `converge` at the court. Their action plans walk back, or explicitly stay there when comparing records. Final actions reach three different destinations.

The source contains seventeen absolute positions, seven camera views, an overview, and six authored paths, including the tower ascent. Changing a source coordinate or waypoint changes the scene without editing JavaScript. Place and entity kinds choose reusable visual templates; the renderer still owns geometry construction, materials, lighting, and animation mechanics.

Only `relight_beacon` operates the beacon, only `launch_pilot` operates the boat, and only `hold_offshore` operates the transmitter. Those physical commands describe issuing the order, not an inferred successful outcome. Narrative resolution uses `snapshot.outcome.id`; the renderer must not derive it from an action name.

Story prose is authored with `display`: interaction headings and `_body`, action labels and `_hint`/`_result`, caveat `_uncertainty`, and outcome labels with `_title`, `_result`, and `_epilogue`. Outcomes explain both what happened and what the missing or observed evidence cost.

## Run and verify

From the repository root:

```sh
cargo run --manifest-path runtime/Cargo.toml --bin caveat -- --game game/the_last_beacon.cav
cargo test --manifest-path runtime/Cargo.toml --test the_last_beacon
cargo run --manifest-path runtime/Cargo.toml --bin caveat-map -- validate game/the_last_beacon.cav
npm run build
npm run test:beacon
```

The native mode accepts a numbered option or its source identifier. The [README](../README.md) supplies browser build prerequisites.

Automated verification explores the actual pending options, checks requirements and attention, retains both reopened commitments, verifies outcome basis and destinations, and exercises save restoration. Browser QA covers the WebAssembly boundary, blocked-action feedback, source-resolved endings, and mobile layout. Screenshot review remains necessary for tower traversal, route geometry, and the three final viewpoints.
