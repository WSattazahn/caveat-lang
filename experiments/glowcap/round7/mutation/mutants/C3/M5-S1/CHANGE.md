# C3 M5-S1

- File: `glowcap.cav`
- Line: 203
- Site: `bind $m.label ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Glowing mushroom" because nothing;
```

After:

```
    bind $m.label = "Glowing mushroom" because taste_pit;
```
