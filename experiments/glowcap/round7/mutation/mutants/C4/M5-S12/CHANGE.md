# C4 M5-S12

- File: `glowcap.cav`
- Line: 222
- Site: `bind $m.label ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "" when $m_consumed == 1 because nothing;
```

After:

```
    bind $m.label = "" when $m_consumed == 1 because taste_pit;
```
