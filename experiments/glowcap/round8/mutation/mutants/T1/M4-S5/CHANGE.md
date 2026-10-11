# T1 M4 S5

Site: render: decision.basis reversed

File: `glowcap.ts`
Line: 514 (line numbers are those of the original file)

Line 514, before:

```
    decision: { state: s.decision.state, basis: [...s.decision.basis], reopenedBy: [...s.decision.reopenedBy], caveats: [...s.decision.caveats],
```

Line 514, after:

```
    decision: { state: s.decision.state, basis: [...s.decision.basis].reverse(), reopenedBy: [...s.decision.reopenedBy], caveats: [...s.decision.caveats],
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
