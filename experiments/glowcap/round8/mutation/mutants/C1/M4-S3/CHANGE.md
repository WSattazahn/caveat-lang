# C1 M4 S3

Site: decision.history entry because (a committed entry's basis): reverse it

File: `adapter.mjs`
Line: 94 (line numbers are those of the original file)

Line 94, before:

```
      history: changes.map((e) => ({ change: e.change, because: names(e.because) })),
```

Line 94, after:

```
      history: changes.map((e) => ({ change: e.change, because: names(e.because).reverse() })),
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
