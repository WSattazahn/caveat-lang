# C2 M5-S3

- File: `glowcap.cav`
- Line: 216
- Site: `bind $m.label ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Glowing mushroom" because nothing;
```

After:

```
    bind $m.label = "Glowing mushroom" because taste_pit;
```
