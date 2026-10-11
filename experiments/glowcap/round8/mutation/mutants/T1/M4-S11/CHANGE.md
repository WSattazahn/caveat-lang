# T1 M4 S11

Site: save decode: journal because reversed

File: `glowcap.ts`
Line: 625 (line numbers are those of the original file)

Line 625, before:

```
  s.journal = v.j.map(([text, because]) => ({ text, because: because.map(decodeEv) }));
```

Line 625, after:

```
  s.journal = v.j.map(([text, because]) => ({ text, because: because.map(decodeEv).reverse() }));
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
