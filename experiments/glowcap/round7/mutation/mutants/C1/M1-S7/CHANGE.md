# C1 M1-S7

- File: `glowcap.cav`
- Line: 216
- Site: label 'Probably a duskcap (taste has faded)': remove `has_caveat($m_taste, taste_faded)` from its condition

Before:

```
        when $m_taste == 2 and has_caveat($m_taste, taste_faded) because $m_taste;
```

After:

```
        when $m_taste == 2 because $m_taste;
```
