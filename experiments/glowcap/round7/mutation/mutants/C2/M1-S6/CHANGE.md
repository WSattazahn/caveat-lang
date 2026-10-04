# C2 M1-S6

- File: `glowcap.cav`
- Line: 224
- Site: label 'Probably a duskcap (tasted in the dark)': remove `has_caveat($m_taste, tasted_in_dark)` from its condition

Before:

```
    bind $m.label = "Probably a duskcap (tasted in the dark)" when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
    bind $m.label = "Probably a duskcap (tasted in the dark)" when $m_taste == 2 because $m_taste;
```
