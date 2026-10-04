# C4 M5-S9

- File: `glowcap.cav`
- Line: 217
- Site: `bind $m.label ... because $m_known`: add to this because an evidence the value never reads

Before:

```
        when $m_known == 2 and carries(taste_$m, tasted_in_dark) because $m_known;
```

After:

```
        when $m_known == 2 and carries(taste_$m, tasted_in_dark) because $m_known, absorb_$m;
```
