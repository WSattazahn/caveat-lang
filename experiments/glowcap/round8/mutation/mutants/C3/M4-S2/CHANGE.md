# C3 M4 S2

Site: decision.basis: reverse the frozen grounds order

File: `adapter.mjs`
Line: 67 (line numbers are those of the original file)

Line 67, before:

```
      basis: grounds ? named(grounds.evidence.filter((e) => !grounds.inherited?.includes(e))) : [],
```

Line 67, after:

```
      basis: grounds ? named(grounds.evidence.filter((e) => !grounds.inherited?.includes(e))).reverse() : [],
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
