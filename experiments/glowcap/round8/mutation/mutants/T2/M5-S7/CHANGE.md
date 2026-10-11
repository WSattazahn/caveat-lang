# T2 M5 S7

Site: add an evidence id the value never reads to the explanation list: "Out of reach" refusal because

File: `glowcap.ts`
Line: 238 (line numbers are those of the original file)

Line 238, before:

```
  const outOfReach: Refusal = OUT_OF_REACH.includes(id) ? { reason: 'Out of reach', because: [] } : null;
```

Line 238, after:

```
  const outOfReach: Refusal = OUT_OF_REACH.includes(id) ? { reason: 'Out of reach', because: ['absorb_cave'] } : null;
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
