# C1 M4-S1

- File: `glowcap.cav`
- Line: 93
- Site: recovery commit `commit trust because enough using r1 + r2;`: change the order the two observations enter to r2 + r1

Before:

```
    when reopened(trust) and r1 > 0 commit trust because enough using r1 + r2;
```

After:

```
    when reopened(trust) and r1 > 0 commit trust because enough using r2 + r1;
```
