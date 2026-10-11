# T2 M4 S4

Site: commit: d.basis = [...basis] reversed

File: `glowcap.ts`
Line: 337 (line numbers are those of the original file)

Line 337, before:

```
  d.basis = [...basis];
```

Line 337, after:

```
  d.basis = [...basis].reverse();
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
