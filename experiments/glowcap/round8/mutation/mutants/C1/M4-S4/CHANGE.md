# C1 M4 S4

Site: recovery accumulation (the using value of the recovery commit): swap the operands so the new observation enters before the earlier one

File: `glowcap.cav`
Line: 232 (line numbers are those of the original file)

Line 232, before:

```
    when found == 1 and reopened(trust) set recovery = recovery + qualified(1, e);
```

Line 232, after:

```
    when found == 1 and reopened(trust) set recovery = qualified(1, e) + recovery;
```

Choice: In the .cav source the operands of the recovery commitment's value are swapped; in the adapter the projected list is reversed with `.reverse()`.
