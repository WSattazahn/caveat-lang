# T1 M5 S28

Site: add an evidence id the value never reads to the explanation list: journal trust entry because

File: `glowcap.ts`
Line: 324 (line numbers are those of the original file)

Line 324, before:

```
    pushBounded(s.journal, { text, because: [...change.because] }, JOURNAL_MEMORY);
```

Line 324, after:

```
    pushBounded(s.journal, { text, because: [...change.because, 'absorb_cave'] }, JOURNAL_MEMORY);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
