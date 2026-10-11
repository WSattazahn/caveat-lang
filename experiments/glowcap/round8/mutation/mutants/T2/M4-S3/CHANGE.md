# T2 M4 S3

Site: viewOf: journal because reversed

File: `glowcap.ts`
Line: 309 (line numbers are those of the original file)

Line 309, before:

```
    journal: s.journal.map((j) => ({ text: journalText(j), because: [...j.because], caveats: caveatsOf(s, j.because) })),
```

Line 309, after:

```
    journal: s.journal.map((j) => ({ text: journalText(j), because: [...j.because].reverse(), caveats: caveatsOf(s, j.because) })),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
