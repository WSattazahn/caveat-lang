# T1 M5 S16

Site: add an evidence id the value never reads to the explanation list: mushroomRule: "Too risky" label because

File: `glowcap.ts`
Line: 261 (line numbers are those of the original file)

Line 261, before:

```
  if (con.length >= 2) return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...con] };
```

Line 261, after:

```
  if (con.length >= 2) return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...con, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
