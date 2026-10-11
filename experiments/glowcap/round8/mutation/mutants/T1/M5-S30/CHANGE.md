# T1 M5 S30

Site: add an evidence id the value never reads to the explanation list: journal undoubt entry because

File: `glowcap.ts`
Line: 475 (line numbers are those of the original file)

Line 475, before:

```
        pushBounded(s.journal, { text: `I believe what I saw at the ${name} mushroom again.`, because: [ev] }, JOURNAL_MEMORY);
```

Line 475, after:

```
        pushBounded(s.journal, { text: `I believe what I saw at the ${name} mushroom again.`, because: [ev, 'absorb_cave'] }, JOURNAL_MEMORY);
```

Choice: Choice (same in T1-T3): the added evidence id is the literal `'absorb_cave'` (the cave mushroom's first-life absorption, which the value is not about), appended to the list (by spread, `.concat`, an extra push or an extra element).
