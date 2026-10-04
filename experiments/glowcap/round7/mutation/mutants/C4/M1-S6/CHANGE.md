# C4 M1-S6

- File: `glowcap.cav`
- Line: 219
- Site: label 'Probably a glowcap (taste has faded)': remove `carries(taste_$m, taste_faded)` from its condition

Before:

```
        when $m_known == 1 and carries(taste_$m, taste_faded) because $m_known;
```

After:

```
        when $m_known == 1 because $m_known;
```
