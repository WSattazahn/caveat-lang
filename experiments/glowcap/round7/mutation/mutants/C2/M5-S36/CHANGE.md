# C2 M5-S36

- File: `glowcap.cav`
- Line: 265
- Site: `bind belief.text ... because supported`: add to this because an evidence the value never reads

Before:

```
bind belief.text = "Glowing mushrooms give you light." when belief_safe because supported;
```

After:

```
bind belief.text = "Glowing mushrooms give you light." when belief_safe because supported, taste_pit;
```
