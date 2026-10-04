# C1 M5-S24

- File: `glowcap.cav`
- Line: 203
- Site: `bind $m.label ... because support`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a glowcap" when belief_safe because support;
```

After:

```
    bind $m.label = "Probably a glowcap" when belief_safe because support, taste_pit;
```
