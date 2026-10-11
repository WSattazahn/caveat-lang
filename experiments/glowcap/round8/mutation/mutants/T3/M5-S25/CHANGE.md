# T3 M5 S25

Site: add an evidence id the value never reads to the explanation list: reopenedBy push

File: `glowcap.ts`
Line: 234 (line numbers are those of the original file)

Line 234, before:

```
    d.reopenedBy.push(ev);
```

Line 234, after:

```
    d.reopenedBy.push(ev, 'absorb_cave');
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
