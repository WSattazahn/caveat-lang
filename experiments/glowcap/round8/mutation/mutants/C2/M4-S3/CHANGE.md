# C2 M4 S3

Site: decision.history entry because: reverse it

File: `adapter.mjs`
Line: 126 (line numbers are those of the original file)

Line 126, before:

```
  out.decision.history = journal.map((j) => ({ change: j.change, because: names(j.because) }));
```

Line 126, after:

```
  out.decision.history = journal.map((j) => ({ change: j.change, because: names(j.because).reverse() }));
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
