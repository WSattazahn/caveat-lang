# T1 M4 S10

Site: save encode: journal because reversed

File: `glowcap.ts`
Line: 592 (line numbers are those of the original file)

Line 592, before:

```
    j: s.journal.map((j) => [j.text, j.because.map(encodeEv)]),
```

Line 592, after:

```
    j: s.journal.map((j) => [j.text, j.because.map(encodeEv).reverse()]),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
