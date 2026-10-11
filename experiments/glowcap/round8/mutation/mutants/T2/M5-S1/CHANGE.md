# T2 M5 S1

Site: add an evidence id the value never reads to the explanation list: supportedBy list

File: `glowcap.ts`
Line: 131 (line numbers are those of the original file)

Line 131, before:

```
  const supportedBy = counted.filter((e) => e.kind === 'glowcap').map((e) => e.id);
```

Line 131, after:

```
  const supportedBy = counted.filter((e) => e.kind === 'glowcap').map((e) => e.id).concat('absorb_cave');
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
