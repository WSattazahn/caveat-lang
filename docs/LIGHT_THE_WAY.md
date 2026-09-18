# Light the Way

**Drag the light left or right. Reach the green harbor.**

Light the Way turns Saint Orin into a short, directly controlled rescue game. A visible ferry advances toward the island while the player guides it past six rocks and a crosscurrent that reverses in a storm. The green harbor ring rescues all thirty-two passengers. Three impacts, missing the harbor, or ninety seconds without arrival ends the attempt. A new session immediately restarts it.

There are no required story decisions or reading panels. The earlier narrative game remains separately available as [The Last Beacon](THE_LAST_BEACON.md).

## The game is a CAVEAT program

[`game/light_the_way.cav`](../game/light_the_way.cav) defines all simulation values, movement equations, steering acceleration, wind, discovery, collisions, cooldown, scoring, success, and failure. It also defines world coordinates, input mappings, the clock, UI screens and values, object poses and colors, and explicit feedback cues. The browser dispatches declared inputs, applies evaluated bindings, and presents emitted cues. It does not infer game meaning from variables such as `phase`, `hull`, or `boat_x`.

The source's events, controls, and bindings follow the [CAVEAT reactive 0.2](../spec/caveat-reactive-0.2.md) presentation contract. [Reactive 0.3](../spec/caveat-reactive-0.3.md) adds qualified values that carry the epistemic graph's evidence and caveats through numeric computation. [Reactive 0.4](../spec/caveat-reactive-0.4.md) adds text expressions and a CAVEAT standard prelude that formats the HUD; source countdowns control notices. Pure functions are reusable expressions; bindings connect evaluated values to generic presentation targets:

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

Holding the light on the center of the current ring for one continuous second measures its flow. A small meter fills in the world and HUD. Moving the light away cancels an incomplete reading. The ferry keeps advancing and turning toward the light throughout: the reading costs actual control time as well as examination budget.

The opening provides more than five seconds of unobstructed water. Six rocks form three paired barriers across the bounded channel, requiring a right, left, then right passage toward the harbor. Holding one edge cannot bypass the course. A successful trip takes roughly a minute to a minute and a half. The collision circle combines the source's rock radius and ferry radius. A hit reduces hull once, nudges the ferry sideways, and grants two seconds of collision immunity.

The source channel runs from `x=-9.5` to `x=13`. At the three barrier centers (`z=45`, `36`, and `27`), an undamaged ferry must pass respectively to the right of `x=2.4`, left of `x=0.1`, and right of `x=3.2`. These limits follow from the authored rock positions and collision radii. They leave clear routes while ruling out a single held direction for most of the game.

## Knowledge changes the simulation

An old chart initially supports `clear_course`. Starting the rescue commits to `follow_light`, retaining six caveats about submerged rocks. Scouting or striking a rock adds its reading as opposing evidence. The old supporting evidence remains in the graph.

Each discovered rock examines its corresponding caveat once. The first contrary observation reopens the course commitment. Near a known rock, the source's `observed(...)` and `reopened(...)` predicates activate a warning and reduce forward speed from `0.6` to `0.42`, allowing more reaction time. Knowledge therefore changes motion; it is not only a journal entry. A collision is also evidence, including the final damaging frame.

### A forecast can be useful and still need revision

The crosscurrent adds a second provisional decision. `morning_forecast`, with tidal-archive provenance, supports `crosscurrent_mild`. `surge_unmeasured` qualifies that evidence. Starting the rescue commits to `trust_forecast` while retaining the caveat.

Completing a held-light sample before the storm yields `crosscurrent_reading`: measured foam drift opposes the forecast claim and supports the provisional claim that the current runs eastward. Examination costs one attention unit. The original supporting evidence is preserved. The observation reopens `trust_forecast`. A qualified measurement then passes through pure functions to produce the steering plan on which `counter_steer` relies.

| Physical truth | Navigation response |
| --- | --- |
| `current_force` depends on the authored field, time pulse, and ferry's position within it. | The forecast supplies a qualified zero-drift estimate, so the original steering plan applies no compensation. |
| Observing the current never weakens or removes it. | The revised policy counters 65% of the estimated current while continuing toward harbor. |

The foam marker, force, opposing reading, warning, response, and cue all come from this source file. The renderer supplies a generic ring asset; it has no crosscurrent-specific behavior. There is no forced reading panel, and an objection does not automatically halt the ferry. The earlier forecast and retained caveat remain inspectable after revision.

### The storm makes an old reading stale

At thirty-four seconds, the sea darkens, rain strengthens, and the current reverses from `+0.9` eastward to `-1.2` westward. The ring turns orange and its arrows reverse. This physical change occurs independently of any observation. A visible `storm_warning` reopens the old commitment without supplying an exact new measurement.

The player can keep the old plan and steer manually, or hold the orange ring for a fresh one-second reading. The original plan continues acting until replaced; after reversal, its correction can worsen the drift. The ring lies beside the route's left-hand leg, giving a fair opportunity to resample while the ferry keeps moving.

A completed second sample creates distinct `storm_reading` evidence and a `revised_counter_steer` commitment. It examines `reading_may_age` at a cost of one attention unit. The earlier supporting observation, contrary storm evidence, old number, and reopened commitment remain inspectable. If the player first samples after the storm, the source creates only the fresh storm reading; it does not invent an earlier observation. The total budget of eight covers six rocks and both readings, and spending it does not prevent continuing the rescue.

### The steering calculation carries its reasons

`forecast_drift = qualified(0, morning_forecast)` is the initial working estimate. The constructor attaches the observed forecast and inherits `surge_unmeasured` from the graph. `trust_forecast` uses that value, retaining its caveat automatically.

The light takes a simulated foam-speed sample when the held observation completes. `qualified(sample, crosscurrent_reading)` carries that reading and its qualifications into `observed_foam`. The pure functions `infer_peak()` and `counter_plan()` produce `estimated_peak` and `steering_plan` without stripping their metadata:

```caveat
fn infer_peak(flow, phase) = flow / phase;
fn counter_plan(peak, fraction) = -peak * fraction;
fn current_field(peak, phase, separation, radius) = peak * phase * clamp(1 - separation / radius, 0, 1);
```

The first revised commitment says `commit counter_steer because enough using steering_plan`. Its evidence basis and retained caveats come from the derived value actually used, rather than another hand-authored `retaining` list. The command is approximately `-0.585` at peak. Its numeric value remains fixed after the observation.

After resampling, `storm_foam` passes through the same functions into `storm_peak` and `revised_plan`, approximately `+0.78` at peak. `revised_counter_steer` commits using that new value. `active_plan` selects the retained command that `current_field()` projects over the local pulse and distance on later ticks. The navigator does not keep borrowing the simulator's current strength as fresh telemetry. Both commitments retain unresolved caveats about later changes; examination has not erased them.

`snapshot.values` remains a plain numeric projection for compatibility. `qualified_values` exposes the associated evidence and caveats; `commitment_bases` preserves the number and provenance used at each commitment. Metadata can also travel through a command into the resulting boat position and a position-dependent force calculation. That is computational lineage, not a declaration that observation changed the physical current or that every qualified value is false. At the same actual position and time, the external field still has the same numeric force.

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
| `forecast_drift`, `observed_foam`, `estimated_peak`, `steering_plan` | Qualified forecast, measurement, inference, and command |
| `measurement_ready` | Preserves the first sampled numeric steering plan |
| `storm_at`, `storm_strength`, `storm_changed` | Authored timing and physical change |
| `sample_charge`, `sample_duration`, `sample_in_range` | Continuous hold requirement and cancelable progress |
| `storm_foam`, `storm_peak`, `revised_plan`, `active_plan` | Fresh qualified sample, inference, revision, and selected response |
| `resample_ready` | Records completion of the storm reading |
| `notice_kind`, `notice_remaining` | Source-selected notice and simulation-time lifetime |
| `failure_reason` | `1` hull lost, `2` missed harbor, `3` time expired |
| `rescued`, `total_passengers`, `score` | Source-authoritative result |
| `aim_x_min/max`, `aim_z_min/max`, `aim_speed` | Input limits and keyboard target speed |

The score rewards remaining hull, time remaining, and discovered rocks. Its equation is authored in the source. CAVEAT's source prelude supplies `number_text`, `time_text`, and `percent_text`; the browser assigns their evaluated strings. Hull indicators also receive individual source bindings.

Sounds and expanding rings are declared with `cue` and emitted by source rules. Observation guards prevent repeated discovery cues; terminal rules emit before changing phase so repeated ticks cannot replay an ending sound. Cue delivery is part of the event transaction, alongside graph and numeric changes. Toast visibility and impact flashes use source bindings and countdowns, so pausing also pauses their lifetime.

## Verify

```sh
cargo test --manifest-path runtime/Cargo.toml --test light_the_way
npm run build
npm run test:rescue
npm run serve
```

Open `/rescue.html` on the local server. Build prerequisites are in the [README](../README.md).

Runtime verification covers the safe opening, real-input rescue, unattended failure, impact immunity, observations and reopened knowledge, source tuning, pause, controls, and explicit cues. Storm checks cover continuous sampling, cancellation, paused timers, manual and revised routes, and preserved old commitments. Qualified-value checks follow both measurements through pure functions into their commands and commitment bases. Browser checks additionally exercise direct controls, a complete rescue, mobile presentation, and source-only changes to HUD text, an object binding, and a previously unknown cue.
