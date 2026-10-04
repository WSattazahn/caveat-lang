# C2 M1-S3

- File: `glowcap.cav`
- Line: 182
- Site: `on taste call learn(taste_$m, sort, if(glow > 0, 0, 1), 2, $index);`: caveat-bearing `dark` argument; replace with 0 so tasted_in_dark is never attached

Before:

```
    on taste call learn(taste_$m, sort, if(glow > 0, 0, 1), 2, $index);
```

After:

```
    on taste call learn(taste_$m, sort, 0, 2, $index);
```
