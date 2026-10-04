# C1 M5-S48

- File: `glowcap.cav`
- Line: 249
- Site: `bind belief.text ... because support, contra`: add to this because an evidence the value never reads

Before:

```
bind belief.text = "Not every glowing mushroom is safe. Taste before absorbing." when belief_mixed because support, contra;
```

After:

```
bind belief.text = "Not every glowing mushroom is safe. Taste before absorbing." when belief_mixed because support, contra, taste_pit;
```
