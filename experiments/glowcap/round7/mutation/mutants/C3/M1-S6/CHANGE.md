# C3 M1-S6

- File: `glowcap.cav`
- Line: 213
- Site: label 'Probably a duskcap (tasted in the dark)': remove `has_caveat($m_taste, tasted_in_dark)` from its condition

Before:

```
        when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
        when $m_taste == 2 because $m_taste;
```
