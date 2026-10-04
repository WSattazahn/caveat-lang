# C3 M1-S7

- File: `glowcap.cav`
- Line: 215
- Site: label 'Probably a glowcap (taste has faded)': remove `has_caveat($m_taste, taste_faded)` from its condition

Before:

```
        when $m_taste == 1 and has_caveat($m_taste, taste_faded) because $m_taste;
```

After:

```
        when $m_taste == 1 because $m_taste;
```
