# T1 M2-S1

- File: `glowcap.ts`
- Line: 230
- Site: render: decision caveats come from the copy stored at commit (clone(s.decision)); read caveatsOf(s, basis) live instead

Before:

```
    decision: clone(s.decision),
```

After:

```
    decision: { ...clone(s.decision), caveats: caveatsOf(s, s.decision.basis) },
```
