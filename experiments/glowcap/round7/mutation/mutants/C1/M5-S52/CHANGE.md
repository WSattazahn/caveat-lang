# C1 M5-S52

- File: `glowcap.cav`
- Line: 254
- Site: `bind belief.supportedBy ... because support`: add to this because an evidence the value never reads

Before:

```
bind belief.supportedBy = support because support;
```

After:

```
bind belief.supportedBy = support because support, taste_pit;
```
