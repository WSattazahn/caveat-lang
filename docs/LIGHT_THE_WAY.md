# Light the Way

**Drag the light left or right. Reach the green harbor.**

Light the Way turns Saint Orin into a short, directly controlled rescue game. A visible ferry advances toward the island while the player guides it past six rocks and a crosscurrent that shifts with two weather fronts. The green harbor ring rescues all thirty-two passengers. Three impacts, missing the harbor, or ninety seconds without arrival ends the attempt. A new session immediately restarts it.

There are no required story decisions or reading panels. The earlier narrative game remains separately available as [The Last Beacon](THE_LAST_BEACON.md).

## The game is a CAVEAT program

[`game/light_the_way.cav`](../game/light_the_way.cav) defines all simulation values, movement equations, steering acceleration, wind, discovery, collisions, cooldown, scoring, success, and failure. It also defines world coordinates, input mappings, the clock, UI screens and values, object poses and colors, and explicit feedback cues. The browser dispatches declared inputs, applies evaluated bindings, and presents emitted cues. It does not infer game meaning from variables such as `phase`, `hull`, or `boat_x`.

The source's events, controls, and bindings follow the [CAVEAT reactive 0.2](../spec/caveat-reactive-0.2.md) presentation contract. [Reactive 0.3](../spec/caveat-reactive-0.3.md) adds qualified values; [Reactive 0.4](../spec/caveat-reactive-0.4.md) adds text expressions and a CAVEAT standard prelude. [Reactive 0.5](../spec/caveat-reactive-0.5.md) adds recurring observations and decision histories. Together they carry evidence and caveats through calculation, revision, and presentation:

```caveat
fn distance(ax, az, bx, bz) = sqrt((ax - bx) * (ax - bx) + (az - bz) * (az - bz));
bind ferry.x = boat_x;
bind app.running = phase == 1 and paused == 0;
bind time.text = time_text(max(0, time_limit - elapsed));
control keyboard = steer;
clock tick every 0.03333333333333333;
```

Expressions can refer directly to source positions such as `reef_one.x` and `harbor_goal.z`. Physics and visuals consequently share one authored location for each rock and the harbor.

Functions and UI bindings provide familiar programming facilities. Their values can preserve evidence and caveats across computation, so that record reaches a decision without manually copying a caveat list. CAVEAT's distinctive contribution remains claims, provenance, qualifications, attention cost, provisional commitments, and revision—not drawing a boat by itself.

## One input, visible consequences

The ferry moves forward automatically. Holding the light steers it laterally toward the light's horizontal position, with acceleration and a capped turn speed. The light's full position also scouts the water: shining near a rock reveals it. Releasing the light lets the ferry's lateral movement settle while it continues forward.

Holding the light on the center of the current ring for one continuous second measures its flow. A small meter fills in the world and HUD. Moving the light away cancels an incomplete reading. The ferry keeps advancing and turning toward the light throughout: every reading costs actual control time, and the first examination of each relevant caveat also spends budget.

The opening provides more than five seconds of unobstructed water. Six rocks form three paired barriers across the bounded channel, requiring a right, left, then right passage toward the harbor. Holding one edge cannot bypass the course. A successful trip takes roughly a minute to a minute and a half. The collision circle combines the source's rock radius and ferry radius. A hit reduces hull once, nudges the ferry sideways, and grants two seconds of collision immunity.

The source channel runs from `x=-9.5` to `x=13`. At the three barrier centers (`z=45`, `36`, and `27`), an undamaged ferry must pass respectively to the right of `x=2.4`, left of `x=0.1`, and right of `x=3.2`. These limits follow from the authored rock positions and collision radii. They leave clear routes while ruling out a single held direction for most of the game.

## Knowledge changes the simulation

An old chart initially supports `clear_course`. Starting the rescue commits to `follow_light`, retaining six caveats about submerged rocks. Scouting or striking a rock adds its reading as opposing evidence. The old supporting evidence remains in the graph.

Each discovered rock examines its corresponding caveat once. The first contrary observation reopens the course commitment. Near a known rock, the source's `observed(...)` and `reopened(...)` predicates activate a warning and reduce forward speed from `0.6` to `0.42`, allowing more reaction time. Knowledge therefore changes motion; it is not only a journal entry. A collision is also evidence, including the final damaging frame.

### One forecast, recurring observations

The crosscurrent adds a provisional navigation decision. `morning_forecast`, with tidal-archive provenance, supports `crosscurrent_mild`. `forecast_drift = qualified(0, morning_forecast)` carries that estimate and its unresolved surge caveat into the first `navigation` commitment.

The source declares reusable histories instead of separate first and second observations:

```caveat
readings flow from foam_sensor limit 128;
readings weather from weather_front limit 8;
decisions navigation limit 128;
```

Each completed one-second scan creates an immutable `flow` occurrence containing the measured foam speed, source provenance, and inherited caveats. Its sign supports or opposes `current_eastward`. Later readings do not overwrite earlier supporting or contrary observations. `foam_sensor` is the source template; declaring it does not count as observing it.

The same sampling block works throughout the crossing. The light must leave the ring or be released before another measurement can begin. Holding it there indefinitely produces one observation, not a stream of perfect telemetry. Each completed scan costs one second of actual steering attention. The source examines `surge_unmeasured` and `reading_may_age` once each; the total budget of eight covers those examinations and six rocks. Repeated sampling remains possible after that budget is spent, and uncertainty remains attached.

### The weather changes twice

At thirty-four seconds, the sea darkens, rain strengthens, and the current reverses from `+0.9` eastward to `-1.2` westward. At forty seconds, another front turns it eastward at `+1.2`. These changes occur independently of the player's knowledge. A flat central flow with a 1.2-unit edge fade makes the difference between the old correction and a revised one visible.

Each front creates a separate `weather` observation containing only coarse direction, `-1` or `+1`. It can reopen the selected navigation revision without supplying an exact new force measurement.

The player can keep the old correction and steer manually, or hold the ring for another sample. The correction keeps acting until revised; after a reversal it can worsen drift. The target sits beside the left-hand leg, providing time to take a reading before the final turn toward harbor.

Bright arrows show visible flow direction. A faint arrow preserves the last measured direction, so their disagreement is visible in the world. The HUD shows `FORECAST` before a measurement and the reading's age afterward. Amber means the visible weather has changed since that reading; it is not a fabricated confidence percentage or a declaration that an older reading must be false.

### Every steering revision keeps its reasons

When a scan completes, the source immediately computes `counter_plan(infer_peak(latest(flow), tidal_pulse(elapsed)), 0.65)`. The pure functions preserve that exact occurrence's evidence and caveats. It reopens the selected navigation revision if needed, then commits the new value. The earlier revision, its number, retained caveats, and reopening cause remain in history.

```caveat
fn infer_peak(flow, phase) = flow / phase;
fn counter_plan(peak, fraction) = -peak * fraction;
fn current_field(peak, phase, separation, radius) = peak * phase * clamp((radius - separation) / 1.2, 0, 1);
```

The resulting peak corrections are approximately `-0.585`, `+0.78`, and `-0.78` when those three physical phases are sampled. Subsequent motion projects the frozen `latest(navigation)` through the local pulse and field. It never recalculates an old foam observation against a later sampling time or quietly borrows actual strength as fresh navigation telemetry.

| Physical truth | Navigation response |
| --- | --- |
| `current_force` depends on the authored field, weather, time pulse, and ferry position. | The selected decision counters 65% of its own estimated current. |
| Observation does not weaken or reverse the external field. | A new reading can produce a different correction while old evidence remains. |

`snapshot.values` remains a plain numeric projection; qualified values and the histories preserve the associated evidence and caveats. Metadata can also travel through a command into boat position and a position-dependent force calculation. That records computational lineage. At the same actual position and time, the external field still has the same numeric force.

## Input and state contract

| Input event | Parameters | Meaning |
| --- | --- | --- |
| `start` | None | Start the rescue and commit to the course |
| `aim` | `x`, `z`, `active` | Move the light and hold/release it |
| `steer` | `dx`, `dz`, `active`, `dt` | Move the target through source rules for keyboard input |
| Source-declared raw key events | `active`, between `0` and `1` | Retain a physical press or release; source rules determine its meaning |
| `tick` | `dt`, between `0` and `0.1` seconds | Advance the source simulation |
| `pause`, `resume` | None | Change the source pause state |

Source `controls` map pointer, keyboard, start, pause, resume, and retry to these events. Start and retry construct a new session. The source clock specifies the tick interval. Pausing releases the light; event guards freeze physics and reject steering changes even if a host submits ticks or inputs while paused.

The browser forwards physical key codes through source-declared controls such
as `key_ArrowLeft`, `key_KeyA`, and `key_Space`. Each alias has an independent
held state in Caveat. Source tick rules combine those states into steering:
two left aliases move at the same speed as one, opposite directions cancel,
and Space holds the beam without moving its target. Releasing one alias cannot
release another held alias. Source pause and pointer takeover clear held keys.
Escape's pause/resume toggle is also a declared source control.
The browser retains only the key-delivery bookkeeping needed to suppress
repeats and release input on focus loss. The existing numeric `steer` event
remains available to native callers. Changing a raw-key control declaration
changes browser behavior without editing JavaScript or Rust.

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
| `current_force`, `compensation` | External current and selected navigation response |
| `forecast_drift` | Qualified forecast used before the first held reading |
| `first_shift_at`, `second_shift_at`, `weather_phase` | Authored weather timing and physical phase |
| `sample_charge`, `sample_duration`, `sample_in_range`, `sample_armed` | Continuous hold requirement, cancellation, and release latch |
| `last_read_at`, `reading_weather_phase` | Reading age and whether weather has changed since it |
| `rain_speed`, `rain_travel` | Source-owned rain motion; water animation uses `elapsed` |
| `notice_kind`, `notice_remaining` | Source-selected notice and simulation-time lifetime |
| `failure_reason` | `1` hull lost, `2` missed harbor, `3` time expired |
| `rescued`, `total_passengers`, `score` | Source-authoritative result |
| `aim_x_min/max`, `aim_z_min/max`, `aim_speed` | Input limits and keyboard target speed |

The score rewards remaining hull, time remaining, and discovered rocks. Its equation is authored in the source. CAVEAT's source prelude supplies `number_text`, `time_text`, and `percent_text`; the browser assigns their evaluated strings. Hull indicators also receive individual source bindings.

Sounds and expanding rings are declared with `cue` and emitted by source rules. Observation guards prevent repeated discovery cues; terminal rules emit before changing phase so repeated ticks cannot replay an ending sound. Cue delivery is part of the event transaction, alongside graph and numeric changes. Toast visibility and impact flashes use source bindings and countdowns, so pausing also pauses their lifetime.

The source supplies water time and integrated rain travel to generic rendering bindings. Pausing or replaying the simulation consequently preserves weather motion alongside the ferry, scan meter, and reading age.

## Verify

```sh
cargo test --manifest-path runtime/Cargo.toml --test light_the_way
npm run build
npm run test:rescue
npm run serve
```

Open `/rescue.html` on the local server. Build prerequisites are in the [README](../README.md).

Runtime verification covers the safe opening, real-input rescue, unattended failure, impact immunity, observations and reopened knowledge, source tuning, pause, controls, and explicit cues. Weather checks cover continuous sampling, cancellation, release latching, paused timers, manual and repeatedly revised routes, and preserved histories. Qualified-value checks follow each immutable measurement through pure functions into its command and commitment basis. Browser checks additionally exercise direct controls, a complete rescue, mobile presentation, and source-only changes to HUD text, an object binding, and a previously unknown cue.
