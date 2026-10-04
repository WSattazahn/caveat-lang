# C4 M5-S2

- File: `glowcap.cav`
- Line: 208
- Site: `bind $m.label ... because support`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a glowcap" when probably_safe because support;
```

After:

```
    bind $m.label = "Probably a glowcap" when probably_safe because support, taste_pit;
```
