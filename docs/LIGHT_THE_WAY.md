# Light the Way

**Drag the light left or right. Reach the green harbor.**

Light the Way turns Saint Orin into a short, directly controlled rescue game. A visible ferry advances toward the island while the player guides it past six rocks. The green harbor ring rescues all thirty-two passengers. Three impacts, missing the harbor, or ninety seconds without arrival ends the attempt. A new session immediately restarts it.

There are no required story decisions or reading panels. The earlier narrative game remains separately available as [The Last Beacon](THE_LAST_BEACON.md).

## The game is a CAVEAT program

[`game/light_the_way.cav`](../game/light_the_way.cav) defines all simulation values, movement equations, steering acceleration, wind, discovery, collisions, cooldown, scoring, success, and failure. It also defines the world coordinates used by the renderer. The browser submits input and time events, then renders the returned state. It does not decide whether a collision or rescue occurred.

The source uses a numeric/event execution profile alongside CAVEAT's existing epistemic graph. For example:

```caveat
state forward_speed = 0.6;
event tick dt min 0 max 0.1;
on tick when phase == 1 set boat_z = boat_z - boat_speed * dt;
```

Expressions can refer directly to source positions such as `reef_one.x` and `harbor_goal.z`. Physics and visuals consequently share one authored location for each rock and the harbor.

## One input, visible consequences

The ferry moves forward automatically. Holding the light steers it laterally toward the light's horizontal position, with acceleration and a capped turn speed. The light's full position also scouts the water: shining near a rock reveals it. Releasing the light lets the ferry's lateral movement settle while it continues forward.

The opening provides more than five seconds of unobstructed water. Six rocks form three paired barriers across the bounded channel, requiring a right, left, then right passage toward the harbor. Holding one edge cannot bypass the course. A successful trip takes roughly a minute to a minute and a half. The collision circle combines the source's rock radius and ferry radius. A hit reduces hull once, nudges the ferry sideways, and grants two seconds of collision immunity.

The source channel runs from `x=-9.5` to `x=13`. At the three barrier centers (`z=45`, `36`, and `27`), an undamaged ferry must pass respectively to the right of `x=2.4`, left of `x=0.1`, and right of `x=3.2`. These limits follow from the authored rock positions and collision radii. They leave clear routes while ruling out a single held direction for most of the game.

## Knowledge changes the simulation

An old chart initially supports `clear_course`. Starting the rescue commits to `follow_light`, retaining six caveats about submerged rocks. Scouting or striking a rock adds its reading as opposing evidence. The old supporting evidence remains in the graph.

Each discovered rock examines its corresponding caveat once. The first contrary observation reopens the course commitment. Near a known rock, the source's `observed(...)` and `reopened(...)` predicates activate a warning and reduce forward speed from `0.6` to `0.42`, allowing more reaction time. Knowledge therefore changes motion; it is not only a journal entry. A collision is also evidence, including the final damaging frame.

## Input and state contract

| Input event | Parameters | Meaning |
| --- | --- | --- |
| `start` | None | Start the rescue and commit to the course |
| `aim` | `x`, `z`, `active` | Move the light and hold/release it |
| `tick` | `dt`, between `0` and `0.1` seconds | Advance the source simulation |

Restart constructs a new reactive session from the same source. Background or paused views stop submitting ticks.

| Source values | Presentation purpose |
| --- | --- |
| `boat_x`, `boat_z`, `boat_vx` | Ferry position and heading |
| `aim_x`, `aim_z`, `light_on` | Light target and beam |
| `phase` | `0` ready, `1` playing, `2` rescued, `3` failed |
| `hull`, `collision_cooldown`, `hit_count` | Hull indicators and impact feedback |
| `elapsed`, `time_limit`, `progress` | Time and route progress |
| `warning` | `0` clear, `1` light released, `2` known rock or impact |
| `reef_one_seen` through `reef_six_seen` | Discovered-rock presentation |
| `failure_reason` | `1` hull lost, `2` missed harbor, `3` time expired |
| `rescued`, `total_passengers`, `score` | Source-authoritative result |
| `aim_x_min/max`, `aim_z_min/max`, `aim_speed` | Input limits and keyboard target speed |

The score rewards remaining hull, time remaining, and discovered rocks. Its equation is authored in the source. The interface may format or round the number for display and retain a local best score.

## Verify

```sh
cargo test --manifest-path runtime/Cargo.toml --test light_the_way
npm run build
npm run test:rescue
npm run serve
```

Open `/rescue.html` on the local server. Build prerequisites are in the [README](../README.md).

Runtime verification covers the safe opening, real-input rescue, unattended failure, impact immunity, observations and reopened knowledge, and changes to trajectories when only source tuning changes. Browser tests additionally check immediate drag response, the visible ferry and harbor, impact feedback, a complete rescue, and mobile controls.
