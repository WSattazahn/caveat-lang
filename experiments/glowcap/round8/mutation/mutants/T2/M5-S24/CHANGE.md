# T2 M5 S24

Site: add an evidence id the value never reads to the explanation list: reopened history entry because

File: `glowcap.ts`
Line: 381 (line numbers are those of the original file)

Line 381, before:

```
    pushBounded(d.history, { change: 'reopened', because: [e.id] }, DECISION_KEEP);
```

Line 381, after:

```
    pushBounded(d.history, { change: 'reopened', because: [e.id, 'absorb_cave'] }, DECISION_KEEP);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
