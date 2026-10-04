# C4 M5-S10

- File: `glowcap.cav`
- Line: 219
- Site: `bind $m.label ... because $m_known`: add to this because an evidence the value never reads

Before:

```
        when $m_known == 1 and carries(taste_$m, taste_faded) because $m_known;
```

After:

```
        when $m_known == 1 and carries(taste_$m, taste_faded) because $m_known, absorb_$m;
```
