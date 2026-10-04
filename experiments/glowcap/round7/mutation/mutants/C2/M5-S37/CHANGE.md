# C2 M5-S37

- File: `glowcap.cav`
- Line: 266
- Site: `bind belief.text ... because contradicted`: add to this because an evidence the value never reads

Before:

```
bind belief.text = "Glowing mushrooms make you heavy." when belief_unsafe because contradicted;
```

After:

```
bind belief.text = "Glowing mushrooms make you heavy." when belief_unsafe because contradicted, taste_pit;
```
