# T1 M5 S19

Site: add an evidence id the value never reads to the explanation list: mushroomRule: "Probably a duskcap" label because

File: `glowcap.ts`
Line: 264 (line numbers are those of the original file)

Line 264, before:

```
  if (bs === 'probably_unsafe') return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: [...con] };
```

Line 264, after:

```
  if (bs === 'probably_unsafe') return { present: true, label: 'Probably a duskcap', canAbsorb: true, canTaste: true, because: [...con, 'absorb_cave'] };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
