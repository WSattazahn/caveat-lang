# C2 M5-S23

- File: `glowcap.cav`
- Line: 239
- Site: `bind $m.why_absorb ... because $m_eaten`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_absorb = "Already eaten" when $m_eaten == 1 because $m_eaten;
```

After:

```
    bind $m.why_absorb = "Already eaten" when $m_eaten == 1 because $m_eaten, taste_$m;
```
