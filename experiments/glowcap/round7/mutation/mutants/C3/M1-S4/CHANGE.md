# C3 M1-S4

- File: `glowcap.cav`
- Line: 164
- Site: `on taste call learn(taste_$m, sort, 1);`: caveat-bearing `tasted` argument; replace 1 with 0 so the taste's caveats are never attached

Before:

```
    on taste call learn(taste_$m, sort, 1);
```

After:

```
    on taste call learn(taste_$m, sort, 0);
```
