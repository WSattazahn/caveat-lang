# C4 M5-S28

- File: `glowcap.cav`
- Line: 277
- Site: `bind journal.text_3 ... because journal_3`: add to this because an evidence the value never reads

Before:

```
bind journal.text_3 = journal_text(journal_3) because journal_3;
```

After:

```
bind journal.text_3 = journal_text(journal_3) because journal_3, taste_pit;
```
