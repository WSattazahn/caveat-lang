# C2 M1-S8

- File: `glowcap.cav`
- Line: 226
- Site: label 'Probably a duskcap (taste has faded)': remove `has_caveat($m_taste, taste_faded)` from its condition

Before:

```
    bind $m.label = "Probably a duskcap (taste has faded)" when $m_taste == 2 and has_caveat($m_taste, taste_faded) because $m_taste;
```

After:

```
    bind $m.label = "Probably a duskcap (taste has faded)" when $m_taste == 2 because $m_taste;
```
