# C4 M2-S2

- File: `adapter.mjs`
- Line: 97
- Site: decision.caveats read from frozen commitment_grounds caveats: compute them live from the basis evidence's current qualifies relations

Before:

```
      caveats: [...(grounds?.caveats ?? [])],
```

After:

```
      caveats: [...new Set(v.relations.filter((r) => r.relation === 'qualifies' && (grounds?.evidence ?? []).includes(r.to)).map((r) => r.from))],
```
