# C1 M5-S40

- File: `glowcap.cav`
- Line: 234
- Site: `bind journal.e6 ... because j6`: add to this because an evidence the value never reads

Before:

```
bind journal.e6 = trust_text(j6) when j6 >= entry_trusted and j6 < 10 because j6;
```

After:

```
bind journal.e6 = trust_text(j6) when j6 >= entry_trusted and j6 < 10 because j6, taste_pit;
```
