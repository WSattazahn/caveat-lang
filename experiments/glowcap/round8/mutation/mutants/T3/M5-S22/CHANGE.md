# T3 M5 S22

Site: add an evidence id the value never reads to the explanation list: journal observation entry because

File: `glowcap.ts`
Line: 222 (line numbers are those of the original file)

Line 222, before:

```
  pushJournal(s, { entry: kind, because: [ev] });
```

Line 222, after:

```
  pushJournal(s, { entry: kind, because: [ev, 'absorb_cave'] });
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
