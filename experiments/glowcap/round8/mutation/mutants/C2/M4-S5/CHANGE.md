# C2 M4 S5

Site: recovery commit (doubted page 5): swap the operands, j6_s + j5_x

File: `glowcap.cav`
Line: 153 (line numbers are those of the original file)

Line 153, before:

```
    when trusting == 1 and j5_n >= 100000000 commit trust because enough using qualified(j5_x + j6_s, d, doubted);
```

Line 153, after:

```
    when trusting == 1 and j5_n >= 100000000 commit trust because enough using qualified(j6_s + j5_x, d, doubted);
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
