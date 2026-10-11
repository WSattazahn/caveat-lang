# T2 M4 S9

Site: save encode: history because reversed

File: `glowcap.ts`
Line: 645 (line numbers are those of the original file)

Line 645, before:

```
    d: [d.state[0], d.basis.map(idCode), d.reopenedBy.map(idCode), [...d.caveats], d.history.map((h): [string, string[]] => [h.change[0], h.because.map(idCode)])],
```

Line 645, after:

```
    d: [d.state[0], d.basis.map(idCode), d.reopenedBy.map(idCode), [...d.caveats], d.history.map((h): [string, string[]] => [h.change[0], h.because.map(idCode).reverse()])],
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
