# C2 M5-S38

- File: `glowcap.cav`
- Line: 267
- Site: `bind belief.text ... because supported, contradicted`: add to this because an evidence the value never reads

Before:

```
bind belief.text = "Not every glowing mushroom is safe. Taste before absorbing." when belief_uncertain because supported, contradicted;
```

After:

```
bind belief.text = "Not every glowing mushroom is safe. Taste before absorbing." when belief_uncertain because supported, contradicted, taste_pit;
```
