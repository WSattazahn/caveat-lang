# C1 M1-S5

- File: `glowcap.cav`
- Line: 212
- Site: label 'Probably a duskcap (tasted in the dark)': remove `has_caveat($m_taste, tasted_in_dark)` from its condition

Before:

```
        when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
        when $m_taste == 2 because $m_taste;
```
