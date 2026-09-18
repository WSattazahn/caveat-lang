# CAVEAT 3D Presentation Contract — 0.1

**Status:** experimental, executable browser presentation layer.

## Purpose

CAVEAT 3D 0.1 separates **program meaning** from **presentation**.

A CAVEAT source file remains authoritative for claims, evidence, caveats, investigations, choices, retained uncertainty, reopening, and any world/action semantics declared by the language.

The 3D renderer consumes that semantic program through the existing WebSession and CAVEAT Map, then applies a presentation manifest:

```
.cav source
  -> WebSession + CAVEAT Map
       -> CAVEAT 3D renderer
            + presentation manifest
              -> camera
              -> primitive scene
              -> emphasis/reveal choreography
              -> player HUD
```

The presentation manifest must not create or remove CAVEAT choices, fabricate evidence, resolve caveats, or reopen commitments on its own.

## Why the manifest is separate

The language specification deliberately keeps semantic declarations independent from meshes, coordinates, colors, materials, and camera movement.

CAVEAT 3D follows that boundary rather than adding rendering coordinates to `.cav` syntax.

This keeps two questions distinct:

1. **What does the program mean?** — CAVEAT source and CAVEAT Map.
2. **How is that meaning shown?** — CAVEAT 3D manifest and renderer.

## Manifest schema

The browser renderer currently recognizes:

```js
{
  schema: "caveat3d/0.1",
  source: "./scenario.cav",
  camera: {
    position: [x, y, z],
    target: [x, y, z],
    fov: degrees
  },
  atmosphere: {
    clear: [r, g, b],
    fog: [r, g, b]
  },
  objects: [
    {
      id: "unique_id",
      primitive: "cube" | "sphere" | "cylinder",
      position: [x, y, z],
      rotation: [xDeg, yDeg, zDeg],
      scale: [x, y, z],
      color: [r, g, b],
      emissive: number,
      opacity: number,
      visible: boolean
    }
  ],
  interactions: {
    interaction_name: {
      eyebrow: "...",
      title: "...",
      body: "..."
    }
  },
  hints: {
    caveat_or_action_id: "..."
  },
  caveats: {
    caveat_id: "..."
  },
  actions: {
    caveat_or_action_id: {
      camera: { position: [...], target: [...] },
      duration: seconds,
      emphasize: ["object_id"],
      reveal: ["object_id"],
      hide: ["object_id"],
      place: "..."
    }
  },
  endings: {
    action_id: {
      title: "...",
      body: "...",
      badge: "...",
      place: "...",
      reveal: ["object_id"],
      camera: { position: [...], target: [...] },
      duration: seconds
    }
  }
}
```

## Runtime contract

The generic renderer:

1. loads the referenced `.cav` source;
2. initializes the real WebSession;
3. builds the real CAVEAT Map;
4. takes player-facing labels from the map when available;
5. renders only options returned by `session.pending()`;
6. applies the actual selection through `session.apply()`;
7. reads actual discoveries and commitment reopening from the runtime;
8. uses manifest choreography only after the semantic transition has succeeded.

Therefore a presentation manifest can make a valid CAVEAT interaction visible, but it cannot make an invalid interaction true.

## 0.1 primitives

The first renderer intentionally keeps geometry small and inspectable:

- cube;
- low-poly sphere/ellipsoid;
- low-poly cylinder.

Complex props are compositions of these primitives. Moon Garden uses them for the pond, bridge, pavilion, greenhouse, lanterns, foliage, fireflies, and Miso.

This is a deliberate quality/performance compromise for mobile WebGL rather than a claim that three primitives are a complete graphics engine.

## Acceptance test

Moon Garden is the second substantially different world after The Door.

The 0.1 milestone succeeds only if:

- the existing 2D Moon Garden remains intact;
- Moon Garden 3D loads on an iPhone-sized WebKit viewport;
- the live CAVEAT investigation/commit/reopen/recommit flow completes;
- camera framing visibly changes in response to choices;
- the ending reveals Miso;
- the canvas visibly changes between start and ending;
- there are no browser console errors;
- The Door 3D regression suite still passes.

## Known caveat

CAVEAT 3D 0.1 presentation choreography is still keyed by CAVEAT option/action identifiers.

That is much better than hardcoding game logic into the renderer, because the semantic transition still belongs to CAVEAT, but it is not yet the final generic command-driven presentation layer.

The next phase should allow presentation choreography to bind to normalized CAVEAT Map/action-runtime concepts (places, entities, `Move`, `Inspect`, `Open`, `Observe`) so more scene behavior can be inferred without per-action presentation entries.
