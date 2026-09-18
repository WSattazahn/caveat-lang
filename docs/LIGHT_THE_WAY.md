# Light the Way

**Drag the light left or right. Reach the green harbor.**

Light the Way turns Saint Orin into a short, directly controlled rescue game. A visible ferry advances toward the island while the player guides it past six rocks and a pulsing crosscurrent. The green harbor ring rescues all thirty-two passengers. Three impacts, missing the harbor, or ninety seconds without arrival ends the attempt. A new session immediately restarts it.

There are no required story decisions or reading panels. The earlier narrative game remains separately available as [The Last Beacon](THE_LAST_BEACON.md).

## The game is a CAVEAT program

[`game/light_the_way.cav`](../game/light_the_way.cav) defines all simulation values, movement equations, steering acceleration, wind, discovery, collisions, cooldown, scoring, success, and failure. It also defines world coordinates, input mappings, the clock, UI screens and values, object poses and colors, and explicit feedback cues. The browser dispatches declared inputs, applies evaluated bindings, and presents emitted cues. It does not infer game meaning from variables such as `phase`, `hull`, or `boat_x`.

The source uses [CAVEAT reactive 0.2](../spec/caveat-reactive-0.2.md) alongside the existing epistemic graph. Pure functions are reusable expressions; bindings connect evaluated values to generic presentation targets:

```caveat
fn distance(ax, az, bx, bz) = sqrt((ax - bx) * (ax - bx) + (az - bz) * (az - bz));
bind ferry.x = boat_x;
bind app.running = phase == 1 and paused == 0;
control keyboard = steer;
clock tick every 0.03333333333333333;
```

Expressions can refer directly to source positions such as `reef_one.x` and `harbor_goal.z`. Physics and visuals consequently share one authored location for each rock and the harbor.

Numeric functions and UI bindings provide ordinary programming facilities. CAVEAT's distinctive contribution remains the record of claims, provenance, caveats, attention cost, provisional commitments, and revision—not arithmetic or drawing a boat by itself.

## One input, visible consequences

The ferry moves forward automatically. Holding the light steers it laterally toward the light's horizontal position, with acceleration and a capped turn speed. The light's full position also scouts the water: shining near a rock reveals it. Releasing the light lets the ferry's lateral movement settle while it continues forward.

The opening provides more than five seconds of unobstructed water. Six rocks form three paired barriers across the bounded channel, requiring a right, left, then right passage toward the harbor. Holding one edge cannot bypass the course. A successful trip takes roughly a minute to a minute and a half. The collision circle combines the source's rock radius and ferry radius. A hit reduces hull once, nudges the ferry sideways, and grants two seconds of collision immunity.

The source channel runs from `x=-9.5` to `x=13`. At the three barrier centers (`z=45`, `36`, and `27`), an undamaged ferry must pass respectively to the right of `x=2.4`, left of `x=0.1`, and right of `x=3.2`. These limits follow from the authored rock positions and collision radii. They leave clear routes while ruling out a single held direction for most of the game.

## Knowledge changes the simulation

An old chart initially supports `clear_course`. Starting the rescue commits to `follow_light`, retaining six caveats about submerged rocks. Scouting or striking a rock adds its reading as opposing evidence. The old supporting evidence remains in the graph.

Each discovered rock examines its corresponding caveat once. The first contrary observation reopens the course commitment. Near a known rock, the source's `observed(...)` and `reopened(...)` predicates activate a warning and reduce forward speed from `0.6` to `0.42`, allowing more reaction time. Knowledge therefore changes motion; it is not only a journal entry. A collision is also evidence, including the final damaging frame.

### A forecast can be useful and still need revision

The crosscurrent adds a second provisional decision. `morning_forecast`, with tidal-archive provenance, supports `crosscurrent_mild`. `surge_unmeasured` qualifies that evidence. Starting the rescue commits to `trust_forecast` while retaining the caveat.

Shining the light into the current yields `crosscurrent_reading`: measured foam drift opposes the forecast claim. Examination costs one attention unit. The original supporting evidence is preserved. The observation reopens `trust_forecast`, then the source commits to partial `counter_steer`, retaining the same unresolved surge caveat.

| Physical truth | Navigation response |
| --- | --- |
| `current_force` depends only on the authored position, radius, strength, and time pulse. | `compensation` remains zero until the revised `counter_steer` commitment exists. |
| Observing the current never weakens or removes it. | The revised policy counters 65% of the current while continuing toward harbor. |

The foam marker, force, opposing reading, warning, response, and cue all come from this source file. The renderer supplies a generic ring asset; it has no crosscurrent-specific behavior. There is no forced reading panel, and an objection does not automatically halt the ferry. The earlier forecast and retained caveat remain inspectable after revision.

## Input and state contract

| Input event | Parameters | Meaning |
| --- | --- | --- |
| `start` | None | Start the rescue and commit to the course |
| `aim` | `x`, `z`, `active` | Move the light and hold/release it |
| `steer` | `dx`, `dz`, `active`, `dt` | Move the target through source rules for keyboard input |
| `tick` | `dt`, between `0` and `0.1` seconds | Advance the source simulation |
| `pause`, `resume` | None | Change the source pause state |

Source `controls` map pointer, keyboard, start, pause, resume, and retry to these events. Start and retry construct a new session. The source clock specifies the tick interval. Pausing releases the light; event guards freeze physics and reject steering changes even if a host submits ticks or inputs while paused.

The following domain values remain available for inspection and testing. Hosts consume their resulting `bindings`, not a fixed list of domain-state identifiers:

| Source values | Presentation purpose |
| --- | --- |
| `boat_x`, `boat_z`, `boat_vx` | Ferry position and heading |
| `aim_x`, `aim_z`, `light_on` | Light target and beam |
| `phase` | `0` ready, `1` playing, `2` rescued, `3` failed |
| `paused` | Source pause state |
| `hull`, `collision_cooldown`, `hit_count` | Hull indicators and impact feedback |
| `elapsed`, `time_limit`, `progress` | Time and route progress |
| `warning` | `0` clear, `1` light released, `2` known rock or impact, `3` current compensation |
| `reef_one_seen` through `reef_six_seen` | Discovered-rock presentation |
| `current_force`, `compensation`, `current_seen` | External current, revised response, and observation state |
| `failure_reason` | `1` hull lost, `2` missed harbor, `3` time expired |
| `rescued`, `total_passengers`, `score` | Source-authoritative result |
| `aim_x_min/max`, `aim_z_min/max`, `aim_speed` | Input limits and keyboard target speed |

The score rewards remaining hull, time remaining, and discovered rocks. Its equation is authored in the source. The interface formats or rounds evaluated values for display.

Sounds, toasts, flashes, and expanding rings are declared with `cue` and emitted by source rules. Observation guards prevent repeated discovery cues; terminal rules emit before changing phase so repeated ticks cannot replay an ending sound. Cue delivery is part of the event transaction, alongside graph and numeric changes.

## Verify

```sh
cargo test --manifest-path runtime/Cargo.toml --test light_the_way
npm run build
npm run test:rescue
npm run serve
```

Open `/rescue.html` on the local server. Build prerequisites are in the [README](../README.md).

Runtime verification covers the safe opening, real-input rescue, unattended failure, impact immunity, observations and reopened knowledge, source tuning, pause, controls, and explicit cues. A paired current test uses legitimate inputs to establish equal external force but different navigation responses after observation, while preserving the original forecast, opposing evidence, and retained caveat. Browser checks additionally exercise direct controls, a complete rescue, mobile presentation, and source-only changes to HUD text, an object binding, and a previously unknown cue.
