# C2 M5-S20

- File: `glowcap.cav`
- Line: 236
- Site: `bind $m.why_absorb ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_absorb = "Marked to avoid" when $m_marked == 1 because nothing;
```

After:

```
    bind $m.why_absorb = "Marked to avoid" when $m_marked == 1 because taste_pit;
```
