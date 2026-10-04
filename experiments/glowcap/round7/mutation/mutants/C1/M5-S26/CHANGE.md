# C1 M5-S26

- File: `glowcap.cav`
- Line: 205
- Site: `bind $m.label ... because contra`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Could be a duskcap — taste first" when belief_mixed because contra;
```

After:

```
    bind $m.label = "Could be a duskcap — taste first" when belief_mixed because contra, taste_pit;
```
