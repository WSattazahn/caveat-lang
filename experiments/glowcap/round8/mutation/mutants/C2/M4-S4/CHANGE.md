# C2 M4 S4

Site: recovery commit (undoubted page 5): swap the operands, j6_s + j5_s

File: `glowcap.cav`
Line: 152 (line numbers are those of the original file)

Line 152, before:

```
    when trusting == 1 and j5_n < 100000000 commit trust because enough using j5_s + j6_s;
```

Line 152, after:

```
    when trusting == 1 and j5_n < 100000000 commit trust because enough using j6_s + j5_s;
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
