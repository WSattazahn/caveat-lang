# C2 M5-S17

- File: `glowcap.cav`
- Line: 230
- Site: `bind $m.can_taste ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.can_taste = $m_eaten == 0 and not $m_out_of_reach and $m_taste == 0 because nothing;
```

After:

```
    bind $m.can_taste = $m_eaten == 0 and not $m_out_of_reach and $m_taste == 0 because taste_pit;
```
