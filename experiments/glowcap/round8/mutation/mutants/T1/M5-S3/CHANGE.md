# T1 M5 S3

Site: add an evidence id the value never reads to the explanation list: whyNot: allowed (reason '') because []

File: `glowcap.ts`
Line: 230 (line numbers are those of the original file)

Line 230, before:

```
  const allowed = mk('', []);
```

Line 230, after:

```
  const allowed = mk('', ['absorb_cave']);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
