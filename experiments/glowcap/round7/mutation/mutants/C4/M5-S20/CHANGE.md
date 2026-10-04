# C4 M5-S20

- File: `glowcap.cav`
- Line: 236
- Site: `bind $m.why_taste ... because $m_known`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_taste = "Already tasted" when $m_known != 0 because $m_known;
```

After:

```
    bind $m.why_taste = "Already tasted" when $m_known != 0 because $m_known, absorb_$m;
```
