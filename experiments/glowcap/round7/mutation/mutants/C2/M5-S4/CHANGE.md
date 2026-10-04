# C2 M5-S4

- File: `glowcap.cav`
- Line: 217
- Site: `bind $m.label ... because supported`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a glowcap" when belief_safe because supported;
```

After:

```
    bind $m.label = "Probably a glowcap" when belief_safe because supported, taste_pit;
```
