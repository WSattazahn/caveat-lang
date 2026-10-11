# T2 M4 S12

Site: save decode: history because reversed

File: `glowcap.ts`
Line: 694 (line numbers are those of the original file)

Line 694, before:

```
        because: because.map(idFromCode),
```

Line 694, after:

```
        because: because.map(idFromCode).reverse(),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
