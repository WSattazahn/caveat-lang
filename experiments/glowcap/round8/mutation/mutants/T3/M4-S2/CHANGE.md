# T3 M4 S2

Site: render: decision.basis reversed

File: `glowcap.ts`
Line: 196 (line numbers are those of the original file)

Line 196, before:

```
      basis: [...d.basis],
```

Line 196, after:

```
      basis: [...d.basis].reverse(),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
