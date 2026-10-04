# C2 M5-S7

- File: `glowcap.cav`
- Line: 220
- Site: `bind $m.label ... because contradicted`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Too risky — taste first" when twice_bitten because contradicted;
```

After:

```
    bind $m.label = "Too risky — taste first" when twice_bitten because contradicted, taste_pit;
```
