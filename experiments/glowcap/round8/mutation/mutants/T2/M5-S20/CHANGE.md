# T2 M5 S20

Site: add an evidence id the value never reads to the explanation list: journal observation entry because

File: `glowcap.ts`
Line: 359 (line numbers are those of the original file)

Line 359, before:

```
  pushBounded(s.journal, { entry: e.kind, because: [e.id] }, JOURNAL_KEEP);
```

Line 359, after:

```
  pushBounded(s.journal, { entry: e.kind, because: [e.id, 'absorb_cave'] }, JOURNAL_KEEP);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
