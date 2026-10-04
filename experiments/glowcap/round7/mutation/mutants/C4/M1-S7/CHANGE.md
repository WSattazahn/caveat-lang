# C4 M1-S7

- File: `glowcap.cav`
- Line: 221
- Site: label 'Probably a duskcap (taste has faded)': remove `carries(taste_$m, taste_faded)` from its condition

Before:

```
        when $m_known == 2 and carries(taste_$m, taste_faded) because $m_known;
```

After:

```
        when $m_known == 2 because $m_known;
```
