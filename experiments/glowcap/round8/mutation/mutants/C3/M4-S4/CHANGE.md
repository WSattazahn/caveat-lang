# C3 M4 S4

Site: recovery commit: swap the operands, e6 + e5

File: `glowcap.cav`
Line: 452 (line numbers are those of the original file)

Line 452, before:

```
        commit trust because enough using e5 + e6;
```

Line 452, after:

```
        commit trust because enough using e6 + e5;
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
