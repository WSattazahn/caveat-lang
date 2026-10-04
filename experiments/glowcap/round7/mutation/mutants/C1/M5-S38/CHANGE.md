# C1 M5-S38

- File: `glowcap.cav`
- Line: 232
- Site: `bind journal.e4 ... because j4`: add to this because an evidence the value never reads

Before:

```
bind journal.e4 = trust_text(j4) when j4 >= entry_trusted and j4 < 10 because j4;
```

After:

```
bind journal.e4 = trust_text(j4) when j4 >= entry_trusted and j4 < 10 because j4, taste_pit;
```
