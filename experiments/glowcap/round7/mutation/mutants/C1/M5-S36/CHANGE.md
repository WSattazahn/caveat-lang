# C1 M5-S36

- File: `glowcap.cav`
- Line: 230
- Site: `bind journal.e2 ... because j2`: add to this because an evidence the value never reads

Before:

```
bind journal.e2 = trust_text(j2) when j2 >= entry_trusted and j2 < 10 because j2;
```

After:

```
bind journal.e2 = trust_text(j2) when j2 >= entry_trusted and j2 < 10 because j2, taste_pit;
```
