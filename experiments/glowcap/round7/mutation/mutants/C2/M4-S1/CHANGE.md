# C2 M4-S1

- File: `glowcap.cav`
- Line: 132
- Site: recovery accumulation `set recovery = recovery + qualified(1, e);` (the recovery commit's using value): change the order evidence enters to qualified(1, e) + recovery

Before:

```
    when sort == sort.glowcap and reopened(trust) set recovery = recovery + qualified(1, e);
```

After:

```
    when sort == sort.glowcap and reopened(trust) set recovery = qualified(1, e) + recovery;
```
