# C1 M5-S20

- File: `glowcap.cav`
- Line: 198
- Site: `bind $m.why.taste ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
    bind $m.why.taste = "Already tasted" when $m_taste != 0 because $m_taste;
```

After:

```
    bind $m.why.taste = "Already tasted" when $m_taste != 0 because $m_taste, absorb_$m;
```
