# C2 M4 S2

Site: decision.basis: reverse the frozen grounds order

File: `adapter.mjs`
Line: 119 (line numbers are those of the original file)

Line 119, before:

```
    out.decision.basis = names(grounds?.evidence ?? []);
```

Line 119, after:

```
    out.decision.basis = names(grounds?.evidence ?? []).reverse();
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
