# CAVEAT Language Specification — Draft 0.3 (world-map integration)

This draft extends the 0.1/0.2 epistemic model with explicit world topology. The goal is to make reasoning state and physical state part of one checked program model.

## 1. CAVEAT Map

Every parsed program can be projected into a machine-readable CAVEAT Map. The map is the shared semantic interface for validators, runtimes, renderers, tests, and AI tooling.

The initial schema is documented in `spec/caveat-map-0.1.md`.

## 2. Places

```
place start_corridor kind corridor;
place stair_landing kind stairwell;
```

A place is a named location in the world graph. Place identifiers are unique within world topology.

## 3. Entities

```
entity door_a kind fire_door at start_corridor;
```

An entity has an identifier, a semantic kind, and a declared location.

The map rejects an entity whose location does not name a declared place.

## 4. Connections

```
connect start_corridor to cross_corridor via door_a;
connect cross_corridor to continuation_corridor;
```

A connection declares traversable adjacency between two places. In Draft 0.3, `connect A to B` describes topology rather than one-way motion; reachability tools treat it as bidirectional unless a future directional construct says otherwise.

A connection may name a connector entity such as a door.

Validation rejects:

- an unknown endpoint;
- an unknown connector entity;
- a connector not located at either endpoint;
- self-connections;
- duplicate topology edges with the same connector.

## 5. Separation from rendering

Places, entities, and connections are semantic declarations. They do not prescribe meshes, coordinates, materials, or animation.

A renderer may map semantic kinds such as `corridor`, `fire_door`, and `stairwell` to presentation templates. It must not invent topology that is absent from the CAVEAT Map.

## 6. The Door milestone

The canonical Door scenario now declares:

```
start_corridor
  -- door_a -- cross_corridor
                  -- continuation_corridor
                  -- stair_door_b -- stair_landing -- stair_flight_down

start_corridor
  -- alternate_route
```

This moves the first authoritative part of the game's physical layout out of `world3d.rs` and into CAVEAT source.

## 7. Next step

Draft 0.3 next adds action plans that reference this topology. That layer must statically reject impossible transitions such as traversing a closed barrier without an opening operation or claiming a stairwell destination that is not reachable through the world graph.


## 8. World start state

A world with executable action plans declares the player's initial logical place:

```
start_at start_corridor;
```

The start place must exist. A world with action plans but no `start_at` is invalid.

## 9. Action plans

Player-facing actions can declare a checked physical plan:

```
action open
  from start_corridor
  to cross_corridor
  steps operate door_a, open door_a, through door_a, observe smoke_report;
```

The bootstrap parser accepts the same declaration on one source statement:

```
action open from start_corridor to cross_corridor steps operate door_a, open door_a, through door_a, observe smoke_report;
```

Available steps in the first executable action-plan profile are:

- `inspect ENTITY`
- `operate ENTITY`
- `open ENTITY`
- `through ENTITY`
- `move PLACE`
- `observe SYMBOL`
- `stay`

An action may declare a state precondition established by an earlier action:

```
action retreat from cross_corridor to start_corridor requires_open door_a steps through door_a;
```

This is different from silently assuming the door is open. The runtime carries barrier state across commitments.

## 10. Static action-plan checks

Before execution, the CAVEAT Map rejects an action plan when:

- its source or destination place does not exist;
- it references an unknown entity or observation symbol;
- it opens an entity that is not openable;
- it operates/inspects an entity inaccessible from the current simulated place;
- it traverses a barrier that is neither opened earlier in the plan nor declared in `requires_open`;
- a `through` step names a connector that does not connect the current place;
- a `move` step lacks a direct connector-free world edge;
- the step sequence finishes somewhere other than the declared destination;
- a player-facing option has no action plan in an action-plan-enabled world;
- two options in one choice accidentally resolve to the same destination without an explicit convergence mechanism.

The validator therefore catches the class of error where a script says “use the stairwell” but the declared physical sequence never reaches a stairwell.

## 11. Generic action runtime

The reference runtime executes validated plans into generic commands:

```
Inspect(entity)
Operate(entity)
Open(entity)
Move(from, to, via?)
Observe(symbol)
Stay(place)
```

The action runtime tracks current logical place and open barriers independently from any renderer. Presentation layers consume these generic commands.

The 3D renderer no longer selects behavior by matching action names such as `open`, `retreat`, or `stairwell`. It receives the action execution and maps places/entities to presentation anchors.

## 12. Current migration boundary

The Door's topology and action consequences now belong to CAVEAT source. The browser still contains scenario-specific geometry and presentation anchors for the canonical demo; moving those templates out of handwritten HTML/Rust is the next presentation phase.

The important boundary is already enforced: changing the semantic route or action sequence is a CAVEAT-source change, not a new Rust action-name branch.
