# T1 M5 S29

Site: add an evidence id the value never reads to the explanation list: journal doubt entry because

File: `glowcap.ts`
Line: 471 (line numbers are those of the original file)

Line 471, before:

```
        pushBounded(s.journal, { text: `I doubt what I saw at the ${name} mushroom.`, because: [ev] }, JOURNAL_MEMORY);
```

Line 471, after:

```
        pushBounded(s.journal, { text: `I doubt what I saw at the ${name} mushroom.`, because: [ev, 'absorb_cave'] }, JOURNAL_MEMORY);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
