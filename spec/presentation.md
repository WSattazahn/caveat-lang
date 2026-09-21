# Source-authored presentation

CAVEAT programs can declare world-space coordinates, camera views and the
physical shape of already connected routes. These declarations accompany the
world graph in the map and live game snapshot. They do not change which actions
are available, move the player, reveal evidence or determine an outcome.

```caveat
place court kind courtyard;
place harbor kind harbor;
entity receiver kind transmitter at court;
connect court to harbor;
start_at court;

position court at 0 3 0;
position harbor at 6 1 12;
position receiver at 2 3.2 -1;
camera court from 18 20 30 toward 0 5 0;
camera harbor from 24 13 32 toward 6 2 12;
overview from 35 30 46 toward 0 6 1;
route court to harbor via 2 3 7 then 4 2 10;
```

## Coordinates

`position TARGET at X Y Z` places a declared place or entity in absolute world
coordinates. X and Z span the ground plane; positive Y is up. Entity coordinates
are absolute even though the entity retains its semantic `at PLACE` owner. A
renderer subtracts the parent's position when it uses a local scene hierarchy.

Numbers may be signed decimals or scientific notation. They must parse as finite
IEEE 754 double values; NaN, infinity, overflow and nonnumeric tokens are errors.
Canonical numeric storage preserves equality for AST and transaction snapshots,
and JSON serialization emits ordinary numbers rather than quoted strings.

## Views

`camera PLACE from X Y Z toward X Y Z` declares the camera eye and look target
used when showing that place. Its target must be a place, not an entity. The eye
and look target must differ. `overview from ... toward ...` declares the initial
island/world view with the same coordinate rules.

## Routes

`route FROM to TO via X Y Z then X Y Z ...` supplies one or more intermediate
world-space points. The endpoints come from the two places' position declarations.
Renderers may interpolate the polyline smoothly, draw a path and animate a
generic runtime `move` along it. The same route works in reverse.

A route requires a preexisting `connect` edge, two distinct declared places and
explicit position declarations for both endpoints. It cannot create a shortcut
through an unconnected world. Consecutive repeated intermediate points are
invalid. Each undirected connection can have at most one route declaration.

## Validation and compatibility

References resolve after collecting declarations, so presentation may precede
world declarations. Duplicate positions, duplicate per-place cameras and more
than one overview are errors. Positions may refer only to a declared place or
entity. All view and route coordinates receive the same finite-number check.

Presentation is optional. Existing programs remain valid and omit the
`world.presentation` field when no directives are present. Renderers can use a
generic layout for older programs; scenario-specific coordinates belong in
CAVEAT source. Geometry templates still own dimensions of their component
meshes, materials and animation details.

When present, `world.presentation` contains `positions`, `cameras`, `overview`
and `routes`. Position records use `{target, position}`; camera records use
`{place, position, target}`; overview uses `{position, target}`; routes use
`{from, to, points}`. Every coordinate is a numeric `[x, y, z]` array.
