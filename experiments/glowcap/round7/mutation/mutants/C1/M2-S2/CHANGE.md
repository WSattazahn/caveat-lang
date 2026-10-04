# C1 M2-S2

- File: `adapter.mjs`
- Line: 83
- Site: decision.caveats read from frozen commitment_grounds[current].caveats: compute them live from the basis evidence's current qualifies relations

Before:

```
      caveats: current ? [...view.commitment_grounds[current].caveats] : [],
```

After:

```
      caveats: current ? [...new Set(view.relations.filter((r) => r.relation === 'qualifies' && view.commitment_grounds[current].evidence.includes(r.to)).map((r) => r.from))] : [],
```
