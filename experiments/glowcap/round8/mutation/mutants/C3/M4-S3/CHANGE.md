# C3 M4 S3

Site: decision.history entry because: reverse it

File: `adapter.mjs`
Line: 69 (line numbers are those of the original file)

Line 69, before:

```
      history: journal.map((e) => ({ change: e.change, because: named(e.because) })),
```

Line 69, after:

```
      history: journal.map((e) => ({ change: e.change, because: named(e.because).reverse() })),
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
