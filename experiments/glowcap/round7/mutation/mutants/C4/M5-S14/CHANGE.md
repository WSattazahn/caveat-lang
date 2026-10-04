# C4 M5-S14

- File: `glowcap.cav`
- Line: 230
- Site: `bind $m.why_absorb ... because contradiction`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_absorb = "Too risky untasted" when $m_known == 0 and twice_bitten because contradiction;
```

After:

```
    bind $m.why_absorb = "Too risky untasted" when $m_known == 0 and twice_bitten because contradiction, taste_pit;
```
