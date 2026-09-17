# CAVEAT 3D

CAVEAT 3D is a scene/world layer for CAVEAT programs. The language owns world identity, state, interaction, evidence, caveats, commitments, and transitions. A renderer owns rasterization.

## Principle

Do not turn CAVEAT into a list of WebGL calls. A CAVEAT world describes objects and epistemic/game relationships; the runtime emits a renderer-neutral scene snapshot and transition events.

```text
.cav source
  -> parser / evaluator
  -> Session
  -> World3D snapshot + events
  -> renderer adapter
  -> WebGL/WebGPU/native renderer
```

## First vertical slice: The Door 3D

The existing The Door scenario becomes a first-person corridor. The player can look around, select the camera or latch sensor, spend attention investigating, see evidence appear, commit to opening/waiting/rerouting, see the door/world react, and reconsider when a commitment reopens.

## Proposed language surface

```caveat
world3d evacuation_corridor;

camera3d player position 0 1.7 5 look_at 0 1.5 -4 fov 70;

object3d floor box position 0 -0.1 0 scale 8 0.2 18;
object3d door box position 0 1.2 -4 scale 2.2 2.4 0.2;
object3d latch box position 1.0 1.2 -3.8 scale 0.15 0.25 0.12 interactive latch_sensor_recently_serviced;
object3d security_camera box position -2.5 2.6 -2 scale 0.3 0.2 0.5 interactive camera_has_blind_spot;
object3d stairwell box position 3 1 -7 scale 2 2 0.2 interactive stairwell;

light3d emergency point position 0 2.7 1 intensity 0.8;

when committed open animate3d door rotate_y 90 over 1.2;
when committed stairwell animate3d player move_to 3 1.7 -6 over 1.5;
```

Exact syntax is provisional until parser work lands. The semantic split is not provisional: scene declarations produce runtime scene state; epistemic symbols remain the authoritative interaction identifiers.

## Runtime types

The initial renderer-neutral API should expose:

- `World3D`: camera, objects, lights.
- `Object3D`: id, primitive/asset, transform, optional interactive CAVEAT symbol.
- `Camera3D`: transform and projection.
- `Light3D`: kind, transform, intensity.
- `WorldEvent3D`: animation/state transition emitted by evaluated CAVEAT actions.
- `pick(symbol)`: maps a rendered object interaction back to a CAVEAT investigation/choice symbol.

Coordinates and transforms use floating-point values even though the epistemic language currently uses integer resource units.

## Renderer boundary

Version 0 should use a tiny browser renderer adapter. It may use WebGL directly or a rendering library, but it must consume CAVEAT's `World3D` snapshot/events. Game rules must not migrate into JavaScript.

The renderer is replaceable. A later native/Unity/Godot/WebGPU renderer should be able to consume the same CAVEAT world representation.

## Milestones

1. Add 3D AST/runtime structures without changing existing CAVEAT semantics.
2. Add parser syntax for camera/object/light declarations.
3. Export renderer-neutral scene JSON through `WebSession`.
4. Render The Door corridor in a canvas on mobile Safari.
5. Ray/pointer picking maps camera/latch/door/stairwell to CAVEAT symbols.
6. Emit door/player animations from commitment events.
7. Add visual state for unresolved caveats without making the renderer decide epistemic truth.
8. Keep existing text UI as an accessibility/debug overlay.

## Non-goals for v0

No custom triangle rasterizer, physics engine, skeletal animation system, asset pipeline, or editor. Those can evolve after the language/runtime boundary is proven.
