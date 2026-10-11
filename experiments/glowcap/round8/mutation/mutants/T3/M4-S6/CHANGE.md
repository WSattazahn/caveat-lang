# T3 M4 S6

Site: commit: decision.basis reversed

File: `glowcap.ts`
Line: 271 (line numbers are those of the original file)

Line 271, before:

```
  d.basis = basis;
```

Line 271, after:

```
  d.basis = [...basis].reverse();
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
