# C4 M5-S8

- File: `glowcap.cav`
- Line: 215
- Site: `bind $m.label ... because $m_known`: add to this because an evidence the value never reads

Before:

```
        when $m_known == 1 and carries(taste_$m, tasted_in_dark) because $m_known;
```

After:

```
        when $m_known == 1 and carries(taste_$m, tasted_in_dark) because $m_known, absorb_$m;
```
