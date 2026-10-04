# C2 M2-S1

- File: `adapter.mjs`
- Line: 89
- Site: decision.basis read from the committed journal entry's because: make it follow live state (the belief's current supported citations)

Before:

```
      basis: current ? named(current.because) : [],
```

After:

```
      basis: current ? cites(v, 'belief', 'supported') : [],
```
