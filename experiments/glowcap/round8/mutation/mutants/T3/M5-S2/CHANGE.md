# T3 M5 S2

Site: add an evidence id the value never reads to the explanation list: con list

File: `glowcap.ts`
Line: 68 (line numbers are those of the original file)

Line 68, before:

```
const con = (s: State): string[] => counted(s).filter((o) => o.kind === 'duskcap').map((o) => o.ev);
```

Line 68, after:

```
const con = (s: State): string[] => counted(s).filter((o) => o.kind === 'duskcap').map((o) => o.ev).concat('absorb_cave');
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
