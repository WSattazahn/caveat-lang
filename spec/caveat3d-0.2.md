# CAVEAT 3D Presentation Contract — 0.2

**Status:** executable semantic presentation layer.

## Purpose

CAVEAT 3D 0.2 removes the main 0.1 shortcut: presentation no longer needs a per-action choreography table for normal movement, inspection, operation, opening, observation, or staying in place.

The runtime path is now:

```
.cav source
  -> CAVEAT Map
  -> ActionRuntime
       -> normalized ActionExecution
            Inspect(entity)
            Operate(entity)
            Open(entity)
            Move(from, to, via?)
            Observe(symbol)
            Stay(place)
  -> CAVEAT 3D semantic bindings
       place -> camera framing
       entity -> visible object(s) / focus
       symbol -> visible object(s) / reveal
  -> renderer
```

The semantic action execution comes from the same Rust action runtime used by CAVEAT simulation tooling. The renderer does not decide what an action means.

## Presentation bindings

A 0.2 manifest binds semantic identifiers to presentation data:

```js
{
  schema: "caveat3d/0.2",

  places: {
    moon_bridge: {
      label: "Moon Garden · Moon Bridge",
      camera: { position: [...], target: [...] },
      duration: 0.6
    }
  },

  entities: {
    tea_cushion: {
      objects: ["tea_cushion"],
      camera: { position: [...], target: [...] }
    }
  },

  symbols: {
    miso_found: {
      objects: ["miso_body", "miso_head"],
      reveal: ["miso_body", "miso_head"],
      camera: { position: [...], target: [...] }
    }
  }
}
```

These bindings answer only presentation questions:

- where should the camera frame a place?
- which scene object represents an entity?
- which visible objects correspond to an observed symbol?
- should an observed symbol reveal a hidden object?

They do not define action legality, action order, destinations, evidence, or reopening.

## Execution

For each player selection, the browser:

1. previews the complete action sequence with `caveat_simulate`;
2. rejects the interaction if the Rust action runtime rejects it;
3. applies the interaction to the real `WebSession`;
4. takes the last normalized `ActionExecution`;
5. maps each command to place/entity/symbol presentation bindings;
6. animates the resulting camera path and emphasis/reveal state.

This means the renderer consumes **executed semantics**, not action names.

## Explicit convergence

Some interactive stories deliberately explore different physical routes and return to a shared decision point.

CAVEAT now supports:

```caveat
choice first_move options follow_bell, follow_tracks, check_greenhouse;
converge first_move;
```

Without `converge`, choices whose action plans all end at the same place remain invalid. The marker is therefore an explicit statement that route convergence is intentional rather than an accidental collapse of distinct choices.

## Moon Garden proof

Moon Garden now authors physical semantics for every player-facing interaction in `game/moon_garden.cav`.

Examples:

```caveat
action cushion_may_hold_old_fur
  from lantern_courtyard
  to lantern_courtyard
  steps move tea_pavilion,
        inspect tea_cushion,
        observe black_fur,
        move lantern_courtyard;

action search_tea_pavilion
  from lantern_courtyard
  to fern_nook
  steps move tea_pavilion,
        move fern_nook,
        observe miso_found;
```

The 3D manifest contains no normal per-action camera choreography. Camera movement comes from the `Move` commands; clue focus comes from `Inspect` and `Observe`; Miso appears because the runtime executes `Observe(miso_found)`.

## Remaining caveats

CAVEAT 3D still requires authored presentation bindings for places, entities, and symbols. It cannot infer attractive camera composition or art direction from semantics alone.

Narrative ending copy may still be keyed by the chosen commitment because different routes can tell different stories even when they converge physically.

Those are presentation concerns. The important 0.2 boundary is that **physical action meaning and command order no longer live in per-action renderer code or manifest entries**.
