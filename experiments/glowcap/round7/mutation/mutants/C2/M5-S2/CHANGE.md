# C2 M5-S2

- File: `glowcap.cav`
- Line: 215
- Site: `bind $m.present ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.present = $m_eaten == 0 because nothing;
```

After:

```
    bind $m.present = $m_eaten == 0 because taste_pit;
```
