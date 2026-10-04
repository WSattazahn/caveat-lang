# C2 M1-S5

- File: `glowcap.cav`
- Line: 223
- Site: label 'Probably a glowcap (tasted in the dark)': remove `has_caveat($m_taste, tasted_in_dark)` from its condition

Before:

```
    bind $m.label = "Probably a glowcap (tasted in the dark)" when $m_taste == 1 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
    bind $m.label = "Probably a glowcap (tasted in the dark)" when $m_taste == 1 because $m_taste;
```
