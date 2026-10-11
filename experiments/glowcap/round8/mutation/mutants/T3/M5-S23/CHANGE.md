# T3 M5 S23

Site: add an evidence id the value never reads to the explanation list: first commit basis argument [ev]

File: `glowcap.ts`
Line: 229 (line numbers are those of the original file)

Line 229, before:

```
    if (beliefState(s) === 'probably_safe') commit(s, [ev]);
```

Line 229, after:

```
    if (beliefState(s) === 'probably_safe') commit(s, [ev, 'absorb_cave']);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
