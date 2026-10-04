# C1 M5-S51

- File: `glowcap.cav`
- Line: 252
- Site: `bind belief.note ... because contra`: add to this because an evidence the value never reads

Before:

```
bind belief.note = observations_text(contra) when belief_unsafe because contra;
```

After:

```
bind belief.note = observations_text(contra) when belief_unsafe because contra, taste_pit;
```
