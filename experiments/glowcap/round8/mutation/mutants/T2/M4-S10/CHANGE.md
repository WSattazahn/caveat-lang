# T2 M4 S10

Site: save encode: journal because reversed

File: `glowcap.ts`
Line: 646 (line numbers are those of the original file)

Line 646, before:

```
    j: s.journal.map((x): [string, string[]] => [JOURNAL_LETTERS[x.entry], x.because.map(idCode)]),
```

Line 646, after:

```
    j: s.journal.map((x): [string, string[]] => [JOURNAL_LETTERS[x.entry], x.because.map(idCode).reverse()]),
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
