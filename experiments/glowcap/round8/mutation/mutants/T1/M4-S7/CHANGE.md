# T1 M4 S7

Site: render: journal because reversed

File: `glowcap.ts`
Line: 517 (line numbers are those of the original file)

Line 517, before:

```
    journal: s.journal.map((j) => ({ text: j.text, because: [...j.because], caveats: caveatsOf(s, j.because) })),
```

Line 517, after:

```
    journal: s.journal.map((j) => ({ text: j.text, because: [...j.because].reverse(), caveats: caveatsOf(s, j.because) })),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
