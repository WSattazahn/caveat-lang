# C2 M2-S2

- File: `adapter.mjs`
- Line: 90
- Site: decision.caveats read from frozen grounds.caveats: compute them live from the basis evidence's current qualifies relations

Before:

```
      caveats: [...grounds.caveats],
```

After:

```
      caveats: current ? [...new Set(v.relations.filter((r) => r.relation === 'qualifies' && current.because.includes(r.to)).map((r) => r.from))] : [],
```
