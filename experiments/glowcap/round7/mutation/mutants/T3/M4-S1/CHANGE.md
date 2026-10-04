# T3 M4-S1

- File: `glowcap.ts`
- Line: 333
- Site: recovery basis `d.recovery.map(...)` passed to commit: reverse it

Before:

```
    const basis = d.recovery.map((id) => findObservation(s, id) ?? fail(`lost evidence ${id}`));
```

After:

```
    const basis = d.recovery.map((id) => findObservation(s, id) ?? fail(`lost evidence ${id}`)).reverse();
```
