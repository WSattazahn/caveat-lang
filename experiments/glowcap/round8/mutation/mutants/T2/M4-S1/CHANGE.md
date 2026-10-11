# T2 M4 S1

Site: viewOf: decision.basis reversed

File: `glowcap.ts`
Line: 304 (line numbers are those of the original file)

Line 304, before:

```
      basis: [...s.decision.basis],
```

Line 304, after:

```
      basis: [...s.decision.basis].reverse(),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
