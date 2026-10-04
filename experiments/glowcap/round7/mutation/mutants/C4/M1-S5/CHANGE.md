# C4 M1-S5

- File: `glowcap.cav`
- Line: 217
- Site: label 'Probably a duskcap (tasted in the dark)': remove `carries(taste_$m, tasted_in_dark)` from its condition

Before:

```
        when $m_known == 2 and carries(taste_$m, tasted_in_dark) because $m_known;
```

After:

```
        when $m_known == 2 because $m_known;
```
