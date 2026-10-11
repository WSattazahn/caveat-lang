# T3 M5 S31

Site: add an evidence id the value never reads to the explanation list: journal undoubt entry because

File: `glowcap.ts`
Line: 378 (line numbers are those of the original file)

Line 378, before:

```
      pushJournal(s, { entry: 'undoubt', because: [obs.ev] });
```

Line 378, after:

```
      pushJournal(s, { entry: 'undoubt', because: [obs.ev, 'absorb_cave'] });
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
