# T2 M4 S6

Site: journal trust entry because (committed branch) reversed

File: `glowcap.ts`
Line: 361 (line numbers are those of the original file)

Line 361, before:

```
    const because = change === 'committed' ? [...s.decision.basis] : [e.id];
```

Line 361, after:

```
    const because = change === 'committed' ? [...s.decision.basis].reverse() : [e.id];
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
