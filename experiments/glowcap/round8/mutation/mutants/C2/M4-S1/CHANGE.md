# C2 M4 S1

Site: journal entry because (holds the new basis for a trust entry): reverse it

File: `adapter.mjs`
Line: 108 (line numbers are those of the original file)

Line 108, before:

```
  const entry = (text, cites) => ({ text: b.journal[text], because: evidence(ex, 'journal', cites), caveats: caveats(ex, 'journal', cites) });
```

Line 108, after:

```
  const entry = (text, cites) => ({ text: b.journal[text], because: evidence(ex, 'journal', cites).reverse(), caveats: caveats(ex, 'journal', cites) });
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
