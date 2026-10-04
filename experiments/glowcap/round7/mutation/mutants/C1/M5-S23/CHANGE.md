# C1 M5-S23

- File: `glowcap.cav`
- Line: 202
- Site: `bind $m.label ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Glowing mushroom" because nothing;
```

After:

```
    bind $m.label = "Glowing mushroom" because taste_pit;
```
