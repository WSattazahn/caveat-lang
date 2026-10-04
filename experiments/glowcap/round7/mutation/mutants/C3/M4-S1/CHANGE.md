# C3 M4-S1

- File: `glowcap.cav`
- Line: 95
- Site: recovery accumulation `set recovery = recovery + qualified(1, e);` (the recovery commit's using value): change the order evidence enters to qualified(1, e) + recovery

Before:

```
    when s == sort.glowcap and recovery < 2 set recovery = recovery + qualified(1, e);
```

After:

```
    when s == sort.glowcap and recovery < 2 set recovery = qualified(1, e) + recovery;
```
