# T1 M4 S8

Site: save encode: basis reversed

File: `glowcap.ts`
Line: 587 (line numbers are those of the original file)

Line 587, before:

```
      s.decision.basis.map(encodeEv),
```

Line 587, after:

```
      s.decision.basis.map(encodeEv).reverse(),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
