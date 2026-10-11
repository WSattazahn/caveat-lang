# T3 M5 S15

Site: add an evidence id the value never reads to the explanation list: mushroomRow: tasted duskcap label because

File: `glowcap.ts`
Line: 158 (line numbers are those of the original file)

Line 158, before:

```
  if (m.tasted === 'duskcap') return { present: true, label: qual !== null ? `Probably a duskcap${qual}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteEv] };
```

Line 158, after:

```
  if (m.tasted === 'duskcap') return { present: true, label: qual !== null ? `Probably a duskcap${qual}` : 'Duskcap — avoid', canAbsorb: false, canTaste: false, because: [tasteEv, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
