# T3 M5 S26

Site: add an evidence id the value never reads to the explanation list: reopened history entry because

File: `glowcap.ts`
Line: 236 (line numbers are those of the original file)

Line 236, before:

```
    pushHistory(s, { change: 'reopened', because: [ev] });
```

Line 236, after:

```
    pushHistory(s, { change: 'reopened', because: [ev, 'absorb_cave'] });
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
