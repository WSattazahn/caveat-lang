# C3 M5-S3

- File: `glowcap.cav`
- Line: 205
- Site: `bind $m.label ... because contradiction`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a duskcap" when belief_unsafe because contradiction;
```

After:

```
    bind $m.label = "Probably a duskcap" when belief_unsafe because contradiction, taste_pit;
```
