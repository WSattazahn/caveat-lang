# C2 M5-S11

- File: `glowcap.cav`
- Line: 224
- Site: `bind $m.label ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a duskcap (tasted in the dark)" when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
    bind $m.label = "Probably a duskcap (tasted in the dark)" when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste, absorb_$m;
```
