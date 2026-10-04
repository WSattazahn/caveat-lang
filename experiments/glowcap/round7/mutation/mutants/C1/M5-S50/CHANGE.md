# C1 M5-S50

- File: `glowcap.cav`
- Line: 251
- Site: `bind belief.note ... because support`: add to this because an evidence the value never reads

Before:

```
bind belief.note = observations_text(support) when belief_safe because support;
```

After:

```
bind belief.note = observations_text(support) when belief_safe because support, taste_pit;
```
