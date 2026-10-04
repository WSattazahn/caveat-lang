# T3 M2-S1

- File: `glowcap.ts`
- Line: 286
- Site: render: decision caveats `[...d.caveats]` (copy taken at commit); read the basis evidence's caveats live instead

Before:

```
    decision: { state: d.state, basis: [...d.basis], reopenedBy: [...d.reopenedBy], caveats: [...d.caveats], history: clone(d.history) },
```

After:

```
    decision: { state: d.state, basis: [...d.basis], reopenedBy: [...d.reopenedBy], caveats: caveatsOf(s.memory.filter((e) => d.basis.includes(e.id))), history: clone(d.history) },
```
