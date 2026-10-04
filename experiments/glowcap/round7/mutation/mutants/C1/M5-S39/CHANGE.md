# C1 M5-S39

- File: `glowcap.cav`
- Line: 233
- Site: `bind journal.e5 ... because j5`: add to this because an evidence the value never reads

Before:

```
bind journal.e5 = trust_text(j5) when j5 >= entry_trusted and j5 < 10 because j5;
```

After:

```
bind journal.e5 = trust_text(j5) when j5 >= entry_trusted and j5 < 10 because j5, taste_pit;
```
