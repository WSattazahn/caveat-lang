# T2 M5 S3

Site: add an evidence id the value never reads to the explanation list: belief caveats: cited list (all)

File: `glowcap.ts`
Line: 133 (line numbers are those of the original file)

Line 133, before:

```
  return { supportedBy, contradictedBy, all: counted.map((e) => e.id) };
```

Line 133, after:

```
  return { supportedBy, contradictedBy, all: counted.map((e) => e.id).concat('absorb_cave') };
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
