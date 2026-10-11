# C1 M4 S1

Site: journal entry because (holds the new basis for a trust entry): reverse it

File: `adapter.mjs`
Line: 74 (line numbers are those of the original file)

Line 74, before:

```
    journal.push({ text: b.journal[`text${k}`], because: evidence(ex, 'journal', `cites${k}`), caveats: caveats(ex, ['journal', `cites${k}`]) });
```

Line 74, after:

```
    journal.push({ text: b.journal[`text${k}`], because: evidence(ex, 'journal', `cites${k}`).reverse(), caveats: caveats(ex, ['journal', `cites${k}`]) });
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
