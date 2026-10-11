# T2 M4 S13

Site: save decode: journal because reversed

File: `glowcap.ts`
Line: 700 (line numbers are those of the original file)

Line 700, before:

```
      return { entry, because: because.map(idFromCode) };
```

Line 700, after:

```
      return { entry, because: because.map(idFromCode).reverse() };
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
