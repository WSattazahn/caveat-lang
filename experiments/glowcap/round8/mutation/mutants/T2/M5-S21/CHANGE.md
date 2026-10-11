# T2 M5 S21

Site: add an evidence id the value never reads to the explanation list: journal trust entry because

File: `glowcap.ts`
Line: 361 (line numbers are those of the original file)

Line 361, before:

```
    const because = change === 'committed' ? [...s.decision.basis] : [e.id];
```

Line 361, after:

```
    const because = [...(change === 'committed' ? [...s.decision.basis] : [e.id]), 'absorb_cave'];
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
