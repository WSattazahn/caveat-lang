# T3 M5 S24

Site: add an evidence id the value never reads to the explanation list: recovery commit basis argument

File: `glowcap.ts`
Line: 231 (line numbers are those of the original file)

Line 231, before:

```
    commit(s, d.sinceContradiction.slice(0, 2));
```

Line 231, after:

```
    commit(s, [...d.sinceContradiction.slice(0, 2), 'absorb_cave']);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
