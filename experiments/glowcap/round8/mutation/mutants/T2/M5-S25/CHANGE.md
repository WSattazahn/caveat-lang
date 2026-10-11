# T2 M5 S25

Site: add an evidence id the value never reads to the explanation list: recovery commit basis argument

File: `glowcap.ts`
Line: 385 (line numbers are those of the original file)

Line 385, before:

```
    return commit(s, s.recovery);
```

Line 385, after:

```
    return commit(s, [...s.recovery, 'absorb_cave']);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
