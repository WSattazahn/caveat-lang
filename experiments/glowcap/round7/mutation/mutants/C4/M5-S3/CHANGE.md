# C4 M5-S3

- File: `glowcap.cav`
- Line: 209
- Site: `bind $m.label ... because contradiction`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a duskcap" when probably_unsafe because contradiction;
```

After:

```
    bind $m.label = "Probably a duskcap" when probably_unsafe because contradiction, taste_pit;
```
