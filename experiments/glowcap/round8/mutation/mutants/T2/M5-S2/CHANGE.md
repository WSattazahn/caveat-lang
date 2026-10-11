# T2 M5 S2

Site: add an evidence id the value never reads to the explanation list: contradictedBy list

File: `glowcap.ts`
Line: 132 (line numbers are those of the original file)

Line 132, before:

```
  const contradictedBy = counted.filter((e) => e.kind === 'duskcap').map((e) => e.id);
```

Line 132, after:

```
  const contradictedBy = counted.filter((e) => e.kind === 'duskcap').map((e) => e.id).concat('absorb_cave');
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
