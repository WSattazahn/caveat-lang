# C3 M4 S1

Site: journal entry because (holds the new basis for a trust record): reverse it

File: `adapter.mjs`
Line: 62 (line numbers are those of the original file)

Line 62, before:

```
        because: cited(v, 'diary', `c${n}`),
```

Line 62, after:

```
        because: cited(v, 'diary', `c${n}`).reverse(),
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
