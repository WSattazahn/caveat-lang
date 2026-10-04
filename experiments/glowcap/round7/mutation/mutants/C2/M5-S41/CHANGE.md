# C2 M5-S41

- File: `glowcap.cav`
- Line: 271
- Site: `bind belief.note ... because supported`: add to this because an evidence the value never reads

Before:

```
bind belief.note = "Based on " + number_text(supported) + " observations." when belief_safe and supported > 1 because supported;
```

After:

```
bind belief.note = "Based on " + number_text(supported) + " observations." when belief_safe and supported > 1 because supported, taste_pit;
```
