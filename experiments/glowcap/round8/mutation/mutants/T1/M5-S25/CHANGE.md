# T1 M5 S25

Site: add an evidence id the value never reads to the explanation list: reopenedBy push

File: `glowcap.ts`
Line: 302 (line numbers are those of the original file)

Line 302, before:

```
      pushBounded(s.decision.reopenedBy, ev, DECISION_MEMORY);
```

Line 302, after:

```
      pushBounded(s.decision.reopenedBy, ev, DECISION_MEMORY); pushBounded(s.decision.reopenedBy, 'absorb_cave', DECISION_MEMORY);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
