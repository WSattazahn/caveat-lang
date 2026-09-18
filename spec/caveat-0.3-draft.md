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
