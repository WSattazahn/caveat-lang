# C1 M5-S34

- File: `glowcap.cav`
- Line: 217
- Site: `bind $m.label ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "" when $m_consumed == 1 because nothing;
```

After:

```
    bind $m.label = "" when $m_consumed == 1 because taste_pit;
```
