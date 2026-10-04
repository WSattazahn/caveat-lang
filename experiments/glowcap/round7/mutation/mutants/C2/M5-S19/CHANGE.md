# C2 M5-S19

- File: `glowcap.cav`
- Line: 235
- Site: `bind $m.why_absorb ... because contradicted`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_absorb = "Too risky untasted" when $m_taste == 0 and twice_bitten because contradicted;
```

After:

```
    bind $m.why_absorb = "Too risky untasted" when $m_taste == 0 and twice_bitten because contradicted, taste_pit;
```
