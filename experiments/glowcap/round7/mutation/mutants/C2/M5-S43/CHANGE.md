# C2 M5-S43

- File: `glowcap.cav`
- Line: 273
- Site: `bind belief.note ... because contradicted`: add to this because an evidence the value never reads

Before:

```
bind belief.note = "Based on " + number_text(contradicted) + " observations." when belief_unsafe and contradicted > 1 because contradicted;
```

After:

```
bind belief.note = "Based on " + number_text(contradicted) + " observations." when belief_unsafe and contradicted > 1 because contradicted, taste_pit;
```
