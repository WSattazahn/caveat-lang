# T2 M4 S11

Site: save decode: basis reversed

File: `glowcap.ts`
Line: 689 (line numbers are those of the original file)

Line 689, before:

```
      basis: basis.map(idFromCode),
```

Line 689, after:

```
      basis: basis.map(idFromCode).reverse(),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
