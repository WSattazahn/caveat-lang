# C2 M5-S15

- File: `glowcap.cav`
- Line: 228
- Site: `bind $m.marked ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.marked = $m_marked == 1 because nothing;
```

After:

```
    bind $m.marked = $m_marked == 1 because taste_pit;
```
