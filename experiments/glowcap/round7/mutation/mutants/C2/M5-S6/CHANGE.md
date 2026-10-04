# C2 M5-S6

- File: `glowcap.cav`
- Line: 219
- Site: `bind $m.label ... because contradicted`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Could be a duskcap — taste first" when belief_uncertain because contradicted;
```

After:

```
    bind $m.label = "Could be a duskcap — taste first" when belief_uncertain because contradicted, taste_pit;
```
