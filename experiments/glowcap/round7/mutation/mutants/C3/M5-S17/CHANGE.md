# C3 M5-S17

- File: `glowcap.cav`
- Line: 225
- Site: `bind $m.why_absorb ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_absorb = "Out of reach" when $index == target.pit because nothing;
```

After:

```
    bind $m.why_absorb = "Out of reach" when $index == target.pit because taste_pit;
```
