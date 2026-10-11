# T1 M5 S10

Site: add an evidence id the value never reads to the explanation list: whyNot: "Too risky untasted" because

File: `glowcap.ts`
Line: 240 (line numbers are those of the original file)

Line 240, before:

```
  else if (m.tasted === null && contradictedBy(s).length >= 2) absorb = mk('Too risky untasted', contradictedBy(s));
```

Line 240, after:

```
  else if (m.tasted === null && contradictedBy(s).length >= 2) absorb = mk('Too risky untasted', [...contradictedBy(s), 'absorb_cave']);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
