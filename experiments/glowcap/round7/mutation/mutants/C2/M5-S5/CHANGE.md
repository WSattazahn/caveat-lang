# C2 M5-S5

- File: `glowcap.cav`
- Line: 218
- Site: `bind $m.label ... because contradicted`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a duskcap" when belief_unsafe because contradicted;
```

After:

```
    bind $m.label = "Probably a duskcap" when belief_unsafe because contradicted, taste_pit;
```
