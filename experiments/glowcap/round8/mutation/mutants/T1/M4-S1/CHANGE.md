# T1 M4 S1

Site: commit: decision.basis = [...basis] reversed

File: `glowcap.ts`
Line: 276 (line numbers are those of the original file)

Line 276, before:

```
  s.decision.basis = [...basis];
```

Line 276, after:

```
  s.decision.basis = [...basis].reverse();
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
