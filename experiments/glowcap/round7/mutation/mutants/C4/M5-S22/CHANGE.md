# C4 M5-S22

- File: `glowcap.cav`
- Line: 238
- Site: `bind $m.why_taste ... because $m_consumed`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_taste = "Already eaten" when $m_consumed == 1 because $m_consumed;
```

After:

```
    bind $m.why_taste = "Already eaten" when $m_consumed == 1 because $m_consumed, taste_$m;
```
