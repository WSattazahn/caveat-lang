# C1 M5-S47

- File: `glowcap.cav`
- Line: 248
- Site: `bind belief.text ... because contra`: add to this because an evidence the value never reads

Before:

```
bind belief.text = "Glowing mushrooms make you heavy." when belief_unsafe because contra;
```

After:

```
bind belief.text = "Glowing mushrooms make you heavy." when belief_unsafe because contra, taste_pit;
```
