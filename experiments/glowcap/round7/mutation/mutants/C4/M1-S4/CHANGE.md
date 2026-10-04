# C4 M1-S4

- File: `glowcap.cav`
- Line: 215
- Site: label 'Probably a glowcap (tasted in the dark)': remove `carries(taste_$m, tasted_in_dark)` from its condition

Before:

```
        when $m_known == 1 and carries(taste_$m, tasted_in_dark) because $m_known;
```

After:

```
        when $m_known == 1 because $m_known;
```
