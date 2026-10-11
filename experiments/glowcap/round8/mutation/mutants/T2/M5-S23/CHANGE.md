# T2 M5 S23

Site: add an evidence id the value never reads to the explanation list: reopenedBy push

File: `glowcap.ts`
Line: 380 (line numbers are those of the original file)

Line 380, before:

```
    pushBounded(d.reopenedBy, e.id, DECISION_KEEP);
```

Line 380, after:

```
    pushBounded(d.reopenedBy, e.id, DECISION_KEEP); pushBounded(d.reopenedBy, 'absorb_cave', DECISION_KEEP);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
