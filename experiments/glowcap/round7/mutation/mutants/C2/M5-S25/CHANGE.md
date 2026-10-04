# C2 M5-S25

- File: `glowcap.cav`
- Line: 241
- Site: `bind $m.why_taste ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_taste = "Already tasted" when $m_taste != 0 because $m_taste;
```

After:

```
    bind $m.why_taste = "Already tasted" when $m_taste != 0 because $m_taste, absorb_$m;
```
