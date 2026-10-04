# C1 M5-S22

- File: `glowcap.cav`
- Line: 200
- Site: `bind $m.why.taste ... because $m_eaten`: add to this because an evidence the value never reads

Before:

```
    bind $m.why.taste = "Already eaten" when $m_eaten > 0 because $m_eaten;
```

After:

```
    bind $m.why.taste = "Already eaten" when $m_eaten > 0 because $m_eaten, taste_$m;
```
