# C2 M5-S14

- File: `glowcap.cav`
- Line: 227
- Site: `bind $m.label ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "" when $m_eaten == 1 because nothing;
```

After:

```
    bind $m.label = "" when $m_eaten == 1 because taste_pit;
```
