# T1 M5 S20

Site: add an evidence id the value never reads to the explanation list: mushroomRule: "Could be a duskcap" label because

File: `glowcap.ts`
Line: 266 (line numbers are those of the original file)

Line 266, before:

```
  return { present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true, because: [...con] };
```

Line 266, after:

```
  return { present: true, label: 'Could be a duskcap — taste first', canAbsorb: true, canTaste: true, because: [...con, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
