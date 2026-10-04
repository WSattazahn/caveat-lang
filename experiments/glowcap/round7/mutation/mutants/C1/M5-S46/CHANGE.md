# C1 M5-S46

- File: `glowcap.cav`
- Line: 247
- Site: `bind belief.text ... because support`: add to this because an evidence the value never reads

Before:

```
bind belief.text = "Glowing mushrooms give you light." when belief_safe because support;
```

After:

```
bind belief.text = "Glowing mushrooms give you light." when belief_safe because support, taste_pit;
```
