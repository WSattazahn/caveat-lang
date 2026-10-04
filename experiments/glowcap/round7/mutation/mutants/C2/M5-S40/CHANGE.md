# C2 M5-S40

- File: `glowcap.cav`
- Line: 270
- Site: `bind belief.note ... because supported`: add to this because an evidence the value never reads

Before:

```
bind belief.note = "Based on one observation." when belief_safe and supported == 1 because supported;
```

After:

```
bind belief.note = "Based on one observation." when belief_safe and supported == 1 because supported, taste_pit;
```
