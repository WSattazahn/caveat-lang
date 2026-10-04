# C1 M5-S35

- File: `glowcap.cav`
- Line: 229
- Site: `bind journal.e1 ... because j1`: add to this because an evidence the value never reads

Before:

```
bind journal.e1 = trust_text(j1) when j1 >= entry_trusted and j1 < 10 because j1;
```

After:

```
bind journal.e1 = trust_text(j1) when j1 >= entry_trusted and j1 < 10 because j1, taste_pit;
```
