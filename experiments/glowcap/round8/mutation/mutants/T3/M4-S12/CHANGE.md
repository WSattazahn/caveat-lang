# T3 M4 S12

Site: save decode: basis reversed

File: `glowcap.ts`
Line: 568 (line numbers are those of the original file)

Line 568, before:

```
  d.basis = unrefs(sv.d.b);
```

Line 568, after:

```
  d.basis = unrefs(sv.d.b).reverse();
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
