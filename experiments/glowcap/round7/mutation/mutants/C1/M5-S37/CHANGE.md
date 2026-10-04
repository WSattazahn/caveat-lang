# C1 M5-S37

- File: `glowcap.cav`
- Line: 231
- Site: `bind journal.e3 ... because j3`: add to this because an evidence the value never reads

Before:

```
bind journal.e3 = trust_text(j3) when j3 >= entry_trusted and j3 < 10 because j3;
```

After:

```
bind journal.e3 = trust_text(j3) when j3 >= entry_trusted and j3 < 10 because j3, taste_pit;
```
