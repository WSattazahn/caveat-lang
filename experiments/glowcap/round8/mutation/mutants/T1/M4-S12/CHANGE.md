# T1 M4 S12

Site: save decode: basis reversed

File: `glowcap.ts`
Line: 633 (line numbers are those of the original file)

Line 633, before:

```
    basis: basis.map(decodeEv),
```

Line 633, after:

```
    basis: basis.map(decodeEv).reverse(),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
