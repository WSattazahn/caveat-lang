# C2 M5-S42

- File: `glowcap.cav`
- Line: 272
- Site: `bind belief.note ... because contradicted`: add to this because an evidence the value never reads

Before:

```
bind belief.note = "Based on one observation." when belief_unsafe and contradicted == 1 because contradicted;
```

After:

```
bind belief.note = "Based on one observation." when belief_unsafe and contradicted == 1 because contradicted, taste_pit;
```
