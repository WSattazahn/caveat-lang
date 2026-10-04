# C1 M5-S14

- File: `glowcap.cav`
- Line: 192
- Site: `bind $m.why.absorb ... because contra`: add to this because an evidence the value never reads

Before:

```
    bind $m.why.absorb = "Too risky untasted" when $m_unknown and twice_bitten because contra;
```

After:

```
    bind $m.why.absorb = "Too risky untasted" when $m_unknown and twice_bitten because contra, taste_pit;
```
