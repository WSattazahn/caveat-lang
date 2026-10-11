# T3 M5 S14

Site: add an evidence id the value never reads to the explanation list: mushroomRow: tasted glowcap label because

File: `glowcap.ts`
Line: 157 (line numbers are those of the original file)

Line 157, before:

```
  if (m.tasted === 'glowcap') return { present: true, label: qual !== null ? `Probably a glowcap${qual}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteEv] };
```

Line 157, after:

```
  if (m.tasted === 'glowcap') return { present: true, label: qual !== null ? `Probably a glowcap${qual}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteEv, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
