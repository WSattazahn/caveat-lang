# T1 M5 S1

Site: add an evidence id the value never reads to the explanation list: supportedBy list

File: `glowcap.ts`
Line: 139 (line numbers are those of the original file)

Line 139, before:

```
  return counted(s).filter((o) => o.glow).map((o) => o.ev);
```

Line 139, after:

```
  return counted(s).filter((o) => o.glow).map((o) => o.ev).concat('absorb_cave');
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
