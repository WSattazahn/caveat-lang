# C1 M2-S1

- File: `adapter.mjs`
- Line: 81
- Site: decision.basis read from frozen commitment_grounds[current].evidence: make it follow live state (the belief's current supportedBy citations)

Before:

```
      basis: current ? view.commitment_grounds[current].evidence.map(life) : [],
```

After:

```
      basis: current ? cites('belief', 'supportedBy') : [],
```
