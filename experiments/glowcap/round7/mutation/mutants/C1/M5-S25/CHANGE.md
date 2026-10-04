# C1 M5-S25

- File: `glowcap.cav`
- Line: 204
- Site: `bind $m.label ... because contra`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a duskcap" when belief_unsafe because contra;
```

After:

```
    bind $m.label = "Probably a duskcap" when belief_unsafe because contra, taste_pit;
```
