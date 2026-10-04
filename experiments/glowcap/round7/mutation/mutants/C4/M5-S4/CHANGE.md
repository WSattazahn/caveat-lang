# C4 M5-S4

- File: `glowcap.cav`
- Line: 210
- Site: `bind $m.label ... because contradiction`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Could be a duskcap — taste first" when uncertain because contradiction;
```

After:

```
    bind $m.label = "Could be a duskcap — taste first" when uncertain because contradiction, taste_pit;
```
