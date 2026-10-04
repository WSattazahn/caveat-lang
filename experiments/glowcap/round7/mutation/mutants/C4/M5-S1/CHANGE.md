# C4 M5-S1

- File: `glowcap.cav`
- Line: 207
- Site: `bind $m.label ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Glowing mushroom" because nothing;
```

After:

```
    bind $m.label = "Glowing mushroom" because taste_pit;
```
