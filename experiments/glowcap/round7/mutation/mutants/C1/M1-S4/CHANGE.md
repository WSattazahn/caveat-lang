# C1 M1-S4

- File: `glowcap.cav`
- Line: 210
- Site: label 'Probably a glowcap (tasted in the dark)': remove `has_caveat($m_taste, tasted_in_dark)` from its condition

Before:

```
        when $m_taste == 1 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
        when $m_taste == 1 because $m_taste;
```
