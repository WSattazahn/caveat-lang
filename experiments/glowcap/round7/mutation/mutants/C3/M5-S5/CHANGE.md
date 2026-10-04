# C3 M5-S5

- File: `glowcap.cav`
- Line: 207
- Site: `bind $m.label ... because contradiction`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Too risky — taste first" when twice_bitten because contradiction;
```

After:

```
    bind $m.label = "Too risky — taste first" when twice_bitten because contradiction, taste_pit;
```
