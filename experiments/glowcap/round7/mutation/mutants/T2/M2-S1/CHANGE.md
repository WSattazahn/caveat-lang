# T2 M2-S1

- File: `glowcap.ts`
- Line: 396
- Site: render: decision caveats `[...s.decision.caveats]` (copy taken at commit); read caveatsOf(s, basis) live instead

Before:

```
      caveats: [...s.decision.caveats],
```

After:

```
      caveats: caveatsOf(s, s.decision.basis),
```
