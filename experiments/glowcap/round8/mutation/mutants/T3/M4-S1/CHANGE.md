# T3 M4 S1

Site: render: journal because reversed

File: `glowcap.ts`
Line: 193 (line numbers are those of the original file)

Line 193, before:

```
    journal: s.journal.map((j) => ({ text: journalText(j), because: [...j.because], caveats: caveatsOf(s, j.because) })),
```

Line 193, after:

```
    journal: s.journal.map((j) => ({ text: journalText(j), because: [...j.because].reverse(), caveats: caveatsOf(s, j.because) })),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
