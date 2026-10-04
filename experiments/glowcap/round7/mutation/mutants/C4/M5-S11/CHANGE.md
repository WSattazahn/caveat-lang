# C4 M5-S11

- File: `glowcap.cav`
- Line: 221
- Site: `bind $m.label ... because $m_known`: add to this because an evidence the value never reads

Before:

```
        when $m_known == 2 and carries(taste_$m, taste_faded) because $m_known;
```

After:

```
        when $m_known == 2 and carries(taste_$m, taste_faded) because $m_known, absorb_$m;
```
