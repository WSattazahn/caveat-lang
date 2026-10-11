# T2 M5 S8

Site: add an evidence id the value never reads to the explanation list: "Marked to avoid" refusal because

File: `glowcap.ts`
Line: 240 (line numbers are those of the original file)

Line 240, before:

```
  const marked: Refusal = m.marked ? { reason: 'Marked to avoid', because: [] } : null;
```

Line 240, after:

```
  const marked: Refusal = m.marked ? { reason: 'Marked to avoid', because: ['absorb_cave'] } : null;
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
