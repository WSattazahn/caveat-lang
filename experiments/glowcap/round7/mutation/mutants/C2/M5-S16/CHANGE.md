# C2 M5-S16

- File: `glowcap.cav`
- Line: 229
- Site: `bind $m.can_absorb ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.can_absorb = $m_eaten == 0 and not $m_out_of_reach and $m_marked == 0 and ($m_taste == 1 or ($m_taste == 0 and not twice_bitten)) because nothing;
```

After:

```
    bind $m.can_absorb = $m_eaten == 0 and not $m_out_of_reach and $m_marked == 0 and ($m_taste == 1 or ($m_taste == 0 and not twice_bitten)) because taste_pit;
```
