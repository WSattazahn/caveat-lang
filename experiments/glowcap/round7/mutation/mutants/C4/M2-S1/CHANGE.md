# C4 M2-S1

- File: `adapter.mjs`
- Line: 96
- Site: decision.basis read from the committed journal entry / commitment grounds: make it follow live state (the belief's current supported citations)

Before:

```
      basis: names(committed?.because ?? grounds?.evidence),
```

After:

```
      basis: current ? evidence(x, 'belief', 'supported') : [],
```
