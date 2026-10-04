# C1 M5-S27

- File: `glowcap.cav`
- Line: 206
- Site: `bind $m.label ... because contra`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Too risky — taste first" when $m_unknown and twice_bitten because contra;
```

After:

```
    bind $m.label = "Too risky — taste first" when $m_unknown and twice_bitten because contra, taste_pit;
```
