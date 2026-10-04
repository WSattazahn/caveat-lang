# C1 M5-S18

- File: `glowcap.cav`
- Line: 196
- Site: `bind $m.why.absorb ... because $m_eaten`: add to this because an evidence the value never reads

Before:

```
    bind $m.why.absorb = "Already eaten" when $m_eaten > 0 because $m_eaten;
```

After:

```
    bind $m.why.absorb = "Already eaten" when $m_eaten > 0 because $m_eaten, taste_$m;
```
