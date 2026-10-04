# C2 M5-S27

- File: `glowcap.cav`
- Line: 243
- Site: `bind $m.why_taste ... because $m_eaten`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_taste = "Already eaten" when $m_eaten == 1 because $m_eaten;
```

After:

```
    bind $m.why_taste = "Already eaten" when $m_eaten == 1 because $m_eaten, taste_$m;
```
