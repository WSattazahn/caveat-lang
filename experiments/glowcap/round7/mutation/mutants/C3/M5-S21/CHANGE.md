# C3 M5-S21

- File: `glowcap.cav`
- Line: 229
- Site: `bind $m.why_taste ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_taste = "Out of reach" when $index == target.pit because nothing;
```

After:

```
    bind $m.why_taste = "Out of reach" when $index == target.pit because taste_pit;
```
