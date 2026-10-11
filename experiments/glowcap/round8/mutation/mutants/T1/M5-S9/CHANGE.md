# T1 M5 S9

Site: add an evidence id the value never reads to the explanation list: whyNot: "Marked to avoid" because

File: `glowcap.ts`
Line: 239 (line numbers are those of the original file)

Line 239, before:

```
  else if (m.marked) absorb = mk('Marked to avoid', []);
```

Line 239, after:

```
  else if (m.marked) absorb = mk('Marked to avoid', ['absorb_cave']);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
