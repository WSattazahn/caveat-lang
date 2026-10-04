# C3 M2-S1

- File: `adapter.mjs`
- Line: 64
- Site: basis read from the committed journal entry's because: make it follow live state (the belief's current supported citations)

Before:

```
  const basis = current < 0 ? [] : names(journal[current].because);
```

After:

```
  const basis = current < 0 ? [] : cited(view, 'belief', 'supported');
```
