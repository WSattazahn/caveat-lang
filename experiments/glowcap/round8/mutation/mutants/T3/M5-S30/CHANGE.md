# T3 M5 S30

Site: add an evidence id the value never reads to the explanation list: journal doubt entry because

File: `glowcap.ts`
Line: 369 (line numbers are those of the original file)

Line 369, before:

```
      pushJournal(s, { entry: 'doubt', because: [obs.ev] });
```

Line 369, after:

```
      pushJournal(s, { entry: 'doubt', because: [obs.ev, 'absorb_cave'] });
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
