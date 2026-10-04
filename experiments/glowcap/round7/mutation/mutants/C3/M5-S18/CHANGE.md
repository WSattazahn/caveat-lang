# C3 M5-S18

- File: `glowcap.cav`
- Line: 226
- Site: `bind $m.why_absorb ... because $m_eaten_by`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_absorb = "Already eaten" when $m_eaten_by == 1 because $m_eaten_by;
```

After:

```
    bind $m.why_absorb = "Already eaten" when $m_eaten_by == 1 because $m_eaten_by, taste_$m;
```
