# C3 M2-S2

- File: `adapter.mjs`
- Line: 65
- Site: basisCaveats read from the committed journal entry's frozen caveats: compute them live from the basis evidence's current qualifies relations

Before:

```
  const basisCaveats = current < 0 ? [] : inOrder(view, journal[current].because, journal[current].caveats);
```

After:

```
  const basisCaveats = current < 0 ? [] : inOrder(view, journal[current].because, [...new Set(view.relations.filter((r) => r.relation === 'qualifies' && journal[current].because.includes(r.to)).map((r) => r.from))]);
```
